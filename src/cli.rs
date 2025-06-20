use clap::{Arg, Command};

pub struct Cli {
    pub command: String,
    pub video_link: Option<String>,
    pub auth_token: Option<String>,
}

pub fn parse_args() -> Cli {
    let matches = Command::new("twitch-dl-rs")
        .version("0.1.0")
        .about("Download a video from Twitch")
        .subcommand(
            Command::new("download")
                .about("Download a Twitch video")
                .arg(
                    Arg::new("video-link")
                        .help("The Twitch video link to download")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("auth-token")
                        .short('a')
                        .long("auth-token")
                        .help("Authentication token for subscriber-only VODs"),
                ),
        )
        .get_matches();

    if let Some(download_matches) = matches.subcommand_matches("download") {
        Cli {
            command: "download".to_string(),
            video_link: download_matches.get_one::<String>("video-link").cloned(),
            auth_token: download_matches.get_one::<String>("auth-token").cloned(),
        }
    } else {
        Cli {
            command: String::new(),
            video_link: None,
            auth_token: None,
        }
    }
} 