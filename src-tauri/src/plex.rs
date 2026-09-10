use crate::settings::{get_client_id, normalize_plex_url};
use once_cell::sync::Lazy;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::time::Instant;
use tauri::AppHandle;

// All Plex transport helpers share this pool; creating it performs no network activity.
static PLEX_CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Library {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShowSummary {
    pub rating_key: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeasonSummary {
    pub rating_key: String,
    pub title: String,
    pub index: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpisodeSummary {
    pub rating_key: String,
    pub title: String,
    pub season_number: i32,
    pub episode_number: i32,
    pub part_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurgeResult {
    pub removed_count: usize,
    pub removed_titles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrashPreview {
    pub count: usize,
    pub titles: Vec<String>,
    pub diagnostics: Vec<String>,
}

struct PlexJsonResponse {
    value: Value,
    status: reqwest::StatusCode,
    response_ms: u128,
    body_ms: u128,
    parse_ms: u128,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PlexItem {
    rating_key: String,
    title: String,
    item_type: String,
    index: Option<i32>,
    parent_index: Option<i32>,
    parent_rating_key: Option<String>,
    grandparent_rating_key: Option<String>,
    is_trashed: bool,
    missing_part_ids: Vec<String>,
    missing_media_ids: Vec<String>,
    part_ids: Vec<String>,
}

async fn plex_get_json(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    path: &str,
    query: Vec<(String, String)>,
) -> Result<Value, String> {
    Ok(plex_get_json_timed(app, server_url, token, path, query)
        .await?
        .value)
}

async fn plex_get_json_timed(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    path: &str,
    query: Vec<(String, String)>,
) -> Result<PlexJsonResponse, String> {
    let base = normalize_plex_url(server_url)?;
    let url = format!("{}/{}", base, path.trim_start_matches('/'));
    let mut url = Url::parse(&url).map_err(|e| e.to_string())?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("X-Plex-Token", token);
        for (k, v) in query {
            pairs.append_pair(&k, &v);
        }
    }

    let client_id = get_client_id(app)?;
    let response_start = Instant::now();
    let response = PLEX_CLIENT
        .get(url)
        .timeout(std::time::Duration::from_secs(30))
        .header("Accept", "application/json")
        .header("X-Plex-Client-Identifier", client_id)
        .header("X-Plex-Product", "PlexSuite")
        .header("X-Plex-Version", "1.0.0")
        .send()
        .await
        .map_err(|e| e.without_url().to_string())?;
    let response_ms = response_start.elapsed().as_millis();
    let status = response.status();
    if !status.is_success() {
        return Err(format!("Plex responded with status {status}"));
    }
    let body_start = Instant::now();
    let body = response.text().await.map_err(|e| e.to_string())?;
    let body_ms = body_start.elapsed().as_millis();
    let parse_start = Instant::now();
    let value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    Ok(PlexJsonResponse {
        value,
        status,
        response_ms,
        body_ms,
        parse_ms: parse_start.elapsed().as_millis(),
    })
}

async fn plex_delete(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    path: &str,
) -> Result<(), String> {
    let base = normalize_plex_url(server_url)?;
    let url = format!("{}/{}", base, path.trim_start_matches('/'));
    let mut url = Url::parse(&url).map_err(|e| e.to_string())?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("X-Plex-Token", token);
    }

    let client_id = get_client_id(app)?;
    let response = PLEX_CLIENT
        .delete(url)
        .timeout(std::time::Duration::from_secs(30))
        .header("X-Plex-Client-Identifier", client_id)
        .header("X-Plex-Product", "PlexSuite")
        .header("X-Plex-Version", "1.0.0")
        .send()
        .await
        .map_err(|e| e.without_url().to_string())?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("Plex responded with status {status} for {path}"));
    }
    Ok(())
}

async fn plex_post_subtitle(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    episode_rating_key: &str,
    file_path: &str,
) -> Result<(), String> {
    let base = normalize_plex_url(server_url)?;
    let url = format!("{}/library/metadata/{}/subtitles", base, episode_rating_key);
    let mut url = Url::parse(&url).map_err(|e| e.to_string())?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("X-Plex-Token", token);
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("subtitle.srt")
            .to_string();
        let ext = Path::new(&file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("srt");
        pairs.append_pair("title", &file_name);
        pairs.append_pair("format", ext);
    }

    let client_id = get_client_id(app)?;
    let data = std::fs::read(file_path).map_err(|e| format!("Failed to read file: {e}"))?;
    let response = PLEX_CLIENT
        .post(url)
        .header("Accept", "text/plain, */*")
        .header("X-Plex-Client-Identifier", client_id)
        .header("X-Plex-Product", "PlexSuite")
        .header("X-Plex-Version", "1.0.0")
        .body(data)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("Upload failed with status {status}"));
    }
    Ok(())
}

fn parse_libraries(value: &Value) -> Vec<Library> {
    let directories = value
        .get("MediaContainer")
        .and_then(|mc| mc.get("Directory"))
        .and_then(|dir| dir.as_array())
        .cloned()
        .unwrap_or_default();
    let mut libraries = Vec::new();
    for dir in directories {
        let section_type = dir.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if section_type != "show" {
            continue;
        }
        if let (Some(id), Some(title)) = (
            dir.get("key").and_then(|v| v.as_str()),
            dir.get("title").and_then(|v| v.as_str()),
        ) {
            libraries.push(Library {
                id: id.to_string(),
                title: title.to_string(),
            });
        }
    }
    libraries
}

fn collect_missing_media(item: &Value) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut part_ids = Vec::new();
    let mut media_ids = Vec::new();
    let mut all_part_ids = Vec::new();
    let medias = item
        .get("Media")
        .and_then(|media| media.as_array())
        .cloned()
        .unwrap_or_default();
    for media in medias {
        let media_missing = media.get("deletedAt").is_some() || media.get("trash").is_some();
        if media_missing {
            let id = media
                .get("id")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string())
                .or_else(|| {
                    media
                        .get("id")
                        .and_then(|value| value.as_i64())
                        .map(|value| value.to_string())
                });
            if let Some(id) = id {
                media_ids.push(id);
            }
        }
        let parts = media
            .get("Part")
            .and_then(|part| part.as_array())
            .cloned()
            .unwrap_or_default();
        for part in parts {
            let is_missing = part.get("deletedAt").is_some()
                || part
                    .get("exists")
                    .and_then(|value| value.as_bool())
                    .map(|value| !value)
                    .unwrap_or(false);
            let id = part
                .get("id")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string())
                .or_else(|| {
                    part.get("id")
                        .and_then(|value| value.as_i64())
                        .map(|value| value.to_string())
                });
            if let Some(id) = id {
                all_part_ids.push(id.clone());
                if is_missing || media_missing {
                    part_ids.push(id);
                }
            }
        }
    }
    (part_ids, media_ids, all_part_ids)
}

fn parse_metadata(value: &Value) -> Vec<PlexItem> {
    let items = value
        .get("MediaContainer")
        .and_then(|mc| mc.get("Metadata"))
        .and_then(|meta| meta.as_array())
        .cloned()
        .unwrap_or_default();
    let mut parsed = Vec::new();
    for item in items {
        let rating_key = match item.get("ratingKey").and_then(|v| v.as_str()) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();
        let item_type = item
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let index = item.get("index").and_then(|v| v.as_i64()).map(|v| v as i32);
        let parent_index = item
            .get("parentIndex")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);
        let parent_rating_key = item
            .get("parentRatingKey")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let grandparent_rating_key = item
            .get("grandparentRatingKey")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let is_trashed = item.get("deletedAt").is_some() || item.get("trash").is_some();
        let (missing_part_ids, missing_media_ids, part_ids) = collect_missing_media(&item);
        parsed.push(PlexItem {
            rating_key,
            title,
            item_type,
            index,
            parent_index,
            parent_rating_key,
            grandparent_rating_key,
            is_trashed,
            missing_part_ids,
            missing_media_ids,
            part_ids,
        });
    }
    parsed
}

async fn fetch_missing_media(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    rating_key: &str,
) -> Result<(Vec<String>, Vec<String>), String> {
    let value = plex_get_json(
        app,
        server_url,
        token,
        &format!("library/metadata/{}", rating_key),
        vec![
            ("includeTrash".to_string(), "1".to_string()),
            ("includeMedia".to_string(), "1".to_string()),
        ],
    )
    .await?;
    let item = value
        .get("MediaContainer")
        .and_then(|mc| mc.get("Metadata"))
        .and_then(|meta| meta.as_array())
        .and_then(|arr| arr.first());
    if let Some(item) = item {
        let (missing_part_ids, missing_media_ids, _) = collect_missing_media(item);
        Ok((missing_part_ids, missing_media_ids))
    } else {
        Ok((Vec::new(), Vec::new()))
    }
}

pub async fn list_libraries(
    app: &AppHandle,
    server_url: &str,
    token: &str,
) -> Result<Vec<Library>, String> {
    let value = plex_get_json(app, server_url, token, "library/sections", vec![]).await?;
    Ok(parse_libraries(&value))
}

pub async fn list_shows(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    library_id: &str,
    query: Option<String>,
) -> Result<Vec<ShowSummary>, String> {
    let path = if query.as_ref().map(|q| q.trim().is_empty()).unwrap_or(true) {
        format!("library/sections/{}/all", library_id)
    } else {
        format!("library/sections/{}/search", library_id)
    };
    let mut params = vec![("type".to_string(), "2".to_string())];
    if let Some(q) = query {
        let trimmed = q.trim();
        if !trimmed.is_empty() {
            params.push(("query".to_string(), trimmed.to_string()));
        }
    }
    let value = plex_get_json(app, server_url, token, &path, params).await?;
    let mut items = parse_metadata(&value);
    items.retain(|item| item.item_type == "show");
    Ok(items
        .into_iter()
        .map(|item| ShowSummary {
            rating_key: item.rating_key,
            title: item.title,
        })
        .collect())
}

pub async fn list_seasons(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    show_rating_key: &str,
) -> Result<Vec<SeasonSummary>, String> {
    let path = format!("library/metadata/{}/children", show_rating_key);
    let value = plex_get_json(
        app,
        server_url,
        token,
        &path,
        vec![("type".to_string(), "3".to_string())],
    )
    .await?;
    let items = parse_metadata(&value);
    Ok(items
        .into_iter()
        .filter(|item| item.item_type == "season")
        .map(|item| SeasonSummary {
            rating_key: item.rating_key,
            title: item.title,
            index: item.index,
        })
        .collect())
}

pub async fn list_episodes(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    show_rating_key: &str,
    season_rating_key: Option<String>,
) -> Result<Vec<EpisodeSummary>, String> {
    let (path, params) = if let Some(season_key) = season_rating_key {
        (
            format!("library/metadata/{}/children", season_key),
            vec![
                ("type".to_string(), "4".to_string()),
                ("includeMedia".to_string(), "1".to_string()),
            ],
        )
    } else {
        (
            format!("library/metadata/{}/allLeaves", show_rating_key),
            vec![("includeMedia".to_string(), "1".to_string())],
        )
    };
    let value = plex_get_json(app, server_url, token, &path, params).await?;
    let items = parse_metadata(&value);
    Ok(items
        .into_iter()
        .filter(|item| item.item_type == "episode")
        .filter_map(|item| {
            Some(EpisodeSummary {
                rating_key: item.rating_key,
                title: item.title,
                season_number: item.parent_index?,
                episode_number: item.index?,
                part_ids: item.part_ids,
            })
        })
        .collect())
}

pub async fn preview_trash(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    library_id: &str,
    show_rating_key: &str,
    season_rating_key: Option<String>,
    debug: bool,
) -> Result<TrashPreview, String> {
    let total_start = Instant::now();
    let path = format!("library/sections/{}/all", library_id);
    let response = plex_get_json_timed(
        app,
        server_url,
        token,
        &path,
        vec![
            ("type".to_string(), "4".to_string()),
            ("includeTrash".to_string(), "1".to_string()),
            ("includeMedia".to_string(), "1".to_string()),
        ],
    )
    .await?;
    let mut diagnostics = Vec::new();
    if debug {
        diagnostics.push(format!(
            "Initial library metadata GET /{} -> {} in {} ms (body {} ms, JSON parse {} ms)",
            path, response.status, response.response_ms, response.body_ms, response.parse_ms
        ));
    }
    let parse_start = Instant::now();
    let parsed_items = parse_metadata(&response.value);
    if debug {
        diagnostics.push(format!(
            "Initial metadata parse: {} library items -> {} ms",
            parsed_items.len(),
            parse_start.elapsed().as_millis()
        ));
    }
    let mut titles = Vec::new();
    let mut count = 0;
    let filtering_start = Instant::now();
    for item in parsed_items {
        let show_match = item
            .grandparent_rating_key
            .as_ref()
            .map(|key| key == show_rating_key)
            .unwrap_or(false);
        if !show_match {
            continue;
        }
        if let Some(ref season_key) = season_rating_key {
            let season_match = item
                .parent_rating_key
                .as_ref()
                .map(|key| key == season_key)
                .unwrap_or(false);
            if !season_match {
                continue;
            }
        }
        let mut missing_part_ids = item.missing_part_ids.clone();
        let mut missing_media_ids = item.missing_media_ids.clone();
        if missing_part_ids.is_empty() && missing_media_ids.is_empty() && !item.is_trashed {
            let fallback_start = Instant::now();
            if let Ok((extra_parts, extra_media)) =
                fetch_missing_media(app, server_url, token, &item.rating_key).await
            {
                missing_part_ids = extra_parts;
                missing_media_ids = extra_media;
            }
            if debug {
                diagnostics.push(format!(
                    "fetch_missing_media ratingKey={} ({}) -> {} ms",
                    item.rating_key,
                    item.title,
                    fallback_start.elapsed().as_millis()
                ));
            }
        }
        let preview_title = match (item.parent_index, item.index) {
            (Some(season), Some(episode)) => {
                format!("S{:02}E{:02} - {}", season, episode, item.title)
            }
            _ => item.title.clone(),
        };
        if item.is_trashed {
            titles.push(preview_title);
            count += 1;
            continue;
        }
        let missing_count = if !missing_media_ids.is_empty() {
            missing_media_ids.len()
        } else {
            missing_part_ids.len()
        };
        if missing_count > 0 {
            titles.push(format!("{} (missing part x{})", preview_title, missing_count));
            count += missing_count;
        }
    }
    if debug {
        diagnostics.push(format!(
            "Filtering/local preview processing -> {} ms",
            filtering_start.elapsed().as_millis()
        ));
    }
    if debug {
        diagnostics.push(format!(
            "Preview complete: {} item(s) in {} ms",
            count,
            total_start.elapsed().as_millis()
        ));
    }
    Ok(TrashPreview {
        count,
        titles,
        diagnostics,
    })
}

pub async fn purge_trash(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    library_id: &str,
    show_rating_key: &str,
    season_rating_key: Option<String>,
) -> Result<PurgeResult, String> {
    let path = format!("library/sections/{}/all", library_id);
    let value = plex_get_json(
        app,
        server_url,
        token,
        &path,
        vec![
            ("type".to_string(), "4".to_string()),
            ("includeTrash".to_string(), "1".to_string()),
            ("includeMedia".to_string(), "1".to_string()),
        ],
    )
    .await?;
    let mut removed_titles = Vec::new();
    let mut removed_count = 0;
    for item in parse_metadata(&value) {
        let show_match = item
            .grandparent_rating_key
            .as_ref()
            .map(|key| key == show_rating_key)
            .unwrap_or(false);
        if !show_match {
            continue;
        }
        if let Some(ref season_key) = season_rating_key {
            let season_match = item
                .parent_rating_key
                .as_ref()
                .map(|key| key == season_key)
                .unwrap_or(false);
            if !season_match {
                continue;
            }
        }
        let mut missing_part_ids = item.missing_part_ids.clone();
        let mut missing_media_ids = item.missing_media_ids.clone();
        if missing_part_ids.is_empty() && missing_media_ids.is_empty() && !item.is_trashed {
            if let Ok((extra_parts, extra_media)) =
                fetch_missing_media(app, server_url, token, &item.rating_key).await
            {
                missing_part_ids = extra_parts;
                missing_media_ids = extra_media;
            }
        }
        if item.is_trashed {
            let delete_path = format!("library/metadata/{}", item.rating_key);
            plex_delete(app, server_url, token, &delete_path).await?;
            removed_titles.push(item.title);
            removed_count += 1;
            continue;
        }
        let missing_count = if !missing_media_ids.is_empty() {
            missing_media_ids.len()
        } else {
            missing_part_ids.len()
        };
        if missing_count > 0 {
            if !missing_media_ids.is_empty() {
                for media_id in &missing_media_ids {
                    let primary_path =
                        format!("library/metadata/{}/media/{}", item.rating_key, media_id);
                    if plex_delete(app, server_url, token, &primary_path)
                        .await
                        .is_err()
                    {
                        let fallback_path = format!("library/media/{}", media_id);
                        plex_delete(app, server_url, token, &fallback_path).await?;
                    }
                }
            } else if !missing_part_ids.is_empty() {
                for part_id in &missing_part_ids {
                    let delete_path = format!("library/parts/{}", part_id);
                    plex_delete(app, server_url, token, &delete_path).await?;
                }
            }
            removed_titles.push(format!("{} (missing part x{})", item.title, missing_count));
            removed_count += missing_count;
        }
    }
    Ok(PurgeResult {
        removed_count,
        removed_titles,
    })
}

pub async fn upload_subtitle_to_episode(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    episode_rating_key: &str,
    file_path: &str,
) -> Result<(), String> {
    plex_post_subtitle(app, server_url, token, episode_rating_key, file_path).await
}

pub mod subtitles;
use subtitles::*;

async fn fetch_subtitle_episode(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    ep: &EpisodeSummary,
    library_id: &str,
    show_key: &str,
    season_key: Option<&str>,
) -> Result<(Vec<SubtitlePart>, PlexJsonResponse, u128), String> {
    let response = plex_get_json_timed(
        app,
        server_url,
        token,
        &format!("library/metadata/{}", ep.rating_key),
        vec![
            ("includeMedia".into(), "1".into()),
            ("includeAllStreams".into(), "1".into()),
        ],
    )
    .await?;
    let processing_start = Instant::now();
    let parts = parse_subtitle_episode(
        &response.value,
        &ep.rating_key,
        library_id,
        show_key,
        season_key,
    )?;
    Ok((parts, response, processing_start.elapsed().as_millis()))
}

fn parse_subtitle_episode(
    value: &Value,
    rating_key: &str,
    library_id: &str,
    show_key: &str,
    season_key: Option<&str>,
) -> Result<Vec<SubtitlePart>, String> {
    let item = value["MediaContainer"]["Metadata"]
        .as_array()
        .and_then(|a| a.first())
        .ok_or("Episode metadata unavailable")?;
    if scalar(&item["ratingKey"]).as_deref() != Some(rating_key)
        || scalar(&item["grandparentRatingKey"]).as_deref() != Some(show_key)
        || scalar(&item["librarySectionID"]).as_deref() != Some(library_id)
        || season_key
            .map(|key| scalar(&item["parentRatingKey"]).as_deref() != Some(key))
            .unwrap_or(false)
    {
        return Err("Episode metadata does not match the selected scope".into());
    }
    let mut parts = Vec::new();
    for media in item["Media"].as_array().into_iter().flatten() {
        for part in media["Part"].as_array().into_iter().flatten() {
            let mut streams = Vec::new();
            for raw in part["Stream"].as_array().into_iter().flatten() {
                match scalar(&raw["streamType"]).as_deref() {
                    Some("3") => streams.push(
                        parse_stream(raw).ok_or("Malformed subtitle flags; episode skipped")?,
                    ),
                    Some("1" | "2" | "4") => {}
                    _ => return Err("Unknown stream type; episode skipped".into()),
                }
            }
            parts.push(SubtitlePart {
                id: scalar(&part["id"]).unwrap_or_default(),
                streams,
            });
        }
    }
    Ok(parts)
}

pub async fn scan_subtitle_streams(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    library_id: &str,
    show_rating_key: &str,
    season_rating_key: Option<String>,
    debug: bool,
) -> Result<SubtitleScan, String> {
    if !numeric_id(library_id)
        || !numeric_id(show_rating_key)
        || season_rating_key
            .as_ref()
            .map(|s| !numeric_id(s))
            .unwrap_or(false)
    {
        return Err("Invalid Plex scope".into());
    }
    let total_start = Instant::now();
    let scope_path = season_rating_key
        .as_ref()
        .map(|key| format!("library/metadata/{}/children", key))
        .unwrap_or_else(|| format!("library/metadata/{}/allLeaves", show_rating_key));
    let scope_start = Instant::now();
    let episodes = list_episodes(
        app,
        server_url,
        token,
        show_rating_key,
        season_rating_key.clone(),
    )
    .await?;
    let mut diagnostics = Vec::new();
    if debug {
        diagnostics.push(format!(
            "Episode scope GET /{} -> {} episodes in {} ms",
            scope_path,
            episodes.len(),
            scope_start.elapsed().as_millis()
        ));
    }
    let mut results = Vec::new();
    for ep in episodes {
        let episode_start = Instant::now();
        let result = fetch_subtitle_episode(
            app,
            server_url,
            token,
            &ep,
            library_id,
            show_rating_key,
            season_rating_key.as_deref(),
        )
        .await;
        let (parts, error) = match result {
            Ok((parts, response, process_ms)) => {
                if debug {
                    let tracks = parts.iter().map(|part| part.streams.len()).sum::<usize>();
                    diagnostics.push(format!("{} ratingKey={} metadata GET /library/metadata/{} -> {} in {} ms (body {} ms, JSON parse {} ms)", ep_code(&ep), ep.rating_key, ep.rating_key, response.status, response.response_ms, response.body_ms, response.parse_ms));
                    diagnostics.push(format!(
                        "{} parse/process -> {} ms, {} subtitle streams",
                        ep_code(&ep),
                        process_ms,
                        tracks
                    ));
                }
                (parts, None)
            }
            Err(e) => {
                if debug {
                    diagnostics.push(format!(
                        "{} ratingKey={} metadata GET /library/metadata/{} failed after {} ms: {}",
                        ep_code(&ep),
                        ep.rating_key,
                        ep.rating_key,
                        episode_start.elapsed().as_millis(),
                        e
                    ));
                }
                (vec![], Some(e))
            }
        };
        results.push(SubtitleEpisode {
            rating_key: ep.rating_key,
            code: format!("S{:02}E{:02}", ep.season_number, ep.episode_number),
            parts,
            error,
        });
    }
    let mut scan = summarize(results);
    if debug {
        diagnostics.push(format!(
            "Scan complete: {} episodes, {} subtitle tracks, total {} ms",
            scan.scanned,
            scan.tracks,
            total_start.elapsed().as_millis()
        ));
    }
    scan.diagnostics = diagnostics;
    Ok(scan)
}

fn ep_code(ep: &EpisodeSummary) -> String {
    format!("S{:02}E{:02}", ep.season_number, ep.episode_number)
}

async fn plex_select_subtitle(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    part: &str,
    stream: &str,
) -> Result<(), String> {
    if !numeric_id(part) || !numeric_id(stream) {
        return Err("Invalid part or stream ID".into());
    }
    let url = format!("{}/library/parts/{}", normalize_plex_url(server_url)?, part);
    let response = PLEX_CLIENT
        .put(url)
        // Each part is matched independently: never propagate an ID to a different part.
        .query(&[("subtitleStreamID", stream), ("allParts", "0")])
        .header("X-Plex-Token", token)
        .header("X-Plex-Client-Identifier", get_client_id(app)?)
        .header("X-Plex-Product", "PlexSuite")
        .header("X-Plex-Version", "1.0.0")
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| e.without_url().to_string())?;
    if !response.status().is_success() {
        return Err(format!("Plex responded with status {}", response.status()));
    }
    Ok(())
}

pub async fn set_subtitle_variant(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    library_id: &str,
    show_rating_key: &str,
    season_rating_key: Option<String>,
    variant: VariantKey,
) -> Result<SubtitleActionResult, String> {
    let scan = scan_subtitle_streams(
        app,
        server_url,
        token,
        library_id,
        show_rating_key,
        season_rating_key,
        false,
    )
    .await?;
    let mut result = SubtitleActionResult {
        errors: scan.errors,
        ..Default::default()
    };
    for ep in scan.episodes {
        if ep.error.is_some() {
            continue;
        }
        let mut found = false;
        let mut failed = false;
        for part in ep.parts {
            let matching = part.streams.iter().filter(|s| s.variant == variant);
            // Prefer an already selected equivalent track, otherwise use metadata order.
            let stream = matching
                .clone()
                .find(|s| s.selected)
                .or_else(|| matching.into_iter().next());
            if let Some(stream) = stream {
                found = true;
                let update = plex_select_subtitle(
                    app,
                    server_url,
                    token,
                    &part.id,
                    stream.id.as_deref().unwrap_or(""),
                )
                .await;
                if let Err(e) = update {
                    failed = true;
                    result
                        .errors
                        .push(format!("{} (part {}): {}", ep.code, part.id, e));
                }
            }
        }
        if found && !failed {
            result.applied += 1;
        }
        if !found {
            result.missing.push(ep.code);
        }
    }
    Ok(result)
}

pub async fn remove_selected_subtitles(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    library_id: &str,
    show_rating_key: &str,
    season_rating_key: Option<String>,
    reviewed_keys: Vec<String>,
    categories: Vec<SubtitleCategory>,
) -> Result<SubtitleActionResult, String> {
    if categories.is_empty() {
        return Ok(SubtitleActionResult::default());
    }
    // Only keys shown in the confirmed preview are eligible. All metadata comes from Plex again.
    let scan = scan_subtitle_streams(
        app,
        server_url,
        token,
        library_id,
        show_rating_key,
        season_rating_key.clone(),
        false,
    )
    .await?;
    let safe = candidates(&scan.episodes, &categories);
    let reviewed: std::collections::BTreeSet<_> = reviewed_keys.into_iter().collect();
    let mut result = SubtitleActionResult {
        errors: scan.errors.clone(),
        ..Default::default()
    };
    for key in reviewed {
        if !safe.contains(&key) {
            result.skipped_unsafe += 1;
            continue;
        }
        // Re-fetch the owning episode immediately before deletion, not just at batch start.
        let owner = scan.episodes.iter().find(|e| {
            e.parts
                .iter()
                .any(|p| p.streams.iter().any(|s| s.key.as_ref() == Some(&key)))
        });
        if let Some(owner) = owner {
            let ep = EpisodeSummary {
                rating_key: owner.rating_key.clone(),
                title: String::new(),
                season_number: 0,
                episode_number: 0,
                part_ids: vec![],
            };
            match fetch_subtitle_episode(
                app,
                server_url,
                token,
                &ep,
                library_id,
                show_rating_key,
                season_rating_key.as_deref(),
            )
            .await
            {
                Ok((parts, _, _)) => {
                    let fresh = SubtitleEpisode {
                        parts,
                        ..owner.clone()
                    };
                    if !candidates(&[fresh], &categories).contains(&key) {
                        result.skipped_unsafe += 1;
                        continue;
                    }
                    match plex_delete(app, server_url, token, &key).await {
                        Ok(()) => result.removed += 1,
                        Err(e) => result.errors.push(format!("{}: {}", owner.code, e)),
                    }
                }
                Err(e) => result.errors.push(format!("{}: {}", owner.code, e)),
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod subtitle_metadata_tests {
    use super::*;
    use serde_json::json;
    fn metadata() -> Value {
        json!({"MediaContainer":{"Metadata":[{"ratingKey":"10","grandparentRatingKey":"2","parentRatingKey":"3","librarySectionID":1,"Media":[
            {"Part":[{"id":20,"Stream":[{"id":30,"streamType":3,"index":0,"languageTag":"fr-FR"}, {"streamType":1}]}, {"id":21,"Stream":null}]},
            {"Part":[{"id":22,"Stream":[{"id":31,"streamType":3,"index":-1,"key":"/library/streams/31","transient":"0"}]}]}
        ]}]}})
    }
    #[test]
    fn validates_library_show_season_and_episode() {
        let value = metadata();
        assert!(parse_subtitle_episode(&value, "10", "1", "2", Some("3")).is_ok());
        assert!(parse_subtitle_episode(&value, "10", "1", "2", None).is_ok());
        for (ep, lib, show, season) in [
            ("11", "1", "2", "3"),
            ("10", "9", "2", "3"),
            ("10", "1", "9", "3"),
            ("10", "1", "2", "9"),
        ] {
            assert!(parse_subtitle_episode(&value, ep, lib, show, Some(season)).is_err());
        }
    }
    #[test]
    fn handles_multiple_versions_parts_empty_and_malformed_streams() {
        let mut value = metadata();
        let parts = parse_subtitle_episode(&value, "10", "1", "2", None).unwrap();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts.iter().map(|p| p.streams.len()).sum::<usize>(), 2);
        assert_eq!(parts[2].streams[0].source, "Plex Uploaded");
        value["MediaContainer"]["Metadata"][0]["Media"][0]["Part"][0]["Stream"][0]["forced"] =
            json!("invalid");
        assert!(parse_subtitle_episode(&value, "10", "1", "2", None).is_err());
        value["MediaContainer"]["Metadata"][0]["Media"] = Value::Null;
        assert!(parse_subtitle_episode(&value, "10", "1", "2", None)
            .unwrap()
            .is_empty());
        assert!(parse_subtitle_episode(&json!({}), "10", "1", "2", None).is_err());
    }
}
