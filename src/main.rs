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
            skip_metadata,
        } => {
            if let Err(error) =
                download_vod(&video_link, auth_token.as_deref(), &output_root, quality, None, skip_metadata).await
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
        cli::CliCommand::QueueVideo { url, output_root } => {
            if let Err(error) = queue_video(&url, &output_root).await {
                eprintln!("Queue-video failed: {error}");
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
            video_id,
        } => {
            if let Err(error) =
                download_all(channel.as_deref(), &output_root, quality, continue_on_error, video_id.as_deref()).await
            {
                eprintln!("Download-all failed: {error}");
                std::process::exit(1);
            }
        }
        cli::CliCommand::TranscribeAll {
            output_root,
            continue_on_error,
            video_id,
        } => {
            if let Err(error) = transcribe_all(&output_root, continue_on_error, video_id.as_deref()).await {
                eprintln!("Transcribe-all failed: {error}");
                std::process::exit(1);
            }
        }
        cli::CliCommand::Cleanup {
            output_root,
            delete,
            delete_all,
            video_id,
        } => {
            if let Err(error) = cleanup(&output_root, delete, delete_all, video_id).await {
                eprintln!("Cleanup failed: {error}");
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
    vod_context: Option<(&str, &str, &str)>,
    skip_metadata: bool,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let video_id = twitch::extract_video_id(video_link)?;
    let env_auth = env::var("TWITCH_DL_AUTH").ok();
    let auth_token = auth_token.or(env_auth.as_deref());

    println!("Resolving VOD {video_id}...");
    let (token, sig) = twitch::fetch_vod_access_token(&video_id, auth_token).await?;
    let m3u8_url = twitch::get_m3u8_url_with_token_sig(&video_id, &token, &sig);

    println!("Selecting stream variant...");
    let stream = downloader::resolve_stream(&m3u8_url, quality).await?;

    // Resolve VOD context: if provided use it, otherwise optionally fetch from GQL
    let (title_opt, channel_opt, uploaded_at_opt): (Option<String>, Option<String>, Option<String>) = 
        if let Some((t, c, u)) = vod_context {
            (Some(t.to_string()), Some(c.to_string()), Some(u.to_string()))
        } else if skip_metadata {
            (None, None, None)
        } else {
            match twitch::fetch_vod_metadata_by_id(&video_id).await {
                Ok((title, channel, uploaded_at)) => (Some(title), Some(channel), Some(uploaded_at)),
                Err(e) => {
                    eprintln!("Failed to fetch VOD metadata: {e}");
                    eprintln!("Hint: use --skip-metadata to download without metadata");
                    return Err(format!("GQL metadata fetch failed: {e}").into());
                }
            }
        };

    let ctx: Option<(&str, &str, &str)> = match (&title_opt, &channel_opt, &uploaded_at_opt) {
        (Some(t), Some(c), Some(u)) => Some((t.as_str(), c.as_str(), u.as_str())),
        _ => None,
    };

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
        ctx,
    )?;
    artifact::write_metadata(&artifact_dir, &metadata)?;

    let mut status = artifact::ProcessStatus::new(&video_id, video_link);
    status.downloaded = true;
    status.media_file = Some(output_name.to_string());
    artifact::write_status(&artifact_dir, &status)?;

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

async fn queue_video(
    url: &str,
    output_root: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let video_id = twitch::extract_video_id(url)?;
    
    let (title, channel, uploaded_at) = match twitch::fetch_vod_metadata_by_id(&video_id).await {
        Ok((title, channel, uploaded_at)) => (title, channel, uploaded_at),
        Err(e) => {
            eprintln!("Failed to resolve VOD metadata: {e}");
            return Err(format!("GQL metadata fetch failed: {e}").into());
        }
    };
    
    // Read existing queue for this channel, or start with empty vec if file doesn't exist
    let existing_entries = match artifact::read_queue_file(output_root, &channel) {
        Ok(queue_file) => queue_file.queued,
        Err(_) => vec![],
    };
    
    // Check if this video_id is already queued
    if existing_entries.iter().any(|v| v.video_id == video_id) {
        println!("Already queued: {video_id}");
        return Ok(());
    }
    
    // Create new VodEntry with duration placeholder
    let new_entry = twitch::VodEntry {
        channel: channel.clone(),
        title,
        url: url.to_string(),
        video_id: video_id.clone(),
        uploaded_at,
        duration: "PT0S".to_string(),
    };
    
    // Append new entry to existing queue
    let mut queued = existing_entries;
    queued.push(new_entry);
    
    // Write updated queue file with default filter settings
    let queue_path = artifact::write_queue_file(
        output_root,
        &channel,
        false,
        0,
        queued,
        vec![],
    )?;
    
    println!("Queued {video_id} into {}", queue_path.display());
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
    let downloaded_dir = download_vod(
        &vod.url,
        None,
        output_root,
        quality,
        Some((vod.title.as_str(), vod.channel.as_str(), vod.uploaded_at.as_str())),
        false,
    ).await?;
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
    channel: Option<&str>,
    output_root: &std::path::Path,
    quality: cli::QualityPreference,
    continue_on_error: bool,
    video_id: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    match channel {
        Some(ch) => {
            // Existing single-channel path
            let queue_file = artifact::read_queue_file(output_root, ch)?;
            let pending: Vec<_> = queue_file
                .queued
                .into_iter()
                .filter(|vod| {
                    let artifact_dir = output_root.join(&vod.video_id);
                    let status = artifact::read_status(&artifact_dir).unwrap_or(None);
                    !status.map(|s| s.downloaded).unwrap_or(false)
                })
                .collect();

            let pending = if let Some(id) = video_id {
                let filtered: Vec<_> = pending.into_iter().filter(|v| v.video_id == id).collect();
                if filtered.is_empty() {
                    return Err(format!("video ID {id} not found in any queue").into());
                }
                filtered
            } else {
                pending
            };

            if pending.is_empty() {
                println!("All queued VODs already downloaded.");
                return Ok(());
            }
            println!("Downloading {} pending VOD(s) for {ch}...", pending.len());
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
        None => {
            // New no-channel path: walk all queues and download pending items
            let all_vods = artifact::scan_queue_files(output_root)?;
            let artifact_statuses = artifact::scan_artifact_statuses(output_root)?;
            
            // Build HashSet of downloaded video IDs for O(1) lookup
            let downloaded_ids: std::collections::HashSet<String> = artifact_statuses
                .iter()
                .filter_map(|(video_id, status_opt)| {
                    status_opt.as_ref()
                        .filter(|s| s.downloaded)
                        .map(|_| video_id.clone())
                })
                .collect();
            
            // Filter to pending (not downloaded)
            let pending: Vec<_> = all_vods
                .into_iter()
                .filter(|vod| !downloaded_ids.contains(&vod.video_id))
                .collect();

            let pending = if let Some(id) = video_id {
                let filtered: Vec<_> = pending.into_iter().filter(|v| v.video_id == id).collect();
                if filtered.is_empty() {
                    return Err(format!("video ID {id} not found in any queue").into());
                }
                filtered
            } else {
                pending
            };

            if pending.is_empty() {
                println!("All queued VODs already downloaded.");
                return Ok(());
            }
            println!("Downloading {} pending VOD(s) across all channels...", pending.len());
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
    }
}

async fn transcribe_all(
    output_root: &std::path::Path,
    continue_on_error: bool,
    video_id: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let items = artifact::scan_artifact_statuses(output_root)?;
    let pending: Vec<_> = items
        .into_iter()
        .filter_map(|(vid, status)| {
            let s = status?;
            if s.downloaded
                && !s.transcribed
                && s.transcription_outcome.as_deref() != Some("suspect")
            {
                Some((vid, s))
            } else {
                None
            }
        })
        .collect();

    let pending = if let Some(id) = video_id {
        let filtered: Vec<_> = pending.into_iter().filter(|(vid, _)| vid == id).collect();
        if filtered.is_empty() {
            return Err(format!("video ID {id} not found in any artifact").into());
        }
        filtered
    } else {
        pending
    };

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

async fn cleanup(
    output_root: &std::path::Path,
    delete: bool,
    delete_all: bool,
    video_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate delete mode arguments
    if delete {
        if !delete_all && video_id.is_none() {
            return Err("Error: --delete requires either --all or --video-id <video_id>".into());
        }
        if delete_all && video_id.is_some() {
            return Err("Error: Cannot use both --all and --video-id together".into());
        }
    }

    // Scan all artifacts and find cleanup candidates
    let items = artifact::scan_artifact_statuses(output_root)?;
    let mut candidates = Vec::new();

    for (vid, status_opt) in &items {
        if let Some(status) = status_opt {
            // Only include if ready_for_notes is true
            // and transcription_outcome is "completed" (not "suspect" or "failed")
            if status.ready_for_notes
                && status.transcription_outcome.as_deref() == Some("completed")
            {
                let artifact_dir = output_root.join(vid);
                let audio_path = artifact_dir.join("audio.m4a");
                let transcript_path = artifact_dir.join("transcript.srt");

                let audio_size = std::fs::metadata(&audio_path).map(|m| m.len()).ok();
                let transcript_size = std::fs::metadata(&transcript_path).map(|m| m.len()).ok();

                candidates.push((
                    vid.clone(),
                    audio_path,
                    transcript_path,
                    audio_size,
                    transcript_size,
                ));
            }
        }
    }

    if candidates.is_empty() {
        println!("No cleanup candidates found.");
        return Ok(());
    }

    // Display candidates with file sizes
    println!("Cleanup candidates ({} found):", candidates.len());
    println!("{}", "-".repeat(70));
    println!(
        "{:<15} {:<20} {:<20}",
        "VIDEO_ID", "audio.m4a", "transcript.srt"
    );
    println!("{}", "-".repeat(70));

    let mut total_bytes = 0u64;
    for (vid, _audio_path, _transcript_path, audio_size, transcript_size) in &candidates {
        let audio_display = audio_size
            .map(|s| format_bytes(s))
            .unwrap_or_else(|| "(missing)".to_string());
        let transcript_display = transcript_size
            .map(|s| format_bytes(s))
            .unwrap_or_else(|| "(missing)".to_string());

        println!("{:<15} {:<20} {:<20}", vid, audio_display, transcript_display);

        if let Some(size) = audio_size {
            total_bytes += size;
        }
        if let Some(size) = transcript_size {
            total_bytes += size;
        }
    }

    println!("{}", "-".repeat(70));
    println!("Total space to be freed: {}", format_bytes(total_bytes));

    // If no deletion requested, return now
    if !delete {
        return Ok(());
    }

    // Perform deletion
    let to_delete: Vec<_> = if delete_all {
        candidates.clone()
    } else if let Some(ref vid) = video_id {
        candidates
            .into_iter()
            .filter(|(v, _, _, _, _)| v == vid)
            .collect()
    } else {
        return Err("Internal error: delete mode not properly validated".into());
    };

    if to_delete.is_empty() {
        if let Some(ref vid) = video_id {
            println!("Video {} is not a cleanup candidate", vid);
        }
        return Ok(());
    }

    println!("\nDeleting {} artifact(s)...", to_delete.len());
    for (vid, audio_path, transcript_path, _, _) in to_delete {
        let mut deleted_count = 0;
        
        if audio_path.exists() {
            std::fs::remove_file(&audio_path)?;
            println!("  Deleted {}/audio.m4a", vid);
            deleted_count += 1;
        }
        
        if transcript_path.exists() {
            std::fs::remove_file(&transcript_path)?;
            println!("  Deleted {}/transcript.srt", vid);
            deleted_count += 1;
        }

        if deleted_count == 0 {
            println!("  Warning: {} had no eligible files to delete", vid);
        }
    }

    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

fn derive_stage(status: &Option<artifact::ProcessStatus>, artifact_dir: &std::path::Path) -> &'static str {
    match status {
        None => {
            if artifact::find_media_file(artifact_dir).is_some() {
                "downloaded"
            } else {
                "queued"
            }
        }
        Some(s) => {
            if !s.downloaded {
                "queued"
            } else if s.transcription_outcome.as_deref() == Some("failed") {
                "failed"
            } else if s.transcription_outcome.as_deref() == Some("suspect") {
                "suspect"
            } else if s.ready_for_notes {
                "ready"
            } else if s.transcribed {
                "ready"
            } else {
                "downloaded"
            }
        }
    }
}

async fn show_status(output_root: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Artifact-dir items
    let artifact_items = artifact::scan_artifact_statuses(output_root)?;
    let artifact_ids: std::collections::HashSet<String> = artifact_items
        .iter()
        .map(|(id, _)| id.clone())
        .collect();

    // 2. Queued-only items (not already in artifact dirs)
    let queued_vods = artifact::scan_queue_files(output_root)?;
    let queued_only: Vec<_> = queued_vods
        .into_iter()
        .filter(|v| !artifact_ids.contains(&v.video_id))
        .collect();

    // Early exit if nothing to show
    if artifact_items.is_empty() && queued_only.is_empty() {
        println!("No artifacts found in {}", output_root.display());
        return Ok(());
    }

    // TODO(sort): rows appear in filesystem/queue-walk order; sort-by-date-desc is a future enhancement
    let total = artifact_items.len() + queued_only.len();

    println!(
        "{:<10} {:<42} {:<16} {:<12} {:<12} {}",
        "STAGE", "TITLE", "CHANNEL", "DATE", "OUTCOME", "REASON"
    );
    println!("{}", "-".repeat(105));

    // Queued-only rows first
    for vod in &queued_only {
        let date = if vod.uploaded_at.len() >= 10 {
            vod.uploaded_at[..10].to_string()
        } else {
            "—".to_string()
        };
        println!(
            "{:<10} {:<42} {:<16} {:<12} {:<12} {}",
            "queued",
            truncate(&vod.title, 40),
            truncate(&vod.channel, 14),
            date,
            "—",
            "—",
        );
    }

    // Artifact-dir rows
    for (video_id, status) in &artifact_items {
        let artifact_dir = output_root.join(video_id);
        let metadata = artifact::read_metadata(&artifact_dir).unwrap_or(None);

        let title = metadata
            .as_ref()
            .and_then(|m| m.title.as_deref())
            .unwrap_or("—")
            .to_string();
        let channel = metadata
            .as_ref()
            .and_then(|m| m.channel.as_deref())
            .unwrap_or("—")
            .to_string();
        let uploaded_at = metadata
            .as_ref()
            .and_then(|m| m.uploaded_at.as_deref())
            .unwrap_or("");
        let date = if uploaded_at.len() >= 10 {
            uploaded_at[..10].to_string()
        } else {
            "—".to_string()
        };

        let stage = derive_stage(status, &artifact_dir);

        let outcome = status
            .as_ref()
            .and_then(|s| s.transcription_outcome.as_deref())
            .unwrap_or("—");
        let reason = status
            .as_ref()
            .and_then(|s| {
                s.transcription_reason
                    .as_deref()
                    .or(s.last_error.as_deref())
            })
            .unwrap_or("—");

        println!(
            "{:<10} {:<42} {:<16} {:<12} {:<12} {}",
            stage,
            truncate(&title, 40),
            truncate(&channel, 14),
            date,
            outcome,
            truncate(reason, 35),
        );
    }

    println!("\n{} item(s) total", total);
    Ok(())
}
