use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{App, AppHandle, Manager};

/// Persisted user preferences that the backend itself acts on (things a
/// hidden webview shouldn't be the single source of truth for). Stored in
/// the same config dir as `tunnels.json`.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    /// Show the active tunnel count as text next to the tray icon.
    pub show_tray_badge: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self { show_tray_badge: true }
    }
}

pub struct SettingsStore {
    path: PathBuf,
    settings: Mutex<AppSettings>,
}

impl SettingsStore {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let dir = app
            .path()
            .app_config_dir()
            .map_err(|e| format!("failed to resolve config dir: {e}"))?;
        fs::create_dir_all(&dir).map_err(|e| format!("failed to create config dir: {e}"))?;
        let path = dir.join("settings.json");

        let settings = match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => AppSettings::default(),
        };

        Ok(Self {
            path,
            settings: Mutex::new(settings),
        })
    }

    pub fn show_tray_badge(&self) -> bool {
        self.settings.lock().unwrap().show_tray_badge
    }

    pub fn set_show_tray_badge(&self, show: bool) -> Result<(), String> {
        let mut settings = self.settings.lock().unwrap();
        settings.show_tray_badge = show;
        let json = serde_json::to_string_pretty(&*settings)
            .map_err(|e| format!("failed to serialize settings: {e}"))?;
        fs::write(&self.path, json).map_err(|e| format!("failed to write settings file: {e}"))
    }
}

pub fn setup(app: &App) -> Result<(), String> {
    let store = SettingsStore::load(app.handle())?;
    app.manage(store);
    Ok(())
}