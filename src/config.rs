use crate::swap_info::SizeUnits;
use crate::theme::ThemeType;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub aggregated: bool,
    pub size_unit: SizeUnits,
    pub display_devices: bool,
    pub theme: ThemeType,
    pub timeout: u64,
    pub devices_split_ratio: f64,
    pub show_info_panel: bool,
    pub info_split_ratio: f64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            aggregated: false,
            size_unit: SizeUnits::KB,
            display_devices: false,
            theme: ThemeType::Dracula,
            timeout: 1000,
            devices_split_ratio: 0.70,
            show_info_panel: false,
            info_split_ratio: 0.35,
        }
    }
}

fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("swaptop"))
}

fn config_path() -> Option<PathBuf> {
    config_dir().map(|p| p.join("config.toml"))
}

pub fn load_config() -> AppConfig {
    let Some(path) = config_path() else {
        return AppConfig::default();
    };

    if !path.exists() {
        return AppConfig::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

pub fn save_config(config: &AppConfig) {
    let Some(dir) = config_dir() else {
        return;
    };

    if fs::create_dir_all(&dir).is_err() {
        return;
    };

    let Some(path) = config_path() else {
        return;
    };

    if let Ok(content) = toml::to_string_pretty(config) {
        let _ = fs::write(&path, content);
    }
}
