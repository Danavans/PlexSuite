use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct StoredSettings {
    pub plex_url: Option<String>,
    pub plex_token: Option<String>,
    pub tmdb_key: Option<String>,
    pub client_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsView {
    pub plex_url: Option<String>,
    pub plex_token: Option<String>,
    pub tmdb_key: Option<String>,
    pub client_id: String,
}

pub fn normalize_plex_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Plex URL is empty.".to_string());
    }
    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{}", trimmed)
    };
    Ok(with_scheme.trim_end_matches('/').to_string())
}

fn app_data_dir(_app: &AppHandle) -> Result<PathBuf, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Failed to resolve executable path: {e}"))?;
    exe.parent()
        .map(|path| path.to_path_buf())
        .ok_or_else(|| "Failed to resolve executable directory.".to_string())
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app_data_dir(app)?;
    Ok(dir.join("settings.json"))
}

fn ensure_parent(path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn ensure_client_id(settings: &mut StoredSettings) {
    if settings.client_id.is_none() {
        settings.client_id = Some(Uuid::new_v4().to_string());
    }
}

pub fn read_settings(app: &AppHandle) -> Result<StoredSettings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        let mut defaults = StoredSettings::default();
        ensure_client_id(&mut defaults);
        return Ok(defaults);
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut settings: StoredSettings = serde_json::from_str(&raw).unwrap_or_default();
    ensure_client_id(&mut settings);
    Ok(settings)
}

pub fn write_settings(app: &AppHandle, settings: &StoredSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    ensure_parent(&path)?;
    let raw = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, raw).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_settings_view(app: &AppHandle) -> Result<SettingsView, String> {
    let settings = read_settings(app)?;
    Ok(SettingsView {
        plex_url: settings.plex_url,
        plex_token: settings.plex_token,
        tmdb_key: settings.tmdb_key,
        client_id: settings
            .client_id
            .unwrap_or_else(|| Uuid::new_v4().to_string()),
    })
}

pub fn save_settings(
    app: &AppHandle,
    plex_url: String,
    plex_token: String,
    tmdb_key: String,
) -> Result<(), String> {
    let mut settings = read_settings(app)?;
    let normalized = normalize_plex_url(&plex_url)?;
    settings.plex_url = Some(normalized);

    if plex_token.trim().is_empty() {
        settings.plex_token = None;
    } else {
        settings.plex_token = Some(plex_token.trim().to_string());
    }

    if tmdb_key.trim().is_empty() {
        settings.tmdb_key = None;
    } else {
        settings.tmdb_key = Some(tmdb_key.trim().to_string());
    }

    ensure_client_id(&mut settings);
    write_settings(app, &settings)
}

pub fn get_client_id(app: &AppHandle) -> Result<String, String> {
    let settings = read_settings(app)?;
    settings
        .client_id
        .ok_or_else(|| "Missing client id.".to_string())
}
