# M001/S01 — Research

**Date:** 2026-03-17

## Summary

S01 owns the first real durable job model for the pipeline: it needs to turn the current "artifact folder exists" heuristic plus `downloaded` / `transcribed` booleans into an explicit per-item state contract that survives reruns and can be inspected from the CLI. The codebase already has the right substrate for this work: filesystem-backed artifact folders, queue JSON writing, per-item `status.json`, and a small Rust CLI surface in `src/cli.rs` and `src/main.rs`. What is missing is not storage infrastructure, but a richer schema and commands that treat it as the product surface.

The current implementation partially covers S01 requirements but in a fragile way. `queue` skips items solely because a numeric artifact directory already exists, even if that directory is partial or failed. `process` rebuilds a fresh queue from Twitch instead of consuming durable queue state, and it only records booleans plus one `last_error` string in `status.json`. There is no `status` or `inspect` command for the operator to view queued items and artifact lifecycle. Recommendation: keep the filesystem-first approach, introduce a richer artifact/job state schema in `src/artifact.rs`, then wire new queue/status-oriented CLI commands around that shared model before attempting any stage decoupling in S02.

## Recommendation

Use `src/artifact.rs` as the authoritative state layer and expand it from helper functions into the durable contract for both queue files and per-item artifact state. Keep JSON-on-disk and stable per-video artifact directories; do not introduce a database or hidden scheduler for this slice. Add explicit lifecycle/state enums or string fields for each stage and for the overall job, plus timestamps and failure metadata. The planner should prefer additive schema evolution over replacing files outright, because S02 and S03 will build on the same state files.

For S01 specifically, build around three user-visible behaviors:
1. `queue` creates durable queue records without treating "directory exists" as equivalent to "done".
2. Per-item artifact folders always contain a status record legible enough to tell whether the item is queued, downloaded, transcribed, failed, or partial.
3. A new CLI inspection surface reports both queue contents and artifact/job state, so the operator can tell what exists without reading raw JSON manually.

This slice should avoid solving S02’s execution orchestration. It only needs to make queue and artifact state durable and inspectable enough that later staged processing can trust them.

## Implementation Landscape

### Key Files

- `src/artifact.rs` — Current home for all durable JSON/file helpers. Defines `ArtifactMetadata`, `QueueFile`, `ProcessStatus`, `existing_artifact_ids`, `write_queue_file`, `read_status`, `write_status`, and `find_media_file`. This is the natural place to introduce the richer durable schema S01 needs.
- `src/main.rs` — Current command behavior. `build_queue()` filters fetched VODs using `existing_artifact_ids()`. `process_channel()` duplicates queue-building logic instead of consuming queue state. `process_vod()` reads and writes `ProcessStatus`, but only as booleans. This file will need rewiring once the artifact/job model is richer and when a status inspection command is added.
- `src/cli.rs` — Existing `download`, `queue`, and `process` clap subcommands. There is no command for listing artifact state or inspecting queue contents. S01 likely needs at least one new read-only command here (`status`, `list`, or equivalent) and possibly a way to inspect an individual video/job.
- `README.md` — Documents the current artifact layout and queue/process behavior. It still describes queue skipping based on existing artifact directories and `status.json` as a rerun helper, not as the main product surface. Must be updated once the state contract and CLI surface change.
- `src/twitch.rs` — Supplies `VodEntry` and queue inputs from Twitch. Relevant because `QueueFile` currently stores raw `Vec<VodEntry>`; planner should check whether the durable queue format should continue embedding `VodEntry` directly or wrap it in a queue-item/job envelope.
- `Cargo.toml` — Confirms a simple Rust CLI stack with `clap`, `serde`, and `serde_json`; no extra state-management library is needed for S01.

### Build Order

1. **Define the durable state contract first in `src/artifact.rs`.**
   - This is the load-bearing seam for S01 and the dependency seam for S02/S03.
   - Replace or extend `ProcessStatus` so it can represent explicit stage state, not just booleans.
   - Rework `QueueFile` so queued work is distinguishable from already-known artifact state rather than just a filtered list plus `skipped_existing_ids`.
   - Add read helpers for queue files and any artifact summary/listing functions needed by the CLI.

2. **Refactor queue creation in `src/main.rs` to use state-aware artifact detection.**
   - Today `build_queue()` uses `existing_artifact_ids()` and treats any numeric directory as existing work.
   - This is too coarse for R001/R005 because partial or failed artifacts become invisible.
   - Queue building should classify known items from durable status/metadata, not from directory presence alone.

3. **Add CLI-visible inspection in `src/cli.rs` + `src/main.rs`.**
   - S01’s roadmap output explicitly requires a “CLI-visible status inspection surface.”
   - Add command(s) to inspect queue state and artifact/job state without having to infer from loose files.
   - Keep this read-only and state-reporting; do not mix it with S02 stage runners yet.

4. **Update writer paths in `download_vod()` / `process_vod()` to always maintain valid status.**
   - Even before stage decoupling, artifact creation and media reuse should leave behind an explicit status file that reflects reality.
   - Ensure successful download paths, transcript reuse paths, and error paths all produce coherent persisted state.

5. **Document the contract in `README.md`.**
   - The artifact layout and CLI behavior need to match the new durable model so later slices can rely on documentation during verification.

### Verification Approach

Use contract-style verification first; this slice does not need live Twitch integration to prove the state model.

- `cargo test` — There are currently no visible tests; S01 should likely introduce unit tests around `src/artifact.rs` JSON read/write behavior and any queue classification logic.
- `cargo run -- queue <channel> ... --output-root <temp-dir>` — Verify the queue file shape is durable and distinguishes queued items from already-known artifact state.
- `cargo run -- <new status command> ... --output-root <temp-dir>` — Verify the CLI can report queue contents and per-item state without opening JSON by hand.
- Inspect generated files under `artifacts/` (or a temp output root) to confirm that a newly created artifact folder contains coherent metadata/status files even for partial work.
- For regression coverage, create fixture directories with combinations like: directory only, metadata only, media only, transcript only, failed status, and complete status. Confirm queue/status commands classify them correctly.

## Constraints

- The milestone context explicitly requires preserving the filesystem-backed artifact model; S01 should not introduce a database or daemon.
- The current codebase is small and centralized. Natural work seams are module-level (`artifact.rs`, `main.rs`, `cli.rs`), not subsystem-level.
- `process_channel()` currently regenerates queue state directly from Twitch fetches. If S01 leaves queue state as write-only JSON with no read path, S02 will have to rework it again.
- The test skill guidance applies here: match existing Rust conventions, use `cargo test`, and add tests around the durable contract before wiring more commands.

## Common Pitfalls

- **Treating directory existence as job existence** — current `existing_artifact_ids()` logic only checks for numeric directories, which hides partial/failed work and makes queue semantics unreliable. Queue classification should inspect durable state files, not just folder names.
- **Encoding stage truth in duplicated booleans** — `downloaded` / `transcribed` can drift from file reality and do not express pending/running/failed/resumable states. Prefer a richer per-stage status model now so S02 does not need a schema break.
- **Making queue files a one-shot export instead of a durable work record** — S01 needs queue state the CLI can later inspect. If queue JSON remains just a generated snapshot of Twitch responses, the operator still has to infer status indirectly.
- **Leaving status as process-only implementation detail** — roadmap explicitly says the operator must inspect durable per-item stage state. The CLI must surface it directly in this slice.

## Open Risks

- The planner must choose how much status richness to add now versus later. Under-building the schema will force churn in S02; over-building it may create fields whose semantics are not yet exercised.
- `QueueFile` currently embeds raw `VodEntry`. If `VodEntry` changes in later source work, the durable queue format could become more coupled than desired. A stable queue-item wrapper may be safer.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Rust testing / verification | installed `test` skill | available |

## Sources

- Existing code shows the durable-state seam is already centralized in `src/artifact.rs`, but currently only stores `downloaded`, `transcribed`, and `last_error` in `ProcessStatus`. (source: local codebase: `src/artifact.rs`)
- Existing CLI behavior shows `queue` and `process` still classify work primarily from Twitch fetch + artifact directory presence, not from a durable job model. (source: local codebase: `src/main.rs`)
- Current user-facing docs still describe `status.json` as a rerun helper rather than an inspection surface, confirming the missing S01 CLI/product layer. (source: local codebase: `README.md`)
