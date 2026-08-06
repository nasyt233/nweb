use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
}

fn default_index_file() -> String {
    "index.html".to_string()
}
fn default_admin_user() -> String {
    "nweb".to_string()
}
fn default_admin_pass() -> String {
    "nweb".to_string()
}
fn default_default_sort() -> String {
    "name".to_string()
}
fn default_show_file_size() -> bool {
    true
}
fn default_show_admin_btn() -> bool {
    true
}
fn default_clear_log() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: "nweb".to_string(),
            description: "本网页由 nweb 自动生成".to_string(),
            background_api: "https://www.loliapi.com/acg/".to_string(),
            opacity: 0.3,
            blur: "5px".to_string(),
            show_hidden: false,
            index_file: "index.html".to_string(),
            admin_user: "nweb".to_string(),
            admin_pass: "nweb".to_string(),
            default_sort: "name".to_string(),
            show_file_size: true,
            show_admin_btn: true,
            clear_log_on_start: true,
            home_dir: "".to_string(),
        }
    }
}

pub fn ensure_config(root: &PathBuf) {
    let config_path = root.join("nweb.yml");
    if !config_path.exists() {
        let default_config = Config::default();
        match save_config(root, &default_config) {
            Ok(_) => println!("✅ 默认配置文件: {}", config_path.display()),
            Err(e) => eprintln!("⚠️  无法写入配置文件: {}", e),
        }
    }
}

pub fn load_config(root: &PathBuf) -> Option<Config> {
    let config_path = root.join("nweb.yml");
    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => {
                match serde_yaml::from_str::<Config>(&content) {
                    Ok(cfg) => {
                        if cfg.title.is_empty() {
                            eprintln!("⚠️  title 为空，使用默认配置");
                            return None;
                        }
                        return Some(cfg);
                    }
                    Err(e) => {
                        eprintln!("⚠️  配置文件解析失败: {}", e);
                        return None;
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️  读取配置文件失败: {}", e);
                return None;
            }
        }
    }
    None
}

pub fn save_config(root: &PathBuf, config: &Config) -> Result<(), String> {
    let config_path = root.join("nweb.yml");
    let yaml = serde_yaml::to_string(config).map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&config_path, yaml).map_err(|e| format!("写入配置文件失败: {}", e))?;
    println!("✅ 配置已保存: {}", config_path.display());
    Ok(())
}