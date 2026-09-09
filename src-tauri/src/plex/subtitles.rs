//! Subtitle metadata and conservative source classification. HTTP stays in plex.rs.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleStream {
    pub id: Option<String>,
    pub index: Option<i64>,
    pub stream_type: Option<i64>,
    pub codec: Option<String>,
    pub format: Option<String>,
    pub language: Option<String>,
    pub language_code: Option<String>,
    pub language_tag: Option<String>,
    pub title: Option<String>,
    pub display_title: Option<String>,
    pub extended_display_title: Option<String>,
    pub forced: bool,
    pub hearing_impaired: bool,
    pub default: bool,
    pub selected: bool,
    pub key: Option<String>,
    pub source_key: Option<String>,
    pub provider_title: Option<String>,
    pub user_id: Option<String>,
    pub location: Option<String>,
    pub transient: Option<i64>,
    pub source: String,
    pub variant: VariantKey,
}

pub fn scalar(v: &Value) -> Option<String> {
    v.as_str()
        .map(str::to_owned)
        .or_else(|| v.as_i64().map(|n| n.to_string()))
}
fn integer(v: &Value) -> Option<i64> {
    v.as_i64().or_else(|| v.as_str()?.trim().parse().ok())
}
fn flag(v: &Value) -> Option<bool> {
    if v.is_null() {
        return Some(false);
    }
    if let Some(value) = v.as_bool() {
        return Some(value);
    }
    match v.as_str() {
        Some("true") => return Some(true),
        Some("false") => return Some(false),
        _ => {}
    }
    match integer(v) {
        Some(0) => Some(false),
        Some(1) => Some(true),
        _ => None,
    }
}
pub fn numeric_id(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}
fn stream_key(s: &SubtitleStream) -> bool {
    s.key
        .as_deref()
        .and_then(|k| k.strip_prefix("/library/streams/"))
        .map(|id| numeric_id(id) && s.id.as_deref() == Some(id))
        .unwrap_or(false)
}
pub fn is_plex_uploaded_subtitle(s: &SubtitleStream) -> bool {
    s.stream_type == Some(3) && s.index == Some(-1) && stream_key(s) && s.transient == Some(0)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct VariantKey {
    pub language_tag: String,
    pub forced: bool,
    pub hearing_impaired: bool,
}

pub fn parse_stream(v: &Value) -> Option<SubtitleStream> {
    if integer(&v["streamType"]) != Some(3) {
        return None;
    }
    let text = |k: &str| scalar(&v[k]);
    let mut s = SubtitleStream {
        id: text("id"),
        index: integer(&v["index"]),
        stream_type: Some(3),
        codec: text("codec"),
        format: text("format"),
        language: text("language"),
        language_code: text("languageCode"),
        language_tag: text("languageTag"),
        title: text("title"),
        display_title: text("displayTitle"),
        extended_display_title: text("extendedDisplayTitle"),
        forced: flag(&v["forced"])?,
        hearing_impaired: flag(&v["hearingImpaired"])?,
        default: flag(&v["default"]).unwrap_or(false),
        selected: flag(&v["selected"]).unwrap_or(false),
        key: text("key"),
        source_key: text("sourceKey"),
        provider_title: text("providerTitle"),
        user_id: text("userID"),
        location: text("location"),
        transient: integer(&v["transient"]),
        ..Default::default()
    };
    // An unrecognised transient value must never become a sidecar or upload.
    s.source = if is_plex_uploaded_subtitle(&s) {
        "Plex Uploaded"
    } else if s.index.map(|i| i >= 0).unwrap_or(false)
        && s.key.is_none()
        && v["transient"].is_null()
    {
        "Embedded"
    } else if s.index == Some(-1) && stream_key(&s) && v["transient"].is_null() {
        "Physical Sidecar"
    } else {
        "Unknown External"
    }
    .into();
    let tag = s
        .language_tag
        .as_deref()
        .filter(|s| !s.trim().is_empty() && *s != "und")
        .or(s
            .language_code
            .as_deref()
            .filter(|s| !s.trim().is_empty() && *s != "und"))
        .or(s.language.as_deref())
        .unwrap_or("und");
    s.variant = VariantKey {
        language_tag: tag.trim().replace('_', "-").to_lowercase(),
        forced: s.forced,
        hearing_impaired: s.hearing_impaired,
    };
    Some(s)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitlePart {
    pub id: String,
    pub streams: Vec<SubtitleStream>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleEpisode {
    pub rating_key: String,
    pub code: String,
    pub parts: Vec<SubtitlePart>,
    pub error: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleVariant {
    pub key: VariantKey,
    pub language: String,
    pub language_code: Option<String>,
    pub episode_count: usize,
    pub total_episodes: usize,
    pub selected_count: usize,
    pub missing: Vec<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleScan {
    pub episodes: Vec<SubtitleEpisode>,
    pub variants: Vec<SubtitleVariant>,
    pub counts: BTreeMap<String, usize>,
    pub tracks: usize,
    pub scanned: usize,
    pub errors: Vec<String>,
}
pub fn candidates(episodes: &[SubtitleEpisode]) -> BTreeSet<String> {
    // Conflicting observations of the same key veto deletion.
    let mut safe = BTreeSet::new();
    let mut unsafe_keys = BTreeSet::new();
    for s in episodes
        .iter()
        .flat_map(|e| &e.parts)
        .flat_map(|p| &p.streams)
    {
        if let Some(key) = &s.key {
            if is_plex_uploaded_subtitle(s) {
                safe.insert(key.clone());
            } else {
                unsafe_keys.insert(key.clone());
            }
        }
    }
    safe.retain(|k| !unsafe_keys.contains(k));
    safe
}
pub fn summarize(episodes: Vec<SubtitleEpisode>) -> SubtitleScan {
    let mut variants = BTreeMap::new();
    let mut counts: BTreeMap<String, usize> = [
        "Embedded",
        "Physical Sidecar",
        "Plex Uploaded",
        "Unknown External",
    ]
    .into_iter()
    .map(|s| (s.into(), 0))
    .collect();
    let mut tracks = 0;
    for ep in &episodes {
        for s in ep.parts.iter().flat_map(|p| &p.streams) {
            tracks += 1;
            *counts.entry(s.source.clone()).or_default() += 1;
            variants
                .entry(s.variant.clone())
                .or_insert_with(|| SubtitleVariant {
                    key: s.variant.clone(),
                    language: s
                        .language
                        .clone()
                        .unwrap_or_else(|| s.variant.language_tag.clone()),
                    language_code: s.language_code.clone(),
                    episode_count: 0,
                    total_episodes: episodes.len(),
                    selected_count: 0,
                    missing: vec![],
                });
        }
    }
    for variant in variants.values_mut() {
        for ep in &episodes {
            if ep.error.is_some() {
                continue;
            }
            let matching: Vec<_> = ep
                .parts
                .iter()
                .flat_map(|p| &p.streams)
                .filter(|s| s.variant == variant.key)
                .collect();
            if matching.is_empty() {
                variant.missing.push(ep.code.clone());
            } else {
                variant.episode_count += 1;
            }
            if matching.iter().any(|s| s.selected) {
                variant.selected_count += 1;
            }
        }
    }
    let errors = episodes
        .iter()
        .filter_map(|e| e.error.as_ref().map(|err| format!("{}: {}", e.code, err)))
        .collect();
    SubtitleScan {
        scanned: episodes.iter().filter(|e| e.error.is_none()).count(),
        episodes,
        variants: variants.into_values().collect(),
        counts,
        tracks,
        errors,
    }
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleActionResult {
    pub applied: usize,
    pub removed: usize,
    pub missing: Vec<String>,
    pub errors: Vec<String>,
    pub skipped_unsafe: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn stream(tag: &str, sdh: bool, forced: bool) -> SubtitleStream {
        parse_stream(&json!({"streamType":3,"id":12,"index":-1,"key":"/library/streams/12","languageTag":tag,"hearingImpaired":sdh,"forced":forced})).unwrap()
    }
    #[test]
    fn exact_variants() {
        let mut keys = BTreeSet::new();
        for tag in [
            "fr-FR", "fr-CA", "es-ES", "es-419", "pt-BR", "pt-PT", "zh-Hans", "zh-Hant",
        ] {
            for sdh in [false, true] {
                for forced in [false, true] {
                    keys.insert(stream(tag, sdh, forced).variant);
                }
            }
        }
        assert_eq!(keys.len(), 32);
        assert_eq!(
            stream("fr-FR", false, false).variant,
            stream("FR-fr", false, false).variant
        );
    }
    #[test]
    fn fail_safe_classification() {
        for transient in [
            json!(null),
            json!(1),
            json!("garbage"),
            json!(false),
            json!({}),
            json!(0.5),
        ] {
            let s = parse_stream(&json!({"streamType":3,"id":12,"index":-1,"key":"/library/streams/12","transient":transient})).unwrap();
            assert!(!is_plex_uploaded_subtitle(&s));
        }
        for transient in [json!(0), json!("0")] {
            let mut s = parse_stream(&json!({"streamType":3,"id":12,"index":-1,"key":"/library/streams/12","transient":transient})).unwrap();
            assert!(is_plex_uploaded_subtitle(&s));
            s.key = Some("/library/metadata/12".into());
            assert!(!is_plex_uploaded_subtitle(&s));
        }
        assert_eq!(stream("fr", false, false).source, "Physical Sidecar");
        let s = parse_stream(&json!({"streamType":3,"index":0})).unwrap();
        assert_eq!(s.source, "Embedded");
        assert!(!is_plex_uploaded_subtitle(&s));
    }
    fn episode(code: &str, streams: Vec<SubtitleStream>) -> SubtitleEpisode {
        SubtitleEpisode {
            rating_key: code.into(),
            code: code.into(),
            parts: vec![SubtitlePart {
                id: "1".into(),
                streams,
            }],
            error: None,
        }
    }
    #[test]
    fn coverage_selected_and_candidates() {
        let mut upload = stream("fr-FR", false, false);
        upload.transient = Some(0);
        upload.selected = true;
        let sidecar = stream("fr-CA", false, false);
        let embedded = parse_stream(&json!({"streamType":3,"index":0})).unwrap();
        let episodes = vec![
            episode("S01E01", vec![upload.clone(), upload.clone(), embedded]),
            episode("S01E02", vec![]),
        ];
        assert_eq!(candidates(&episodes).len(), 1);
        let scan = summarize(episodes);
        let v = scan
            .variants
            .iter()
            .find(|v| v.key == upload.variant)
            .unwrap();
        assert_eq!(
            (v.episode_count, v.selected_count, v.total_episodes),
            (1, 1, 2)
        );
        assert_eq!(v.missing, vec!["S01E02"]);
        assert!(candidates(&[episode("1", vec![sidecar.clone()])]).is_empty());
        assert!(candidates(&[episode("1", vec![upload, sidecar])]).is_empty());
    }

    #[test]
    fn fallback_flags_and_unknown_external() {
        let generic = parse_stream(&json!({"streamType":"3","languageTag":"fr","languageCode":"fra","forced":"1","hearingImpaired":"true"})).unwrap();
        assert_eq!(generic.variant.language_tag, "fr");
        assert!(generic.variant.forced && generic.variant.hearing_impaired);
        let code =
            parse_stream(&json!({"streamType":3,"languageTag":null,"languageCode":"jpn"})).unwrap();
        assert_eq!(code.variant.language_tag, "jpn");
        let name = parse_stream(&json!({"streamType":3,"language":"Japanese"})).unwrap();
        assert_eq!(name.variant.language_tag, "japanese");
        assert!(parse_stream(&json!({"streamType":3,"forced":"invalid"})).is_none());
        assert!(parse_stream(&json!({"streamType":"invalid"})).is_none());
        let unknown = parse_stream(&json!({"streamType":3,"index":-1,"id":12,"key":"/library/streams/12","transient":"unknown"})).unwrap();
        assert_eq!(unknown.source, "Unknown External");
        assert!(candidates(&[episode("1", vec![unknown])]).is_empty());
    }

    #[test]
    fn failed_episodes_are_not_reported_as_missing() {
        let mut failed = episode("S01E02", vec![]);
        failed.error = Some("Network failure".into());
        let scan = summarize(vec![
            episode("S01E01", vec![stream("en", false, false)]),
            failed,
        ]);
        assert_eq!(scan.scanned, 1);
        assert_eq!(scan.errors.len(), 1);
        assert!(scan.variants[0].missing.is_empty());
        assert_eq!(scan.variants[0].total_episodes, 2);
    }
}
