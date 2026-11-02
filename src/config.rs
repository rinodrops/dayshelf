use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::card::CardOrder;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalConfig {
    #[serde(default)]
    pub sync_folder: String,
    #[serde(default)]
    pub cards_order: CardOrder,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            sync_folder: String::new(),
            cards_order: CardOrder::PastToFuture,
        }
    }
}

impl LocalConfig {
    pub fn folder(&self) -> Option<&Path> {
        if self.sync_folder.is_empty() {
            None
        } else {
            Some(Path::new(&self.sync_folder))
        }
    }
}

pub fn app_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("jp", "EmotionGraphics", "DayShelf")
        .map(|dirs| dirs.config_dir().to_path_buf())
}

pub fn config_path() -> Option<PathBuf> {
    app_dir().map(|dir| dir.join("config.toml"))
}

pub fn device_id_path() -> Option<PathBuf> {
    app_dir().map(|dir| dir.join("device-id"))
}

pub fn load_config() -> LocalConfig {
    let Some(path) = config_path() else {
        return LocalConfig::default();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return LocalConfig::default();
    };
    toml::from_str(&text).unwrap_or_default()
}

pub fn save_config(config: &LocalConfig) -> std::io::Result<()> {
    let Some(path) = config_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = toml::to_string_pretty(config).unwrap_or_default();
    fs::write(path, text)
}

pub fn load_device_id() -> String {
    if let Some(path) = device_id_path() {
        if let Ok(text) = fs::read_to_string(&path) {
            let text = text.trim();
            if !text.is_empty() {
                return text.to_owned();
            }
        }
        let id = uuid::Uuid::new_v4().to_string();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, &id);
        return id;
    }
    uuid::Uuid::new_v4().to_string()
}
