use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;

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
    #[error("Failed to parse Twitch response: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VodEntry {
    pub channel: String,
    pub title: String,
    pub url: String,
    pub video_id: String,
    pub uploaded_at: String,
    pub duration: String,
}

#[derive(Debug, Deserialize)]
struct ChannelVideosResponse {
    data: Option<ChannelVideosData>,
}

#[derive(Debug, Deserialize)]
struct ChannelVideosData {
    user: Option<ChannelUser>,
}

#[derive(Debug, Deserialize)]
struct ChannelUser {
    videos: ChannelVideoConnection,
}

#[derive(Debug, Deserialize)]
struct ChannelVideoConnection {
    edges: Vec<ChannelVideoEdge>,
}

#[derive(Debug, Deserialize)]
struct ChannelVideoEdge {
    node: ChannelVideoNode,
}

#[derive(Debug, Deserialize)]
struct ChannelVideoNode {
    id: String,
    title: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
    #[serde(rename = "lengthSeconds")]
    length_seconds: u64,
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
pub async fn fetch_vod_access_token(video_id: &str, oauth_token: Option<&str>) -> Result<(String, String), TwitchError> {
    let client = Client::new();
    let url = "https://gql.twitch.tv/gql";
    let client_id = "kimne78kx3ncx6brgo4mv6wki5h1ko";
    let query = format!(
        "{{\n  videoPlaybackAccessToken(\n    id: \"{}\",\n    params: {{\n      platform: \"web\",\n      playerBackend: \"mediaplayer\",\n      playerType: \"site\"\n    }}\n  ) {{\n    signature\n    value\n  }}\n}}",
        video_id
    );
    let body = serde_json::json!({"query": query});
    let mut request = client
        .post(url)
        .header("Client-ID", client_id)
        .json(&body);

    if let Some(oauth_token) = oauth_token {
        request = request.header("Authorization", format!("OAuth {}", oauth_token));
    }

    let resp = request.send().await?.error_for_status()?;
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

pub async fn fetch_channel_archive_vods(
    channel: &str,
    limit: usize,
) -> Result<Vec<VodEntry>, TwitchError> {
    let client = Client::new();
    let query = r#"
        query($login: String!, $first: Int!) {
          user(login: $login) {
            videos(first: $first, type: ARCHIVE, sort: TIME) {
              edges {
                node {
                  id
                  title
                  publishedAt
                  lengthSeconds
                }
              }
            }
          }
        }
    "#;
    let body = serde_json::json!({
        "query": query,
        "variables": {
            "login": channel.to_ascii_lowercase(),
            "first": limit.min(100),
        }
    });
    let response: ChannelVideosResponse = client
        .post("https://gql.twitch.tv/gql")
        .header("Client-ID", "kimne78kx3ncx6brgo4mv6wki5h1ko")
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let user = response
        .data
        .and_then(|data| data.user)
        .ok_or_else(|| TwitchError::Parse(format!("channel not found: {channel}")))?;

    Ok(user
        .videos
        .edges
        .into_iter()
        .map(|edge| VodEntry {
            channel: channel.to_ascii_lowercase(),
            title: edge.node.title,
            url: format!("https://www.twitch.tv/videos/{}", edge.node.id),
            video_id: edge.node.id,
            uploaded_at: edge.node.published_at,
            duration: format!("PT{}S", edge.node.length_seconds),
        })
        .collect())
}
