---
estimated_steps: 77
estimated_files: 2
skills_used: []
---

# T01: Rewrite transcribe.rs with hear invocation, SRT capture, VTT conversion, and quality heuristics

Replace the mlx-whisper implementation in src/transcribe.rs with a hear-backed transcription function. Extend ProcessStatus in src/artifact.rs with three new fields. Add unit tests covering all pure logic.

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

## Inputs

- ``src/transcribe.rs` — existing mlx-whisper implementation to be replaced`
- ``src/artifact.rs` — ProcessStatus struct to be extended with three new fields`

## Expected Output

- ``src/transcribe.rs` — rewritten with hear invocation, TranscriptionOutcome enum, srt_to_vtt(), quality heuristics, and unit tests`
- ``src/artifact.rs` — ProcessStatus extended with transcription_outcome, transcription_reason, transcript_word_count fields (all serde(default) / Option)`

## Verification

cargo test transcribe::tests && cargo test artifact::tests

## Observability Impact

TranscriptionOutcome enum variants carry structured reason strings; Failed variant surfaces hear exit code and stderr; Suspect variant carries heuristic trigger name and word count — these feed into status.json in T02
