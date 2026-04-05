---
id: S01
milestone: M001
status: ready
---

# S01: Durable artifact and queue state — Context

## Goal

Replace the current flat `downloaded`/`transcribed` boolean model with a per-stage lifecycle state model, update the queue file format to distinguish queued-but-not-started items from known artifact state, and expose a `status` command that shows both queued items and existing artifacts without requiring the user to infer progress from raw files.

## Why this Slice

This is the foundation slice — every downstream slice (S02 staged processing, S03 transcript reliability, S04 ready-for-notes) depends on a richer artifact state model to track and resume staged work. Without explicit per-stage lifecycle state, S02 can't implement independent download/transcription progress or resume semantics. The inspect surface built here is also what the user reaches for first when something goes wrong. Order matters because all other slices build on the state contract established here.

## Scope

### In Scope

- New per-stage lifecycle state model in `status.json` replacing `downloaded: bool` / `transcribed: bool` — each stage (download, transcription) gets an explicit state field (e.g. `pending`, `running`, `completed`, `failed`)
- Migration on first read: infer new stage state from existing boolean fields in old `status.json` files; 26 existing artifacts should become first-class citizens automatically
- Updated queue file format that can distinguish items queued-but-not-started from items that already have an artifact folder
- `status` CLI command with two modes:
  - Default: one-line summary table per item (`video_id | channel | download:state transcription:state`)
  - `--verbose` / detail mode: compact per-item block showing all stage states, last error if any, and relevant file presence
- Status command shows queued items (from queue file) that do not yet have an artifact folder, clearly marked as "not started"
- Status command shows existing artifact folders with their migrated stage state

### Out of Scope

- `ready-for-notes` lifecycle state — that belongs to S04
- Stage running/pending states driven by active processing — those are S02 concerns; S01 only needs the state fields to exist and be readable
- Cleanup candidate discovery — S04
- Transcription reliability improvements — S03
- Any new Twitch API calls or queue-building logic changes — queue format update only, not queue-building behavior
- YouTube or other non-Twitch sources

## Constraints

- Filesystem-backed artifact model must be preserved — no database or service introduced
- Build on existing Rust CLI and module layout (`src/artifact.rs`, `src/cli.rs`, `src/main.rs`) rather than restructuring the codebase
- Migration must be non-destructive: reading an old `status.json` must produce correct new state without corrupting the file until an explicit write occurs (or migration is safe to write on first read)
- The status model produced here is the contract S02, S03, and S04 all build against — keep the stage state fields stable and extensible

## Integration Points

### Consumes

- `src/artifact.rs` — existing artifact directory helpers, `ProcessStatus` struct, queue file writer; this slice extends or replaces the state model here
- `src/cli.rs` — existing clap command surface; the new `status` subcommand is added here
- `artifacts/queues/<channel>.json` — existing queue file format; updated to distinguish queued-not-started from known artifact state
- `artifacts/<video_id>/status.json` — existing per-item status files with boolean fields; read and migrated

### Produces

- Updated `ProcessStatus` (or equivalent) in `src/artifact.rs` with per-stage lifecycle state fields replacing flat booleans
- Migration logic: read old boolean-format `status.json` → return new stage-state model (safe on first read, optionally writes migrated version back)
- Updated queue file format distinguishing queued-not-started items from artifact-known items
- `status` subcommand in `src/cli.rs` / `src/main.rs` with summary (default) and verbose (`--verbose`) modes
- Stable stage state contract for S02/S03/S04 to build against

## Open Questions

- Should migration write the upgraded `status.json` back to disk on first read, or only upgrade in memory until the next natural write? — current thinking is write-back on first read keeps the on-disk state consistent and avoids repeated migration logic, but either is safe given the boolean model maps cleanly
- What channel name to surface in the summary table when an artifact predates the queue file (no channel association)? — current thinking is show the video ID and leave channel as `unknown` or omit it; this is cosmetic and can be refined later
