use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::sync::Mutex;

static SETTINGS_WRITE: Mutex<()> = Mutex::new(());
use std::path::PathBuf;
use tauri::AppHandle;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Default)]
pub struct StoredSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plex_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plex_token: Option<String>,
    pub tmdb_key: Option<String>,
    pub client_id: Option<String>,
    #[serde(default)]
    pub plex: PlexSettings,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct PlexSettings {
    pub active_server_id: Option<String>,
    pub servers: Vec<PlexProfile>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct PlexProfile {
    pub id: String,
    pub name: String,
    pub url: String,
    pub token: String,
}
pub type SettingsView = StoredSettings;
impl StoredSettings {
    fn migrate(&mut self) {
        if self.plex.servers.is_empty() && (self.plex_url.is_some() || self.plex_token.is_some()) {
            let id = Uuid::new_v4().to_string();
            self.plex.servers.push(PlexProfile {
                id: id.clone(), name: "My Plex Server".into(),
                url: self.plex_url.take().unwrap_or_default(),
                token: self.plex_token.take().unwrap_or_default(),
            });
            self.plex.active_server_id = Some(id);
        }
        if !self.plex.servers.iter().any(|s| Some(&s.id) == self.plex.active_server_id.as_ref()) {
            self.plex.active_server_id = self.plex.servers.first().map(|s| s.id.clone());
        }
        ensure_client_id(self);
    }
}

pub fn normalize_plex_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Plex URL is empty.".to_string());
    }
    let with_scheme = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("http://{}", trimmed)
    };
    let url = reqwest::Url::parse(&with_scheme).map_err(|_| "Invalid Plex URL.".to_string())?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none()
        || !url.username().is_empty() || url.password().is_some() || url.query().is_some() || url.fragment().is_some() {
        return Err("Use an HTTP(S) Plex URL without credentials, query parameters, or fragments.".into());
    }
    Ok(url.as_str().trim_end_matches('/').to_string())
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
    read_settings_file(&settings_path(app)?)
}

fn read_settings_file(path: &PathBuf) -> Result<StoredSettings, String> {
    if !path.exists() {
        let mut defaults = StoredSettings::default();
        ensure_client_id(&mut defaults);
        return Ok(defaults);
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut settings: StoredSettings = serde_json::from_str(&raw)
        .map_err(|_| "Settings file is invalid; it has been left unchanged.".to_string())?;
    settings.migrate();
    Ok(settings)
}

pub fn write_settings(app: &AppHandle, settings: &StoredSettings) -> Result<(), String> {
    write_settings_file(&settings_path(app)?, settings)
}

fn write_settings_file(path: &PathBuf, settings: &StoredSettings) -> Result<(), String> {
    ensure_parent(path)?;
    let raw = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    let temporary = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    let result = (|| {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|e| e.to_string())?;
        file.write_all(raw.as_bytes()).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        fs::rename(&temporary, path).map_err(|e| e.to_string())
    })();
    if result.is_err() { let _ = fs::remove_file(&temporary); }
    result
}

pub fn load_settings_view(app: &AppHandle) -> Result<SettingsView, String> {
    let _guard = SETTINGS_WRITE.lock().map_err(|_| "Settings are unavailable.".to_string())?;
    let settings = read_settings(app)?;
    write_settings(app, &settings)?;
    Ok(settings)
}
pub fn save_plex_config(app: &AppHandle, plex: PlexSettings) -> Result<(), String> {
    let _guard = SETTINGS_WRITE.lock().map_err(|_| "Settings are unavailable.".to_string())?;
    validate_profiles(&plex)?;
    let mut settings = read_settings(app)?;
    settings.plex = plex;
    settings.plex_url = None;
    settings.plex_token = None;
    write_settings(app, &settings)
}
fn validate_profiles(plex: &PlexSettings) -> Result<(), String> {
    let mut ids = std::collections::HashSet::new();
    for profile in &plex.servers {
        if profile.id.trim().is_empty() || !ids.insert(&profile.id) {
            return Err("Server IDs must be present and unique.".into());
        }
    }
    match &plex.active_server_id {
        Some(id) if !ids.contains(id) => Err("Active server does not exist.".into()),
        None if !plex.servers.is_empty() => Err("Select an active server.".into()),
        _ => Ok(()),
    }
}
pub fn save_tmdb_key(app: &AppHandle, tmdb_key: String) -> Result<(), String> {
    let _guard = SETTINGS_WRITE.lock().map_err(|_| "Settings are unavailable.".to_string())?;
    let mut settings = read_settings(app)?;
    settings.tmdb_key = Some(tmdb_key.trim().to_string());
    write_settings(app, &settings)
}

pub fn get_client_id(app: &AppHandle) -> Result<String, String> {
    let settings = read_settings(app)?;
    settings
        .client_id
        .ok_or_else(|| "Missing client id.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn legacy() -> StoredSettings {
        serde_json::from_str(r#"{"plex_url":"http://old:32400/","plex_token":"fixture-secret","tmdb_key":"global-key","client_id":"installation","unrelated":true}"#).unwrap()
    }
    #[test]
    fn migration_preserves_credentials_global_settings_and_stable_identity() {
        let mut value = legacy(); value.migrate();
        let id = value.plex.active_server_id.clone();
        assert_eq!(value.plex.servers[0].url, "http://old:32400/");
        assert_eq!(value.plex.servers[0].token, "fixture-secret");
        assert_eq!(value.tmdb_key.as_deref(), Some("global-key"));
        assert_eq!(value.client_id.as_deref(), Some("installation"));
        assert_eq!(value.extra["unrelated"], true);
        assert!(value.plex_url.is_none() && value.plex_token.is_none());
        value.plex.servers[0].name = "Renamed".into();
        let mut restarted: StoredSettings = serde_json::from_str(&serde_json::to_string(&value).unwrap()).unwrap();
        restarted.migrate();
        assert_eq!(restarted.plex.active_server_id, id);
        assert_eq!(restarted.plex.servers.len(), 1);
    }
    #[test]
    fn validates_identity_independently_of_duplicate_names_and_offline_credentials() {
        let mut value = legacy(); value.migrate();
        let mut other = value.plex.servers[0].clone(); other.id = "other".into();
        value.plex.servers.push(other);
        assert!(validate_profiles(&value.plex).is_ok());
        value.plex.active_server_id = Some("missing".into());
        assert!(validate_profiles(&value.plex).is_err());
        value.migrate(); assert!(validate_profiles(&value.plex).is_ok());
        value.plex.servers[1].id = value.plex.servers[0].id.clone();
        assert!(validate_profiles(&value.plex).is_err());
    }
    #[test]
    fn empty_and_tmdb_only_configurations_are_valid() {
        let mut value = StoredSettings { tmdb_key: Some("key".into()), ..Default::default() };
        value.migrate();
        assert!(value.plex.active_server_id.is_none());
        assert!(value.plex.servers.is_empty());
        assert!(validate_profiles(&value.plex).is_ok());
    }
    #[test]
    fn normalizes_urls_and_rejects_unsafe_or_invalid_urls() {
        assert_eq!(normalize_plex_url(" host:32400/ ").unwrap(), "http://host:32400");
        for bad in ["", "ftp://host", "http://", "http://user:secret@host", "http://host?token=secret", "http://host#fragment"] {
            assert!(normalize_plex_url(bad).is_err(), "URL was accepted");
        }
    }
    #[test]
    fn atomic_file_roundtrip_and_corrupt_file_preservation() {
        let directory = std::env::temp_dir().join(format!("plexsuite-settings-test-{}", Uuid::new_v4()));
        let path = directory.join("settings.json");
        let mut value = legacy(); value.migrate();
        write_settings_file(&path, &value).unwrap();
        value.tmdb_key = Some("changed".into());
        write_settings_file(&path, &value).unwrap();
        let loaded = read_settings_file(&path).unwrap();
        assert_eq!(loaded.tmdb_key, value.tmdb_key);
        assert_eq!(loaded.plex.active_server_id, value.plex.active_server_id);
        fs::write(&path, "invalid JSON").unwrap();
        assert!(read_settings_file(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "invalid JSON");
        fs::remove_file(path).unwrap(); fs::remove_dir(directory).unwrap();
    }
}
