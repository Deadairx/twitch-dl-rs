---
id: S03
parent: M001
milestone: M001
provides:
  - Three-outcome transcription (completed/suspect/failed) with structured failure reasons
  - SRT and VTT transcript artifacts with quality heuristics applied
  - Backward-compatible ProcessStatus extension with transcription metadata
  - hear-based transcription replacing mlx-whisper with verifiable output
requires:
  - slice: S01
    provides: Durable artifact state and status.json persistence
  - slice: S02
    provides: Staged processing dispatch and transcribe-all command
affects:
  - S04
  - S05
key_files:
  - src/transcribe.rs
  - src/artifact.rs
  - src/main.rs
  - src/lib.rs
key_decisions:
  - Replaced mlx-whisper with hear -d -i -S invocation for on-device transcription
  - Suspect transcriptions do NOT block pipeline (outcome visible, retried on rerun)
  - Word-count threshold: 50 words/hour of audio duration
  - Repetition detection: trigrams appearing >10 times in 200-word window
  - Both SRT and VTT must succeed for completed outcome
patterns_established:
  - TranscriptionOutcome enum as non-Result error type
  - Heuristics applied at transcription boundary, not downstream
  - Partial file cleanup at start of transcription to prevent stale state
  - Status display shows three-valued outcome column with reason truncation
observability_surfaces:
  - show_status command: OUTCOME and REASON columns
  - ProcessStatus.transcription_outcome, transcription_reason, transcript_word_count in artifact state
  - hear stderr captured and persisted as reason on failed outcome
drill_down_paths:
  - .gsd/milestones/M001/slices/S03/tasks/T01-SUMMARY.md
  - .gsd/milestones/M001/slices/S03/tasks/T02-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-04-06T03:10:51.661Z
blocker_discovered: false
---

# S03: Trustworthy transcription and failure surfacing

**Replaced mlx-whisper with hear-backed transcription, added quality heuristics, and surfaced three terminal outcomes (completed/suspect/failed) with structured failure reasons in artifact state.**

## What Happened

T01 rewrote src/transcribe.rs to replace the mlx-whisper invocation with a hear-based transcription pipeline. Implemented TranscriptionOutcome enum (Completed/Suspect/Failed variants) with no Result wrapper, hear invocation as `hear -d -i <audio-file> -S` capturing stdout as transcript.srt, pure Rust SRT→VTT conversion (prepend WEBVTT header, strip sequence numbers, normalize timestamps), word-count threshold heuristic (flag suspect if < 50 words/hour of audio), and repetition detection heuristic (flag suspect if trigram appears >10 times in 200-word window). Extended ProcessStatus in src/artifact.rs with three new optional fields (backward-compatible via serde defaults): transcription_outcome, transcription_reason, transcript_word_count. Implemented partial file cleanup at transcription start.

T02 wired TranscriptionOutcome into src/main.rs. transcribe_artifact() now maps outcome variants to ProcessStatus fields: Completed sets transcribed=true and outcome="completed"; Suspect sets transcribed=false and outcome="suspect" without failing (pipeline continues); Failed sets transcribed=false and outcome="failed" and returns error. Added get_audio_duration_secs() helper to retrieve audio duration via ffprobe for word-count heuristic. Updated transcribe_all() filter to skip items with transcription_outcome=="suspect". Updated show_status() table with OUTCOME column (completed/suspect/failed/—) and REASON column with 40-char truncation and fallback to last_error.

All 11 unit tests pass (7 transcribe, 4 artifact). Cargo build produces zero warnings."

## Verification

All 11 unit tests pass: `cargo test transcribe::tests` (7/7) and `cargo test artifact::tests` (4/4). Build successful with zero warnings. Code inspection confirms: TranscriptionOutcome mapping in transcribe_artifact() handles all three variants correctly with proper ProcessStatus field assignments; transcribe_all() filter skips suspect items; show_status() displays OUTCOME and REASON columns with truncation. Backward compatibility test confirms old JSON files without new fields deserialize cleanly."

## Requirements Advanced

- R004 — Completed transcript artifacts now pass quality heuristics (word-count and repetition checks); suspect items are visible and separately marked; failed items show clear reasons. Trustworthiness is enhanced vs. the old fast path.
- R005 — Three-outcome transcription (completed/suspect/failed) is now visible in status.json and status command output; failure reasons (heuristic triggers or hear errors) are structured and displayed; items remain in durable artifact state for recovery.

## Requirements Validated

- R004 — Unit tests verify heuristics work correctly (word-count threshold, repetition detection); completed artifacts have both SRT and VTT; backward compat test confirms state integrity
- R005 — Status display shows OUTCOME and REASON columns; transcription_reason field captures structured failure reasons; suspect and failed items remain visible and recoverable

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Deviations


  "None. Implementation matches plan exactly."

## Known Limitations


  "1. No manual force-retry for suspect items (deferred to M002). 2. Assumes hear handles multi-hour .m4a files (chunking becomes blocker if it doesn't). 3. No AI correction pass (deferred to M002). 4. Locale and punctuation flags not wired. 5. No plain-text transcript format."

## Follow-ups


  "1. Real-world testing on long VODs (>2h) to validate hear performance and heuristic accuracy. 2. Suspect item handling UX (should transcribe-all have --force-retry flag?). 3. Downstream integration with S04: how should suspect transcriptions be treated in ready-for-notes state?"

## Files Created/Modified

- `src/transcribe.rs` — Replaced mlx-whisper with hear-backed transcription, added TranscriptionOutcome enum, SRT→VTT conversion, quality heuristics, unit tests
- `src/artifact.rs` — Extended ProcessStatus with transcription_outcome, transcription_reason, transcript_word_count fields (backward-compatible)
- `src/main.rs` — Wired TranscriptionOutcome into transcribe_artifact(), added get_audio_duration_secs(), updated transcribe_all() filter, updated show_status() display
- `src/lib.rs` — Created to enable unit testing on binary-only crate
