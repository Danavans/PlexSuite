mod matching;
mod plex;
mod settings;
mod tmdb;

use matching::parse_episode_code;
use plex::{EpisodeSummary, PurgeResult, TrashPreview};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubtitlePreviewEntry {
    path: String,
    file_name: String,
    season: Option<i32>,
    episode: Option<i32>,
    status: String,
    episode_title: Option<String>,
    episode_rating_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubtitlePreview {
    total: usize,
    matched: usize,
    entries: Vec<SubtitlePreviewEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubtitleUploadItem {
    path: String,
    episode_rating_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubtitleUploadResult {
    uploaded: usize,
    failed: Vec<String>,
}

#[tauri::command]
async fn save_text_file(path: String, content: String) -> Result<(), String> {
    let target = PathBuf::from(&path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, content).map_err(|e| e.to_string())
}

fn subtitle_extensions() -> &'static [&'static str] {
    &["srt", "ass", "ssa", "vtt"]
}

fn scan_subtitle_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    if !root.is_dir() {
        return Err("Subtitle folder not found.".to_string());
    }
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).map_err(|e| e.to_string())?;
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let lower = ext.to_lowercase();
                if subtitle_extensions()
                    .iter()
                    .any(|allowed| allowed == &lower)
                {
                    files.push(path);
                }
            }
        }
    }
    Ok(files)
}

fn build_episode_map(episodes: Vec<EpisodeSummary>) -> HashMap<(i32, i32), EpisodeSummary> {
    let mut map = HashMap::new();
    for ep in episodes {
        map.insert((ep.season_number, ep.episode_number), ep);
    }
    map
}

#[tauri::command]
async fn get_settings(app: AppHandle) -> Result<settings::SettingsView, String> {
    settings::load_settings_view(&app)
}

#[tauri::command]
async fn save_settings(
    app: AppHandle,
    plex_url: String,
    plex_token: String,
    tmdb_key: String,
) -> Result<(), String> {
    settings::save_settings(&app, plex_url, plex_token, tmdb_key)
}

#[tauri::command]
async fn list_libraries(
    app: AppHandle,
    server_url: String,
    token: String,
) -> Result<Vec<plex::Library>, String> {
    plex::list_libraries(&app, &server_url, &token).await
}

#[tauri::command]
async fn list_shows(
    app: AppHandle,
    server_url: String,
    token: String,
    library_id: String,
    query: Option<String>,
) -> Result<Vec<plex::ShowSummary>, String> {
    plex::list_shows(&app, &server_url, &token, &library_id, query).await
}

#[tauri::command]
async fn list_seasons(
    app: AppHandle,
    server_url: String,
    token: String,
    show_rating_key: String,
) -> Result<Vec<plex::SeasonSummary>, String> {
    plex::list_seasons(&app, &server_url, &token, &show_rating_key).await
}

#[tauri::command]
async fn list_episodes(
    app: AppHandle,
    server_url: String,
    token: String,
    show_rating_key: String,
    season_rating_key: Option<String>,
) -> Result<Vec<plex::EpisodeSummary>, String> {
    plex::list_episodes(
        &app,
        &server_url,
        &token,
        &show_rating_key,
        season_rating_key,
    )
    .await
}

#[tauri::command]
async fn preview_trash(
    app: AppHandle,
    server_url: String,
    token: String,
    library_id: String,
    show_rating_key: String,
    season_rating_key: Option<String>,
) -> Result<TrashPreview, String> {
    plex::preview_trash(
        &app,
        &server_url,
        &token,
        &library_id,
        &show_rating_key,
        season_rating_key,
    )
    .await
}

#[tauri::command]
async fn purge_trash(
    app: AppHandle,
    server_url: String,
    token: String,
    library_id: String,
    show_rating_key: String,
    season_rating_key: Option<String>,
) -> Result<PurgeResult, String> {
    plex::purge_trash(
        &app,
        &server_url,
        &token,
        &library_id,
        &show_rating_key,
        season_rating_key,
    )
    .await
}

#[tauri::command]
async fn tmdb_search_series(
    api_key: String,
    query: String,
) -> Result<Vec<tmdb::TmdbSeries>, String> {
    tmdb::search_series(&api_key, &query).await
}

#[tauri::command]
async fn tmdb_list_episodes(
    api_key: String,
    series_id: i64,
) -> Result<Vec<tmdb::TmdbEpisode>, String> {
    tmdb::list_episodes(&api_key, series_id).await
}

#[tauri::command]
async fn preview_subtitles(
    app: AppHandle,
    server_url: String,
    token: String,
    show_rating_key: String,
    season_number: Option<i32>,
    episode_number: Option<i32>,
    subs_root: String,
) -> Result<SubtitlePreview, String> {
    let all_episodes =
        plex::list_episodes(&app, &server_url, &token, &show_rating_key, None).await?;
    let map = build_episode_map(all_episodes);
    let subs = scan_subtitle_files(Path::new(&subs_root))?;

    let mut entries = Vec::new();
    let mut matched = 0;
    for path in subs {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        let text = file_name.clone();
        let parsed = parse_episode_code(&text, season_number);
        if parsed.is_none() {
            entries.push(SubtitlePreviewEntry {
                path: path.to_string_lossy().to_string(),
                file_name,
                season: None,
                episode: None,
                status: "No episode code found".to_string(),
                episode_title: None,
                episode_rating_key: None,
            });
            continue;
        }
        let parsed = parsed.unwrap();
        if let Some(scope_season) = season_number {
            if parsed.season != scope_season {
                entries.push(SubtitlePreviewEntry {
                    path: path.to_string_lossy().to_string(),
                    file_name,
                    season: Some(parsed.season),
                    episode: Some(parsed.episode),
                    status: "Out of scope".to_string(),
                    episode_title: None,
                    episode_rating_key: None,
                });
                continue;
            }
        }
        if let Some(scope_episode) = episode_number {
            if parsed.episode != scope_episode {
                entries.push(SubtitlePreviewEntry {
                    path: path.to_string_lossy().to_string(),
                    file_name,
                    season: Some(parsed.season),
                    episode: Some(parsed.episode),
                    status: "Out of scope".to_string(),
                    episode_title: None,
                    episode_rating_key: None,
                });
                continue;
            }
        }
        let ep = map.get(&(parsed.season, parsed.episode));
        if let Some(ep) = ep {
            matched += 1;
            entries.push(SubtitlePreviewEntry {
                path: path.to_string_lossy().to_string(),
                file_name,
                season: Some(parsed.season),
                episode: Some(parsed.episode),
                status: "Matched".to_string(),
                episode_title: Some(ep.title.clone()),
                episode_rating_key: Some(ep.rating_key.clone()),
            });
        } else {
            entries.push(SubtitlePreviewEntry {
                path: path.to_string_lossy().to_string(),
                file_name,
                season: Some(parsed.season),
                episode: Some(parsed.episode),
                status: "No match in Plex scope".to_string(),
                episode_title: None,
                episode_rating_key: None,
            });
        }
    }

    entries.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
    Ok(SubtitlePreview {
        total: entries.len(),
        matched,
        entries,
    })
}

#[tauri::command]
async fn upload_subtitles(
    app: AppHandle,
    server_url: String,
    token: String,
    items: Vec<SubtitleUploadItem>,
) -> Result<SubtitleUploadResult, String> {
    let mut uploaded = 0;
    let mut failed = Vec::new();
    for item in items {
        match plex::upload_subtitle_to_episode(
            &app,
            &server_url,
            &token,
            &item.episode_rating_key,
            &item.path,
        )
        .await
        {
            Ok(_) => uploaded += 1,
            Err(err) => failed.push(format!("{} -> {}", item.path, err)),
        }
    }
    Ok(SubtitleUploadResult { uploaded, failed })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            list_libraries,
            list_shows,
            list_seasons,
            list_episodes,
            preview_trash,
            purge_trash,
            tmdb_search_series,
            tmdb_list_episodes,
            preview_subtitles,
            upload_subtitles,
            scan_subtitle_streams,
            set_subtitle_variant,
            remove_uploaded_subtitles,
            save_text_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn scan_subtitle_streams(
    app: AppHandle,
    server_url: String,
    token: String,
    library_id: String,
    show_rating_key: String,
    season_rating_key: Option<String>,
) -> Result<plex::subtitles::SubtitleScan, String> {
    plex::scan_subtitle_streams(
        &app,
        &server_url,
        &token,
        &library_id,
        &show_rating_key,
        season_rating_key,
    )
    .await
}
#[tauri::command]
async fn set_subtitle_variant(
    app: AppHandle,
    server_url: String,
    token: String,
    library_id: String,
    show_rating_key: String,
    season_rating_key: Option<String>,
    variant: plex::subtitles::VariantKey,
) -> Result<plex::subtitles::SubtitleActionResult, String> {
    plex::set_subtitle_variant(
        &app,
        &server_url,
        &token,
        &library_id,
        &show_rating_key,
        season_rating_key,
        variant,
    )
    .await
}
#[tauri::command]
async fn remove_uploaded_subtitles(
    app: AppHandle,
    server_url: String,
    token: String,
    library_id: String,
    show_rating_key: String,
    season_rating_key: Option<String>,
    reviewed_keys: Vec<String>,
) -> Result<plex::subtitles::SubtitleActionResult, String> {
    plex::remove_uploaded_subtitles(
        &app,
        &server_url,
        &token,
        &library_id,
        &show_rating_key,
        season_rating_key,
        reviewed_keys,
    )
    .await
}
