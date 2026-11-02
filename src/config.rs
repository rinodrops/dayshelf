use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::card::CardOrder;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalConfig {
    #[serde(default)]
    pub sync: SyncSection,
    #[serde(default)]
    pub calendar: CalendarSection,
    #[serde(default)]
    pub cards: CardsSection,
    #[serde(default = "default_language")]
    pub language: String,
    /// BCP-47 tag so the settings window can follow `language`. Not shown in the form.
    #[serde(default)]
    pub language_bcp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SyncSection {
    #[serde(default)]
    pub folder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarSection {
    #[serde(default = "default_week_start")]
    pub week_start: String,
    #[serde(default = "default_month_count")]
    pub month_count: u8,
    #[serde(default = "default_span")]
    pub span: String,
}

impl Default for CalendarSection {
    fn default() -> Self {
        Self {
            week_start: default_week_start(),
            month_count: default_month_count(),
            span: default_span(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardsSection {
    #[serde(default)]
    pub order: CardOrder,
}

impl Default for CardsSection {
    fn default() -> Self {
        Self {
            order: CardOrder::PastToFuture,
        }
    }
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            sync: SyncSection::default(),
            calendar: CalendarSection::default(),
            cards: CardsSection::default(),
            language: default_language(),
            language_bcp: String::new(),
        }
    }
}

fn default_language() -> String {
    "os".to_owned()
}

fn default_week_start() -> String {
    "locale".to_owned()
}

fn default_month_count() -> u8 {
    1
}

fn default_span() -> String {
    "include_previous".to_owned()
}

impl LocalConfig {
    pub fn folder(&self) -> Option<&Path> {
        if self.sync.folder.is_empty() {
            None
        } else {
            Some(Path::new(&self.sync.folder))
        }
    }

    pub fn sync_language_bcp(&mut self) {
        self.language_bcp = whichlang_to_bcp(&self.language)
            .unwrap_or_default()
            .to_owned();
    }
}

pub fn whichlang_to_bcp(code: &str) -> Option<&'static str> {
    Some(match code {
        "ara" => "ar",
        "cmn" => "zh",
        "deu" => "de",
        "eng" => "en",
        "fra" => "fr",
        "hin" => "hi",
        "ita" => "it",
        "jpn" => "ja",
        "kor" => "ko",
        "nld" => "nl",
        "por" => "pt",
        "rus" => "ru",
        "spa" => "es",
        "swe" => "sv",
        "tur" => "tr",
        "vie" => "vi",
        _ => return None,
    })
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

pub fn settings_binary() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("SETTINGS_BIN") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in ["settings", "settings.exe"] {
                let path = dir.join(name);
                if path.is_file() {
                    return Some(path);
                }
            }
        }
    }
    which_on_path("settings")
}

fn which_on_path(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
}
