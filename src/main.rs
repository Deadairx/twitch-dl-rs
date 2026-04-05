mod artifact;
mod cli;
mod downloader;
mod ffmpeg;
mod transcribe;
mod twitch;

use std::env;

#[tokio::main]
async fn main() {
    let cli = cli::parse_args();
    match cli.command {
        cli::CliCommand::Download {
            video_link,
            auth_token,
            output_root,
            quality,
        } => {
            if let Err(error) =
                download_vod(&video_link, auth_token.as_deref(), &output_root, quality).await
            {
                eprintln!("Download failed: {error}");
                std::process::exit(1);
            }
        }
        cli::CliCommand::Queue {
            channel,
            output_root,
            limit,
            past_broadcasts_only,
            min_seconds,
        } => {
            if let Err(error) =
                build_queue(&channel, &output_root, limit, past_broadcasts_only, min_seconds).await
            {
                eprintln!("Queue build failed: {error}");
                std::process::exit(1);
            }
        }
        cli::CliCommand::Process {
            channel,
            output_root,
            limit,
            past_broadcasts_only,
            min_seconds,
            quality,
            continue_on_error,
        } => {
            if let Err(error) = process_channel(
                &channel,
                &output_root,
                limit,
                past_broadcasts_only,
                min_seconds,
                quality,
                continue_on_error,
            )
            .await
            {
                eprintln!("Process failed: {error}");
                std::process::exit(1);
            }
        }
        cli::CliCommand::Status { output_root } => {
            if let Err(error) = show_status(&output_root).await {
                eprintln!("Status failed: {error}");
                std::process::exit(1);
            }
        }
    }
}

async fn download_vod(
    video_link: &str,
    auth_token: Option<&str>,
    output_root: &std::path::Path,
    quality: cli::QualityPreference,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let video_id = twitch::extract_video_id(video_link)?;
    let env_auth = env::var("TWITCH_DL_AUTH").ok();
    let auth_token = auth_token.or(env_auth.as_deref());

    println!("Resolving VOD {video_id}...");
    let (token, sig) = twitch::fetch_vod_access_token(&video_id, auth_token).await?;
    let m3u8_url = twitch::get_m3u8_url_with_token_sig(&video_id, &token, &sig);

    println!("Selecting stream variant...");
    let stream = downloader::resolve_stream(&m3u8_url, quality).await?;

    let artifact_dir = output_root.join(&video_id);
    let output_name = if stream.is_audio_only {
        "audio.m4a"
    } else {
        "video.mp4"
    };
    let video_path = artifact_dir.join(output_name);

    artifact::prepare_artifact_dir(&artifact_dir)?;
    artifact::write_source_url(&artifact_dir, video_link)?;

    println!(
        "Downloading {} stream to {}...",
        describe_stream(&stream),
        video_path.display()
    );
    ffmpeg::download_to_mp4(&stream.playlist_url, &video_path)?;

    let metadata = artifact::ArtifactMetadata::from_download(
        &video_id,
        video_link,
        &video_path,
        &stream,
        auth_token.is_some(),
    )?;
    artifact::write_metadata(&artifact_dir, &metadata)?;

    println!("Saved artifact to {}", artifact_dir.display());
    Ok(artifact_dir)
}

fn describe_stream(stream: &downloader::StreamInfo) -> String {
    if stream.is_audio_only {
        return stream
            .name
            .clone()
            .unwrap_or_else(|| "audio-only".to_string());
    }

    stream
        .resolution
        .clone()
        .or_else(|| stream.name.clone())
        .unwrap_or_else(|| "video".to_string())
}

async fn build_queue(
    channel: &str,
    output_root: &std::path::Path,
    limit: usize,
    past_broadcasts_only: bool,
    min_seconds: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching archive VODs for {channel}...");
    let vods = twitch::fetch_channel_archive_vods(channel, limit).await?;
    let existing_ids = artifact::existing_artifact_ids(output_root)?;
    let existing_id_set: std::collections::HashSet<_> = existing_ids.iter().cloned().collect();

    let queued: Vec<_> = vods
        .into_iter()
        .filter(|vod| parse_duration_seconds(&vod.duration).unwrap_or_default() >= min_seconds)
        .filter(|vod| !existing_id_set.contains(&vod.video_id))
        .collect();

    let skipped_existing_ids: Vec<_> = existing_ids
        .into_iter()
        .filter(|id| queued.iter().all(|vod| vod.video_id != *id))
        .collect();

    let queue_path = artifact::write_queue_file(
        output_root,
        channel,
        past_broadcasts_only,
        min_seconds,
        queued.clone(),
        skipped_existing_ids,
    )?;

    if past_broadcasts_only {
        println!("Filtering to past broadcasts only");
    }
    println!("Minimum duration: {} seconds", min_seconds);
    println!("Queued {} VOD(s)", queued.len());
    for vod in queued.iter().take(10) {
        println!("- {} | {} | {}", vod.video_id, vod.uploaded_at, vod.title);
    }
    if queued.len() > 10 {
        println!("... and {} more", queued.len() - 10);
    }
    println!("Saved queue to {}", queue_path.display());
    Ok(())
}

async fn process_channel(
    channel: &str,
    output_root: &std::path::Path,
    limit: usize,
    past_broadcasts_only: bool,
    min_seconds: u64,
    quality: cli::QualityPreference,
    continue_on_error: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Building queue for processing...");
    let vods = twitch::fetch_channel_archive_vods(channel, limit).await?;
    let existing_ids = artifact::existing_artifact_ids(output_root)?;
    let existing_id_set: std::collections::HashSet<_> = existing_ids.iter().cloned().collect();
    let queued: Vec<_> = vods
        .into_iter()
        .filter(|vod| parse_duration_seconds(&vod.duration).unwrap_or_default() >= min_seconds)
        .filter(|vod| {
            let artifact_dir = output_root.join(&vod.video_id);
            let has_existing_artifact = existing_id_set.contains(&vod.video_id);
            let has_existing_media = artifact::find_media_file(&artifact_dir).is_some();

            !has_existing_artifact || has_existing_media
        })
        .collect();

    let skipped_existing_ids: Vec<_> = existing_ids
        .into_iter()
        .filter(|id| queued.iter().all(|vod| vod.video_id != *id))
        .collect();
    let queue_path = artifact::write_queue_file(
        output_root,
        channel,
        past_broadcasts_only,
        min_seconds,
        queued.clone(),
        skipped_existing_ids,
    )?;
    println!("Saved queue to {}", queue_path.display());

    if queued.is_empty() {
        println!("No queued VODs to process");
        return Ok(());
    }

    println!("Processing {} queued VOD(s)...", queued.len());
    for vod in queued {
        let result = process_vod(&vod, output_root, quality).await;
        match result {
            Ok(()) => println!("Finished {}", vod.video_id),
            Err(error) => {
                eprintln!("Failed {}: {error}", vod.video_id);
                if !continue_on_error {
                    return Err(error);
                }
            }
        }
    }

    Ok(())
}

async fn process_vod(
    vod: &twitch::VodEntry,
    output_root: &std::path::Path,
    quality: cli::QualityPreference,
) -> Result<(), Box<dyn std::error::Error>> {
    let artifact_dir = output_root.join(&vod.video_id);
    artifact::prepare_artifact_dir(&artifact_dir)?;
    let mut status = artifact::read_status(&artifact_dir)?
        .unwrap_or_else(|| artifact::ProcessStatus::new(&vod.video_id, &vod.url));

    let media_path = if let Some(existing) = artifact::find_media_file(&artifact_dir) {
        println!("Reusing existing media for {}", vod.video_id);
        status.downloaded = true;
        status.media_file = existing.file_name().map(|name| name.to_string_lossy().to_string());
        existing
    } else {
        println!("Downloading {} | {}", vod.video_id, vod.title);
        let artifact_dir = download_vod(&vod.url, None, output_root, quality).await?;
        let media_path = artifact::find_media_file(&artifact_dir)
            .ok_or_else(|| format!("missing media file after download for {}", vod.video_id))?;
        status.downloaded = true;
        status.media_file = media_path.file_name().map(|name| name.to_string_lossy().to_string());
        status.last_error = None;
        status.updated_at_epoch_s = now_epoch_s();
        artifact::write_status(&artifact_dir, &status)?;
        media_path
    };

    let transcript_path = artifact_dir.join("transcript.txt");
    if transcript_path.exists() {
        println!("Reusing existing transcript for {}", vod.video_id);
        status.transcribed = true;
        status.transcript_file = Some("transcript.txt".to_string());
        status.last_error = None;
        status.updated_at_epoch_s = now_epoch_s();
        artifact::write_status(&artifact_dir, &status)?;
        return Ok(());
    }

    println!("Transcribing {} with mlx-whisper...", vod.video_id);
    match transcribe::transcribe_to_txt(&media_path, &artifact_dir) {
        Ok(path) => {
            status.transcribed = true;
            status.transcript_file = path.file_name().map(|name| name.to_string_lossy().to_string());
            status.last_error = None;
        }
        Err(error) => {
            status.last_error = Some(error.to_string());
            status.updated_at_epoch_s = now_epoch_s();
            artifact::write_status(&artifact_dir, &status)?;
            return Err(Box::new(error));
        }
    }

    status.updated_at_epoch_s = now_epoch_s();
    artifact::write_status(&artifact_dir, &status)?;
    Ok(())
}

fn now_epoch_s() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn parse_duration_seconds(duration: &str) -> Option<u64> {
    duration
        .strip_prefix("PT")
        .and_then(|value| value.strip_suffix('S'))
        .and_then(|value| value.parse::<u64>().ok())
}

async fn show_status(output_root: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let items = artifact::scan_artifact_statuses(output_root)?;
    if items.is_empty() {
        println!("No artifacts found in {}", output_root.display());
        return Ok(());
    }
    println!("{:<15} {:<12} {:<12} {}", "VIDEO_ID", "DOWNLOADED", "TRANSCRIBED", "LAST_ERROR");
    println!("{}", "-".repeat(70));
    for (video_id, status) in &items {
        match status {
            Some(s) => {
                let last_err = s.last_error.as_deref().unwrap_or("-");
                let truncated = if last_err.len() > 40 { &last_err[..40] } else { last_err };
                println!("{:<15} {:<12} {:<12} {}", video_id, s.downloaded, s.transcribed, truncated);
            }
            None => println!("{:<15} {:<12} {:<12} {}", video_id, "(no status)", "-", "-"),
        }
    }
    println!("\n{} artifact(s) total", items.len());
    Ok(())
}
