use warp::Filter;
use std::path::PathBuf;
use std::fs;
use std::env;
use warp::reply::{html as warp_html, Reply};

/// 外部文件 Web 服务器
#[derive(clap::Parser, Debug)]
struct Args {
    /// 要服务的目录路径
    directory: PathBuf,
    
    /// 服务器端口
    port: u16,
}

#[tokio::main]
async fn main() {
    // 解析命令行参数
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("❌ 参数错了，雜鱼: {}", e);
            eprintln!("💡 食用方法: nweb <目录> <端口>");
            eprintln!("💡 示例: nweb ~/Documents 8080");
            eprintln!("💡 如果想使用当前目录，请换成nweb . <端口>");
            std::process::exit(1);
        }
    };
    
    // 展开路径
    let dir_str = args.directory.to_string_lossy().to_string();
    let expanded_dir = expand_path(&dir_str);
    let external_dir = PathBuf::from(&expanded_dir);
    let port = args.port;
    
    print!("\x1B[2J\x1B[1;1H"); // 清屏
    println!("███╗  ██╗██╗  ██╗██╗    ██╗███████╗██████╗");
    println!("████╗ ██║╚██╗██╔╝██║    ██║██╔════╝██╔══██╗");
    println!("██╔██╗██║ ╚███╔╝ ██║ █╗ ██║█████╗  ██████╔╝");
    println!("██║╚████║ ██╔██╗ ██║███╗██║██╔══╝  ██╔══██╗");
    println!("██║ ╚███║██╔╝ ██╗╚███╔███╔╝███████╗██║  ██║");
    println!("╚═╝  ╚══╝╚═╝  ╚═╝ ╚══╝╚══╝ ╚══════╝╚═╝  ╚═╝");
    println!("_________________________________");
    println!("🤓 本项目由rust制作，NAS油条 开发");
    println!("🌍 请将index.html网页放在你选择的目录底下");
    println!("_________________________________");
    println!("📁 服务目录: {}", external_dir.display());
    println!("🌐 服务器地址: http://127.0.0.1:{}", port);
    println!("🔗 访问示例: http://127.0.0.1:{}/index.html", port);
    
    // 检查目录是否存在
    if !external_dir.exists() {
        println!("❌ 错误: 目录不存在 - {}", external_dir.display());
        std::process::exit(1);
    }
    
    if !external_dir.is_dir() {
        println!("❌ 错误: 不是目录 - {}", external_dir.display());
        std::process::exit(1);
    }
    
    // 获取绝对路径
    let abs_dir = match fs::canonicalize(&external_dir) {
        Ok(path) => path,
        Err(e) => {
            println!("❌ 无法获取绝对路径: {}", e);
            external_dir
        }
    };
    
    // 静态文件服务
    let static_files = warp::fs::dir(abs_dir.clone());
    
    // 默认路由：显示目录列表或 index.html
    let index = warp::path::end()
        .and_then(move || {
            let dir = abs_dir.clone();
            async move {
                handle_root(&dir).await
            }
        });
    
    // 组合路由
    let routes = index.or(static_files);
    
    println!("🚀 启动中，雜鱼...");
    println!("🛑 按 Ctrl+C 停止服务器");
    println!("_________________________________");
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

// 处理根路径请求
async fn handle_root(dir: &PathBuf) -> Result<Box<dyn Reply>, warp::Rejection> {
    // 尝试打开 index.html
    let index_path = dir.join("index.html");
    if index_path.exists() && index_path.is_file() {
        match fs::read_to_string(&index_path) {
            Ok(content) => Ok(Box::new(warp_html(content))),
            Err(_) => {
                match list_directory(dir).await {
                    Ok(reply) => Ok(Box::new(reply)),
                    Err(e) => Err(e),
                }
            }
        }
    } else {
        match list_directory(dir).await {
            Ok(reply) => Ok(Box::new(reply)),
            Err(e) => Err(e),
        }
    }
}

// 解析命令行参数（手动实现，避免依赖 clap 的 Parse trait）
struct ArgsParsed {
    directory: PathBuf,
    port: u16,
}

fn parse_args() -> Result<ArgsParsed, String> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        return Err("需要两个参数：目录和端口".to_string());
    }
    
    let directory = PathBuf::from(&args[1]);
    let port = args[2].parse::<u16>()
        .map_err(|e| format!("端口必须是 1-65535 之间的数字: {}", e))?;
    
    Ok(ArgsParsed { directory, port })
}

// 展开路径（支持 ~ 和环境变量）
fn expand_path(path: &str) -> String {
    let mut result = path.to_string();
    
    // 展开 ~ 到用户目录
    if result.starts_with('~') {
        if let Ok(home) = env::var("HOME") {
            result = result.replacen('~', &home, 1);
        }
    }
    
    // 展开环境变量
    if result.contains('$') {
        if result.contains("$HOME") || result.contains("${HOME}") {
            if let Ok(home) = env::var("HOME") {
                result = result.replace("$HOME", &home).replace("${HOME}", &home);
            }
        }
        if result.contains("$USER") || result.contains("${USER}") {
            if let Ok(user) = env::var("USER") {
                result = result.replace("$USER", &user).replace("${USER}", &user);
            }
        }
    }
    
    result
}

// 生成目录列表
async fn list_directory(dir: &PathBuf) -> Result<impl Reply, warp::Rejection> {
    let mut html = String::new();
    
    html.push_str(r#"<!DOCTYPE html><html><head>
        <title>目录列表</title>
        <meta charset="UTF-8">
        <style>
            body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; 
                   padding: 20px; background: #f5f5f5; }
            .container { max-width: 800px; margin: 0 auto; background: white; 
                        padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
            h1 { color: #333; margin-bottom: 20px; border-bottom: 2px solid #eee; padding-bottom: 10px; }
            ul { list-style: none; padding: 0; }
            li { padding: 10px; border-radius: 5px; }
            li:hover { background: #f0f0f0; }
            a { text-decoration: none; color: #0066cc; display: flex; align-items: center; }
            a:hover { text-decoration: underline; }
            .dir::before { content: "📁 "; font-size: 1.2em; margin-right: 8px; }
            .file::before { content: "📄 "; font-size: 1.2em; margin-right: 8px; }
            .parent::before { content: "🔙 "; font-size: 1.2em; margin-right: 8px; color: #666; }
            .file-size { margin-left: auto; color: #666; font-size: 0.9em; }
            .error { color: #d32f2f; padding: 20px; background: #ffebee; border-radius: 5px; }
        </style>
    </head><body>
    <div class="container">"#);
    
    html.push_str(&format!("<h1>📁 {}</h1>", dir.display()));
    html.push_str("<ul>");
    
    // 上级目录链接
    if let Some(_parent) = dir.parent() {
        html.push_str(r#"<li class="parent"><a href="/../"><span>上级目录</span></a></li>"#);
    }
    
    // 列出文件和目录
    match fs::read_dir(dir) {
        Ok(entries) => {
            let mut entries: Vec<_> = entries.collect();
            entries.sort_by(|a, b| {
                let a = a.as_ref().unwrap();
                let b = b.as_ref().unwrap();
                let a_is_dir = a.path().is_dir();
                let b_is_dir = b.path().is_dir();
                
                if a_is_dir && !b_is_dir {
                    std::cmp::Ordering::Less
                } else if !a_is_dir && b_is_dir {
                    std::cmp::Ordering::Greater
                } else {
                    a.file_name().cmp(&b.file_name())
                }
            });
            
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        let name = entry.file_name().to_string_lossy().to_string();
                        let is_dir = path.is_dir();
                        
                        let class = if is_dir { "dir" } else { "file" };
                        let size = if is_dir {
                            "目录".to_string()
                        } else {
                            match fs::metadata(&path) {
                                Ok(metadata) => format!("{} bytes", metadata.len()),
                                Err(_) => "未知".to_string()
                            }
                        };
                        
                        html.push_str(&format!(
                            r#"<li class="{}"><a href="/{}"><span>{}</span><span class="file-size">{}</span></a></li>"#,
                            class, name, name, size
                        ));
                    }
                    Err(e) => {
                        html.push_str(&format!(r#"<li class="error">读取错误: {}</li>"#, e));
                    }
                }
            }
        }
        Err(e) => {
            html.push_str(&format!(r#"<div class="error">无法读取目录: {}</div>"#, e));
        }
    }
    
    html.push_str("</ul></div></body></html>");
    
    Ok(warp_html(html))
}