use crate::settings::{get_client_id, normalize_plex_url};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use tauri::AppHandle;

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
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Accept", "application/json")
        .header("X-Plex-Client-Identifier", client_id)
        .header("X-Plex-Product", "PlexTools")
        .header("X-Plex-Version", "0.1.0")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("Plex responded with status {status}"));
    }
    response.json::<Value>().await.map_err(|e| e.to_string())
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
    let client = reqwest::Client::new();
    let response = client
        .delete(url)
        .header("X-Plex-Client-Identifier", client_id)
        .header("X-Plex-Product", "PlexTools")
        .header("X-Plex-Version", "0.1.0")
        .send()
        .await
        .map_err(|e| e.to_string())?;
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
    let url = format!(
        "{}/library/metadata/{}/subtitles",
        base, episode_rating_key
    );
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
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header("Accept", "text/plain, */*")
        .header("X-Plex-Client-Identifier", client_id)
        .header("X-Plex-Product", "PlexTools")
        .header("X-Plex-Version", "0.1.0")
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
        let section_type = dir
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
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
) -> Result<TrashPreview, String> {
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
    let mut titles = Vec::new();
    let mut count = 0;
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
            titles.push(item.title);
            count += 1;
            continue;
        }
        let missing_count = if !missing_media_ids.is_empty() {
            missing_media_ids.len()
        } else {
            missing_part_ids.len()
        };
        if missing_count > 0 {
            titles.push(format!("{} (missing part x{})", item.title, missing_count));
            count += missing_count;
        }
    }
    Ok(TrashPreview { count, titles })
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
                    if plex_delete(app, server_url, token, &primary_path).await.is_err() {
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

pub async fn fetch_episode_parts(
    app: &AppHandle,
    server_url: &str,
    token: &str,
    episode_rating_key: &str,
) -> Result<Vec<String>, String> {
    let value = plex_get_json(
        app,
        server_url,
        token,
        &format!("library/metadata/{}", episode_rating_key),
        vec![("includeMedia".to_string(), "1".to_string())],
    )
    .await?;
    let items = parse_metadata(&value);
    let part_ids = items
        .into_iter()
        .find(|item| item.rating_key == episode_rating_key)
        .map(|item| item.part_ids)
        .unwrap_or_default();
    Ok(part_ids)
}
