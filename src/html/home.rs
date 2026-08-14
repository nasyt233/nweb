use crate::config::Config;

/// HTML 转义函数
pub fn html_escape(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;")
}

/// 主页面 HTML
pub fn generate_index_html(config: &Config) -> String {
    let title = html_escape(&config.title);
    let description = html_escape(&config.description);
    let bg_api = &config.background_api;
    let opacity = config.opacity;
    let blur = &config.blur;
    let admin_btn_style = if config.show_admin_btn {
        "display:inline-block;"
    } else {
        "display:none;"
    };
    let show_file_size_js = if config.show_file_size { "true" } else { "false" };

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        /* ========== 全局样式 ========== */
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        html {{
            margin: 0;
            padding: 0;
            min-height: 100vh;
            height: 100%;
        }}
        
        body {{
            margin: 0;
            padding: 0;
            min-height: 100vh;
            height: 100%;
            font-family:'Segoe UI',Arial,sans-serif;
            background: url('{}') no-repeat center center fixed;
            background-size: cover;
            transition: background-color 0.3s, color 0.3s;
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
            transition: background 0.3s, backdrop-filter 0.3s;
        }}
        .header {{
            display:flex;
            justify-content:space-between;
            align-items:center;
            flex-wrap:wrap;
            gap:15px;
            margin-bottom:20px;
            padding-bottom:15px;
            border-bottom:2px solid rgba(102,126,234,0.2);
        }}
        .header-left {{
            display:flex;
            align-items:center;
            gap:12px;
        }}
        .header-left h1 {{
            font-size:28px;
            font-weight:700;
            background:linear-gradient(135deg,#667eea,#764ba2);
            -webkit-background-clip:text;
            -webkit-text-fill-color:transparent;
            transition: background 0.3s;
        }}
        .btn {{
            padding:8px 18px;
            border:none;
            border-radius:25px;
            font-size:14px;
            font-weight:500;
            cursor:pointer;
            transition:all 0.3s;
            background:rgba(102,126,234,0.15);
            color:#667eea;
            border:1px solid rgba(102,126,234,0.2);
        }}
        .btn:hover {{
            transform:translateY(-2px);
            box-shadow:0 4px 12px rgba(102,126,234,0.3);
        }}
        .btn-primary {{
            background:linear-gradient(135deg,#667eea,#764ba2);
            color:white;
            border:none;
        }}
        .btn-primary:hover {{
            box-shadow:0 4px 15px rgba(102,126,234,0.4);
        }}
        /* ========== 主题切换按钮 ========== */
        .theme-btn {{
            background: rgba(255, 255, 255, 0.3);
            border: 1px solid rgba(255, 255, 255, 0.5);
            border-radius: 30px;
            padding: 8px 16px;
            cursor: pointer;
            font-size: 14px;
            backdrop-filter: blur(5px);
            transition: all 0.3s;
            color: #333;
        }}
        .theme-btn:hover {{
            background: rgba(255, 255, 255, 0.5);
            transform: scale(1.05);
        }}
        .project-info {{
            background: rgba(255, 255, 255, 0.5);
            border-radius:12px;
            padding:15px 20px;
            margin-bottom:15px;
            backdrop-filter: blur(5px);
            border: 1px solid rgba(255, 255, 255, 0.3);
            transition: background 0.3s, border-color 0.3s;
        }}
        .project-info p {{ margin:0; color:#555; line-height:1.6; transition: color 0.3s; }}
        .stats {{
            display:flex;
            gap:15px;
            flex-wrap:wrap;
            margin-bottom:20px;
        }}
        .stats-item {{
            display:flex;
            align-items:center;
            gap:6px;
            font-size:14px;
            color:#666;
            background:rgba(255,255,255,0.6);
            padding:4px 14px;
            border-radius:20px;
            border:1px solid rgba(255,255,255,0.3);
            transition: background 0.3s, color 0.3s, border-color 0.3s;
        }}
        .stats-item .num {{ font-weight:700; color:#667eea; transition: color 0.3s; }}
        .file-tree {{ margin-top:10px; }}
        .tree-root {{ list-style:none; padding-left:0; }}
        .tree-root li {{ margin:4px 0; list-style:none; }}
        .entry {{
            display:flex;
            align-items:center;
            justify-content:space-between;
            padding:10px 14px;
            border-radius:10px;
            transition:all 0.3s;
            text-decoration:none;
            color:#333;
            background:rgba(255,255,255,0.5);
            border:1px solid transparent;
            cursor:pointer;
        }}
        .entry:hover {{
            background:rgba(102,126,234,0.08);
            border-color:rgba(102,126,234,0.15);
            transform:translateX(4px);
        }}
        .folder-entry {{
            background:rgba(102,126,234,0.06);
            font-weight:600;
            border-left:4px solid #667eea;
        }}
        .file-entry {{
            background:rgba(255,255,255,0.3);
        }}
        .entry-left {{
            display:flex;
            align-items:center;
            gap:10px;
            flex:1;
            min-width:0;
        }}
        .entry-left .icon {{ font-size:20px; flex-shrink:0; }}
        .entry-left .name {{ word-break:break-word; font-size:15px; }}
        .file-info {{ color:#999; font-size:13px; flex-shrink:0; margin-left:10px; transition: color 0.3s; }}
        .children {{ padding-left:30px; }}
        .toggle-icon {{ transition:transform 0.3s; display:inline-block; margin-right:6px; color:#667eea; }}
        .toggle-icon.open {{ transform:rotate(90deg); }}
        .footer {{
            margin-top:30px;
            padding-top:20px;
            border-top:1px solid rgba(102,126,234,0.1);
            text-align:center;
            font-size:13px;
            color:#999;
            transition: border-color 0.3s, color 0.3s;
        }}
        .footer span {{ color:#667eea; transition: color 0.3s; }}
        .loading {{ text-align:center; padding:20px; color:#666; }}
        @keyframes spin {{ 100% {{ transform:rotate(360deg); }} }}
        .spinner {{ display:inline-block; width:20px; height:20px; border:2px solid rgba(102,126,234,0.2); border-top:2px solid #667eea; border-radius:50%; animation:spin 0.8s linear infinite; }}

        /* 返回顶部按钮 */
        .back-to-top {{
            position: fixed;
            bottom: 30px;
            right: 30px;
            width: 50px;
            height: 50px;
            background: linear-gradient(135deg, #667eea, #764ba2);
            color: white;
            border: none;
            border-radius: 50%;
            font-size: 24px;
            cursor: pointer;
            box-shadow: 0 4px 15px rgba(0,0,0,0.3);
            transition: all 0.3s;
            display: none;
            z-index: 999;
        }}
        .back-to-top:hover {{
            transform: scale(1.1);
            box-shadow: 0 6px 20px rgba(102,126,234,0.4);
        }}
        .back-to-top.show {{ display: block; }}

        /* ========== 夜间模式 ========== */
        html.night-mode,
        body.night-mode {{
            background: #1a1a1a;
            background-size: cover;
            background-position: center;
            min-height: 100vh;
            height: 100%;
        }}
        body.night-mode .container {{
            background: rgba(30, 30, 30, {});
            backdrop-filter: blur({}) saturate(180%);
            border-color: rgba(255, 255, 255, 0.1);
            color: #eee;
        }}
        body.night-mode .project-info {{
            background: rgba(40, 40, 40, 0.6);
            border-color: rgba(255, 255, 255, 0.1);
        }}
        body.night-mode .project-info p {{
            color: #eee;
        }}
        body.night-mode .stats-item {{
            background: rgba(50, 50, 50, 0.6);
            border-color: rgba(255, 255, 255, 0.1);
            color: #ccc;
        }}
        body.night-mode .stats-item .num {{
            color: #7ab0e0;
        }}
        body.night-mode .header-left h1 {{
            background: linear-gradient(135deg, #7ab0e0, #a78bfa);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }}
        body.night-mode .theme-btn {{
            background: rgba(50, 50, 50, 0.5);
            border-color: rgba(255, 255, 255, 0.2);
            color: #eee;
        }}
        body.night-mode .theme-btn:hover {{
            background: rgba(80, 80, 80, 0.6);
        }}
        body.night-mode .btn {{
            background: rgba(122, 176, 224, 0.2);
            color: #7ab0e0;
            border-color: rgba(122, 176, 224, 0.3);
        }}
        body.night-mode .btn-primary {{
            background: linear-gradient(135deg, #5a9acf, #8b6fe8);
            color: white;
        }}
        body.night-mode .tree-root .folder-entry {{
            background: rgba(40, 60, 80, 0.8);
            border-left-color: #5a9acf;
            color: #ddd;
        }}
        body.night-mode .tree-root .file-entry {{
            background: rgba(50, 50, 50, 0.8);
            color: #ccc;
        }}
        body.night-mode .tree-root .entry:hover {{
            background: rgba(122, 176, 224, 0.15);
            border-color: rgba(122, 176, 224, 0.3);
        }}
        body.night-mode .file-info {{
            color: #aaa;
        }}
        body.night-mode .footer {{
            border-top-color: rgba(255, 255, 255, 0.1);
            color: #888;
        }}
        body.night-mode .footer span {{
            color: #7ab0e0;
        }}
        body.night-mode .entry {{
            color: #ddd;
            background: rgba(40, 40, 40, 0.6);
        }}
        body.night-mode .entry:hover {{
            background: rgba(122, 176, 224, 0.15);
        }}
        body.night-mode .project-info h2 {{
            color: #eee;
        }}
        body.night-mode .loading {{
            color: #999;
        }}
        body.night-mode .back-to-top {{
            background: linear-gradient(135deg, #5a9acf, #8b6fe8);
        }}

        @media (max-width:768px) {{
            .container {{ margin:15px; padding:20px; }}
            .header-left h1 {{ font-size:22px; }}
            .stats {{ gap:8px; }}
            .stats-item {{ font-size:12px; padding:3px 10px; }}
            .entry {{ padding:8px 12px; font-size:14px; }}
            .entry-left .icon {{ font-size:17px; }}
            .file-info {{ font-size:11px; }}
            .children {{ padding-left:20px; }}
            .back-to-top {{ bottom:20px; right:20px; width:45px; height:45px; font-size:20px; }}
        }}
        @media (max-width:480px) {{
            .container {{ margin:10px; padding:15px; }}
            .header {{ flex-direction:column; align-items:stretch; }}
            .header-right {{ justify-content:flex-start; flex-wrap:wrap; }}
            .btn {{ padding:6px 14px; font-size:13px; }}
            .children {{ padding-left:15px; }}
            .back-to-top {{ bottom:15px; right:15px; width:40px; height:40px; font-size:18px; }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="header-left">
                <h1>📁 {}</h1>
            </div>
            <div class="header-right">
                <button class="theme-btn" id="themeToggle">🌙 夜间模式</button>
                <button class="btn" onclick="window.location.reload()">🔄 刷新</button>
                <button class="btn btn-primary" onclick="collapseAll()">📁 全部折叠</button>
                <button class="btn" onclick="window.location.href='/@admin'" id="adminBtn" style="{};">
                    ⚙️ 管理
                </button>
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

    <!-- 返回顶部按钮 -->
    <button id="backToTop" class="back-to-top" onclick="scrollToTop()">⬆</button>

    <script>
        // ========== 从后端传入配置 ==========
        const showFileSize = {};

        // ========== 夜间模式 ==========
        (function() {{
            const themeToggle = document.getElementById('themeToggle');
            const body = document.body;

            const savedTheme = localStorage.getItem('nweb_theme');
            if (savedTheme === 'night') {{
                body.classList.add('night-mode');
                themeToggle.textContent = '☀️ 日间模式';
            }} else {{
                themeToggle.textContent = '🌙 夜间模式';
            }}

            themeToggle.addEventListener('click', function() {{
                body.classList.toggle('night-mode');
                const isNight = body.classList.contains('night-mode');
                themeToggle.textContent = isNight ? '☀️ 日间模式' : '🌙 夜间模式';
                localStorage.setItem('nweb_theme', isNight ? 'night' : 'day');
            }});
        }})();

        // ========== 文件浏览 ==========
        let loadedPaths = new Set();
        let loadingPaths = new Set();
        let totalFiles = 0;
        let totalDirs = 0;

        document.addEventListener('DOMContentLoaded', function() {{
            loadDirectory('');
            window.addEventListener('scroll', function() {{
                const btn = document.getElementById('backToTop');
                if (window.scrollY > 300) {{
                    btn.classList.add('show');
                }} else {{
                    btn.classList.remove('show');
                }}
            }});
        }});

        function scrollToTop() {{
            window.scrollTo({{ top: 0, behavior: 'smooth' }});
        }}

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

                        let sizeHtml = '';
                        if (showFileSize) {{
                            sizeHtml = `<span class="file-info">${{formatSize(node.size)}}</span>`;
                        }}

                        entry.innerHTML = `
                            <span class="entry-left">
                                <span class="icon">${{getFileIcon(node.name)}}</span>
                                <span class="name">${{node.name}}</span>
                            </span>
                            ${{sizeHtml}}
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
        // 参数对应
        title, bg_api, opacity, blur, blur, opacity, blur, title, admin_btn_style, description,
        show_file_size_js
    )
}