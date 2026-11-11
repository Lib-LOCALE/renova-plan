use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

pub struct I18nData {
    pub fr: Value,
    pub en: Value,
}

impl I18nData {
    pub fn load() -> Self {
        let root: Value = serde_json::from_str(include_str!("../ui/data/i18n.json"))
            .expect("Fichier i18n.json valide");
        Self {
            fr: root["fr"].clone(),
            en: root["en"].clone(),
        }
    }

    pub fn get(&self, key: &str, lang_idx: i32) -> String {
        let table = if lang_idx == 1 { &self.en } else { &self.fr };
        table[key].as_str().unwrap_or(key).to_owned()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preferences {
    pub language: usize, // 0 = FR, 1 = EN
    pub theme: usize,    // 0 = COSMIC Dark, 1 = Atelier Clair (default), 2 = Mocha, 3 = Nord
    #[serde(default)]
    pub large_text: bool,
    #[serde(default)]
    pub expert_mode: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        let locale = sys_locale::get_locale()
            .or_else(|| {
                ["LC_ALL", "LC_MESSAGES", "LANG"]
                    .iter()
                    .filter_map(|k| std::env::var(k).ok())
                    .find(|s| !s.is_empty())
            })
            .unwrap_or_default()
            .to_lowercase();
        let language = if locale.starts_with("en") { 1 } else { 0 };
        Self {
            language,
            theme: 1, // Thème clair « Papier » par défaut pour une lisibilité immédiate
            large_text: false,
            expert_mode: false,
        }
    }
}

fn preferences_base_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join("AppData").join("Roaming")))
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
    }
}

fn preferences_path() -> Option<PathBuf> {
    preferences_base_dir().map(|p| p.join("RenovaPlan").join("preferences.json"))
}

impl Preferences {
    pub fn load() -> Self {
        let try_paths = [
            preferences_path(),
            preferences_base_dir().map(|p| p.join("io.github.tonyboysuper.RenovaPlan").join("preferences.json")),
        ];

        for opt_p in try_paths {
            if let Some(path) = opt_p
                && let Ok(b) = std::fs::read(&path)
                && let Ok(saved) = serde_json::from_slice::<Self>(&b)
            {
                return Self {
                    language: saved.language.min(1),
                    theme: saved.theme.min(3),
                    large_text: saved.large_text,
                    expert_mode: saved.expert_mode,
                };
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = preferences_path().ok_or("Répertoire de préférences introuvable")?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(self)?;
        let temp = path.with_extension(format!("{}.tmp", std::process::id()));
        std::fs::write(&temp, &bytes)?;
        #[cfg(windows)]
        let _ = std::fs::remove_file(&path);
        if std::fs::rename(&temp, &path).is_err() {
            std::fs::write(&path, &bytes)?;
            let _ = std::fs::remove_file(&temp);
        }
        Ok(())
    }
}

pub fn hex_color(hex: &str) -> slint::Color {
    let rgb = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0x14161b);
    slint::Color::from_rgb_u8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

pub struct PaletteColors {
    pub bg: &'static str,
    pub panel: &'static str,
    pub surface: &'static str,
    pub hover: &'static str,
    pub border: &'static str,
    pub text: &'static str,
    pub muted: &'static str,
    pub accent: &'static str,
    pub on_accent: &'static str,
    pub green: &'static str,
    pub amber: &'static str,
    pub red: &'static str,
    pub dark_canvas: bool,
}

pub fn theme_palette(index: usize) -> PaletteColors {
    match index % 4 {
        // 1: Atelier Clair (Papier Architecte chaleureux, contraste élevé)
        1 => PaletteColors {
            bg: "#faf8f5",
            panel: "#f1ede4",
            surface: "#ffffff",
            hover: "#e4dfd3",
            border: "#cfc8b8",
            text: "#18181b",
            muted: "#52525b",
            accent: "#0369a1",
            on_accent: "#ffffff",
            green: "#059669",
            amber: "#d97706",
            red: "#dc2626",
            dark_canvas: false,
        },
        // 2: Catppuccin Mocha
        2 => PaletteColors {
            bg: "#1e1e2e",
            panel: "#181825",
            surface: "#313244",
            hover: "#45475a",
            border: "#313244",
            text: "#cdd6f4",
            muted: "#a6adc8",
            accent: "#89b4fa",
            on_accent: "#11111b",
            green: "#a6e3a1",
            amber: "#f9e2af",
            red: "#f38ba8",
            dark_canvas: true,
        },
        // 3: Nord Architect
        3 => PaletteColors {
            bg: "#2e3440",
            panel: "#242933",
            surface: "#3b4252",
            hover: "#434c5e",
            border: "#4c566a",
            text: "#eceff4",
            muted: "#d8dee9",
            accent: "#88c0d0",
            on_accent: "#2e3440",
            green: "#a3be8c",
            amber: "#ebcb8b",
            red: "#bf616a",
            dark_canvas: true,
        },
        // 0: COSMIC Dark (Pop!_OS Default)
        _ => PaletteColors {
            bg: "#14161b",
            panel: "#1c1f26",
            surface: "#262a34",
            hover: "#323846",
            border: "#2e3440",
            text: "#f1f5f9",
            muted: "#94a3b8",
            accent: "#38bdf8",
            on_accent: "#0f172a",
            green: "#34d399",
            amber: "#fbbf24",
            red: "#f87171",
            dark_canvas: true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i18n_keys_exist_in_both_french_and_english() {
        let i18n = I18nData::load();
        let fr_obj = i18n.fr.as_object().expect("table FR");
        let en_obj = i18n.en.as_object().expect("table EN");
        assert_eq!(fr_obj.len(), en_obj.len());
        for key in fr_obj.keys() {
            assert!(
                en_obj.contains_key(key),
                "Missing English translation key: {key}"
            );
            assert!(!i18n.get(key, 0).is_empty());
            assert!(!i18n.get(key, 1).is_empty());
        }
    }

    #[test]
    fn all_four_themes_have_valid_colors() {
        for idx in 0..4 {
            let p = theme_palette(idx);
            let _ = hex_color(p.bg);
            let _ = hex_color(p.panel);
            let _ = hex_color(p.accent);
        }
    }

    #[test]
    fn preferences_path_and_save_load_work() {
        let path = preferences_path();
        assert!(path.is_some(), "preferences_path() should resolve on this platform");
        let initial_prefs = Preferences {
            language: 1,
            theme: 2,
            large_text: true,
            expert_mode: true,
        };
        assert!(initial_prefs.save().is_ok(), "Preferences::save should succeed");
        let loaded = Preferences::load();
        assert_eq!(loaded.language, 1);
        assert_eq!(loaded.theme, 2);
        assert!(loaded.large_text);
        assert!(loaded.expert_mode);
    }
}
