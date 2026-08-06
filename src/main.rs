mod config;
mod handler;
mod html;
mod utils;

use clap::Parser;
use warp::Filter;
use warp::http::{Response, StatusCode};
use std::path::PathBuf;
use std::net::IpAddr;
use config::{ensure_config, load_config, Config};
use utils::expand_path;
use handler::{handle_request, is_valid_auth, handle_admin_page, get_admin_config, update_admin_config, get_admin_logs, get_admin_status};

/// nweb - Rust 文件服务器
#[derive(Parser, Debug)]
#[command(name = "nweb")]
#[command(version = "0.5.0")]
#[command(author = "NAS油条")]
#[command(about = "一个用 Rust 编写的轻量级文件服务器", long_about = None)]
#[command(disable_help_flag = true)]
#[command(disable_version_flag = true)]
struct Cli {
    #[arg(short = 'd', long, default_value = "")]
    dir: String,
    #[arg(short = 'p', long, default_value_t = 0)]
    port: u16,
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    host: String,
    #[arg(short, long)]
    help: bool,
    #[arg(short, long)]
    version: bool,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        eprintln!("❌ 参数错了，雜鱼🐟");
        eprintln!("💡 输入 nweb -h 查看帮助");
        eprintln!("💡 食用方法: nweb <目录> <端口>");
        eprintln!("💡 示例: nweb ~/文档 7891");
        eprintln!("💡 如果想使用当前目录，请换成 nweb . <端口>");
        eprintln!();
        eprintln!("  也可以使用参数形式:");
        eprintln!("  nweb -d <目录> -p <端口> -H <主机IP>");
        eprintln!();
        eprintln!("  其他选项:");
        eprintln!("  -h, --help     显示帮助信息");
        eprintln!("  -v, --version  显示版本号");
        std::process::exit(1);
    }

    if args.iter().any(|a| a == "-h" || a == "--help") {
        eprintln!("nweb - Rust 文件服务器 v1.0");
        eprintln!("");
        eprintln!("用法:");
        eprintln!("  nweb <目录> <端口>");
        eprintln!("  nweb [选项]");
        eprintln!("");
        eprintln!("选项:");
        eprintln!("  -d, --dir <DIR>     服务目录路径 (默认: .)");
        eprintln!("  -p, --port <PORT>   服务端口 (默认: 8080)");
        eprintln!("  -H, --host <HOST>   绑定主机IP (默认: 0.0.0.0)");
        eprintln!("  -h, --help          显示帮助信息");
        eprintln!("  -v, --version       显示版本号");
        eprintln!("");
        eprintln!("示例:");
        eprintln!("  nweb . 7891");
        eprintln!("  nweb ~/Documents 8080");
        eprintln!("  nweb -d ~/Documents -p 8080");
        eprintln!("  nweb -d ~/Documents -p 8080 -H 127.0.0.1");
        std::process::exit(0);
    }

    if args.iter().any(|a| a == "-v" || a == "--version") {
        eprintln!("nweb v1.0.0");
        eprintln!("🤓 NAS油条 出品");
        std::process::exit(0);
    }

    let (dir, port, host) = if args.len() == 3 {
        let dir = args[1].clone();
        let port = match args[2].parse::<u16>() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("❌ 端口格式错误: {}", args[2]);
                eprintln!("💡 输入 nweb -h 查看帮助");
                std::process::exit(1);
            }
        };
        (dir, port, "0.0.0.0".to_string())
    } else {
        let cli = Cli::parse();
        if cli.port == 0 && !cli.dir.is_empty() {
            eprintln!("❌ 参数错了，雜鱼🐟");
            eprintln!("💡 输入 nweb -h 查看帮助");
            eprintln!("💡 食用方法: nweb <目录> <端口>");
            std::process::exit(1);
        }
        let dir = if cli.dir.is_empty() { ".".to_string() } else { cli.dir };
        let port = if cli.port == 0 { 8080 } else { cli.port };
        (dir, port, cli.host)
    };

    let expanded_dir = expand_path(&dir);
    let root_dir = PathBuf::from(&expanded_dir);
    let ip: IpAddr = host.parse().unwrap_or([0, 0, 0, 0].into());

    if !root_dir.exists() || !root_dir.is_dir() {
        eprintln!("❌ 目录不存在: {}", root_dir.display());
        std::process::exit(1);
    }

    ensure_config(&root_dir);
    let config = load_config(&root_dir).unwrap_or_else(Config::default);

    // 清理日志
    let log_path = root_dir.join("nweb.log");
    if config.clear_log_on_start {
        if log_path.exists() {
            let _ = std::fs::remove_file(&log_path);
        }
        let _ = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&log_path)
            .await;
    }

    // ============================================
    // 启动信息
    // ============================================
    println!("╔══════════════════════════════════╗");
    println!("║  🦀 nweb - Rust 文件服务器       ║");
    println!("║  🤓 NAS油条 出品                 ║");
    println!("╚══════════════════════════════════╝");
    println!();
    println!("📁 服务目录: {}", root_dir.display());
    println!("🌐 访问地址: http://{}:{}", host, port);
    println!("🔨️ 管理后台: http://{}:{}/@admin", host, port);
    println!("📄 日志文件: {}", log_path.display());
    println!();
    println!("🔐 管理后台认证信息:");
    println!("   用户名: {}", config.admin_user);
    println!("   密码:   {}", config.admin_pass);
    println!();
    println!("🐧 问题反馈: QQ群 610699712");
    println!();
    println!("💡 按 Ctrl+C 停止服务");


    // 1. 主路由 获取客户端 IP

    let main_routes = warp::any()
        .and(warp::path::tail())
        .and(warp::path::full())
        .and(warp::addr::remote())
        .and_then({
            let root = root_dir.clone();
            move |tail: warp::path::Tail, full: warp::path::FullPath, remote_addr: Option<std::net::SocketAddr>| {
                let root = root.clone();
                async move {
                    handle_request(&root, tail.as_str(), full.as_str(), remote_addr).await
                }
            }
        });

    // 2. 管理路由 认证 /@admin

    // 管理页面 - /@admin
    let admin_page = warp::path!("@admin")
        .and(warp::get())
        .and(warp::header::optional::<String>("authorization"))
        .and_then({
            let root = root_dir.clone();
            move |auth_header: Option<String>| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth_header, true).await {
                        return Ok(Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .header("WWW-Authenticate", "Basic realm=\"nweb admin\"")
                            .body(b"Unauthorized".to_vec())
                            .unwrap());
                    }
                    handle_admin_page(root).await
                }
            }
        });

    // GET /@admin/config
    let admin_config_get = warp::path!("@admin" / "config")
        .and(warp::get())
        .and(warp::header::optional::<String>("authorization"))
        .and_then({
            let root = root_dir.clone();
            move |auth_header: Option<String>| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth_header, true).await {
                        return Ok(Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .header("WWW-Authenticate", "Basic realm=\"nweb admin\"")
                            .body(b"Unauthorized".to_vec())
                            .unwrap());
                    }
                    get_admin_config(root).await
                }
            }
        });

    // POST /@admin/config
    let admin_config_post = warp::path!("@admin" / "config")
        .and(warp::post())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::body::json())
        .and_then({
            let root = root_dir.clone();
            move |auth_header: Option<String>, body: serde_json::Value| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth_header, true).await {
                        return Ok(Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .header("WWW-Authenticate", "Basic realm=\"nweb admin\"")
                            .body(b"Unauthorized".to_vec())
                            .unwrap());
                    }
                    update_admin_config(root, body).await
                }
            }
        });

    // GET /@admin/logs
    let admin_logs = warp::path!("@admin" / "logs")
        .and(warp::get())
        .and(warp::header::optional::<String>("authorization"))
        .and_then({
            let root = root_dir.clone();
            move |auth_header: Option<String>| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth_header, true).await {
                        return Ok(Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .header("WWW-Authenticate", "Basic realm=\"nweb admin\"")
                            .body(b"Unauthorized".to_vec())
                            .unwrap());
                    }
                    get_admin_logs(root).await
                }
            }
        });

    // GET /@admin/status
    let admin_status = warp::path!("@admin" / "status")
        .and(warp::get())
        .and(warp::header::optional::<String>("authorization"))
        .and_then({
            let root = root_dir.clone();
            move |auth_header: Option<String>| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth_header, true).await {
                        return Ok(Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .header("WWW-Authenticate", "Basic realm=\"nweb admin\"")
                            .body(b"Unauthorized".to_vec())
                            .unwrap());
                    }
                    get_admin_status(root).await
                }
            }
        });

    // 3. 组合设置
    let routes = admin_page
        .or(admin_config_get)
        .or(admin_config_post)
        .or(admin_logs)
        .or(admin_status)
        .or(main_routes);

    warp::serve(routes)
        .run((ip, port))
        .await;
}