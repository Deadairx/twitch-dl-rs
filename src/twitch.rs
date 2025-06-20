// Placeholder for Twitch API interaction and video info extraction 

use regex::Regex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TwitchError {
    #[error("Invalid Twitch video URL")] 
    InvalidUrl,
    #[error("Failed to extract video ID")] 
    VideoIdNotFound,
}

/// Extracts the video ID from a Twitch video URL.
pub fn extract_video_id(url: &str) -> Result<String, TwitchError> {
    // Example URLs:
    // https://www.twitch.tv/videos/123456789
    // https://m.twitch.tv/videos/123456789
    let re = Regex::new(r"twitch\.tv/videos/(\d+)").map_err(|_| TwitchError::InvalidUrl)?;
    if let Some(caps) = re.captures(url) {
        if let Some(id) = caps.get(1) {
            return Ok(id.as_str().to_string());
        }
    }
    Err(TwitchError::VideoIdNotFound)
}

/// Constructs the m3u8 URL for a given video ID and optional auth token.
pub fn get_m3u8_url(video_id: &str, auth_token: Option<&str>) -> String {
    let base = format!("https://usher.ttvnw.net/vod/{}.m3u8", video_id);
    match auth_token {
        Some(token) => format!("{}?nauth={}", base, urlencoding::encode(token)),
        None => base,
    }
} 