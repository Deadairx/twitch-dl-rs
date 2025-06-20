mod cli;
mod twitch;
mod downloader;

use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let cli = cli::parse_args();
    match cli.command.as_str() {
        "download" => {
            let video_link = match &cli.video_link {
                Some(link) => link,
                None => {
                    eprintln!("No video link provided.");
                    return;
                }
            };
            let auth_token = match &cli.auth_token {
                Some(token) => token,
                None => {
                    eprintln!("No auth token provided. This is required for VOD access.");
                    return;
                }
            };
            match twitch::extract_video_id(video_link) {
                Ok(video_id) => {
                    match twitch::fetch_vod_access_token(&video_id, auth_token).await {
                        Ok((token, sig)) => {
                            let m3u8_url = twitch::get_m3u8_url_with_token_sig(&video_id, &token, &sig);
                            println!("m3u8 URL: {}", m3u8_url);
                            let output_dir = PathBuf::from(format!("output_{}", video_id));
                            if let Err(e) = std::fs::create_dir_all(&output_dir) {
                                eprintln!("Failed to create output directory: {}", e);
                                return;
                            }
                            match downloader::download_m3u8_stream(&m3u8_url, &output_dir).await {
                                Ok(()) => println!("All segments downloaded to {:?}", output_dir),
                                Err(e) => eprintln!("Download error: {}", e),
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to get VOD access token: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
        _ => {
            eprintln!("Unknown command");
        }
    }
}
