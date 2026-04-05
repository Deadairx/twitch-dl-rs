---
id: S02
milestone: M001
status: ready
---

# S02: Decoupled staged processing — Context

## Goal

Split the current monolithic `process` command into two explicit CLI commands — `download-all` and `transcribe-all` — that operate on queue and artifact state independently, process items sequentially oldest-first, continue past individual failures, and treat partial downloads as corrupt artifacts to be cleaned up and retried.

## Why this Slice

The current `process` command couples download and transcription into a single synchronous loop — a transcription failure or slow run blocks all subsequent downloads. S02 breaks that coupling so downloads can make progress overnight regardless of transcription state. It also establishes the resume semantics (skip completed stages, clean up partial artifacts) that S03 and S04 depend on for trustworthy artifact state. Must follow S01 because it builds on the per-stage lifecycle state model S01 establishes.

## Scope

### In Scope

- `download-all` CLI command: iterates the queue oldest-first, skips items whose download stage is already `completed` in artifact state, deletes partial download files and restarts if download state is not `completed`, continues to next item on failure (marking the failed item with error in status), processes items sequentially one at a time
- `transcribe-all` CLI command: iterates artifact folders oldest-first, skips items whose transcription stage is already `completed`, continues to next item on failure, processes sequentially; invokes the transcription backend (currently `mlx-whisper`, to be replaced by `hear` in S03)
- Oldest-first ordering for both commands — oldest VODs are prioritized because they are most at risk of expiry on the Twitch origin server; ordering derived from video ID numeric order (Twitch video IDs are monotonically increasing) or artifact directory creation time as fallback
- Partial download detection and cleanup: if a media file exists but `status.download` is not `completed`, delete the file and restart the download from scratch — artifact integrity over time savings
- Continue-on-error as default behavior for both commands — no flag needed, this is the only supported mode; the entire point is unattended overnight runs that don't need intervention
- Durable stage transitions written to `status.json` at each stage boundary: `pending → running → completed / failed`
- `running` state written at stage start so an interrupted run leaves a visible signal; on next invocation, `running` is treated as incomplete and retried (simple re-run semantics, no heartbeat or TTL needed for M001)
- Optional `--channel` filter on both commands to scope work to a single channel's queue; if omitted, process all known queue files

### Out of Scope

- Background scheduler or daemon mode — deferred to a later milestone
- Parallel / concurrent downloads — deferred to a later milestone
- Transcription reliability improvements and the `hear` backend switch — that is S03's scope; S02 invokes whatever transcription backend is currently wired
- `ready-for-notes` state transitions — S04
- Concurrency limits or `--jobs` flag
- Automatic retry with backoff on transient failures — fail fast, mark error, move on; retry is manual re-invocation

## Constraints

- Sequential execution only — no async work queues, no thread pools, no tokio concurrency beyond what already exists for HTTP calls
- Artifact integrity over time savings: partial files must be deleted before restart, never appended to or treated as resumable via ffmpeg HLS continuation
- Continue-on-error is not optional — both commands always continue past individual item failures
- Oldest-first is the required ordering for both commands, not a configurable option in this slice
- `running` state must not be left permanently stuck: re-invoking either command treats any item in `running` state as incomplete and retries it
- Build on the stage state model from S01 — do not introduce a parallel status representation
- Do not change the transcription backend in this slice — backend swap is S03's responsibility

## Integration Points

### Consumes

- `src/artifact.rs` — per-stage lifecycle state model from S01 (`StageState` enum or equivalent); read and written at each stage transition
- `artifacts/<video_id>/status.json` — authoritative source for whether a stage is completed, running, or failed; used to determine skip/retry/clean behavior
- `artifacts/queues/<channel>.json` — queue file from S01 providing the ordered list of items to process
- `src/cli.rs` — existing clap command surface; `download-all` and `transcribe-all` subcommands added here
- `src/transcribe.rs` — existing transcription invocation (currently `mlx-whisper`); called unchanged by `transcribe-all` until S03 replaces it
- `src/ffmpeg.rs` — existing ffmpeg download invocation; called by `download-all`

### Produces

- `download-all` subcommand: processes queue oldest-first, writes stage transitions to `status.json`, cleans partial files, continues on failure
- `transcribe-all` subcommand: processes artifact folders oldest-first, writes stage transitions to `status.json`, continues on failure
- Durable `running` → `completed` / `failed` stage transitions visible in artifact state after each command run
- Resume semantics contract for S03/S04: completed stages are never re-executed; `running` or `failed` stages are retried on next invocation
- Clean handoff surface for S03: `transcribe-all` calls into `src/transcribe.rs` via a stable interface so S03 can swap the backend without touching command dispatch logic

## Open Questions

- How is "oldest" determined when a VOD predates the queue file and has no Twitch upload timestamp in artifact state? — current thinking is fall back to video ID numeric order (monotonically increasing on Twitch), which is a reliable proxy for age; artifact directory creation time is secondary fallback
- Should `download-all` and `transcribe-all` accept a `--channel` filter to scope work to a single channel's queue? — current thinking is yes, since queue files are per-channel; omitting the flag processes all known queues
