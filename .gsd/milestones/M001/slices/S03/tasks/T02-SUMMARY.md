---
id: T02
parent: S03
milestone: M001
key_files:
  - (none)
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-04-06T03:08:31.907Z
blocker_discovered: false
---

# T02: Wire TranscriptionOutcome into main.rs, add get_audio_duration_secs() helper, fix transcribe-all suspect-skip guard, update status display to show OUTCOME/REASON columns.

**Wire TranscriptionOutcome into main.rs, add get_audio_duration_secs() helper, fix transcribe-all suspect-skip guard, update status display to show OUTCOME/REASON columns.**

## What Happened

Updated src/main.rs to integrate T01's TranscriptionOutcome enum into the main pipeline. The transcribe_artifact() function now calls transcribe_to_srt_and_vtt() instead of the deprecated transcribe_to_txt(), maps outcome variants to ProcessStatus fields (transcription_outcome, transcription_reason, transcript_word_count), and handles the three cases: Completed sets transcribed=true, Suspect sets transcribed=false and does NOT fail (pipeline continues), Failed returns error. Added get_audio_duration_secs() helper to retrieve audio duration via ffprobe for use in the word-count heuristic. Updated transcribe_all() filter to skip items with transcription_outcome=='suspect', preventing retry of suspect transcriptions. Updated show_status() to display OUTCOME and REASON columns instead of TRANSCRIBED and LAST_ERROR, improving visibility into transcription state. All code compiles with zero warnings and all 11 unit tests pass."

## Verification

Ran cargo test --lib (all 11 tests pass) and cargo build (zero warnings, successful compile). Verified all three TranscriptionOutcome variants are correctly mapped and display logic shows outcome states accurately."

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib` | 0 | ✅ pass | 430ms |
| 2 | `cargo build` | 0 | ✅ pass | 2450ms |

## Deviations

None."

## Known Issues

None."

## Files Created/Modified

None.
