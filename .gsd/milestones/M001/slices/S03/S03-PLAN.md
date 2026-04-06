# S03: Trustworthy transcription and failure surfacing

**Goal:** Replace the mlx-whisper transcription path with a `hear -d -i <audio-file> -S` invocation, capture stdout as transcript.srt, convert to transcript.vtt, apply quality heuristics (word-count and repetition), and persist one of three terminal outcomes (completed / suspect / failed) with structured reasons into status.json.
**Demo:** After this: Finished items produce transcript artifacts you can trust more than the current fast path, and failed transcriptions show clear reasons and remain recoverable.

## Tasks
- [x] **T01: Replace mlx-whisper with hear-backed transcription, add TranscriptionOutcome enum, SRT→VTT conversion, quality heuristics, and comprehensive unit tests.** — Replace the mlx-whisper implementation in src/transcribe.rs with a hear-backed transcription function. Extend ProcessStatus in src/artifact.rs with three new fields. Add unit tests covering all pure logic.

**Context for executor:** The current src/transcribe.rs calls mlx-whisper via Command, returns PathBuf. The new implementation must:
1. Extend src/artifact.rs ProcessStatus struct with three optional fields (backward-compat required).
2. Define a TranscriptionOutcome enum in src/transcribe.rs (no artifact.rs coupling).
3. Implement hear invocation, stdout capture, SRT write, VTT conversion, and heuristics.
4. Add unit tests — no audio needed, all logic is testable with synthetic strings.

**ProcessStatus additions (src/artifact.rs):**
Add to the ProcessStatus struct (all must have #[serde(default)] or be Option to preserve backward compat with existing status.json files that lack these fields):
```rust
pub transcription_outcome: Option<String>,  // "completed" | "suspect" | "failed"
pub transcription_reason: Option<String>,   // human-readable reason
pub transcript_word_count: Option<u64>,     // word count of SRT text
```
Also add: `#[serde(default)]` to the entire struct or to each new field. Use `#[serde(skip_serializing_if = "Option::is_none")]` on the three new fields to keep old JSON files clean.

**TranscriptionOutcome enum (src/transcribe.rs):**
```rust
pub enum TranscriptionOutcome {
    Completed { srt_path: PathBuf, vtt_path: PathBuf, word_count: u64 },
    Suspect   { srt_path: PathBuf, vtt_path: PathBuf, word_count: u64, reason: String },
    Failed    { reason: String },
}
```

**Public function signature:**
```rust
pub fn transcribe_to_srt_and_vtt(
    media_path: &Path,
    artifact_dir: &Path,
    duration_secs: Option<f64>,
) -> TranscriptionOutcome
```
Return TranscriptionOutcome directly (not Result) — Failed variant covers all error cases, removing the need for a separate error type. The caller does not need to distinguish IO errors from transcription failures at this level.

**hear invocation:**
```rust
let output = Command::new("hear")
    .args(["-d", "-i"])
    .arg(media_path)
    .arg("-S")
    .output()
    .map_err(|e| TranscriptionOutcome::Failed { reason: e.to_string() });
```
Capture stdout bytes. If hear exits non-zero, return Failed with stderr as reason.

**SRT→VTT conversion (pure Rust, ~15 lines):**
Algorithm: prepend WEBVTT header; iterate SRT lines; skip lines that are all-numeric (sequence numbers — hear -S resets these non-monotonically so strip unconditionally); on timestamp lines replace "," with "."; copy text and blank lines verbatim.
```rust
fn srt_to_vtt(srt: &str) -> String {
    let mut out = String::from("WEBVTT\n\n");
    for line in srt.lines() {
        let trimmed = line.trim();
        if trimmed.chars().all(|c| c.is_ascii_digit()) {
            continue; // strip sequence numbers
        } else if trimmed.contains(" --> ") {
            out.push_str(&line.replace(',', "."));
            out.push('\n');
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}
```

**Word extraction from SRT text:**
Extract text-only lines (not sequence numbers, not timestamp lines, not blank). Split on whitespace, count. Store as word_count.

**Quality heuristics:**
1. Word-count threshold: if duration_secs is Some(d) and word_count < (d / 3600.0) * 50.0, flag suspect with reason "word count {word_count} below threshold {threshold:.0} for {duration:.0}s audio".
2. Repetition detection: slide 200-word window over word list. Build trigrams in window. If any trigram appears >10 times in the window, flag suspect with reason "repeated trigram detected in 200-word window".
Apply in order: if either flags suspect, return Suspect variant. Reason carries the first triggered heuristic.

**File writes:**
Write SRT to artifact_dir/transcript.srt, VTT to artifact_dir/transcript.vtt. Both must succeed before returning Completed or Suspect. If either write fails, return Failed.

**Partial cleanup on retry:**
At the START of transcribe_to_srt_and_vtt, delete transcript.srt and transcript.vtt if they exist in artifact_dir. This handles stale partial files from prior failed runs.

**Unit tests to add (in src/transcribe.rs #[cfg(test)] module):**
- test_srt_to_vtt_basic: provide a 3-line SRT with sequence numbers, timestamps, and text; assert VTT output has WEBVTT header, no sequence lines, timestamps with '.' not ',', and text intact.
- test_srt_to_vtt_resetting_sequence: provide SRT where sequence resets to 1 mid-way; assert all sequence lines stripped.
- test_word_count_threshold_flags_suspect: call heuristic logic with word_count=10 and duration_secs=7200.0 (100 words/hour threshold → 10 < 100 → suspect).
- test_repetition_heuristic_flags_suspect: build a 200-word list with a trigram repeated 11 times; assert suspect.
- test_repetition_heuristic_clean_input: build a 200-word list with no repetition; assert not suspect.
  - Estimate: 2h
  - Files: src/transcribe.rs, src/artifact.rs
  - Verify: cargo test transcribe::tests && cargo test artifact::tests
- [x] **T02: Wire TranscriptionOutcome into main.rs, add get_audio_duration_secs() helper, fix transcribe-all suspect-skip guard, update status display to show OUTCOME/REASON columns.** — Update src/main.rs to consume the new TranscriptionOutcome from transcribe_to_srt_and_vtt(), map outcomes to ProcessStatus fields, fix the transcribe-all suspect-skip logic, and update the status command to show the OUTCOME column. Add a unit test verifying ProcessStatus backward compat deserialization.

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
  - Estimate: 1.5h
  - Files: src/main.rs, src/artifact.rs
  - Verify: cargo test && cargo build 2>&1 | grep -c warning | xargs -I{} sh -c 'test {} -eq 0 && echo "zero warnings" || (cargo build 2>&1; exit 1)'
