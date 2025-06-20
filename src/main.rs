mod cli;
mod twitch;

fn main() {
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
            match twitch::extract_video_id(video_link) {
                Ok(video_id) => {
                    let m3u8_url = twitch::get_m3u8_url(&video_id, cli.auth_token.as_deref());
                    println!("m3u8 URL: {}", m3u8_url);
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
