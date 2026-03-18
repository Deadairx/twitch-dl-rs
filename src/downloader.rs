use regex::Regex;
use reqwest::Client;
use thiserror::Error;
use url::Url;

use crate::cli::QualityPreference;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Failed to parse m3u8 playlist")] 
    Parse,
    #[error("Invalid playlist URL: {0}")]
    Url(#[from] url::ParseError),
}

#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub playlist_url: String,
    pub bandwidth: Option<u64>,
    pub resolution: Option<String>,
    pub codecs: Option<String>,
    pub name: Option<String>,
    pub is_audio_only: bool,
}

pub async fn resolve_stream(
    master_url: &str,
    preference: QualityPreference,
) -> Result<StreamInfo, DownloadError> {
    let client = Client::new();
    let playlist = client.get(master_url).send().await?.error_for_status()?.text().await?;
    let master = Url::parse(master_url)?;

    let bandwidth_re = Regex::new(r#"BANDWIDTH=(\d+)"#).map_err(|_| DownloadError::Parse)?;
    let resolution_re = Regex::new(r#"RESOLUTION=(\d+x\d+)"#).map_err(|_| DownloadError::Parse)?;
    let codecs_re = Regex::new(r#"CODECS=\"([^\"]+)\""#).map_err(|_| DownloadError::Parse)?;
    let name_re = Regex::new(r#"NAME=\"([^\"]+)\""#).map_err(|_| DownloadError::Parse)?;

    let mut variants = Vec::new();
    let mut pending_inf: Option<&str> = None;

    for line in playlist.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line.starts_with("#EXT-X-STREAM-INF:") {
            pending_inf = Some(line);
            continue;
        }

        if line.starts_with('#') {
            continue;
        }

        if let Some(stream_inf) = pending_inf.take() {
            let bandwidth = bandwidth_re
                .captures(stream_inf)
                .and_then(|caps| caps.get(1))
                .and_then(|value| value.as_str().parse::<u64>().ok());
            let resolution = resolution_re
                .captures(stream_inf)
                .and_then(|caps| caps.get(1))
                .map(|value| value.as_str().to_string());
            let codecs = codecs_re
                .captures(stream_inf)
                .and_then(|caps| caps.get(1))
                .map(|value| value.as_str().to_string());
            let name = name_re
                .captures(stream_inf)
                .and_then(|caps| caps.get(1))
                .map(|value| value.as_str().to_string());
            let playlist_url = master.join(line)?.to_string();
            let is_audio_only = playlist_url.contains("audio_only")
                || name
                    .as_deref()
                    .map(|value| value.to_ascii_lowercase().contains("audio"))
                    .unwrap_or(false)
                || resolution.is_none();

            variants.push(StreamInfo {
                playlist_url,
                bandwidth,
                resolution,
                codecs,
                name,
                is_audio_only,
            });
        }
    }

    if variants.is_empty() {
        return Ok(StreamInfo {
            playlist_url: master_url.to_string(),
            bandwidth: None,
            resolution: None,
            codecs: None,
            name: None,
            is_audio_only: false,
        });
    }

    let selected = match preference {
        QualityPreference::AudioOnly => variants
            .iter()
            .filter(|variant| variant.is_audio_only)
            .min_by_key(|variant| variant.bandwidth.unwrap_or(u64::MAX))
            .cloned()
            .or_else(|| {
                variants
                    .iter()
                    .min_by_key(|variant| variant.bandwidth.unwrap_or(u64::MAX))
                    .cloned()
            }),
        QualityPreference::Lowest => variants
            .iter()
            .min_by_key(|variant| variant.bandwidth.unwrap_or(u64::MAX))
            .cloned(),
        QualityPreference::Highest => variants
            .iter()
            .max_by_key(|variant| variant.bandwidth.unwrap_or_default())
            .cloned(),
    };

    selected.ok_or(DownloadError::Parse)
}
