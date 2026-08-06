use std::env;

/// 展开路径环境变量
pub fn expand_path(path: &str) -> String {
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