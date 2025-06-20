use clap::{App, Arg, ArgMatches, SubCommand};

pub struct Cli {
    pub command: String,
    pub video_link: Option<String>,
    pub auth_token: Option<String>,
}

pub fn parse_args() -> Cli {
    let matches = App::new("twitch-dl-rs")
        .version("0.1.0")
        .about("Download a video from Twitch")
        .subcommand(
            SubCommand::with_name("download")
                .about("Download a Twitch video")
                .arg(
                    Arg::with_name("video-link")
                        .help("The Twitch video link to download")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("auth-token")
                        .short("a")
                        .long("auth-token")
                        .takes_value(true)
                        .help("Authentication token for subscriber-only VODs"),
                ),
        )
        .get_matches();

    if let Some(download_matches) = matches.subcommand_matches("download") {
        Cli {
            command: "download".to_string(),
            video_link: download_matches.value_of("video-link").map(|s| s.to_string()),
            auth_token: download_matches.value_of("auth-token").map(|s| s.to_string()),
        }
    } else {
        Cli {
            command: String::new(),
            video_link: None,
            auth_token: None,
        }
    }
} 