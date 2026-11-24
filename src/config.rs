use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

static CACHE: OnceLock<Mutex<HashMap<PathBuf, (SystemTime, Config)>>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub background_api: String,
    #[serde(default)]
    pub opacity: f32,
    #[serde(default)]
    pub blur: String,
    #[serde(default)]
    pub show_hidden: bool,
    #[serde(default = "default_index_file")]
    pub index_file: String,
    #[serde(default = "default_admin_user")]
    pub admin_user: String,
    #[serde(default = "default_admin_pass")]
    pub admin_pass: String,
    #[serde(default = "default_default_sort")]
    pub default_sort: String,
    #[serde(default = "default_show_file_size")]
    pub show_file_size: bool,
    #[serde(default = "default_show_admin_btn")]
    pub show_admin_btn: bool,
    #[serde(default = "default_clear_log")]
    pub clear_log_on_start: bool,
    #[serde(default)]
    pub home_dir: String,
    #[serde(default)]
    pub enable_exec: bool,
}

fn default_index_file() -> String {
    String::from("index.html")
}
fn default_admin_user() -> String {
    String::from("nweb")
}
fn default_admin_pass() -> String {
    String::from("nweb")
}
fn default_default_sort() -> String {
    String::from("name")
}
fn default_show_file_size() -> bool {
    true
}
fn default_show_admin_btn() -> bool {
    true
}
fn default_clear_log() -> bool {
    false
}

impl Default for Config {
    fn default() -> Self {
        Config {
            title: String::from("nweb"),
            description: String::from("本网页由 nweb 自动生成"),
            background_api: String::from("https://www.loliapi.com/acg/"),
            opacity: 0.3,
            blur: String::from("5px"),
            show_hidden: false,
            index_file: default_index_file(),
            admin_user: default_admin_user(),
            admin_pass: default_admin_pass(),
            default_sort: default_default_sort(),
            show_file_size: default_show_file_size(),
            show_admin_btn: default_show_admin_btn(),
            clear_log_on_start: default_clear_log(),
            home_dir: String::new(),
            enable_exec: false,
        }
    }
}

fn random_password() -> String {
    const CHARS: &[u8] = b"abcdefghijkmnpqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut buf = [0u8; 16];
    if let Ok(mut f) = fs::File::open("/dev/urandom") {
        use std::io::Read;
        let _ = f.read_exact(&mut buf);
    }
    buf.iter()
        .map(|b| CHARS[(*b as usize) % CHARS.len()] as char)
        .collect()
}

pub fn ensure_config(root: &PathBuf) {
    let config_path = root.join("nweb.yml");
    if config_path.exists() {
        return;
    }
    let mut generated = Config::default();
    generated.admin_pass = random_password();
    match save_config(root, &generated) {
        Ok(_) => {
            println!("✅ 已生成配置文件: {}", config_path.display());
            println!(
                "🔑 初始管理员口令: {} （请登录 /@admin 后修改）",
                generated.admin_pass
            );
        }
        Err(e) => eprintln!("⚠️  无法写入配置文件: {}", e),
    }
}

fn invalidate(config_path: &PathBuf) {
    if let Ok(mut guard) = CACHE.get_or_init(|| Mutex::new(HashMap::new())).lock() {
        guard.remove(config_path);
    }
}

pub fn load_config(root: &PathBuf) -> Option<Config> {
    let config_path = root.join("nweb.yml");
    let meta = fs::metadata(&config_path).ok()?;
    let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);

    if let Ok(guard) = CACHE.get_or_init(|| Mutex::new(HashMap::new())).lock() {
        if let Some((cached_mtime, cached)) = guard.get(&config_path) {
            if *cached_mtime == mtime {
                return Some(cached.clone());
            }
        }
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️  读取配置文件失败: {}", e);
            return None;
        }
    };
    let cfg: Config = match serde_yaml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠️  配置文件解析失败: {}", e);
            return None;
        }
    };

    if let Ok(mut guard) = CACHE.get_or_init(|| Mutex::new(HashMap::new())).lock() {
        guard.insert(config_path, (mtime, cfg.clone()));
    }
    Some(cfg)
}

pub fn validate(config: &Config) -> Result<(), String> {
    if config.admin_user.trim().is_empty() {
        return Err(String::from("管理员用户名不能为空"));
    }
    if config.admin_pass.is_empty() {
        return Err(String::from("管理员密码不能为空"));
    }
    if config.admin_pass == "nweb" {
        return Err(String::from("管理员密码不能使用默认的 nweb"));
    }
    if !config.opacity.is_finite() || !(0.0..=1.0).contains(&config.opacity) {
        return Err(String::from("透明度必须在 0.0 到 1.0 之间"));
    }
    if !matches!(config.default_sort.as_str(), "name" | "size" | "time") {
        return Err(String::from("默认排序只能是 name、size 或 time"));
    }
    if config.index_file.trim().is_empty()
        || config.index_file.contains('/')
        || config.index_file.contains('\\')
    {
        return Err(String::from("首页文件名必须是单个文件名"));
    }
    if config.blur != crate::html::home::safe_css_length(&config.blur) {
        return Err(String::from("模糊程度格式无效，仅支持数字 px 或百分比"));
    }
    Ok(())
}

pub fn save_config(root: &PathBuf, config: &Config) -> Result<(), String> {
    validate(config)?;
    let config_path = root.join("nweb.yml");
    let yaml = serde_yaml::to_string(config).map_err(|e| format!("序列化失败: {}", e))?;
    let tmp_path = root.join("nweb.yml.tmp");
    fs::write(&tmp_path, yaml).map_err(|e| format!("写入配置文件失败: {}", e))?;
    fs::rename(&tmp_path, &config_path).map_err(|e| format!("保存配置文件失败: {}", e))?;
    invalidate(&config_path);
    println!("✅ 配置已保存: {}", config_path.display());
    Ok(())
}
