use directories::UserDirs;
use ini::Ini;
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub api_key: Option<String>,
    pub api_endpoint: Option<String>,
    pub api_unrated_endpoint: Option<String>,
    pub api_unrated_email: Option<String>,
    pub api_ondemand_key: Option<String>,
    pub api_ondemand_endpoint: Option<String>,
    pub api_size: Option<u32>,
    pub api_maxpage: Option<u32>,
    pub api_trackquery: Option<bool>,
    pub api_calculated: Option<bool>,
    pub api_keepalive: Option<bool>,
}

impl Config {
    pub fn load(path: Option<&str>) -> Self {
        let mut config = Config::default();

        let config_path = match path {
            Some(p) if p.starts_with("~/") => {
                if let Some(user_dirs) = UserDirs::new() {
                    user_dirs.home_dir().join(&p[2..])
                } else {
                    PathBuf::from(p)
                }
            }
            Some(p) => PathBuf::from(p),
            None => {
                if let Some(user_dirs) = UserDirs::new() {
                    user_dirs.home_dir().join(".onyphe.ini")
                } else {
                    return config;
                }
            }
        };

        if let Ok(ini) = Ini::load_from_file(&config_path) {
            for (_, section) in ini.iter() {
                if let Some(val) = section.get("api_key") {
                    config.api_key = Some(val.to_string());
                }
                if let Some(val) = section.get("api_endpoint") {
                    config.api_endpoint = Some(val.to_string());
                }
                if let Some(val) = section.get("api_unrated_endpoint") {
                    config.api_unrated_endpoint = Some(val.to_string());
                }
                if let Some(val) = section.get("api_unrated_email") {
                    config.api_unrated_email = Some(val.to_string());
                }
                if let Some(val) = section.get("api_ondemand_key") {
                    config.api_ondemand_key = Some(val.to_string());
                }
                if let Some(val) = section.get("api_ondemand_endpoint") {
                    config.api_ondemand_endpoint = Some(val.to_string());
                }

                if let Some(s) = section.get("api_size") {
                    config.api_size = s.parse().ok();
                }
                if let Some(s) = section.get("api_maxpage") {
                    config.api_maxpage = s.parse().ok();
                }
                if let Some(s) = section.get("api_trackquery") {
                    config.api_trackquery = Some(s == "1" || s.to_lowercase() == "true");
                }
                if let Some(s) = section.get("api_calculated") {
                    config.api_calculated = Some(s == "1" || s.to_lowercase() == "true");
                }
                if let Some(s) = section.get("api_keepalive") {
                    config.api_keepalive = Some(s == "1" || s.to_lowercase() == "true");
                }
            }
        }
        config
    }
}
