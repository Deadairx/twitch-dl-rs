# S03: Trustworthy transcription and failure surfacing — Research

**Date:** 2026-04-05
**Milestone:** M001
**Slice:** S03
**Requirements owned:** R004 (transcript trustworthiness), R005 (failure visibility)

---

## Summary

S03 is targeted work: replace the current `mlx-whisper` call in `src/transcribe.rs` with `hear -d -i <file> -S`, capture stdout as `transcript.srt`, convert to `transcript.vtt`, apply quality heuristics, and persist a three-outcome transcription result (`completed` / `suspect` / `failed`) into `status.json`. The `hear` binary is confirmed present at `/usr/local/bin/hear` (version 0.6) and works correctly: it writes SRT to stdout, errors to stderr, and exits non-zero on failure.

The `ProcessStatus` struct in `artifact.rs` currently tracks `transcribed: bool` and `last_error: Option<String>`. S03 needs to extend it with three new fields: `transcription_outcome: Option<String>`, `transcription_reason: Option<String>`, and `transcript_word_count: Option<u64>`. The boolean `transcribed` remains but semantics narrow: only `completed` sets it to `true`. The `transcribe-all` filter already handles `failed` items correctly (retries when `downloaded && !transcribed`); `suspect` items need explicit skip logic since they have `transcribed = false` but outcome set.

SRT→VTT conversion is implementable in pure Rust in ~15 lines: prepend `WEBVTT\n\n`, strip sequence number lines, replace `,` with `.` in timestamp lines. No external library or `ffmpeg` needed. The audio duration needed for the word-count heuristic is obtained via `ffprobe -v quiet -print_format json -show_streams <file>`, which is already available (`/opt/homebrew/bin/ffprobe`) since `ffmpeg` is a project dependency.

One important discovery: `hear -S` produces non-standard SRT where the sequence counter resets to `1` at irregular intervals (observed in real output). This does not affect the pipeline — the pure-Rust SRT→VTT converter strips sequence number lines entirely — but it is worth noting for any future tooling that assumes strictly-incrementing SRT sequences.

---

## Recommendation

**Two tasks.** T01 extends `ProcessStatus` and rewrites `transcribe.rs` with the `hear` invocation, stdout capture, and three-outcome result type. T02 wires the new outcome into `transcribe_artifact()` in `main.rs`, updates the `transcribe-all` skip logic for `suspect` items, updates the `status` display command to show outcome, and adds unit tests.

Keep SRT→VTT as a pure-Rust conversion in `transcribe.rs` — no new dependency. Use `ffprobe` for audio duration only when computing the word-count heuristic; call it once per transcription attempt and pass the result through. Partial/suspect transcript files must be deleted before a retry attempt — same pattern as partial download cleanup already in `download_vod_to_artifact`.

---

## Implementation Landscape

### Key Files

- `src/transcribe.rs` — **Replace entirely.** Current implementation calls `mlx-whisper` via `Command`, returns a `PathBuf`. New implementation: `transcribe_to_srt_and_vtt(media_path, artifact_dir, duration_secs) -> TranscriptionResult`. Captures `hear` stdout as the SRT content, writes `transcript.srt`, converts in-memory to VTT and writes `transcript.vtt`, then applies heuristics. The `TranscriptionError` enum stays but adds `Suspect { reason, word_count }` variant (or a separate `TranscriptionResult` enum is cleaner — see below).

- `src/artifact.rs` — **Extend `ProcessStatus`.** Add three fields:
  ```rust
  pub transcription_outcome: Option<String>,   // "completed" | "suspect" | "failed"
  pub transcription_reason: Option<String>,    // human-readable reason string
  pub transcript_word_count: Option<u64>,      // word count of SRT text content
  ```
  These are `Option` so existing `status.json` files (from S01/S02 artifacts) deserialize without error — `serde` fills missing fields with `None`. Add a unit test to `artifact::tests` verifying the new fields round-trip and that an old-schema JSON (missing the new fields) deserializes successfully.

- `src/main.rs` — **Update `transcribe_artifact()` and `transcribe_all()`.** 
  - `transcribe_artifact()` calls the new `transcribe::transcribe_to_srt_and_vtt()`, maps the result to `transcription_outcome`/`reason`/`word_count` on `status`, and sets `status.transcribed = true` only for `completed`.
  - `transcribe_all()` filter currently: `s.downloaded && !s.transcribed` — must add: `&& s.transcription_outcome.as_deref() != Some("suspect")` to skip suspect items on re-run.
  - `show_status()` — extend table to show `OUTCOME` column or fold the outcome into the existing `TRANSCRIBED` column display. Suspect items must be visually distinct (not silently show as `false`).

### New Types in `transcribe.rs`

```rust
pub enum TranscriptionOutcome {
    Completed { srt_path: PathBuf, vtt_path: PathBuf, word_count: u64 },
    Suspect   { srt_path: PathBuf, vtt_path: PathBuf, word_count: u64, reason: String },
    Failed    { reason: String },
}
```

Return `TranscriptionOutcome` from `transcribe_to_srt_and_vtt()`. The caller (`main.rs`) maps to `status.json` fields. This keeps `transcribe.rs` free of `artifact` module coupling.

### SRT→VTT Conversion (pure Rust, ~15 lines)

SRT and WebVTT differ only in three ways: VTT has a `WEBVTT` header, timestamps use `.` instead of `,`, and sequence number lines are omitted. Algorithm:
1. `output.push_str("WEBVTT\n\n")`
2. For each line: skip lines that are purely numeric (sequence numbers); for timestamp lines replace `,` → `.`; copy text lines verbatim; copy blank lines verbatim.

`hear` outputs lines like:
```
1
00:00:02,160 --> 00:00:04,350
The stuff is difficult
```
where the sequence number resets to `1` at irregular intervals. The conversion handles this correctly since all-numeric lines are stripped unconditionally.

### Quality Heuristics

**Word count threshold:** Extract text-only content from SRT (lines that are neither sequence numbers nor timestamp lines nor blank). Split on whitespace, count words. Compare against `(duration_secs / 3600.0) * 50.0` — less than 50 words per hour flags suspect. For a 3.8-hour VOD this threshold is ~190 words.

**Repetition detection:** Slide a 200-word window over the word list. Build trigrams in each window. If any single trigram appears more than 10 times in a 200-word window (>5% density), flag suspect. This catches the hallucination pattern of repeated phrases without false-positives on normal speech.

**Audio duration source:** `ffprobe -v quiet -print_format json -show_streams <audio_path>` — parse the `duration` field from the first stream. Confirmed working. Call this once before heuristic evaluation. If `ffprobe` fails, skip the word-count heuristic and only apply repetition detection.

### Retry/Cleanup Rule

Before re-transcribing (any `failed` re-attempt or future `--retry-suspect`): delete `transcript.srt` and `transcript.vtt` if they exist in the artifact directory. Same integrity rule as partial download cleanup.

### `transcribe-all` Skip Logic

```
// Skip: already completed, or suspect (don't retry suspects without explicit flag)
if s.transcribed 
   || s.transcription_outcome.as_deref() == Some("suspect") {
    continue;
}
```

### Build Order

1. **T01** — Extend `ProcessStatus`, rewrite `transcribe.rs` (new `hear` invocation + SRT capture + VTT conversion + heuristics + `TranscriptionOutcome` type). Add unit tests for SRT→VTT conversion and heuristics (can test with synthetic SRT strings — no audio needed).
2. **T02** — Wire `TranscriptionOutcome` into `transcribe_artifact()` in `main.rs`, update `transcribe-all` filter, update `show_status()` to surface outcome. Add integration-level test verifying status round-trip with new fields.

T01 must complete before T02 since T02 consumes the new return type.

### Verification Approach

```bash
# Unit tests (no audio needed)
cargo test                      # all 3 existing tests + new T01 tests pass

# Build clean
cargo build                     # no warnings

# CLI smoke test (requires actual audio artifact)
./target/debug/twitch-dl-rs transcribe-all --output-root artifacts --continue-on-error
./target/debug/twitch-dl-rs status --output-root artifacts
# → status table shows OUTCOME column: completed/suspect/failed, not just true/false

# Artifact inspection
cat artifacts/<video_id>/status.json
# → contains transcription_outcome, transcription_reason, transcript_word_count
ls artifacts/<video_id>/
# → transcript.srt and transcript.vtt both present for completed items

# Re-run idempotency
./target/debug/twitch-dl-rs transcribe-all --output-root artifacts
# → completed and suspect items are skipped; failed items are retried
```

---

## Constraints

- `hear` canonical invocation: exactly `hear -d -i <audio-file> -S` — no additional flags.
- Both `transcript.srt` AND `transcript.vtt` must exist for `completed` state — partial outputs → `failed`.
- `suspect` items must NOT silently display as `transcribed = false` in status output.
- `transcribe-all` re-run must skip `suspect` items without a force flag (force flag is out of scope).
- `ProcessStatus` changes must be backward-compatible: existing `status.json` files from S01/S02 without the new fields must still deserialize (use `#[serde(default)]` or `Option` fields).
- No new Cargo dependencies — pure Rust SRT→VTT and `ffprobe` via `Command` (same pattern as existing `ffmpeg` calls).

---

## Common Pitfalls

- **`hear` stdout is the SRT content — use `Command::output()` not `Command::status()`** — the existing `transcribe.rs` uses `.output()` already, so this is already the right pattern. Do not pipe to a file via shell; capture `stdout` bytes directly and write from Rust.
- **`hear -S` outputs non-standard SRT with resetting sequence numbers** — the pure-Rust VTT converter must strip all-numeric lines unconditionally rather than expecting a strict incrementing sequence.
- **`suspect` items have `transcribed = false`** — the `transcribe-all` filter `s.downloaded && !s.transcribed` would retry suspects on every run without an explicit `transcription_outcome` check. Must add the outcome guard.
- **Backward compat for `status.json` deserialization** — annotate new `ProcessStatus` fields with `#[serde(default)]` (not just `Option`) if they have sensible defaults, or use `Option` + `#[serde(skip_serializing_if = "Option::is_none")]` to keep old files readable and keep new files clean.
- **`hear` may take minutes on long audio files** — this is expected; the process call is synchronous within `transcribe_artifact()`. No timeout logic is required for S03; operator cancels with Ctrl-C.
- **Partial transcript cleanup** — delete both `transcript.srt` and `transcript.vtt` if they exist before re-attempting a `failed` item. Otherwise a stale `.vtt` from a previous partial run could survive.

---

## Open Risks

- **`hear` reliability on multi-hour `.m4a` files** — confirmed it starts transcribing (we observed output at 45s on a real 3.8h file), but whether it completes cleanly is untested. If it stalls or OOM-exits mid-file, the `failed` outcome and partial cleanup path handles it, but chunking becomes a follow-up blocker. The slice plan explicitly defers chunking to an escalation path.
- **Word-count threshold calibration** — the 50 words/hour threshold is conservative but untested on real `hear` output at scale. If real streams produce low word counts legitimately (long pauses, music, ambient audio), the threshold may produce false `suspect` labels. The threshold is intentionally tunable and recorded in `status.json` for post-hoc review.
