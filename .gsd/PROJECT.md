# Project

## What This Is

A queue-first CLI for ingesting media and turning it into usable transcript artifacts without babysitting the process. It started as a Twitch downloader and is evolving into a personal media processing pipeline: acquire media, preserve durable artifacts, transcribe reliably, surface clear stage state, and prepare finished transcripts for downstream notes and memory work.

## Core Value

The one thing that must work even if everything else gets cut is this: queue media intake work and come back later to trustworthy transcript artifacts with clear state, failures, and resume behavior.

## Current State (Post-S01)

The project now has durable queue and artifact state contracts. Queue creation writes persistent `queues/<channel>.json` files that list queued VODs and distinguish them from already-known artifacts. Process commands persist per-artifact `status.json` files tracking download/transcribe completion, timestamps, and failure reasons. Media and transcript reuse is detected from durable files, allowing interrupted work to resume without redundant re-processing.

**Critical Gap:** The status CLI inspection command is not yet implemented. Operators cannot inspect queue contents or artifact lifecycle from the CLI without manually reading JSON files. This is a blocking issue for human UAT and milestone sign-off (S01 is code-complete but functionally incomplete).

**Technical State:** Rust CLI using `clap`, `reqwest`, `ffmpeg`, and `serde` for JSON persistence. Artifact directories have stable layout with metadata.json, source_url.txt, media files, and status.json. Both queue and process commands work and persist durable state correctly. Zero tests protect the artifact/queue schema from regression.

## Architecture / Key Patterns

- **Durable state location:** All queue and job state lives in JSON files under the output root (not memory or external services)
- **Artifact directory as job record:** A numeric directory (video_id) + status.json = complete job record
- **Stateless command design:** Queue and process are safe to re-run; they classify existing artifacts from durable state
- **Simple schema versioning:** Added schema_version to QueueFile and ProcessStatus for forward compatibility

## Capability Contract

See `.gsd/REQUIREMENTS.md` for the explicit capability contract, requirement status, and coverage mapping.

Current coverage:
- ✅ R001: Durable tracked jobs with explicit stage state (partial — state is simple booleans, not complex lifecycle types)
- ✅ R002: Twitch media ingestion into durable artifacts
- ✅ R005: Failure visibility in status.json (partial — not accessible from CLI yet)
- ❌ R007, R010, R020, R021: Deferred to later milestones

## Blocking Issues

1. **Status CLI command missing** — S01 promised "operators can inspect queue and artifact lifecycle from the CLI" but no status command exists. Operators must read JSON manually. This is required before human UAT can pass and before S03/S04 can be verified.

2. **No regression tests** — S01 promised test coverage for queue/status serialization and mixed fixture classification. Zero tests were written. Future changes to artifact logic are unguarded.

3. **Task summaries overstate implementation** — T01 claimed complex lifecycle types (JobLifecycleState, StageLifecycleState, etc.) and regression tests that don't exist in the actual code.

## Milestone Sequence

- [x] **M001/S01:** Durable artifact and queue state (CODE COMPLETE, FUNCTIONALLY INCOMPLETE)
  - ✅ Persistent queue and status JSON schemas
  - ✅ Queue and process commands write durable state
  - ✅ Media and transcript reuse detected from durable files
  - ❌ Status CLI inspection command (critical blocker)
  - ❌ Regression tests
  - 🚩 **Recommendation:** Before human UAT, add minimal status CLI command and basic unit tests

- [ ] **M001/S02:** Decoupled staged processing (blocked on S01 status CLI, can proceed on code)

- [ ] **M001/S03:** Trustworthy transcription and failure surfacing

- [ ] **M001/S04:** Ready-for-notes and manual cleanup workflow

- [ ] **M001/S05:** End-to-end operator flow proof

- [ ] **M002:** Notes and Ember memory workflow

- [ ] **M003:** Source expansion and workflow polish

## Next Steps for S02

S02 can begin using the durable queue/status JSON files as a foundation. Recommended:
1. Add explicit stage states (pending, downloading, transcribing, etc.) before S02 extends the schema
2. Add minimal test coverage in src/artifact.rs before modifying the artifact layer
3. Do NOT proceed with human UAT or milestone sign-off until status CLI command is implemented (blocking issue for S01 completion)
