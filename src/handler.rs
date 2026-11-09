use warp::http::{Response, StatusCode};
use warp::hyper::body::{Body, Bytes};
use std::path::{Component, Path, PathBuf};
use std::fs;
use urlencoding::decode;
use chrono::Local;
use tokio::fs as tokio_fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use crate::config::{Config, load_config, save_config};
use crate::html::home::generate_index_html;
use crate::html::admin::generate_admin_html;
use sysinfo::{Pid, System};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::process::Command;
use std::collections::HashMap;

pub type ApiResponse = Response<Body>;

const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024;
const STREAM_BUF: usize = 64 * 1024;

static AUTH_LOGGED: AtomicBool = AtomicBool::new(false);
static AUTH_FAILS: AtomicU32 = AtomicU32::new(0);
static SYS: Mutex<Option<System>> = Mutex::new(None);

#[derive(Debug, Clone, serde::Serialize)]
struct FileNode {
    name: String,
    is_dir: bool,
    size: u64,
    path: String,
    modified_time: u64,
}

fn build_response(status: StatusCode, content_type: Option<&str>, body: Bytes) -> ApiResponse {
    let mut builder = Response::builder().status(status);
    if let Some(ct) = content_type {
        builder = builder.header("Content-Type", ct);
    }
    builder
        .body(Body::from(body))
        .expect("response build failed")
}

pub fn text(status: StatusCode, body: String) -> ApiResponse {
    build_response(status, Some("text/plain; charset=utf-8"), Bytes::from(body.into_bytes()))
}

pub fn html(status: StatusCode, body: String) -> ApiResponse {
    build_response(status, Some("text/html; charset=utf-8"), Bytes::from(body.into_bytes()))
}

pub fn json_bytes(body: String) -> ApiResponse {
    build_response(StatusCode::OK, Some("application/json; charset=utf-8"), Bytes::from(body.into_bytes()))
}

pub fn unauthorized() -> ApiResponse {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("WWW-Authenticate", "Basic realm=nweb")
        .body(Body::from(Bytes::from_static(b"Unauthorized")))
        .expect("response build failed")
}

enum PathError {
    Traversal,
    NotFound,
}

fn safe_relative(rel: &str) -> Option<PathBuf> {
    if rel.as_bytes().contains(&0u8) {
        return None;
    }
    let mut out = PathBuf::new();
    for comp in Path::new(rel).components() {
        match comp {
            Component::Normal(c) => out.push(c),
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(out)
}

fn resolve_inside(root: &Path, rel: &str) -> Result<PathBuf, PathError> {
    let rel_path = match safe_relative(rel) {
        Some(p) => p,
        None => return Err(PathError::Traversal),
    };
    let canon_root = fs::canonicalize(root).map_err(|_| PathError::NotFound)?;
    let canon = match fs::canonicalize(canon_root.join(rel_path)) {
        Ok(p) => p,
        Err(_) => return Err(PathError::NotFound),
    };
    if canon == canon_root || canon.starts_with(&canon_root) {
        Ok(canon)
    } else {
        Err(PathError::Traversal)
    }
}

fn is_sensitive(rel: &str) -> bool {
    for comp in Path::new(rel).components() {
        if let Component::Normal(name) = comp {
            let n = name.to_string_lossy().to_lowercase();
            if n == "nweb.yml" || n == "nweb.yml.tmp" || n == "nweb.log" || n == "nweb.log.1" {
                return true;
            }
        }
    }
    false
}

fn effective_root(root: &PathBuf, config: &Config) -> PathBuf {
    let configured = config.home_dir.trim();
    if configured.is_empty() {
        return root.clone();
    }
    match fs::canonicalize(configured) {
        Ok(p) if p.is_dir() => {
            if p.parent().is_none() {
                eprintln!("⚠️  home_dir 不能是文件系统根目录，已忽略该设置");
                return root.clone();
            }
            p
        }
        _ => {
            eprintln!("⚠️  home_dir 无效: {}，改用共享根目录", configured);
            root.clone()
        }
    }
}

fn decode_path(raw: &str) -> String {
    match decode(raw) {
        Ok(c) => c.to_string(),
        Err(_) => raw.to_string(),
    }
}

fn get_directory_tree(root: &Path, path: &str, config: &Config) -> Result<Vec<FileNode>, String> {
    let current = match resolve_inside(root, path) {
        Ok(p) => p,
        Err(PathError::Traversal) => return Err(String::from("禁止访问")),
        Err(PathError::NotFound) => return Err(String::from("目录不存在")),
    };
    if !current.is_dir() {
        return Err(String::from("目录不存在"));
    }
    let entries = match fs::read_dir(&current) {
        Ok(e) => e,
        Err(_) => return Err(String::from("目录不可读")),
    };
    let parent = path.trim_start_matches("/").trim_end_matches("/");
    let mut nodes: Vec<FileNode> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let lowered = name.to_lowercase();
        if lowered == "nweb.yml" || lowered == "nweb.yml.tmp" || lowered == "nweb.log" || lowered == "nweb.log.1" {
            continue;
        }
        if !config.show_hidden && name.starts_with(".") {
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let modified_time = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let child_rel = if parent.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", parent, name)
        };
        nodes.push(FileNode {
            name,
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            path: child_rel,
            modified_time,
        });
    }
    nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(nodes)
}

fn stream_file(path: PathBuf) -> Body {
    let (mut sender, body) = Body::channel();
    tokio::spawn(async move {
        if let Ok(mut file) = tokio_fs::File::open(&path).await {
            let mut buffer = vec![0u8; STREAM_BUF];
            loop {
                match file.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if sender.send_data(Bytes::copy_from_slice(&buffer[..n])).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    });
    body
}

pub async fn log_request(root: &PathBuf, path: &str, status: u16, client_ip: &str) {
    if (path == "favicon.ico" || path == "sw.js") && status == 404 {
        return;
    }
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let level = match status {
        s if s >= 500 => "ERROR",
        s if s >= 400 => "WARN ",
        _ => "INFO ",
    };
    let mut msg = format!("[{}] [{}] [{}] /{}", timestamp, level, client_ip, path);
    msg.push(char::from(10u8));
    print!("{}", msg);
    let log_path = root.join("nweb.log");
    if let Ok(meta) = tokio_fs::metadata(&log_path).await {
        if meta.len() > MAX_LOG_SIZE {
            let _ = tokio_fs::rename(&log_path, root.join("nweb.log.1")).await;
        }
    }
    if let Ok(mut file) = tokio_fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .await
    {
        let _ = file.write_all(msg.as_bytes()).await;
    }
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    let left = a.as_bytes();
    let right = b.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut diff: u8 = 0;
    let mut i = 0;
    while i < left.len() {
        diff |= left[i] ^ right[i];
        i += 1;
    }
    diff == 0
}

pub async fn is_valid_auth(root: &PathBuf, auth_header: Option<String>, is_admin: bool) -> bool {
    let header = match auth_header {
        Some(h) => h,
        None => return false,
    };
    if !header.starts_with("Basic ") {
        return false;
    }
    let encoded = header.trim_start_matches("Basic ").trim();
    let raw = match STANDARD.decode(encoded) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let text_value = match String::from_utf8(raw) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let parts: Vec<&str> = text_value.splitn(2, ":").collect();
    if parts.len() != 2 {
        return false;
    }
    let config = match load_config(root) {
        Some(c) => c,
        None => return false,
    };
    let user_ok = constant_time_eq(parts[0], &config.admin_user);
    let pass_ok = constant_time_eq(parts[1], &config.admin_pass);
    if user_ok && pass_ok {
        AUTH_FAILS.store(0, Ordering::Relaxed);
        if is_admin && !AUTH_LOGGED.swap(true, Ordering::Relaxed) {
            println!("[AUTH] ✅ 管理员认证成功");
        }
        return true;
    }
    let fails = AUTH_FAILS.fetch_add(1, Ordering::Relaxed) + 1;
    if fails % 10 == 1 {
        println!("[AUTH] ❌ 认证失败，累计 {} 次", fails);
    }
    if fails >= 10 {
        let delay = std::cmp::min(fails, 30) as u64 * 100;
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }
    false
}

// ================= 后台文件管理（仅登录后可用） =================

fn sanitize_css_len(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .filter(|c| {
            c.is_ascii_digit()
                || c.is_ascii_alphabetic()
                || *c == char::from(46u8)
                || *c == char::from(37u8)
        })
        .take(12)
        .collect();
    if cleaned.is_empty() {
        String::from("0px")
    } else {
        cleaned
    }
}
fn admin_base(root: &PathBuf) -> PathBuf {
    match load_config(root) {
        Some(config) => effective_root(root, &config),
        None => root.clone(),
    }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains("/")
        && !name.contains(char::from(92))
        && !name.as_bytes().contains(&0u8)
}

fn clean_rel(raw: &str) -> String {
    raw.trim_matches(|c| c == char::from(47u8)).to_string()
}

/// 列出目录内容（管理员）
pub async fn admin_files_list(
    root: PathBuf,
    query: HashMap<String, String>,
) -> Result<ApiResponse, warp::Rejection> {
    struct Item {
        name: String,
        path: String,
        is_dir: bool,
        size: u64,
        modified: u64,
        sensitive: bool,
    }

    let base = admin_base(&root);
    let rel = clean_rel(query.get("path").map(|s| s.as_str()).unwrap_or(""));
    let dir = match resolve_inside(&base, &rel) {
        Ok(p) => p,
        Err(PathError::Traversal) => {
            return Ok(json_bytes(json!({"error": "禁止访问该路径"}).to_string()))
        }
        Err(PathError::NotFound) => {
            return Ok(json_bytes(json!({"error": "目录不存在"}).to_string()))
        }
    };
    if !dir.is_dir() {
        return Ok(json_bytes(json!({"error": "目标不是目录"}).to_string()));
    }
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) => {
            return Ok(json_bytes(
                json!({"error": format!("目录不可读: {}", e)}).to_string(),
            ))
        }
    };

    let mut items: Vec<Item> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let path = if rel.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", rel, name)
        };
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        items.push(Item {
            sensitive: is_sensitive(&path),
            name,
            path,
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified,
        });
    }
    items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    let entries_json: Vec<serde_json::Value> = items
        .iter()
        .map(|i| {
            json!({
                "name": i.name,
                "path": i.path,
                "is_dir": i.is_dir,
                "size": i.size,
                "modified": i.modified,
                "sensitive": i.sensitive,
            })
        })
        .collect();

    Ok(json_bytes(
        json!({ "path": rel, "entries": entries_json }).to_string(),
    ))
}

/// 上传文件（请求体为文件内容，文件名走 query）
pub async fn admin_upload(
    root: PathBuf,
    query: HashMap<String, String>,
    data: Bytes,
) -> Result<ApiResponse, warp::Rejection> {
    let base = admin_base(&root);
    let dir_rel = clean_rel(query.get("dir").map(|s| s.as_str()).unwrap_or(""));
    let name = query.get("name").cloned().unwrap_or_default();
    let overwrite = matches!(
        query.get("overwrite").map(|s| s.as_str()),
        Some("1") | Some("true")
    );
    if !valid_name(&name) {
        return Ok(json_bytes(json!({"error": "文件名不合法"}).to_string()));
    }
    let dir = match resolve_inside(&base, &dir_rel) {
        Ok(p) => p,
        Err(_) => return Ok(json_bytes(json!({"error": "目标目录不存在"}).to_string())),
    };
    if !dir.is_dir() {
        return Ok(json_bytes(json!({"error": "目标不是目录"}).to_string()));
    }
    let rel = if dir_rel.is_empty() {
        name.clone()
    } else {
        format!("{}/{}", dir_rel, name)
    };
    if is_sensitive(&rel) {
        return Ok(json_bytes(
            json!({"error": "该文件名受保护，禁止写入"}).to_string(),
        ));
    }
    let target = dir.join(&name);
    if target.is_dir() {
        return Ok(json_bytes(json!({"error": "已存在同名文件夹"}).to_string()));
    }
    if target.exists() && !overwrite {
        return Ok(json_bytes(json!({"error": "文件已存在"}).to_string()));
    }
    let size = data.len();
    match tokio_fs::write(&target, &data).await {
        Ok(_) => {
            println!("[FILES] ⬆ 上传 {} ({} 字节)", target.display(), size);
            Ok(json_bytes(
                json!({"status": "success", "name": name, "size": size}).to_string(),
            ))
        }
        Err(e) => Ok(json_bytes(
            json!({"error": format!("写入失败: {}", e)}).to_string(),
        )),
    }
}

/// 新建文件夹
pub async fn admin_mkdir(
    root: PathBuf,
    query: HashMap<String, String>,
) -> Result<ApiResponse, warp::Rejection> {
    let base = admin_base(&root);
    let dir_rel = clean_rel(query.get("path").map(|s| s.as_str()).unwrap_or(""));
    let name = query.get("name").cloned().unwrap_or_default();
    if !valid_name(&name) {
        return Ok(json_bytes(json!({"error": "文件夹名称不合法"}).to_string()));
    }
    let dir = match resolve_inside(&base, &dir_rel) {
        Ok(p) => p,
        Err(_) => return Ok(json_bytes(json!({"error": "目标目录不存在"}).to_string())),
    };
    if !dir.is_dir() {
        return Ok(json_bytes(json!({"error": "目标不是目录"}).to_string()));
    }
    let target = dir.join(&name);
    if target.exists() {
        return Ok(json_bytes(
            json!({"error": "已存在同名文件或文件夹"}).to_string(),
        ));
    }
    match fs::create_dir(&target) {
        Ok(_) => {
            println!("[FILES] 📁 新建文件夹 {}", target.display());
            Ok(json_bytes(json!({"status": "success", "name": name}).to_string()))
        }
        Err(e) => Ok(json_bytes(
            json!({"error": format!("创建失败: {}", e)}).to_string(),
        )),
    }
}

/// 删除文件或空文件夹
pub async fn admin_delete(
    root: PathBuf,
    query: HashMap<String, String>,
) -> Result<ApiResponse, warp::Rejection> {
    let base = admin_base(&root);
    let rel = clean_rel(query.get("path").map(|s| s.as_str()).unwrap_or(""));
    if rel.is_empty() {
        return Ok(json_bytes(json!({"error": "不能删除根目录"}).to_string()));
    }
    if is_sensitive(&rel) {
        return Ok(json_bytes(
            json!({"error": "该文件受保护，禁止删除"}).to_string(),
        ));
    }
    let target = match resolve_inside(&base, &rel) {
        Ok(p) => p,
        Err(PathError::Traversal) => {
            return Ok(json_bytes(json!({"error": "禁止访问该路径"}).to_string()))
        }
        Err(PathError::NotFound) => {
            return Ok(json_bytes(json!({"error": "文件不存在"}).to_string()))
        }
    };
    let is_dir = target.is_dir();
    let result = if is_dir {
        fs::remove_dir_all(&target)
    } else {
        fs::remove_file(&target)
    };
    match result {
        Ok(_) => {
            println!("[FILES] 🗑 删除 {}", target.display());
            Ok(json_bytes(json!({"status": "success"}).to_string()))
        }
        Err(e) => Ok(json_bytes(
            json!({"error": format!("删除失败: {}", e)}).to_string(),
        )),
    }
}

pub async fn handle_request(
    root: &PathBuf,
    tail: &str,
    _full_path: &str,
    remote_addr: Option<std::net::SocketAddr>,
) -> Result<ApiResponse, warp::Rejection> {
    let config = match load_config(root) {
        Some(c) => c,
        None => {
            eprintln!("⚠️  配置文件读取失败，本次请求使用默认配置");
            Config::default()
        }
    };

    let client_ip = match remote_addr {
        Some(addr) => addr.ip().to_string(),
        None => String::from("unknown"),
    };

    let base = effective_root(root, &config);

    if is_sensitive(tail) {
        log_request(root, tail, 404, &client_ip).await;
        return Ok(text(StatusCode::NOT_FOUND, String::from("404 Not Found")));
    }

    let api_rel = if let Some(r) = tail.strip_prefix("api/tree/") {
        Some(r)
    } else if let Some(r) = tail.strip_prefix("api/") {
        Some(r)
    } else {
        None
    };
    if let Some(raw) = api_rel {
        let rel = decode_path(raw);
        let rel_clean = rel.trim_start_matches("/").to_string();
        match get_directory_tree(&base, &rel_clean, &config) {
            Ok(tree) => {
                log_request(root, tail, 200, &client_ip).await;
                let body = serde_json::to_string(&tree).unwrap_or_else(|_| String::from("[]"));
                return Ok(json_bytes(body));
            }
            Err(e) => {
                log_request(root, tail, 404, &client_ip).await;
                return Ok(text(StatusCode::NOT_FOUND, format!("Error: {}", e)));
            }
        }
    }

    if tail.is_empty() || tail == "/" {
        let index_name = config.index_file.trim().to_string();
        if !index_name.is_empty() && !is_sensitive(&index_name) {
            if let Ok(index_path) = resolve_inside(&base, &index_name) {
                if index_path.is_file() {
                    if let Ok(content) = tokio_fs::read(&index_path).await {
                        log_request(root, tail, 200, &client_ip).await;
                        return Ok(html(StatusCode::OK, String::from_utf8_lossy(&content).to_string()));
                    }
                }
            }
        }
        log_request(root, tail, 200, &client_ip).await;
        return Ok(html(StatusCode::OK, generate_index_html(&config)));
    }

    let rel = decode_path(tail);
    let rel_clean = rel.trim_start_matches("/").to_string();
    match resolve_inside(&base, &rel_clean) {
        Ok(fs_path) => {
            if fs_path.is_file() {
                let mime = mime_guess::from_path(&fs_path).first_or_octet_stream();
                let size = fs::metadata(&fs_path).map(|m| m.len()).unwrap_or(0);
                let file_name = fs_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let disposition = format!("inline; filename*=UTF-8{}", urlencoding::encode(&file_name));
                log_request(root, tail, 200, &client_ip).await;
                let body = stream_file(fs_path);
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", mime.as_ref())
                    .header("Content-Length", size)
                    .header("Content-Disposition", disposition)
                    .header("X-Content-Type-Options", "nosniff")
                    .body(body)
                    .expect("response build failed"));
            }
        }
        Err(PathError::Traversal) => {
            log_request(root, tail, 403, &client_ip).await;
            return Ok(text(StatusCode::FORBIDDEN, String::from("403 禁止访问")));
        }
        Err(PathError::NotFound) => {}
    }

    log_request(root, tail, 404, &client_ip).await;
    Ok(text(StatusCode::NOT_FOUND, String::from("404 Not Found")))
}

pub async fn handle_admin_page(root: PathBuf) -> Result<ApiResponse, warp::Rejection> {
    match load_config(&root) {
        Some(config) => {
            let mut page = generate_admin_html(&config);
            // 后台透明度 / 模糊度跟随配置文件（nweb.yml 的 opacity / blur）
            let adm_opacity = format!("{:.2}", config.opacity.clamp(0.0, 1.0));
            let adm_blur = sanitize_css_len(&config.blur);
            let vars = format!(
                "<style>:root{{--adm-opacity:{};--adm-blur:{}}}</style>",
                adm_opacity, adm_blur
            );
            if let Some(pos) = page.rfind("</head>") {
                page.insert_str(pos, &vars);
            }
            // 注入后台文件管理模块（该页面仅在管理员认证通过后生成）
            if let Some(pos) = page.rfind("</div>") {
                page.insert_str(pos, &crate::html::files::generate_files_html());
            }
            Ok(html(StatusCode::OK, page))
        }
        None => Ok(text(StatusCode::INTERNAL_SERVER_ERROR, String::from("配置读取失败"))),
    }
}

pub async fn get_admin_config(root: PathBuf) -> Result<ApiResponse, warp::Rejection> {
    match load_config(&root) {
        Some(config) => {
            let body = serde_json::to_string(&config).unwrap_or_else(|_| String::from("{}"));
            Ok(json_bytes(body))
        }
        None => Ok(text(StatusCode::INTERNAL_SERVER_ERROR, String::from("配置读取失败"))),
    }
}

pub async fn update_admin_config(
    root: PathBuf,
    body: serde_json::Value,
) -> Result<ApiResponse, warp::Rejection> {
    let existing = load_config(&root).unwrap_or_default();
    let mut merged = serde_json::to_value(&existing).unwrap_or_else(|_| json!({}));
    if let (Some(dst), Some(src)) = (merged.as_object_mut(), body.as_object()) {
        for (k, v) in src.iter() {
            dst.insert(k.clone(), v.clone());
        }
    }
    let new_config: Config = match serde_json::from_value(merged) {
        Ok(c) => c,
        Err(e) => {
            return Ok(json_bytes(json!({"error": format!("配置格式错误: {}", e)}).to_string()))
        }
    };
    match save_config(&root, &new_config) {
        Ok(_) => Ok(json_bytes(
            json!({"status": "success", "message": "配置已保存"}).to_string(),
        )),
        Err(e) => Ok(json_bytes(json!({"error": e}).to_string())),
    }
}

pub async fn get_admin_logs(root: PathBuf) -> Result<ApiResponse, warp::Rejection> {
    let log_path = root.join("nweb.log");
    let content = match tokio_fs::read_to_string(&log_path).await {
        Ok(c) => c,
        Err(_) => String::new(),
    };
    let lines: Vec<&str> = content.lines().collect();
    let start = if lines.len() > 200 { lines.len() - 200 } else { 0 };
    let sep = char::from(10u8).to_string();
    let recent = lines[start..].join(&sep);
    Ok(json_bytes(json!({ "logs": recent }).to_string()))
}

pub async fn get_admin_status(_root: PathBuf, start_time: u64) -> Result<ApiResponse, warp::Rejection> {
    let pid = Pid::from_u32(std::process::id());
    let (memory, cpu) = {
        let mut guard = match SYS.lock() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        if guard.is_none() {
            let mut sys = System::new();
            sys.refresh_process(pid);
            *guard = Some(sys);
        }
        let sys = match guard.as_mut() {
            Some(s) => s,
            None => return Ok(text(StatusCode::INTERNAL_SERVER_ERROR, String::from("状态不可用"))),
        };
        sys.refresh_process(pid);
        let mut mem = 0u64;
        let mut usage = 0.0f32;
        if let Some(p) = sys.process(pid) {
            mem = p.memory() / 1024;
            usage = p.cpu_usage();
        }
        (mem, usage)
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let status_json = json!({
        "pid": pid.as_u32(),
        "cpu_usage": cpu,
        "memory": memory,
        "uptime": now.saturating_sub(start_time),
        "version": env!("CARGO_PKG_VERSION"),
    });
    Ok(json_bytes(status_json.to_string()))
}

pub async fn exec_command(root: PathBuf, command: String) -> Result<ApiResponse, warp::Rejection> {
    let config = match load_config(&root) {
        Some(c) => c,
        None => {
            return Ok(json_bytes(json!({"error": "配置读取失败"}).to_string()));
        }
    };
    if !config.enable_exec {
        println!("[EXEC] ⛔ 命令执行功能未启用，已拒绝: {}", command);
        return Ok(json_bytes(
            json!({"error": "命令执行功能未启用，请在 nweb.yml 中设置 enable_exec: true"}).to_string(),
        ));
    }
    println!("[EXEC] ▶ {}", command);
    let result = tokio::task::spawn_blocking(move || {
        Command::new("sh").arg("-c").arg(&command).output()
    })
    .await;
    match result {
        Ok(Ok(out)) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let body = json!({
                "stdout": stdout,
                "stderr": stderr,
                "code": out.status.code().unwrap_or(-1),
            });
            Ok(json_bytes(body.to_string()))
        }
        _ => Ok(json_bytes(json!({"error": "命令执行失败"}).to_string())),
    }
}
