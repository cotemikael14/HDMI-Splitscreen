//! Lecture / écriture de config.json (compatible avec l'ancienne version Python).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn default_true() -> bool {
    true
}
fn default_width() -> u32 {
    1280
}
fn default_height() -> u32 {
    720
}
fn default_fps() -> u32 {
    60
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamConfig {
    pub name: String,
    /// Index du périphérique de capture (0, 1, 2...)
    pub source: u32,
    pub display_name: String,
    pub enabled: bool,
    /// Nom du périphérique, pour le retrouver si l'ordre des ports USB change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_fps")]
    pub fps: u32,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            name: "New Stream".into(),
            source: 0,
            display_name: "HDMI".into(),
            enabled: true,
            device_name: None,
            width: default_width(),
            height: default_height(),
            fps: default_fps(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub streams: Vec<StreamConfig>,
    /// Garder les proportions de l'image (bandes noires) au lieu d'étirer.
    #[serde(default = "default_true")]
    pub keep_aspect: bool,
    /// Afficher le nom du flux au-dessus de l'image.
    #[serde(default = "default_true")]
    pub show_labels: bool,
    /// Afficher les infos du flux (format, résolution, fps).
    #[serde(default = "default_true")]
    pub show_info: bool,
    /// Synchro verticale. Désactivée = moins de latence (possible "tearing").
    #[serde(default)]
    pub vsync: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            streams: Vec::new(),
            keep_aspect: true,
            show_labels: true,
            show_info: true,
            vsync: false,
        }
    }
}

/// config.json est placé à côté du .exe (marche même en double-clic).
pub fn config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("config.json")))
        .unwrap_or_else(|| PathBuf::from("config.json"))
}

pub fn load() -> Config {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(text) => match serde_json::from_str::<Config>(&text) {
            Ok(cfg) => {
                // Réécrit le fichier pour ajouter les nouvelles options par défaut.
                save(&cfg);
                cfg
            }
            Err(e) => {
                eprintln!("config.json invalide ({e}), configuration par défaut utilisée");
                Config::default()
            }
        },
        Err(_) => {
            let cfg = Config::default();
            save(&cfg);
            cfg
        }
    }
}

pub fn save(cfg: &Config) {
    let path = config_path();
    match serde_json::to_string_pretty(cfg) {
        Ok(text) => {
            if let Err(e) = std::fs::write(&path, text) {
                eprintln!("Impossible d'écrire {}: {e}", path.display());
            }
        }
        Err(e) => eprintln!("Erreur de sérialisation: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_old_python_config() {
        let old = r#"{"streams":[{"name":"New Stream","source":1,"display_name":"HDMI","enabled":true}]}"#;
        let cfg: Config = serde_json::from_str(old).unwrap();
        assert_eq!(cfg.streams[0].source, 1);
        assert_eq!(cfg.streams[0].width, 1280);
        assert_eq!(cfg.streams[0].fps, 60);
        assert!(cfg.keep_aspect && !cfg.vsync);
    }
}
