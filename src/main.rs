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
        cli::CliCommand::DownloadAll {
            channel,
            output_root,
            quality,
            continue_on_error,
        } => {
            if let Err(error) =
                download_all(&channel, &output_root, quality, continue_on_error).await
            {
                eprintln!("Download-all failed: {error}");
                std::process::exit(1);
            }
        }
        cli::CliCommand::TranscribeAll {
            output_root,
            continue_on_error,
        } => {
            if let Err(error) = transcribe_all(&output_root, continue_on_error).await {
                eprintln!("Transcribe-all failed: {error}");
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

async fn download_vod_to_artifact(
    vod: &twitch::VodEntry,
    output_root: &std::path::Path,
    quality: cli::QualityPreference,
    status: &mut artifact::ProcessStatus,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let artifact_dir = output_root.join(&vod.video_id);
    if let Some(existing) = artifact::find_media_file(&artifact_dir) {
        println!("Reusing existing media for {}", vod.video_id);
        status.downloaded = true;
        status.media_file = existing.file_name().map(|n| n.to_string_lossy().to_string());
        return Ok(existing);
    }
    println!("Downloading {} | {}", vod.video_id, vod.title);
    let downloaded_dir = download_vod(&vod.url, None, output_root, quality).await?;
    let media_path = artifact::find_media_file(&downloaded_dir)
        .ok_or_else(|| format!("missing media file after download for {}", vod.video_id))?;
    status.downloaded = true;
    status.media_file = media_path.file_name().map(|n| n.to_string_lossy().to_string());
    status.last_error = None;
    status.updated_at_epoch_s = now_epoch_s();
    artifact::write_status(&downloaded_dir, status)?;
    Ok(media_path)
}

fn get_audio_duration_secs(audio_path: &std::path::Path) -> Option<f64> {
    let output = std::process::Command::new("ffprobe")
        .args(["-v", "quiet", "-print_format", "json", "-show_streams"])
        .arg(audio_path)
        .output()
        .ok()?;
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    json["streams"][0]["duration"]
        .as_str()
        .and_then(|d| d.parse().ok())
}

fn transcribe_artifact(
    video_id: &str,
    artifact_dir: &std::path::Path,
    media_path: &std::path::Path,
    status: &mut artifact::ProcessStatus,
) -> Result<(), Box<dyn std::error::Error>> {
    // Reuse check for existing srt+vtt
    let srt_path = artifact_dir.join("transcript.srt");
    let vtt_path = artifact_dir.join("transcript.vtt");
    if srt_path.exists() && vtt_path.exists() && status.transcribed {
        println!("Reusing existing transcript for {}", video_id);
        return Ok(());
    }

    // Get audio duration for word-count heuristic
    let duration_secs = get_audio_duration_secs(media_path);

    println!("Transcribing {} with hear...", video_id);
    match transcribe::transcribe_to_srt_and_vtt(media_path, artifact_dir, duration_secs) {
        transcribe::TranscriptionOutcome::Completed {
            srt_path,
            vtt_path: _,
            word_count,
        } => {
            status.transcribed = true;
            status.ready_for_notes = true;
            status.transcription_outcome = Some("completed".to_string());
            status.transcription_reason = None;
            status.transcript_word_count = Some(word_count);
            status.transcript_file = srt_path.file_name().map(|n| n.to_string_lossy().to_string());
            status.last_error = None;
            status.updated_at_epoch_s = now_epoch_s();
            artifact::write_status(artifact_dir, status)?;
            Ok(())
        }
        transcribe::TranscriptionOutcome::Suspect {
            srt_path,
            vtt_path: _,
            word_count,
            reason,
        } => {
            // suspect: leave transcribed=false, set outcome fields, do NOT return Err
            status.transcribed = false;
            status.transcription_outcome = Some("suspect".to_string());
            status.transcription_reason = Some(reason);
            status.transcript_word_count = Some(word_count);
            status.transcript_file = srt_path.file_name().map(|n| n.to_string_lossy().to_string());
            status.last_error = None;
            status.updated_at_epoch_s = now_epoch_s();
            artifact::write_status(artifact_dir, status)?;
            Ok(()) // NOT an error — pipeline continues
        }
        transcribe::TranscriptionOutcome::Failed { reason } => {
            status.transcribed = false;
            status.transcription_outcome = Some("failed".to_string());
            status.transcription_reason = Some(reason.clone());
            status.last_error = Some(reason.clone());
            status.updated_at_epoch_s = now_epoch_s();
            artifact::write_status(artifact_dir, status)?;
            Err(reason.into())
        }
    }
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
    let media_path = download_vod_to_artifact(vod, output_root, quality, &mut status).await?;
    transcribe_artifact(&vod.video_id, &artifact_dir, &media_path, &mut status)?;
    Ok(())
}

async fn download_all(
    channel: &str,
    output_root: &std::path::Path,
    quality: cli::QualityPreference,
    continue_on_error: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let queue_file = artifact::read_queue_file(output_root, channel)?;
    let pending: Vec<_> = queue_file
        .queued
        .into_iter()
        .filter(|vod| {
            let artifact_dir = output_root.join(&vod.video_id);
            let status = artifact::read_status(&artifact_dir).unwrap_or(None);
            !status.map(|s| s.downloaded).unwrap_or(false)
        })
        .collect();

    if pending.is_empty() {
        println!("All queued VODs already downloaded.");
        return Ok(());
    }
    println!("Downloading {} pending VOD(s) for {channel}...", pending.len());
    for vod in pending {
        let artifact_dir = output_root.join(&vod.video_id);
        artifact::prepare_artifact_dir(&artifact_dir)?;
        let mut status = artifact::read_status(&artifact_dir)?
            .unwrap_or_else(|| artifact::ProcessStatus::new(&vod.video_id, &vod.url));
        match download_vod_to_artifact(&vod, output_root, quality, &mut status).await {
            Ok(_) => println!("Downloaded {}", vod.video_id),
            Err(e) => {
                eprintln!("Failed {}: {e}", vod.video_id);
                if !continue_on_error {
                    return Err(e);
                }
            }
        }
    }
    Ok(())
}

async fn transcribe_all(
    output_root: &std::path::Path,
    continue_on_error: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let items = artifact::scan_artifact_statuses(output_root)?;
    let pending: Vec<_> = items
        .into_iter()
        .filter_map(|(video_id, status)| {
            let s = status?;
            if s.downloaded
                && !s.transcribed
                && s.transcription_outcome.as_deref() != Some("suspect")
            {
                Some((video_id, s))
            } else {
                None
            }
        })
        .collect();

    if pending.is_empty() {
        println!("No artifacts pending transcription.");
        return Ok(());
    }
    println!("Transcribing {} artifact(s)...", pending.len());
    for (video_id, mut status) in pending {
        let artifact_dir = output_root.join(&video_id);
        let media_path = artifact::find_media_file(&artifact_dir)
            .ok_or_else(|| format!("media file missing for {} despite downloaded=true", video_id))?;
        match transcribe_artifact(&video_id, &artifact_dir, &media_path, &mut status) {
            Ok(()) => println!("Transcribed {}", video_id),
            Err(e) => {
                eprintln!("Failed {}: {e}", video_id);
                if !continue_on_error {
                    return Err(e);
                }
            }
        }
    }
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
    println!("{:<15} {:<12} {:<12} {:<8} {}", "VIDEO_ID", "DOWNLOADED", "OUTCOME", "READY", "REASON");
    println!("{}", "-".repeat(80));
    for (video_id, status) in &items {
        match status {
            Some(s) => {
                let outcome = s.transcription_outcome.as_deref().unwrap_or("-");
                let ready = if s.ready_for_notes { "yes" } else { "-" };
                let reason = s
                    .transcription_reason
                    .as_deref()
                    .or(s.last_error.as_deref())
                    .unwrap_or("-");
                let truncated = if reason.len() > 40 {
                    &reason[..40]
                } else {
                    reason
                };
                println!(
                    "{:<15} {:<12} {:<12} {:<8} {}",
                    video_id, s.downloaded, outcome, ready, truncated
                );
            }
            None => println!("{:<15} {:<12} {:<12} {:<8} {}", video_id, "(no status)", "-", "-", "-"),
        }
    }
    println!("\n{} artifact(s) total", items.len());
    Ok(())
}
