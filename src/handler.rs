use warp::http::{Response, StatusCode};
use std::path::PathBuf;
use std::fs;
use urlencoding::decode;
use chrono::Local;
use tokio::fs as tokio_fs;
use tokio::io::AsyncWriteExt;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::config::{Config, load_config, save_config};
use crate::html::home::generate_index_html;
use crate::html::admin::generate_admin_html;
use sysinfo::{System, Pid};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::process::Command;

/// 认证成功日志标志（只打印一次）
static AUTH_LOGGED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, serde::Serialize)]
struct FileNode {
    name: String,
    is_dir: bool,
    size: u64,
    path: String,
    modified_time: u64,
}

/// 日志记录（过滤 favicon.ico / sw.js 404）
pub async fn log_request(root: &PathBuf, path: &str, status: u16, client_ip: &str) {
    if (path == "favicon.ico" || path == "sw.js") && status == 404 {
        return;
    }
    let time = Local::now().format("%H:%M:%S");
    let status_str = match status {
        200 => "✓",
        201 => "✓",
        204 => "✓",
        301 => "↷",
        302 => "↷",
        304 => "↷",
        400 => "⚠",
        401 => "⚠",
        403 => "✗",
        404 => "✗",
        500 => "✗",
        _ => "•",
    };
    let msg = format!("[{}] {} {} {} {}\n", time, status_str, status, client_ip, path);
    print!("{}", msg);

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

/// ★ 目录树生成（核心修复）
/// 
/// 关键点：
/// - 使用请求传入的 path 拼接相对路径，不依赖 strip_prefix
/// - 彻底避免符号链接/挂载点/相对路径导致的路径错乱
fn get_directory_tree(root: &PathBuf, path: &str, config: &Config) -> Result<Vec<FileNode>, String> {
    // 严格清理路径：去掉首尾斜杠，防止绝对路径注入
    let path = path.trim_matches('/');

    let current_path = if path.is_empty() {
        root.clone()
    } else {
        root.join(path)
    };

    // 安全检查
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
        let name = entry.file_name().to_string_lossy().to_string();

        // 隐藏文件过滤
        if name == "nweb.yml" || name == "nweb.log" {
            continue;
        }
        if !config.show_hidden && name.starts_with('.') {
            continue;
        }

        let entry_path = entry.path();
        let is_dir = entry_path.is_dir();
        let size = if is_dir {
            0
        } else {
            fs::metadata(&entry_path).map(|m| m.len()).unwrap_or(0)
        };

        let modified_time = fs::metadata(&entry_path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // ★ 核心修复：使用请求的 path 拼接，确保路径始终是相对路径
        let relative_path = if path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", path, name)
        };

        nodes.push(FileNode {
            name,
            is_dir,
            size,
            path: relative_path,
            modified_time,
        });
    }

    // 排序（文件夹优先）
    match config.default_sort.as_str() {
        "size" => {
            nodes.sort_by(|a, b| {
                if a.is_dir != b.is_dir {
                    return if a.is_dir { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
                }
                a.size.cmp(&b.size)
            });
        }
        "time" => {
            nodes.sort_by(|a, b| {
                if a.is_dir != b.is_dir {
                    return if a.is_dir { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
                }
                b.modified_time.cmp(&a.modified_time)
            });
        }
        _ => {
            nodes.sort_by(|a, b| {
                if a.is_dir != b.is_dir {
                    return if a.is_dir { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
                }
                a.name.cmp(&b.name)
            });
        }
    }

    Ok(nodes)
}

/// 主请求处理（永不返回 401）
pub async fn handle_request(
    root: &PathBuf,
    tail: &str,
    _full_path: &str,
    remote_addr: Option<std::net::SocketAddr>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let config = load_config(root).unwrap_or_else(Config::default);
    let client_ip = remote_addr
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // ========== 处理 home_dir ==========
    let effective_root = if !config.home_dir.is_empty() {
        let home_path = PathBuf::from(&config.home_dir);
        if home_path.exists() && home_path.is_dir() {
            home_path
        } else {
            root.clone()
        }
    } else {
        root.clone()
    };

    // 隐藏敏感文件
    if tail == "nweb.yml" || tail == "nweb.log" {
        log_request(root, tail, 404, &client_ip).await;
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(b"404 Not Found".to_vec())
            .unwrap());
    }

    // ========== API: /api/tree/xxx ==========
    if tail.starts_with("api/tree/") {
        let raw_path = &tail[9..];
        let decoded = decode(raw_path).unwrap_or_else(|_| raw_path.into());
        // 严格清理路径
        let path = decoded
            .trim_start_matches('/')
            .trim_start_matches("./")
            .trim_end_matches('/')
            .to_string();

        match get_directory_tree(&effective_root, &path, &config) {
            Ok(tree) => {
                let json = serde_json::to_string(&tree).unwrap_or("[]".to_string());
                log_request(root, &format!("/api/tree/{}", path), 200, &client_ip).await;
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(json.into_bytes())
                    .unwrap());
            }
            Err(e) => {
                log_request(root, &format!("/api/tree/{}", path), 404, &client_ip).await;
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(format!("Error: {}", e).into_bytes())
                    .unwrap());
            }
        }
    }

    // ========== API: /api/xxx ==========
    if tail.starts_with("api/") {
        let raw_path = &tail[4..];
        let decoded = decode(raw_path).unwrap_or_else(|_| raw_path.into());
        let path = decoded
            .trim_start_matches('/')
            .trim_start_matches("./")
            .trim_end_matches('/')
            .to_string();

        match get_directory_tree(&effective_root, &path, &config) {
            Ok(tree) => {
                let json = serde_json::to_string(&tree).unwrap_or("[]".to_string());
                log_request(root, &format!("/api/{}", path), 200, &client_ip).await;
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(json.into_bytes())
                    .unwrap());
            }
            Err(e) => {
                log_request(root, &format!("/api/{}", path), 404, &client_ip).await;
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(format!("Error: {}", e).into_bytes())
                    .unwrap());
            }
        }
    }

    // ========== 根路径：优先 index_file ==========
    if tail.is_empty() {
        let index_path = effective_root.join(&config.index_file);
        if index_path.exists() && index_path.is_file() {
            match tokio_fs::read_to_string(&index_path).await {
                Ok(content) => {
                    log_request(root, "/", 200, &client_ip).await;
                    return Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(content.into_bytes())
                        .unwrap());
                }
                Err(_) => {
                    log_request(root, "/", 500, &client_ip).await;
                    let html = generate_index_html(&config);
                    return Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(html.into_bytes())
                        .unwrap());
                }
            }
        } else {
            let html = generate_index_html(&config);
            log_request(root, "/", 200, &client_ip).await;
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(html.into_bytes())
                .unwrap());
        }
    }

    // ========== 文件下载 ==========
    let decoded_tail = decode(tail).unwrap_or_else(|_| tail.into());
    // ★ 关键修复：严格清理路径，防止绝对路径
    let decoded_tail = decoded_tail
        .trim_start_matches('/')
        .trim_start_matches("./")
        .to_string();
    let fs_path = effective_root.join(&decoded_tail);

    if !fs_path.starts_with(&effective_root) {
        log_request(root, &decoded_tail, 403, &client_ip).await;
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("雜鱼，禁止访问".as_bytes().to_vec())
            .unwrap());
    }

    if fs_path.is_file() {
        match tokio_fs::read(&fs_path).await {
            Ok(data) => {
                let mime = mime_guess::from_path(&fs_path).first_or_octet_stream();
                log_request(root, &decoded_tail, 200, &client_ip).await;
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", mime.as_ref())
                    .body(data)
                    .unwrap());
            }
            Err(_) => {
                log_request(root, &decoded_tail, 500, &client_ip).await;
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("雜鱼，文件读取失败".as_bytes().to_vec())
                    .unwrap());
            }
        }
    }

    log_request(root, tail, 404, &client_ip).await;
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(b"404 Not Found".to_vec())
        .unwrap())
}

/// ==================== 管理后台 ====================

/// 验证 Basic Auth（只在管理接口打印日志，成功只打印一次）
pub async fn is_valid_auth(root: &PathBuf, auth_header: Option<String>, is_admin: bool) -> bool {
    match auth_header {
        Some(header) => {
            if !header.starts_with("Basic ") {
                return false;
            }
            let encoded = &header[6..];
            let decoded = match STANDARD.decode(encoded) {
                Ok(d) => d,
                Err(_) => return false,
            };
            let decoded_str = match String::from_utf8(decoded) {
                Ok(s) => s,
                Err(_) => return false,
            };
            let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
            if parts.len() != 2 {
                return false;
            }
            let config = load_config(root).unwrap_or_else(Config::default);
            let valid = parts[0] == config.admin_user && parts[1] == config.admin_pass;

            if is_admin {
                if valid {
                    if !AUTH_LOGGED.load(Ordering::Relaxed) {
                        println!("[AUTH] ✅ 认证成功");
                        AUTH_LOGGED.store(true, Ordering::Relaxed);
                    }
                } else {
                    println!("[AUTH] ❌ 认证失败");
                }
            }
            valid
        }
        None => false,
    }
}

/// GET /@admin - 管理页面
pub async fn handle_admin_page(root: PathBuf) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let config = load_config(&root).unwrap_or_else(Config::default);
    let html = generate_admin_html(&config);
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(html.into_bytes())
        .unwrap())
}

/// GET /@admin/config
pub async fn get_admin_config(root: PathBuf) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let config = load_config(&root).unwrap_or_else(Config::default);
    let json = serde_json::to_string(&config).unwrap_or("{}".to_string());
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(json.into_bytes())
        .unwrap())
}

/// POST /@admin/config
pub async fn update_admin_config(
    root: PathBuf,
    body: serde_json::Value,
) -> Result<Response<Vec<u8>>, warp::Rejection> {
    println!("[ADMIN] 配置文件更新请求");
    let new_config: Config = match serde_json::from_value::<Config>(body) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ADMIN] ❌ 配置解析失败: {}", e);
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("配置格式错误: {}", e).into_bytes())
                .unwrap());
        }
    };
    match save_config(&root, &new_config) {
        Ok(_) => match load_config(&root) {
            Some(_) => {
                println!("[ADMIN] ✅ 配置已更新");
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(b"Config updated successfully".to_vec())
                    .unwrap())
            }
            None => {
                println!("[ADMIN] ❌ 配置保存成功但加载失败");
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(b"Config saved but load failed".to_vec())
                    .unwrap())
            }
        },
        Err(e) => {
            eprintln!("[ADMIN] ❌ 配置更新失败: {}", e);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("保存失败: {}", e).into_bytes())
                .unwrap())
        }
    }
}

/// GET /@admin/logs
pub async fn get_admin_logs(root: PathBuf) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let log_path = root.join("nweb.log");
    if !log_path.exists() {
        let msg = json!({ "logs": "暂无日志文件" }).to_string();
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(msg.into_bytes())
            .unwrap());
    }
    match tokio_fs::read_to_string(&log_path).await {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let last_lines = if lines.len() > 100 {
                &lines[lines.len() - 100..]
            } else {
                &lines
            };
            let logs = last_lines.join("\n");
            let msg = json!({ "logs": logs }).to_string();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(msg.into_bytes())
                .unwrap())
        }
        Err(_) => {
            let msg = json!({ "logs": "无法读取日志文件" }).to_string();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(msg.into_bytes())
                .unwrap())
        }
    }
}

/// GET /@admin/status
pub async fn get_admin_status(_root: PathBuf, start_time: u64) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let mut sys = System::new();
    sys.refresh_all();
    let pid = std::process::id();
    let process = sys.process(Pid::from_u32(pid));
    let memory = process.map(|p| p.memory() / 1024).unwrap_or(0);
    let cpu = process.map(|p| p.cpu_usage()).unwrap_or(0.0);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let uptime = now.saturating_sub(start_time);

    let status = json!({
        "pid": pid,
        "memory": memory,
        "cpu": cpu,
        "uptime": uptime,
    });
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(status.to_string().into_bytes())
        .unwrap())
}

/// POST /@admin/exec - 执行命令
pub async fn exec_command(
    root: PathBuf,
    body: serde_json::Value,
) -> Result<Response<Vec<u8>>, warp::Rejection> {
    let cmd = match body.get("cmd").and_then(|v| v.as_str()) {
        Some(c) => c.trim(),
        None => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(b"Missing 'cmd' field".to_vec())
                .unwrap());
        }
    };

    if cmd.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(b"Empty command".to_vec())
            .unwrap());
    }

    let log_msg = format!("[EXEC] 管理员: {}\n", cmd);
    println!("{}", log_msg.trim());
    let log_path = root.join("nweb.log");
    if let Ok(mut file) = tokio_fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&log_path)
        .await
    {
        let _ = file.write_all(log_msg.as_bytes()).await;
    }

    let cmd_owned = cmd.to_string();
    let result = tokio::task::spawn_blocking(move || {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", &cmd_owned]).output()
        } else {
            Command::new("sh").args(["-c", &cmd_owned]).output()
        };

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let exit_code = out.status.code().unwrap_or(-1);
                (exit_code, stdout, stderr)
            }
            Err(e) => (-1, String::new(), format!("执行失败: {}", e)),
        }
    })
    .await
    .unwrap_or((-1, String::new(), "任务 panic".to_string()));

    let (exit_code, stdout, stderr) = result;
    let response = json!({
        "exit_code": exit_code,
        "stdout": stdout,
        "stderr": stderr,
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(response.to_string().into_bytes())
        .unwrap())
}