use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub telegram_token: String,
    pub admin_id: u64,
    #[serde(default)]
    pub password_hash: Option<String>,
}

impl Config {
    pub fn config_path() -> PathBuf {
        let mut candidates = Vec::<PathBuf>::new();

        if let Ok(exe) = env::current_exe() {
            if let Some(parent) = exe.parent() {
                candidates.push(parent.join("config.json"));
            }
        }

        if let Ok(dir) = env::current_dir() {
            candidates.push(dir.join("config.json"));
        }

        if let Some(existing) = candidates.iter().find(|p| p.is_file()) {
            return existing.clone();
        }

        candidates.into_iter().next().unwrap_or_else(|| {
            PathBuf::from("config.json")
        })
    }

    pub fn load() -> Result<Self, String> {
        let path = Self::config_path();

        if !path.is_file() {
            return Err("config.json not found".to_string());
        }

        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("Cannot read config.json: {e}"))?;

        let config: Config = serde_json::from_str(&contents)
            .map_err(|e| format!("Invalid config.json: {e}"))?;

        if config.telegram_token.trim().is_empty() {
            return Err("Telegram token is empty".to_string());
        }

        if config.admin_id == 0 {
            return Err("Admin ID is invalid".to_string());
        }

        Ok(config)
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();

        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Cannot serialize config: {e}"))?;

        fs::write(path, contents)
            .map_err(|e| format!("Cannot save config: {e}"))
    }
}
