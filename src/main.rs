mod config;
mod handler;
mod html;
mod utils;

use warp::Filter;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use config::{ensure_config, load_config};
use utils::expand_path;
use std::collections::HashMap;
use crate::handler::{admin_delete, admin_files_list, admin_mkdir, admin_upload};
use handler::{
    exec_command, get_admin_config, get_admin_logs, get_admin_status, handle_admin_page,
    handle_request, is_valid_auth, unauthorized, update_admin_config,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!("nweb v{} - 极简网页文件服务器", VERSION);
    println!();
    println!("用法: nweb [选项] [目录] [端口]");
    println!();
    println!("选项:");
    println!("  -d, --dir <目录>    共享目录 (默认: 当前目录)");
    println!("  -p, --port <端口>   监听端口 (默认: 8080)");
    println!("  -H, --host <地址>   监听地址 (默认: 0.0.0.0)");
    println!("  -h, --help          显示帮助");
    println!("  -v, --version       显示版本");
    println!();
    println!("示例:");
    println!("  nweb -d /data -p 7891 -H 127.0.0.1");
}

fn fail(msg: &str) -> ! {
    eprintln!("❌ {}", msg);
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.is_empty() {
        print_help();
        std::process::exit(1);
    }
    if argv.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        std::process::exit(0);
    }
    if argv.iter().any(|a| a == "--version" || a == "-v") {
        println!("nweb v{}", VERSION);
        std::process::exit(0);
    }

    let mut dir = String::new();
    let mut port: u16 = 0;
    let mut host = String::from("0.0.0.0");
    let mut positional: Vec<String> = Vec::new();

    let mut i = 0;
    while i < argv.len() {
        let arg = argv[i].clone();
        match arg.as_str() {
            "-d" | "--dir" => {
                i += 1;
                dir = match argv.get(i) {
                    Some(v) => v.clone(),
                    None => fail("缺少 --dir 参数"),
                };
            }
            "-p" | "--port" => {
                i += 1;
                port = match argv.get(i).and_then(|v| v.parse::<u16>().ok()) {
                    Some(p) => p,
                    None => fail("端口格式错误"),
                };
            }
            "-H" | "--host" => {
                i += 1;
                host = match argv.get(i) {
                    Some(v) => v.clone(),
                    None => fail("缺少 --host 参数"),
                };
            }
            _ => positional.push(arg.clone()),
        }
        i += 1;
    }

    if dir.is_empty() {
        if let Some(first) = positional.first() {
            dir = first.clone();
        }
    }
    if let Some(second) = positional.get(1) {
        match second.parse::<u16>() {
            Ok(value) => {
                if port == 0 {
                    port = value;
                }
            }
            Err(_) => fail(&format!("端口格式错误: {}", second)),
        }
    }
    if dir.is_empty() {
        dir = String::from(".");
    }
    if port == 0 {
        port = 8080;
    }

    let ip: IpAddr = match host.parse() {
        Ok(value) => value,
        Err(_) => fail(&format!("监听地址无效: {}", host)),
    };

    let root_dir = match std::fs::canonicalize(expand_path(&dir)) {
        Ok(p) if p.is_dir() => p,
        Ok(_) => fail(&format!("不是有效目录: {}", dir)),
        Err(e) => fail(&format!("目录无法访问: {} ({})", dir, e)),
    };

    ensure_config(&root_dir);

    let cfg = match load_config(&root_dir) {
        Some(c) => c,
        None => {
            eprintln!("❌ 配置文件 nweb.yml 无效或不可读，请修复后重试（不会回退默认口令启动）");
            std::process::exit(1);
        }
    };

    if cfg.clear_log_on_start {
        let log_path = root_dir.join("nweb.log");
        if log_path.exists() {
            let _ = std::fs::rename(&log_path, root_dir.join("nweb.log.1"));
        }
    }

    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if cfg.admin_user == "nweb" && cfg.admin_pass == "nweb" {
        println!("⚠️  检测到默认管理员口令 nweb/nweb，请登录后台立即修改");
    }
    if cfg.enable_exec {
        println!("⚠️  命令执行功能已启用 (/@admin/exec)，请确保管理员口令足够强");
    }
    if !cfg.home_dir.trim().is_empty() {
        match std::fs::canonicalize(cfg.home_dir.trim()) {
            Ok(p) if p.is_dir() => println!("📂 home_dir 生效: {}", p.display()),
            _ => println!("⚠️  home_dir 无效: {}，将使用 {}", cfg.home_dir, root_dir.display()),
        }
    }

    let admin_page = warp::path!("@admin")
        .and(warp::get())
        .and(warp::header::optional::<String>("authorization"))
        .and_then({
            let root = root_dir.clone();
            move |auth: Option<String>| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth, true).await {
                        return Ok(unauthorized());
                    }
                    handle_admin_page(root).await
                }
            }
        });

    let admin_config_get = warp::path!("@admin" / "config")
        .and(warp::get())
        .and(warp::header::optional::<String>("authorization"))
        .and_then({
            let root = root_dir.clone();
            move |auth: Option<String>| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth, true).await {
                        return Ok(unauthorized());
                    }
                    get_admin_config(root).await
                }
            }
        });

    let admin_config_post = warp::path!("@admin" / "config")
        .and(warp::post())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::body::json())
        .and_then({
            let root = root_dir.clone();
            move |auth: Option<String>, payload: serde_json::Value| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth, true).await {
                        return Ok(unauthorized());
                    }
                    update_admin_config(root, payload).await
                }
            }
        });

    let admin_logs = warp::path!("@admin" / "logs")
        .and(warp::get())
        .and(warp::header::optional::<String>("authorization"))
        .and_then({
            let root = root_dir.clone();
            move |auth: Option<String>| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth, true).await {
                        return Ok(unauthorized());
                    }
                    get_admin_logs(root).await
                }
            }
        });

    let admin_status = warp::path!("@admin" / "status")
        .and(warp::get())
        .and(warp::header::optional::<String>("authorization"))
        .and_then({
            let root = root_dir.clone();
            move |auth: Option<String>| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth, true).await {
                        return Ok(unauthorized());
                    }
                    get_admin_status(root, start_time).await
                }
            }
        });

    let admin_exec = warp::path!("@admin" / "exec")
        .and(warp::post())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::body::json())
        .and_then({
            let root = root_dir.clone();
            move |auth: Option<String>, payload: serde_json::Value| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth, true).await {
                        return Ok(unauthorized());
                    }
                    let command = payload
                        .get("command")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    exec_command(root, command).await
                }
            }
        });

    let admin_files_route = warp::path!("@admin" / "files")
        .and(warp::get())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::query::<HashMap<String, String>>())
        .and_then({
            let root = root_dir.clone();
            move |auth: Option<String>, query: HashMap<String, String>| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth, true).await {
                        return Ok(unauthorized());
                    }
                    admin_files_list(root, query).await
                }
            }
        });

    let admin_upload_route = warp::path!("@admin" / "upload")
        .and(warp::post())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::query::<HashMap<String, String>>())
        .and(warp::body::content_length_limit(268435456))
        .and(warp::body::bytes())
        .and_then({
            let root = root_dir.clone();
            move |auth: Option<String>,
                  query: HashMap<String, String>,
                  data: warp::hyper::body::Bytes| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth, true).await {
                        return Ok(unauthorized());
                    }
                    admin_upload(root, query, data).await
                }
            }
        });

    let admin_mkdir_route = warp::path!("@admin" / "files" / "mkdir")
        .and(warp::post())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::query::<HashMap<String, String>>())
        .and_then({
            let root = root_dir.clone();
            move |auth: Option<String>, query: HashMap<String, String>| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth, true).await {
                        return Ok(unauthorized());
                    }
                    admin_mkdir(root, query).await
                }
            }
        });

    let admin_delete_route = warp::path!("@admin" / "files" / "delete")
        .and(warp::post())
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::query::<HashMap<String, String>>())
        .and_then({
            let root = root_dir.clone();
            move |auth: Option<String>, query: HashMap<String, String>| {
                let root = root.clone();
                async move {
                    if !is_valid_auth(&root, auth, true).await {
                        return Ok(unauthorized());
                    }
                    admin_delete(root, query).await
                }
            }
        });
    let main_routes = warp::path::tail()
        .and(warp::path::full())
        .and(warp::addr::remote())
        .and_then({
            let root = root_dir.clone();
            move |tail: warp::path::Tail,
                  full: warp::path::FullPath,
                  remote: Option<std::net::SocketAddr>| {
                let root = root.clone();
                async move { handle_request(&root, tail.as_str(), full.as_str(), remote).await }
            }
        });

    let routes = admin_page
        .or(admin_config_get)
        .or(admin_config_post)
        .or(admin_logs)
        .or(admin_status)
        .or(admin_exec)
        .or(admin_files_route)
        .or(admin_upload_route)
        .or(admin_mkdir_route)
        .or(admin_delete_route)
        .or(main_routes);

    println!("🚀 nweb v{} 已启动", VERSION);
    println!("📍 监听: http://{}:{}", ip, port);
    println!("📂 共享根目录: {}", root_dir.display());
    println!("👤 管理后台: /@admin （用户名: {}）", cfg.admin_user);
    println!("🔐 配置文件: {}", root_dir.join("nweb.yml").display());

    warp::serve(routes).run((ip, port)).await;
}
