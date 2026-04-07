---
estimated_steps: 19
estimated_files: 3
skills_used: []
---

# T01: Add --force-suspect flag to transcribe-all and update filter predicate

Add `force_suspect: bool` to the `TranscribeAll` CLI variant, wire the `--force-suspect` clap arg, thread the flag through dispatch into `transcribe_all()`, update the filter predicate to include suspect items when `force_suspect=true`, and add a unit test verifying the filter behavior.

The executor should follow the exact pattern established by `--continue-on-error` and `--video-id` — all three are bool/Option fields on `TranscribeAll`, clap args using `ArgAction::SetTrue` (for the bool) or value args (for Option<String>), and the flag is destructured in the dispatch match arm.

**No changes to `transcribe_artifact()` are needed.** Suspect items have `status.transcribed = false`, so the existing reuse guard (`srt_path.exists() && vtt_path.exists() && status.transcribed`) evaluates to `false` for suspect items and does not block re-transcription. `transcribe_to_srt_and_vtt()` already cleans up stale SRT/VTT at entry.

The filter predicate in `transcribe_all()` currently is:
```rust
if s.downloaded && !s.transcribed && s.transcription_outcome.as_deref() != Some("suspect")
```
Replace with:
```rust
let is_suspect = s.transcription_outcome.as_deref() == Some("suspect");
let include = s.downloaded && ((!s.transcribed && !is_suspect) || (force_suspect && is_suspect));
if include
```
This correctly composes with the `--video-id` post-filter (which applies to `pending` after it is built) without any additional logic.

Add a unit test in `src/lib.rs` (the project's test home, following the lib.rs re-export pattern established in M001/S03) that:
1. Creates three mock `(vid, ProcessStatus)` entries: one normal pending (downloaded=true, transcribed=false, no outcome), one suspect (downloaded=true, transcribed=false, outcome="suspect"), one completed (downloaded=true, transcribed=true)
2. Applies the filter predicate with `force_suspect=false` and asserts only the normal pending item passes
3. Applies the filter with `force_suspect=true` and asserts both the normal pending item and the suspect item pass (completed still excluded)

The test should exercise the predicate logic directly (not call `transcribe_all` which is async and touches the filesystem).

## Inputs

- ``src/cli.rs` — TranscribeAll variant and transcribe-all subcommand arg definitions`
- ``src/main.rs` — dispatch match arm for TranscribeAll and transcribe_all() function`
- ``src/lib.rs` — existing test module location`

## Expected Output

- ``src/cli.rs` — TranscribeAll struct with force_suspect field; --force-suspect arg in transcribe-all subcommand; parse in match arm`
- ``src/main.rs` — transcribe_all() accepts force_suspect param; dispatch passes it through; filter predicate updated`
- ``src/lib.rs` — unit test verifying force_suspect filter predicate logic (suspect included when true, excluded when false)`

## Verification

cargo test 2>&1 | grep -E 'result|FAILED'; cargo build 2>&1 | grep -E 'error|warning.*unused'
