use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TmdbSeries {
    pub id: i64,
    pub name: String,
    pub first_air_year: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TmdbEpisode {
    pub season_number: i32,
    pub episode_number: i32,
    pub name: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<SearchSeries>,
}

#[derive(Debug, Deserialize)]
struct SearchSeries {
    id: i64,
    name: String,
    first_air_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ShowDetails {
    seasons: Vec<TmdbSeasonRef>,
}

#[derive(Debug, Deserialize)]
struct TmdbSeasonRef {
    season_number: i32,
}

#[derive(Debug, Deserialize)]
struct SeasonDetails {
    episodes: Vec<TmdbEpisodeRef>,
}

#[derive(Debug, Deserialize)]
struct TmdbEpisodeRef {
    episode_number: i32,
    name: String,
}

fn base_url() -> Result<Url, String> {
    Url::parse("https://api.themoviedb.org/3/").map_err(|e| e.to_string())
}

pub async fn search_series(api_key: &str, query: &str) -> Result<Vec<TmdbSeries>, String> {
    if api_key.trim().is_empty() {
        return Err("TMDb API key is missing.".to_string());
    }
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut url = base_url()?.join("search/tv").map_err(|e| e.to_string())?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("api_key", api_key);
        pairs.append_pair("query", query.trim());
    }

    let response = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("TMDb responded with status {}", response.status()));
    }
    let data: SearchResponse = response.json().await.map_err(|e| e.to_string())?;
    Ok(data
        .results
        .into_iter()
        .map(|s| TmdbSeries {
            id: s.id,
            name: s.name,
            first_air_year: s.first_air_date.as_ref().map(|d| d.get(0..4).unwrap_or("").to_string()),
        })
        .collect())
}

pub async fn list_episodes(api_key: &str, series_id: i64) -> Result<Vec<TmdbEpisode>, String> {
    if api_key.trim().is_empty() {
        return Err("TMDb API key is missing.".to_string());
    }
    let client = reqwest::Client::new();
    let mut show_url = base_url()?
        .join(&format!("tv/{}", series_id))
        .map_err(|e| e.to_string())?;
    {
        let mut pairs = show_url.query_pairs_mut();
        pairs.append_pair("api_key", api_key);
    }
    let show_resp = client
        .get(show_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !show_resp.status().is_success() {
        return Err(format!("TMDb responded with status {}", show_resp.status()));
    }
    let show: ShowDetails = show_resp.json().await.map_err(|e| e.to_string())?;

    let mut all = Vec::new();
    for season in show.seasons {
        let mut season_url = base_url()?
            .join(&format!("tv/{}/season/{}", series_id, season.season_number))
            .map_err(|e| e.to_string())?;
        {
            let mut pairs = season_url.query_pairs_mut();
            pairs.append_pair("api_key", api_key);
        }
        let season_resp = client
            .get(season_url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !season_resp.status().is_success() {
            return Err(format!(
                "TMDb responded with status {} for season {}",
                season_resp.status(),
                season.season_number
            ));
        }
        let season_details: SeasonDetails =
            season_resp.json().await.map_err(|e| e.to_string())?;
        for ep in season_details.episodes {
            let code = format!("S{:02}E{:02}", season.season_number, ep.episode_number);
            all.push(TmdbEpisode {
                season_number: season.season_number,
                episode_number: ep.episode_number,
                name: ep.name,
                code,
            });
        }
    }
    Ok(all)
}
