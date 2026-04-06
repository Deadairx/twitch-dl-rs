---
estimated_steps: 97
estimated_files: 2
skills_used: []
---

# T02: Wire TranscriptionOutcome into main.rs, update transcribe-all filter, and update status display

Update src/main.rs to consume the new TranscriptionOutcome from transcribe_to_srt_and_vtt(), map outcomes to ProcessStatus fields, fix the transcribe-all suspect-skip logic, and update the status command to show the OUTCOME column. Add a unit test verifying ProcessStatus backward compat deserialization.

**Context for executor:** T01 rewrote src/transcribe.rs and extended src/artifact.rs ProcessStatus. This task wires those changes into src/main.rs. The key changes are:
1. transcribe_artifact() — call new transcribe_to_srt_and_vtt() instead of transcribe_to_txt(); map TranscriptionOutcome variants to status fields.
2. transcribe_all() — add suspect-skip guard to the filter.
3. show_status() — extend table with OUTCOME column.
4. Get audio duration via ffprobe before calling transcribe_to_srt_and_vtt().

**transcribe_artifact() rewrite in src/main.rs:**
The function signature stays the same. Replace body logic:
```rust
// Delete stale transcripts before retry — handled inside transcribe_to_srt_and_vtt
// Get audio duration for word-count heuristic
let duration_secs = get_audio_duration_secs(media_path);

println!("Transcribing {} with hear...", video_id);
match transcribe::transcribe_to_srt_and_vtt(media_path, artifact_dir, duration_secs) {
    TranscriptionOutcome::Completed { srt_path, vtt_path, word_count } => {
        status.transcribed = true;
        status.transcription_outcome = Some("completed".to_string());
        status.transcription_reason = None;
        status.transcript_word_count = Some(word_count);
        status.transcript_file = srt_path.file_name().map(|n| n.to_string_lossy().to_string());
        status.last_error = None;
        artifact::write_status(artifact_dir, status)?;
        Ok(())
    }
    TranscriptionOutcome::Suspect { srt_path, vtt_path, word_count, reason } => {
        // suspect: leave transcribed=false, set outcome fields, do NOT return Err
        status.transcribed = false;
        status.transcription_outcome = Some("suspect".to_string());
        status.transcription_reason = Some(reason);
        status.transcript_word_count = Some(word_count);
        status.transcript_file = srt_path.file_name().map(|n| n.to_string_lossy().to_string());
        status.last_error = None;
        artifact::write_status(artifact_dir, status)?;
        Ok(()) // NOT an error — pipeline continues
    }
    TranscriptionOutcome::Failed { reason } => {
        status.transcribed = false;
        status.transcription_outcome = Some("failed".to_string());
        status.transcription_reason = Some(reason.clone());
        status.last_error = Some(reason.clone());
        artifact::write_status(artifact_dir, status)?;
        Err(reason.into())
    }
}
```
Note: the existing transcript.txt reuse check at the top of transcribe_artifact() must be REMOVED — it checks for the old transcript.txt format. The reuse check should instead look for transcript.srt + transcript.vtt both present AND outcome == Some("completed") in status. Or more simply: if status.transcribed == true (meaning completed), the transcribe_all filter already skips it — so the per-item reuse check in transcribe_artifact() can just be removed. The transcribe_all filter handles idempotency.

**get_audio_duration_secs() helper:**
Add a private fn in src/main.rs:
```rust
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
```
If ffprobe fails or duration is unavailable, return None — transcribe_to_srt_and_vtt() handles None by skipping the word-count heuristic.

**transcribe_all() filter update:**
The current filter: `if s.downloaded && !s.transcribed`. Add suspect guard:
```rust
if s.downloaded
    && !s.transcribed
    && s.transcription_outcome.as_deref() != Some("suspect")
{
    Some((video_id, s))
} else {
    None
}
```

**show_status() update:**
Replace the TRANSCRIBED column with an OUTCOME column. Show: 'completed', 'suspect', 'failed', or '-' (for not yet transcribed and no outcome set). Keep DOWNLOADED column. Show REASON as a truncated column (40 chars) replacing LAST_ERROR — since transcription_reason is more useful than last_error for debugging.
```
VIDEO_ID        DOWNLOADED   OUTCOME      REASON
```
Or keep LAST_ERROR as a fallback if transcription_reason is None — use whichever is Some.

**Reuse check for existing srt+vtt:**
In transcribe_artifact(), add a reuse check at the top for the NEW format:
```rust
let srt_path = artifact_dir.join("transcript.srt");
let vtt_path = artifact_dir.join("transcript.vtt");
if srt_path.exists() && vtt_path.exists() && status.transcribed {
    println!("Reusing existing transcript for {}", video_id);
    return Ok(());
}
```
This replaces the old transcript.txt check.

**Unit test to add (in src/artifact.rs #[cfg(test)] module):**
- test_process_status_backward_compat: deserialize a JSON string that lacks the three new fields (transcription_outcome, transcription_reason, transcript_word_count) and assert deserialization succeeds with all three as None.

The test JSON should be the minimal old-schema representation:
```json
{"schema_version":1,"video_id":"abc","source_url":"https://twitch.tv/videos/abc","media_file":null,"transcript_file":null,"downloaded":true,"transcribed":false,"last_error":null,"updated_at_epoch_s":0}
```

## Inputs

- ``src/transcribe.rs` — TranscriptionOutcome enum and transcribe_to_srt_and_vtt() from T01`
- ``src/artifact.rs` — ProcessStatus with new transcription_outcome/reason/word_count fields from T01`
- ``src/main.rs` — existing transcribe_artifact(), transcribe_all(), show_status() to be updated`

## Expected Output

- ``src/main.rs` — transcribe_artifact() consuming TranscriptionOutcome, transcribe_all() with suspect-skip guard, show_status() with OUTCOME column, get_audio_duration_secs() helper`
- ``src/artifact.rs` — backward compat unit test added`

## Verification

cargo test && cargo build 2>&1 | grep -c warning | xargs -I{} sh -c 'test {} -eq 0 && echo "zero warnings" || (cargo build 2>&1; exit 1)'

## Observability Impact

status command now shows OUTCOME column (completed/suspect/failed/-) and REASON column; suspects are visually distinct from not-yet-transcribed; failed items show reason for diagnosis
