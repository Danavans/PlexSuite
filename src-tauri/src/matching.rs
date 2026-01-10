use once_cell::sync::Lazy;
use regex::Regex;

static RE_SXXEYY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)S(\d{1,2})[.\s_-]*E(\d{1,3})").unwrap());
static RE_X_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(\d{1,2})x(\d{1,3})\b").unwrap());
static RE_WORDS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(?:season|saison)\s*(\d{1,2})\s*(?:episode|ep)\s*(\d{1,3})\b")
        .unwrap()
});
static RE_E_ONLY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bE(?:p|pisode)?[.\s_-]?(\d{1,3})\b").unwrap());
static RE_TRAILING_02: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)[\s._-](\d{1,3})(?:\D|$)").unwrap());
static RE_MULTI_JOIN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bS(\d{1,2})[.\s_-]*E(\d{1,3})(?:[.\s_-]*E(\d{1,3})|\s*-\s*(\d{1,3}))\b")
        .unwrap()
});

#[derive(Debug, Clone, Copy)]
pub struct EpisodeCode {
    pub season: i32,
    pub episode: i32,
}

pub fn parse_episode_code(text: &str, season_hint: Option<i32>) -> Option<EpisodeCode> {
    if let Some(caps) = RE_SXXEYY.captures(text) {
        return Some(EpisodeCode {
            season: caps[1].parse().ok()?,
            episode: caps[2].parse().ok()?,
        });
    }

    if let Some(caps) = RE_X_PATTERN.captures(text) {
        return Some(EpisodeCode {
            season: caps[1].parse().ok()?,
            episode: caps[2].parse().ok()?,
        });
    }

    if let Some(caps) = RE_WORDS.captures(text) {
        return Some(EpisodeCode {
            season: caps[1].parse().ok()?,
            episode: caps[2].parse().ok()?,
        });
    }

    if let Some(caps) = RE_MULTI_JOIN.captures(text) {
        return Some(EpisodeCode {
            season: caps[1].parse().ok()?,
            episode: caps[2].parse().ok()?,
        });
    }

    if let Some(season) = season_hint {
        if let Some(caps) = RE_E_ONLY.captures(text) {
            return Some(EpisodeCode {
                season,
                episode: caps[1].parse().ok()?,
            });
        }

        if let Some(caps) = RE_TRAILING_02.captures(text) {
            return Some(EpisodeCode {
                season,
                episode: caps[1].parse().ok()?,
            });
        }
    }

    None
}
