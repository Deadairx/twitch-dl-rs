---
id: S06
milestone: M002-z48awz
status: ready
---

# S06: Retry And Operational Hardening — Context

## Goal

Add `--force-suspect` to `transcribe-all` so suspect transcriptions can be retried without re-downloading, and add blocking file locking to `status.json` writes so parallel `download-all` and `transcribe-all` invocations don't corrupt artifact state.

## Why this Slice

S06 closes two gaps that make the pipeline unreliable in real operator usage. Force-retry is the missing recovery path when `hear` produces a suspect result — without it the operator is stuck. File locking is necessary because the operator runs `download-all` and `transcribe-all` concurrently, and both commands write `status.json`; without a lock, one process silently overwrites the other's changes. S07 (Additional Source Support) depends on S06 being complete first.

## Scope

### In Scope

- `--force-suspect` flag on `transcribe-all` — when passed, suspect items are included in the transcription run instead of skipped; normal items (already completed) are still skipped
- `--force-suspect` composes with `--video-id` from S04: `transcribe-all --force-suspect --video-id <id>` retries a specific suspect item
- After a force-retry, `status.json` is updated with the new outcome (`completed`, `suspect`, or `failed`) — the prior suspect state is overwritten
- Blocking file lock on all `status.json` writes — when a second process tries to write while a lock is held, it waits (does not error) until the lock is released; no UX change to the operator
- `cargo test` passes; `cargo build` succeeds

### Out of Scope

- Separate `retry` command — force-retry is a flag on `transcribe-all`, not a new top-level command
- Force-retrying `failed` items (as opposed to `suspect`) — `--force-suspect` targets suspect only; failed items require re-download which is out of scope here
- Auto-retry on failure — retry is always an explicit operator action
- Read locking on `status.json` — only writes need the lock; reads remain unlockable

## Constraints

- `--force-suspect` must reuse the existing `transcribe_artifact` helper — no parallel transcription path
- The file lock must be a blocking lock (not try-lock): the waiting process must not emit errors or warnings while waiting; it simply proceeds once the lock is available
- `hear` invocation is unchanged
- The lock mechanism must work on macOS (the operator's platform) — use OS-level file locking (`flock` or equivalent via a Rust crate) rather than a `.lock` file advisory scheme

## Integration Points

### Consumes

- `src/cli.rs` — `transcribe-all` subcommand (gets `--force-suspect` flag)
- `src/main.rs` — `transcribe_all` function and `transcribe_artifact` helper; `write_status` call sites
- `src/artifact.rs` — `write_status` function (gets locking added)
- `src/transcribe.rs` — `TranscriptionOutcome` variants; existing transcription logic unchanged

### Produces

- `src/cli.rs` — `--force-suspect` flag on `transcribe-all`
- `src/main.rs` — `transcribe_all` updated to include suspect items when `force_suspect` is true
- `src/artifact.rs` — `write_status` wrapped with a blocking file lock; lock acquired before write, released after flush

## Open Questions

- **Rust crate for file locking**: `fs2` and `file-lock` are the common options on crates.io. Executor should pick whichever is more actively maintained at implementation time and document the choice in DECISIONS.md.
