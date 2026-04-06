---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Add ready_for_notes field to ProcessStatus and wire transition in transcribe_artifact

Add `ready_for_notes: bool` to `ProcessStatus` in `src/artifact.rs` with `#[serde(default)]` for backward compatibility. Set `status.ready_for_notes = true` in `transcribe_artifact()` in `src/main.rs` inside the `Completed` match arm, alongside the existing `status.transcribed = true`. Update `show_status()` to include a READY column (shows `yes` / `-`) after the OUTCOME column. Add two unit tests in `src/artifact.rs`: one confirming that old `status.json` JSON without the field deserializes as `false`, and one confirming `ready_for_notes = true` round-trips through write/read. Do NOT set `ready_for_notes` on `Suspect` or `Failed` outcomes.

## Inputs

- `src/artifact.rs`
- `src/main.rs`

## Expected Output

- `src/artifact.rs`
- `src/main.rs`

## Verification

cargo test artifact::tests 2>&1 | grep -E 'test result|FAILED' && cargo build 2>&1 | grep -E 'error|warning'

## Observability Impact

show_status gains a READY column; status.json gains a durable ready_for_notes boolean field
