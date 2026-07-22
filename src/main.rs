use warp::Filter;
use std::path::PathBuf;
use std::fs;
use std::env;
use warp::http::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use urlencoding::decode;
use chrono::Local;
use tokio::fs as tokio_fs;
use tokio::io::AsyncWriteExt;

// ==================== 配置 ====================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub title: String,
    pub description: String,
    pub background_api: String,
    pub opacity: f32,
    pub blur: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: "nweb".to_string(),
            description: "本网页由 nweb 自动生成".to_string(),
            background_api: "https://www.loliapi.com/acg/".to_string(),
            opacity: 0.3,
            blur: "5px".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct FileNode {
    name: String,
    is_dir: bool,
    size: u64,
    path: String,
}

// ==================== 主函数 ====================
#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("❌ 参数错了，雜鱼", );
        eprintln!("💡 食用方法: nweb <目录> <端口>");
        eprintln!("💡 示例: nweb ~/文档 7891");
        eprintln!("💡 如果想使用当前目录，请换成nweb . <端口>");
        std::process::exit(1);
    }

    let dir_str = args[1].clone();
    let expanded_dir = expand_path(&dir_str);
    let root_dir = PathBuf::from(&expanded_dir);
    let port = args[2].parse::<u16>().unwrap_or_else(|e| {
        eprintln!("❌ 雜鱼，端口错误: {}", e);
        std::process::exit(1);
    });

    if !root_dir.exists() || !root_dir.is_dir() {
        eprintln!("❌ 雜鱼，目录不存在或不是目录: {}", root_dir.display());
        std::process::exit(1);
    }

    // 初始化日志文件（清空）
    let log_path = root_dir.join("nweb.log");
    let _ = tokio_fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&log_path)
        .await;

    ensure_config(&root_dir);
    let config = load_config(&root_dir).unwrap_or_else(Config::default);
    
    print!("\x1B[2J\x1B[1;1H"); // 清屏
    println!("██    ██  ██     ██ ███████  ██████ ");
    println!("███   ██  ██     ██ ██       ██   ██ ");
    println!("██ ██ ██  ██  █  ██ ███████  ██████  ");
    println!("██   ███  ██ ███ ██ ██       ██   ██ ");
    println!("██    ██   ███ ███  ███████  ██████  ");
    println!("_________________________________");
    println!("🤓 本项目由Rust语言开发，NAS油条 制作");
    println!("🐧 问题反馈QQ群: 610699712 ");
    println!("🌍 请将index.html网页放在你选择的目录底下");
    println!("_________________________________");
    println!("📁 服务目录: {}", root_dir.display());
    println!("📄 日志文件: {}", log_path.display());
    println!("🌐 服务器地址: http://127.0.0.1:{}", port);
    println!("🚀 雜鱼服务器启动中...");
    println!("🛑 按 Ctrl+C 停止服务器");
    println!("_________________________________");
    
    let routes = warp::any()
        .and(warp::path::tail())
        .and(warp::path::full())
        .and_then(move |tail: warp::path::Tail, full: warp::path::FullPath| {
            let root = root_dir.clone();
            let config = config.clone();
            async move {
                handle_request(&root, tail.as_str(), full.as_str(), &config).await
            }
        });

    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

// ==================== 日志记录 ====================
async fn log_request(root: &PathBuf, path: &str, status: u16) {
    let time = Local::now().format("[%Y-%m-%d %H:%M:%S]");
    let msg = format!("{} {} {}\n", time, path, status);

    // 终端输出
    print!("{}", msg);

    // 写入日志文件
    let log_path = root.join("nweb.log");
    if let Ok(mut file) = tokio_fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_path)
        .await
    {
        let _ = file.write_all(msg.as_bytes()).await;
    }
}

// ==================== 请求处理 ====================
async fn handle_request(
    root: &PathBuf,
    tail: &str,
    _full_path: &str,
    config: &Config,
) -> Result<Response<Vec<u8>>, warp::Rejection> {
    // 禁止访问配置文件 & 日志文件
    if tail == "nweb.yml" || tail == "nweb.log" {
        log_request(root, tail, 404).await;
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(b"404 Not Found".to_vec())
            .unwrap());
    }

    // ===== API 路由：支持 /api/tree/ 和 /api/ =====
    if tail.starts_with("api/tree/") {
        let raw_path = &tail[9..];
        let decoded = decode(raw_path).unwrap_or_else(|_| raw_path.into());
        let path = decoded.trim_start_matches('/').trim_end_matches('/');

        match get_directory_tree(root, path) {
            Ok(tree) => {
                let json = serde_json::to_string(&tree).unwrap_or("[]".to_string());
                log_request(root, &format!("/api/tree/{}", path), 200).await;
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(json.into_bytes())
                    .unwrap());
            }
            Err(e) => {
                log_request(root, &format!("/api/tree/{}", path), 404).await;
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(format!("Error: {}", e).into_bytes())
                    .unwrap());
            }
        }
    }

    // 2. 新格式 /api/路径（外部调用）
    if tail.starts_with("api/") {
        let raw_path = &tail[4..]; // 去掉 "api/"
        let decoded = decode(raw_path).unwrap_or_else(|_| raw_path.into());
        let path = decoded.trim_start_matches('/').trim_end_matches('/');

        match get_directory_tree(root, path) {
            Ok(tree) => {
                let json = serde_json::to_string(&tree).unwrap_or("[]".to_string());
                log_request(root, &format!("/api/{}", path), 200).await;
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(json.into_bytes())
                    .unwrap());
            }
            Err(e) => {
                log_request(root, &format!("/api/{}", path), 404).await;
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(format!("Error: {}", e).into_bytes())
                    .unwrap());
            }
        }
    }

    // 根路径 → 优先检查 index.html
    if tail.is_empty() {
        let index_path = root.join("index.html");
        if index_path.exists() && index_path.is_file() {
            match tokio_fs::read_to_string(&index_path).await {
                Ok(content) => {
                    log_request(root, "/", 200).await;
                    return Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(content.into_bytes())
                        .unwrap());
                }
                Err(_) => {
                    // 读取失败则回退到默认页面
                    log_request(root, "/", 500).await;
                    let html = generate_index_html(config);
                    return Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(html.into_bytes())
                        .unwrap());
                }
            }
        } else {
            // 没有 index.html，显示默认页面
            let html = generate_index_html(config);
            log_request(root, "/", 200).await;
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(html.into_bytes())
                .unwrap());
        }
    }

    // ===== 文件下载 =====
    let decoded_tail = decode(tail).unwrap_or_else(|_| tail.into());
    let fs_path = root.join(&decoded_tail as &str);

    if !fs_path.starts_with(root) {
        log_request(root, &decoded_tail, 403).await;
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("雜鱼，禁止访问".as_bytes().to_vec())
            .unwrap());
    }

    if fs_path.is_file() {
        match tokio_fs::read(&fs_path).await {
            Ok(data) => {
                let mime = mime_guess::from_path(&fs_path)
                    .first_or_octet_stream();
                log_request(root, &decoded_tail, 200).await;
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", mime.as_ref())
                    .body(data)
                    .unwrap());
            }
            Err(_) => {
                log_request(root, &decoded_tail, 500).await;
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("雜鱼，文件读取失败".as_bytes().to_vec())
                    .unwrap());
            }
        }
    }

    log_request(root, tail, 404).await;
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(b"404 Not Found".to_vec())
        .unwrap())
}

// ==================== 目录树生成 ====================
fn get_directory_tree(root: &PathBuf, path: &str) -> Result<Vec<FileNode>, String> {
    let current_path = if path.is_empty() {
        root.clone()
    } else {
        root.join(path)
    };

    if !current_path.starts_with(root) {
        return Err("Access denied".to_string());
    }
    if !current_path.exists() || !current_path.is_dir() {
        return Err(format!("雜鱼，目录不存在: {}", current_path.display()));
    }

    let mut nodes = Vec::new();
    let entries = fs::read_dir(&current_path).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // 隐藏文件
        if name.starts_with('.') || name == "nweb.yml" || name == "nweb.log" {
            continue;
        }

        let is_dir = path.is_dir();
        let size = if is_dir {
            0
        } else {
            fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
        };

        let relative_path = path.strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        nodes.push(FileNode { name, is_dir, size, path: relative_path });
    }

    nodes.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    Ok(nodes)
}

// ==================== 配置文件管理 ====================
fn ensure_config(root: &PathBuf) {
    let config_path = root.join("nweb.yml");
    if !config_path.exists() {
        let default_config = Config::default();
        let yaml = serde_yaml::to_string(&default_config).unwrap();
        match fs::write(&config_path, yaml) {
            Ok(_) => println!("✅ 默认配置文件: {}", config_path.display()),
            Err(e) => eprintln!("⚠️  无法写入配置文件: {}", e),
        }
    }
}

fn load_config(root: &PathBuf) -> Option<Config> {
    let config_path = root.join("nweb.yml");
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(cfg) = serde_yaml::from_str::<Config>(&content) {
                println!("✅ 已加载配置文件: {}", config_path.display());
                return Some(cfg);
            }
        }
    }
    None
}

// ==================== 生成 HTML（单页应用） ====================
fn generate_index_html(config: &Config) -> String {
    let title = html_escape(&config.title);
    let description = html_escape(&config.description);
    let bg_api = &config.background_api;
    let opacity = config.opacity;
    let blur = &config.blur;

    format!(r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        ul, li {{ margin:0; padding:0; list-style:none; }}
        * {{ margin:0; padding:0; box-sizing:border-box; }}

        html,body {{
            min-height:100vh;
            font-family:'Segoe UI',Arial,sans-serif;
            background: url('{}') no-repeat center center fixed;
            background-size: cover;
        }}
        .container {{
            max-width:960px;
            margin:30px auto;
            padding:30px;
            background: rgba(255, 255, 255, {});
            border-radius:20px;
            box-shadow:0 20px 60px rgba(0,0,0,0.3);
            backdrop-filter: blur({});
            -webkit-backdrop-filter: blur({});
            border:1px solid rgba(255,255,255,0.3);
        }}
        .header {{ display:flex; justify-content:space-between; align-items:center; flex-wrap:wrap; gap:15px; margin-bottom:20px; padding-bottom:15px; border-bottom:2px solid rgba(102,126,234,0.2); }}
        .header-left h1 {{ font-size:28px; font-weight:700; background:linear-gradient(135deg,#667eea,#764ba2); -webkit-background-clip:text; -webkit-text-fill-color:transparent; }}
        .btn {{ padding:8px 18px; border:none; border-radius:25px; font-size:14px; font-weight:500; cursor:pointer; transition:all 0.3s; background:rgba(102,126,234,0.15); color:#667eea; border:1px solid rgba(102,126,234,0.2); }}
        .btn:hover {{ transform:translateY(-2px); box-shadow:0 4px 12px rgba(102,126,234,0.3); }}
        .btn-primary {{ background:linear-gradient(135deg,#667eea,#764ba2); color:white; border:none; }}
        .btn-primary:hover {{ box-shadow:0 4px 15px rgba(102,126,234,0.4); }}
        .project-info {{ background:rgba(255,255,255,0.5); border-radius:12px; padding:15px 20px; margin-bottom:15px; }}
        .project-info p {{ margin:0; color:#555; line-height:1.6; }}
        .stats {{ display:flex; gap:15px; flex-wrap:wrap; margin-bottom:20px; }}
        .stats-item {{ display:flex; align-items:center; gap:6px; font-size:14px; color:#666; background:rgba(255,255,255,0.6); padding:4px 14px; border-radius:20px; }}
        .stats-item .num {{ font-weight:700; color:#667eea; }}
        .file-tree {{ margin-top:10px; }}
        .tree-root {{ list-style:none; padding-left:0; }}
        .tree-root li {{ margin:4px 0; list-style:none; }}
        .entry {{ display:flex; align-items:center; justify-content:space-between; padding:10px 14px; border-radius:10px; transition:all 0.3s; text-decoration:none; color:#333; background:rgba(255,255,255,0.5); border:1px solid transparent; cursor:pointer; }}
        .entry:hover {{ background:rgba(102,126,234,0.08); border-color:rgba(102,126,234,0.15); transform:translateX(4px); }}
        .folder-entry {{ background:rgba(102,126,234,0.06); font-weight:600; border-left:4px solid #667eea; }}
        .file-entry {{ background:rgba(255,255,255,0.3); }}
        .entry-left {{ display:flex; align-items:center; gap:10px; flex:1; min-width:0; }}
        .entry-left .icon {{ font-size:20px; flex-shrink:0; }}
        .entry-left .name {{ word-break:break-word; font-size:15px; }}
        .file-info {{ color:#999; font-size:13px; flex-shrink:0; margin-left:10px; }}
        .children {{ padding-left:30px; }}
        .toggle-icon {{ transition:transform 0.3s; display:inline-block; margin-right:6px; color:#667eea; }}
        .toggle-icon.open {{ transform:rotate(90deg); }}
        .footer {{ margin-top:30px; padding-top:20px; border-top:1px solid rgba(102,126,234,0.1); text-align:center; font-size:13px; color:#999; }}
        .footer span {{ color:#667eea; }}
        .loading {{ text-align:center; padding:20px; color:#666; }}
        @keyframes spin {{ 100% {{ transform:rotate(360deg); }} }}
        .spinner {{ display:inline-block; width:20px; height:20px; border:2px solid rgba(102,126,234,0.2); border-top:2px solid #667eea; border-radius:50%; animation:spin 0.8s linear infinite; }}
        @media (max-width:768px) {{ .container {{ margin:15px; padding:20px; }} .header-left h1 {{ font-size:22px; }} .stats {{ gap:8px; }} .stats-item {{ font-size:12px; padding:3px 10px; }} .entry {{ padding:8px 12px; font-size:14px; }} .entry-left .icon {{ font-size:17px; }} .file-info {{ font-size:11px; }} .children {{ padding-left:20px; }} }}
        @media (max-width:480px) {{ .container {{ margin:10px; padding:15px; }} .header {{ flex-direction:column; align-items:stretch; }} .header-right {{ justify-content:flex-start; }} .btn {{ padding:6px 14px; font-size:13px; }} .children {{ padding-left:15px; }} }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="header-left"><h1>📁 {}</h1></div>
            <div class="header-right">
                <button class="btn" onclick="window.location.reload()">🔄 刷新</button>
                <button class="btn btn-primary" onclick="collapseAll()">📁 全部折叠</button>
            </div>
        </div>
        <div class="project-info">
            <p>{}</p>
        </div>
        <div class="stats" id="stats">
            <span class="stats-item" id="fileCount">📄 文件 <span class="num">0</span></span>
            <span class="stats-item" id="dirCount">📁 目录 <span class="num">0</span></span>
        </div>
        <div class="file-tree">
            <div id="treeRoot" class="tree-root"></div>
        </div>
        <div class="footer">
            <p>⚡ 由 <span>Rust</span> 强力驱动 · 快速文件浏览</p>
        </div>
    </div>

    <script>
        let loadedPaths = new Set();
        let loadingPaths = new Set();
        let totalFiles = 0;
        let totalDirs = 0;

        document.addEventListener('DOMContentLoaded', function() {{
            loadDirectory('');
        }});

        async function loadDirectory(path, containerId = null) {{
            const container = containerId ? document.getElementById(containerId) : document.getElementById('treeRoot');
            if (loadedPaths.has(path) || loadingPaths.has(path)) return;
            loadingPaths.add(path);

            const loader = document.createElement('div');
            loader.className = 'loading';
            loader.innerHTML = '<div class="spinner"></div><p>加载中...</p>';
            container.appendChild(loader);

            try {{
                const cleanPath = path.replace(/^\/+|\/+$/g, '');
                const response = await fetch(`/api/tree/${{encodeURIComponent(cleanPath)}}`);
                if (!response.ok) {{
                    const errText = await response.text();
                    throw new Error(errText || `HTTP ${{response.status}}`);
                }}
                const data = await response.json();

                loader.remove();
                loadingPaths.delete(path);
                loadedPaths.add(path);

                const ul = document.createElement('ul');
                ul.className = 'children';
                ul.id = `children-${{path.replace(/[^a-zA-Z0-9]/g, '_')}}`;

                let filesInDir = 0;
                let dirsInDir = 0;

                data.forEach(node => {{
                    const li = document.createElement('li');

                    if (node.is_dir) {{
                        dirsInDir++;
                        const entry = document.createElement('div');
                        entry.className = 'entry folder-entry';
                        const toggleSpan = document.createElement('span');
                        toggleSpan.className = 'toggle-icon';
                        toggleSpan.innerHTML = '▶';
                        const entryLeft = document.createElement('span');
                        entryLeft.className = 'entry-left';
                        entryLeft.innerHTML = `
                            <span class="icon">📁</span>
                            <span class="name">${{node.name}}</span>
                        `;
                        entry.appendChild(toggleSpan);
                        entry.appendChild(entryLeft);

                        const fileInfo = document.createElement('span');
                        fileInfo.className = 'file-info';
                        fileInfo.textContent = '目录';
                        entry.appendChild(fileInfo);

                        const childrenContainer = document.createElement('div');
                        childrenContainer.className = 'children';
                        childrenContainer.style.display = 'none';
                        childrenContainer.id = `children-container-${{node.path.replace(/[^a-zA-Z0-9]/g, '_')}}`;

                        entry.onclick = function(e) {{
                            e.stopPropagation();
                            const isOpen = childrenContainer.style.display !== 'none';
                            if (isOpen) {{
                                childrenContainer.style.display = 'none';
                                toggleSpan.className = 'toggle-icon';
                                toggleSpan.innerHTML = '▶';
                            }} else {{
                                childrenContainer.style.display = 'block';
                                toggleSpan.className = 'toggle-icon open';
                                toggleSpan.innerHTML = '▼';
                                const childId = `children-${{node.path.replace(/[^a-zA-Z0-9]/g, '_')}}`;
                                if (!document.getElementById(childId)) {{
                                    loadDirectory(node.path, childrenContainer.id);
                                }}
                            }}
                        }};

                        li.appendChild(entry);
                        li.appendChild(childrenContainer);
                    }} else {{
                        filesInDir++;
                        const entry = document.createElement('a');
                        entry.className = 'entry file-entry';
                        const filePath = node.path || node.name;
                        entry.href = `/${{filePath}}`;
                        entry.target = '_blank';
                        entry.innerHTML = `
                            <span class="entry-left">
                                <span class="icon">${{getFileIcon(node.name)}}</span>
                                <span class="name">${{node.name}}</span>
                            </span>
                            <span class="file-info">${{formatSize(node.size)}}</span>
                        `;
                        li.appendChild(entry);
                    }}
                    ul.appendChild(li);
                }});

                if (data.length === 0) {{
                    const emptyLi = document.createElement('li');
                    emptyLi.style.color = '#999';
                    emptyLi.style.padding = '10px';
                    emptyLi.textContent = '雜鱼 这目录是空的';
                    ul.appendChild(emptyLi);
                }}

                container.innerHTML = '';
                container.appendChild(ul);

                totalFiles += filesInDir;
                totalDirs += dirsInDir;
                document.getElementById('fileCount').querySelector('.num').textContent = totalFiles;
                document.getElementById('dirCount').querySelector('.num').textContent = totalDirs;

            }} catch (error) {{
                loader.innerHTML = `<p style="color:#ef4444;">❌ 加载失败: ${{error.message}}</p>`;
                loadingPaths.delete(path);
                console.error('加载失败:', error);
            }}
        }}

        function getFileIcon(filename) {{
            const ext = filename.split('.').pop().toLowerCase();
            const icons = {{
                'html': '🌐', 'htm': '🌐',
                'css': '🎨',
                'js': '⚡', 'jsx': '⚡', 'ts': '⚡', 'tsx': '⚡',
                'rs': '🦀',
                'py': '🐍',
                'go': '🐹',
                'java': '☕',
                'c': '⚙️', 'cpp': '⚙️', 'h': '⚙️', 'hpp': '⚙️',
                'json': '📋',
                'xml': '📝', 'yaml': '📝', 'yml': '📝', 'toml': '📝',
                'md': '📖',
                'txt': '📄', 'log': '📄',
                'pdf': '📕',
                'jpg': '🖼️', 'jpeg': '🖼️', 'png': '🖼️', 'gif': '🖼️', 'svg': '🖼️', 'webp': '🖼️',
                'mp4': '🎬', 'avi': '🎬', 'mov': '🎬', 'mkv': '🎬',
                'mp3': '🎵', 'wav': '🎵', 'flac': '🎵',
                'zip': '📦', 'rar': '📦', '7z': '📦', 'tar': '📦', 'gz': '📦',
                'exe': '📦', 'msi': '📦', 'dmg': '📦', 'apk': '📦',
                'sh': '📃', 'bash': '📃', 'zsh': '📃', 'rb': '📃', 'fish': '📃',
                'dockerfile': '🐳',
                'gitignore': '📌', 'gitattributes': '📌',
                'lock': '🔒',
                'wasm': '⚙️',
                'proto': '📡',
            }};
            return icons[ext] || '📄';
        }}

        function formatSize(bytes) {{
            if (bytes === 0) return '0 B';
            const units = ['B', 'KB', 'MB', 'GB', 'TB'];
            const k = 1024;
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            const size = (bytes / Math.pow(k, i)).toFixed(i > 0 ? 1 : 0);
            return size + ' ' + units[i];
        }}

        function collapseAll() {{
            document.querySelectorAll('.children:not(:first-child)').forEach(el => {{
                el.style.display = 'none';
            }});
            document.querySelectorAll('.toggle-icon').forEach(el => {{
                el.className = 'toggle-icon';
                el.innerHTML = '▶';
            }});
        }}
    </script>
</body>
</html>"#,
        title, bg_api, opacity, blur, blur, title, description
    )
}

// ==================== 辅助函数 ====================
fn expand_path(path: &str) -> String {
    let mut result = path.to_string();
    if result.starts_with('~') {
        if let Ok(home) = env::var("HOME") {
            result = result.replacen('~', &home, 1);
        }
    }
    if result.contains('$') {
        if let Ok(home) = env::var("HOME") {
            result = result.replace("$HOME", &home).replace("${HOME}", &home);
        }
        if let Ok(user) = env::var("USER") {
            result = result.replace("$USER", &user).replace("${USER}", &user);
        }
    }
    result
}

fn html_escape(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;")
}