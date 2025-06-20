use reqwest::Client;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse m3u8 playlist")] 
    Parse,
}

/// Recursively fetches the first media playlist with .ts segments.
async fn fetch_media_playlist(client: &Client, initial_url: &str) -> Result<(String, Vec<String>), DownloadError> {
    let mut m3u8_url = initial_url.to_string();
    loop {
        let playlist = client.get(&m3u8_url).send().await?.text().await?;
        println!("\n--- Playlist from {} ---\n{}\n--- End Playlist ---\n", m3u8_url, playlist);
        let mut ts_segments = vec![];
        let mut m3u8_links = vec![];
        for line in playlist.lines() {
            let line = line.trim();
            if line.ends_with(".ts") {
                ts_segments.push(line.to_string());
            } else if line.ends_with(".m3u8") {
                m3u8_links.push(line.to_string());
            }
        }
        if !ts_segments.is_empty() {
            return Ok((m3u8_url.clone(), ts_segments));
        } else if !m3u8_links.is_empty() {
            // Pick the first variant playlist (could be improved to select quality)
            let next_url = if m3u8_links[0].starts_with("http") {
                m3u8_links[0].clone()
            } else {
                let base = m3u8_url.rsplit_once('/').map(|(b, _)| b).unwrap_or("");
                format!("{}/{}", base, m3u8_links[0])
            };
            m3u8_url = next_url;
            continue;
        } else {
            return Err(DownloadError::Parse);
        }
    }
}

/// Downloads the m3u8 playlist and all segments to the given directory.
pub async fn download_m3u8_stream(m3u8_url: &str, output_dir: &Path) -> Result<(), DownloadError> {
    let client = Client::new();
    let (playlist_url, segment_urls) = fetch_media_playlist(&client, m3u8_url).await?;

    // Download each segment
    for (i, segment_url) in segment_urls.iter().enumerate() {
        let url = if segment_url.starts_with("http") {
            segment_url.to_string()
        } else {
            let base = playlist_url.rsplit_once('/').map(|(b, _)| b).unwrap_or("");
            format!("{}/{}", base, segment_url)
        };
        let resp = client.get(&url).send().await?.bytes().await?;
        let segment_path = output_dir.join(format!("segment_{:05}.ts", i));
        let mut file = File::create(&segment_path)?;
        file.write_all(&resp)?;
        println!("Downloaded segment {}", i + 1);
    }
    Ok(())
} 