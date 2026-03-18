use std::path::PathBuf;

use clap::{Arg, ArgAction, Command};

pub struct Cli {
    pub command: CliCommand,
}

pub enum CliCommand {
    Download {
        video_link: String,
        auth_token: Option<String>,
        output_root: PathBuf,
        quality: QualityPreference,
    },
    Queue {
        channel: String,
        output_root: PathBuf,
        limit: usize,
        past_broadcasts_only: bool,
        min_seconds: u64,
    },
    Process {
        channel: String,
        output_root: PathBuf,
        limit: usize,
        past_broadcasts_only: bool,
        min_seconds: u64,
        quality: QualityPreference,
        continue_on_error: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum QualityPreference {
    AudioOnly,
    Lowest,
    Highest,
}

impl QualityPreference {
    fn parse(value: &str) -> Self {
        match value {
            "audio-only" => Self::AudioOnly,
            "lowest" => Self::Lowest,
            "highest" => Self::Highest,
            _ => Self::AudioOnly,
        }
    }
}

pub fn parse_args() -> Cli {
    let matches = Command::new("twitch-dl-rs")
        .version("0.1.0")
        .about("Download a Twitch VOD into a local artifact directory")
        .subcommand_required(true)
        .arg_required_else_help(true)
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
                )
                .arg(
                    Arg::new("output-root")
                        .long("output-root")
                        .help("Directory where artifact folders are created")
                        .default_value("artifacts"),
                )
                .arg(
                    Arg::new("quality")
                        .long("quality")
                        .help("Preferred stream type for download")
                        .value_parser(["audio-only", "lowest", "highest"])
                        .default_value("audio-only"),
                ),
        )
        .subcommand(
            Command::new("queue")
                .about("Build a backlog queue for a Twitch channel")
                .arg(
                    Arg::new("channel")
                        .help("Twitch channel login name")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("output-root")
                        .long("output-root")
                        .help("Directory where artifact folders and queue files are created")
                        .default_value("artifacts"),
                )
                .arg(
                    Arg::new("limit")
                        .long("limit")
                        .help("Maximum number of archive VODs to queue")
                        .value_parser(clap::value_parser!(usize))
                        .default_value("25"),
                )
                .arg(
                    Arg::new("past-broadcasts-only")
                        .long("past-broadcasts-only")
                        .help("Keep only Twitch past broadcasts (current queue behavior)")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("min-seconds")
                        .long("min-seconds")
                        .help("Skip VODs shorter than this many seconds")
                        .value_parser(clap::value_parser!(u64))
                        .default_value("600"),
                ),
        )
        .subcommand(
            Command::new("process")
                .about("Queue, download, and transcribe VODs for a Twitch channel")
                .arg(
                    Arg::new("channel")
                        .help("Twitch channel login name")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("output-root")
                        .long("output-root")
                        .help("Directory where artifact folders and queue files are created")
                        .default_value("artifacts"),
                )
                .arg(
                    Arg::new("limit")
                        .long("limit")
                        .help("Maximum number of archive VODs to process")
                        .value_parser(clap::value_parser!(usize))
                        .default_value("25"),
                )
                .arg(
                    Arg::new("past-broadcasts-only")
                        .long("past-broadcasts-only")
                        .help("Keep only Twitch past broadcasts")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("min-seconds")
                        .long("min-seconds")
                        .help("Skip VODs shorter than this many seconds")
                        .value_parser(clap::value_parser!(u64))
                        .default_value("600"),
                )
                .arg(
                    Arg::new("quality")
                        .long("quality")
                        .help("Preferred stream type for download")
                        .value_parser(["audio-only", "lowest", "highest"])
                        .default_value("audio-only"),
                )
                .arg(
                    Arg::new("continue-on-error")
                        .long("continue-on-error")
                        .help("Continue processing later VODs if one step fails")
                        .action(ArgAction::SetTrue),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("download", download_matches)) => Cli {
            command: CliCommand::Download {
                video_link: download_matches
                    .get_one::<String>("video-link")
                    .expect("video-link is required by clap")
                    .clone(),
                auth_token: download_matches.get_one::<String>("auth-token").cloned(),
                output_root: PathBuf::from(
                    download_matches
                        .get_one::<String>("output-root")
                        .expect("output-root has a default value"),
                ),
                quality: QualityPreference::parse(
                    download_matches
                        .get_one::<String>("quality")
                        .expect("quality has a default value"),
                ),
            },
        },
        Some(("queue", queue_matches)) => Cli {
            command: CliCommand::Queue {
                channel: queue_matches
                    .get_one::<String>("channel")
                    .expect("channel is required by clap")
                    .clone(),
                output_root: PathBuf::from(
                    queue_matches
                        .get_one::<String>("output-root")
                        .expect("output-root has a default value"),
                ),
                limit: *queue_matches
                    .get_one::<usize>("limit")
                    .expect("limit has a default value"),
                past_broadcasts_only: queue_matches.get_flag("past-broadcasts-only"),
                min_seconds: *queue_matches
                    .get_one::<u64>("min-seconds")
                    .expect("min-seconds has a default value"),
            },
        },
        Some(("process", process_matches)) => Cli {
            command: CliCommand::Process {
                channel: process_matches
                    .get_one::<String>("channel")
                    .expect("channel is required by clap")
                    .clone(),
                output_root: PathBuf::from(
                    process_matches
                        .get_one::<String>("output-root")
                        .expect("output-root has a default value"),
                ),
                limit: *process_matches
                    .get_one::<usize>("limit")
                    .expect("limit has a default value"),
                past_broadcasts_only: process_matches.get_flag("past-broadcasts-only"),
                min_seconds: *process_matches
                    .get_one::<u64>("min-seconds")
                    .expect("min-seconds has a default value"),
                quality: QualityPreference::parse(
                    process_matches
                        .get_one::<String>("quality")
                        .expect("quality has a default value"),
                ),
                continue_on_error: process_matches.get_flag("continue-on-error"),
            },
        },
        _ => unreachable!("clap enforces a subcommand"),
    }
}
