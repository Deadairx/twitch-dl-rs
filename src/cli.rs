use std::path::PathBuf;

use clap::{Arg, ArgAction, Command};

const OUTPUT_ROOT_ENV: &str = "VOD_PIPELINE_OUTPUT_ROOT";

pub struct Cli {
    pub command: CliCommand,
}

pub enum CliCommand {
    Download {
        video_link: String,
        auth_token: Option<String>,
        output_root: PathBuf,
        quality: QualityPreference,
        skip_metadata: bool,
    },
    Queue {
        channel: String,
        output_root: PathBuf,
        limit: usize,
        past_broadcasts_only: bool,
        min_seconds: u64,
    },
    QueueVideo {
        url: String,
        output_root: PathBuf,
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
    Status {
        output_root: PathBuf,
    },
    DownloadAll {
        channel: String,
        output_root: PathBuf,
        quality: QualityPreference,
        continue_on_error: bool,
    },
    TranscribeAll {
        output_root: PathBuf,
        continue_on_error: bool,
    },
    Cleanup {
        output_root: PathBuf,
        delete: bool,
        delete_all: bool,
        video_id: Option<String>,
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

fn output_root_arg(help: &'static str) -> Arg {
    Arg::new("output-root")
        .long("output-root")
        .help(help)
        .env(OUTPUT_ROOT_ENV)
        .default_value("artifacts")
}

pub fn parse_args() -> Cli {
    let matches = Command::new("vod-pipeline")
        .version("0.1.0")
        .about("Queue-first media pipeline for Twitch VOD artifacts")
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
                .arg(output_root_arg(
                    "Directory where artifact folders are created (default: VOD_PIPELINE_OUTPUT_ROOT or artifacts)",
                ))
                .arg(
                    Arg::new("quality")
                        .long("quality")
                        .help("Preferred stream type for download")
                        .value_parser(["audio-only", "lowest", "highest"])
                        .default_value("audio-only"),
                )
                .arg(
                    Arg::new("skip-metadata")
                        .long("skip-metadata")
                        .help("Skip fetching VOD title/channel/date from Twitch API; those fields will be absent in metadata.json")
                        .action(ArgAction::SetTrue),
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
                .arg(output_root_arg(
                    "Directory where artifact folders and queue files are created (default: VOD_PIPELINE_OUTPUT_ROOT or artifacts)",
                ))
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
            Command::new("queue-video")
                .about("Queue a single Twitch VOD by URL")
                .arg(
                    Arg::new("url")
                        .help("The Twitch video URL to queue")
                        .required(true)
                        .index(1),
                )
                .arg(output_root_arg(
                    "Directory where artifact folders and queue files are created (default: VOD_PIPELINE_OUTPUT_ROOT or artifacts)",
                )),
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
                .arg(output_root_arg(
                    "Directory where artifact folders and queue files are created (default: VOD_PIPELINE_OUTPUT_ROOT or artifacts)",
                ))
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
        .subcommand(
            Command::new("status")
                .about("Show status of all downloaded/transcribed artifacts")
                .arg(output_root_arg(
                    "Directory where artifact folders are stored (default: VOD_PIPELINE_OUTPUT_ROOT or artifacts)",
                )),
        )
        .subcommand(
            Command::new("download-all")
                .about("Download all pending queued VODs for a channel")
                .arg(
                    Arg::new("channel")
                        .help("Twitch channel login name")
                        .required(true)
                        .index(1),
                )
                .arg(output_root_arg(
                    "Directory where artifact folders and queue files are stored (default: VOD_PIPELINE_OUTPUT_ROOT or artifacts)",
                ))
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
                        .help("Continue downloading other VODs if one fails")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("transcribe-all")
                .about("Transcribe all downloaded-but-not-transcribed artifacts")
                .arg(output_root_arg(
                    "Directory where artifact folders are stored (default: VOD_PIPELINE_OUTPUT_ROOT or artifacts)",
                ))
                .arg(
                    Arg::new("continue-on-error")
                        .long("continue-on-error")
                        .help("Continue transcribing other artifacts if one fails")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("cleanup")
                .about("List and optionally delete ready-for-notes artifact files")
                .arg(output_root_arg(
                    "Directory where artifact folders are stored (default: VOD_PIPELINE_OUTPUT_ROOT or artifacts)",
                ))
                .arg(
                    Arg::new("delete")
                        .long("delete")
                        .help("Enable deletion mode")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("all")
                        .long("all")
                        .help("Delete all candidates (use with --delete)")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("video-id")
                        .long("video-id")
                        .help("Delete specific artifact by video_id (use with --delete)")
                        .value_name("VIDEO_ID"),
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
                skip_metadata: download_matches.get_flag("skip-metadata"),
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
        Some(("queue-video", queue_video_matches)) => Cli {
            command: CliCommand::QueueVideo {
                url: queue_video_matches
                    .get_one::<String>("url")
                    .expect("url is required by clap")
                    .clone(),
                output_root: PathBuf::from(
                    queue_video_matches
                        .get_one::<String>("output-root")
                        .expect("output-root has a default value"),
                ),
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
        Some(("status", status_matches)) => Cli {
            command: CliCommand::Status {
                output_root: PathBuf::from(
                    status_matches
                        .get_one::<String>("output-root")
                        .expect("output-root has a default value"),
                ),
            },
        },
        Some(("download-all", download_all_matches)) => Cli {
            command: CliCommand::DownloadAll {
                channel: download_all_matches
                    .get_one::<String>("channel")
                    .expect("channel is required by clap")
                    .clone(),
                output_root: PathBuf::from(
                    download_all_matches
                        .get_one::<String>("output-root")
                        .expect("output-root has a default value"),
                ),
                quality: QualityPreference::parse(
                    download_all_matches
                        .get_one::<String>("quality")
                        .expect("quality has a default value"),
                ),
                continue_on_error: download_all_matches.get_flag("continue-on-error"),
            },
        },
        Some(("transcribe-all", transcribe_all_matches)) => Cli {
            command: CliCommand::TranscribeAll {
                output_root: PathBuf::from(
                    transcribe_all_matches
                        .get_one::<String>("output-root")
                        .expect("output-root has a default value"),
                ),
                continue_on_error: transcribe_all_matches.get_flag("continue-on-error"),
            },
        },
        Some(("cleanup", cleanup_matches)) => {
            let output_root = PathBuf::from(
                cleanup_matches
                    .get_one::<String>("output-root")
                    .expect("output-root has a default value"),
            );
            let delete = cleanup_matches.get_flag("delete");
            let delete_all = cleanup_matches.get_flag("all");
            let video_id = cleanup_matches.get_one::<String>("video-id").cloned();

            Cli {
                command: CliCommand::Cleanup {
                    output_root,
                    delete,
                    delete_all,
                    video_id,
                },
            }
        }
        _ => unreachable!("clap enforces a subcommand"),
    }
}
