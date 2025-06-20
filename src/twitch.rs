// Placeholder for Twitch API interaction and video info extraction 

use regex::Regex;
use thiserror::Error;
use reqwest::Client;
use serde::Deserialize;
use serde_json;

#[derive(Debug, Error)]
pub enum TwitchError {
    #[error("Invalid Twitch video URL")] 
    InvalidUrl,
    #[error("Failed to extract video ID")] 
    VideoIdNotFound,
    #[error("Failed to get access token: {0}")]
    AccessToken(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, Deserialize)]
struct GqlAccessTokenData {
    signature: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct GqlAccessTokenResponse {
    data: Option<GqlAccessTokenDataWrapper>,
}

#[derive(Debug, Deserialize)]
struct GqlAccessTokenDataWrapper {
    videoPlaybackAccessToken: Option<GqlAccessTokenData>,
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

/// Fetches the VOD access token and signature from Twitch GQL API using a plain query.
pub async fn fetch_vod_access_token(video_id: &str, oauth_token: &str) -> Result<(String, String), TwitchError> {
    let client = Client::new();
    let url = "https://gql.twitch.tv/gql";
    let client_id = "kimne78kx3ncx6brgo4mv6wki5h1ko";
    let query = format!(
        "{{\n  videoPlaybackAccessToken(\n    id: \"{}\",\n    params: {{\n      platform: \"web\",\n      playerBackend: \"mediaplayer\",\n      playerType: \"site\"\n    }}\n  ) {{\n    signature\n    value\n  }}\n}}",
        video_id
    );
    let body = serde_json::json!({"query": query});
    let resp = client
        .post(url)
        .header("Client-ID", client_id)
        .header("Authorization", format!("OAuth {}", oauth_token))
        .json(&body)
        .send()
        .await?;
    let json: serde_json::Value = resp.json().await?;
    let token = json["data"]["videoPlaybackAccessToken"]["value"].as_str();
    let sig = json["data"]["videoPlaybackAccessToken"]["signature"].as_str();
    match (token, sig) {
        (Some(token), Some(sig)) => Ok((token.to_string(), sig.to_string())),
        _ => Err(TwitchError::AccessToken(format!("Response: {:?}", json))),
    }
}

/// Constructs the m3u8 URL for a given video ID, token, and signature.
pub fn get_m3u8_url_with_token_sig(video_id: &str, token: &str, sig: &str) -> String {
    format!(
        "https://usher.ttvnw.net/vod/{}.m3u8?nauth={}&nauthsig={}&allow_source=true&allow_audio_only=true",
        video_id,
        urlencoding::encode(token),
        sig
    )
} 