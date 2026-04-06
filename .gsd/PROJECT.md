# Project

## What This Is

A queue-first CLI for ingesting media and turning it into usable transcript artifacts without babysitting the process. It started as a Twitch downloader and is now a complete personal media processing pipeline: acquire media, preserve durable artifacts, transcribe reliably, surface clear stage state, and prepare finished transcripts for downstream notes and memory work.

## Core Value

The one thing that must work even if everything else gets cut: queue media intake work and come back later to trustworthy transcript artifacts with clear state, failures, and resume behavior.

## Current State (Post-M001)

M001 is **complete**. The project is a fully functional queue-first, artifact-first media-to-transcript pipeline.

**What works today:**
- `queue` — Build a backlog queue for a Twitch channel (`queues/<channel>.json`)
- `process` — Queue, download, and transcribe in one shot
- `status` — Show per-artifact stage state (DOWNLOADED, OUTCOME, READY, REASON columns)
- `download-all` — Batch download all pending queued VODs, resumable
- `transcribe-all` — Batch transcribe all downloaded-but-untranscribed artifacts, resumable
- `cleanup` — List (or delete) ready-for-notes artifact files, with safety checks

**Pipeline stages for each artifact:**
1. Queued → Downloaded → Transcribed (completed/suspect/failed) → Ready-for-notes → Cleaned up

**Transcript quality:** `hear`-backed transcription with word-count threshold (50 words/hour) and repetition detection (trigrams >10× in 200-word window). Both SRT and VTT required for a `completed` outcome. Failed and suspect transcriptions show clear reasons in status.json and in the `status` CLI output.

**Durability:** All state lives in JSON files under the output root. Per-artifact `status.json` persists download/transcribe completion, timestamps, failure reasons, transcription outcome, and ready-for-notes flag. All batch commands check status.json before acting — safe to interrupt and resume.

## Architecture / Key Patterns

- **Durable state:** All queue and job state in JSON files under output root (not memory, not external services)
- **Artifact directory as job record:** `<video_id>/` directory + `status.json` = complete job record
- **Stateless command design:** All commands are safe to re-run; they classify existing artifacts from durable state
- **Composable helpers:** `download_vod_to_artifact` and `transcribe_artifact` are standalone async helpers used by both single-item and batch commands
- **Backward-compatible schema evolution:** All new `ProcessStatus` fields use `#[serde(default)]`; old status.json files always deserialize cleanly
- **Three-outcome transcription:** `TranscriptionOutcome::Completed | Suspect | Failed` — only Completed sets `ready_for_notes=true`
- **Safe cleanup:** Two-step operator review before any deletion; protected files (transcript.vtt, status.json, metadata.json, source_url.txt) are never deleted

## Capability Contract

See `.gsd/REQUIREMENTS.md` for the explicit capability contract.

### M001-Validated Requirements
- ✅ R001: Queue-first media job pipeline — durable per-item tracked jobs with explicit stage state
- ✅ R002: Twitch media intake — ingests Twitch VODs into durable artifact directories
- ✅ R003: Decoupled download and transcription — independent batch commands with resume behavior
- ✅ R004: Trustworthy transcript artifacts — hear-backed with quality heuristics; SRT+VTT required
- ✅ R005: Durable per-item artifact state and failure visibility — reason surfaced in status.json and CLI
- ✅ R006: Ready-for-notes downstream stage — auto-set on completion; separate from cleanup/notes
- ✅ R009: Manual cleanup candidate workflow with safety checks — explicit operator action only
- ✅ R012: Resume long-running work without babysitting — safe re-run via durable stage state

### Future Requirements (M002/M003)
- R007: Manual-first notes generation with prompt/style choice → M002/S01
- R008: Ember memory persistence for selected note outputs → M002/S02
- R010: Additional media sources beyond Twitch → M003/S01
- R011: Support/contradict analysis against memory context → M002/S03

## Known Gaps and Follow-ups

1. **Force-retry for suspect transcriptions** — `transcribe-all` skips suspect items; no `--force-retry` flag exists. Operator recovery requires manually editing status.json.
2. **Long VOD validation needed** — hear's behavior on multi-hour audio files (>2h) is unverified in production. Chunking may be required.
3. **No concurrent access locking** — simultaneous writes to status.json from multiple processes could corrupt state.
4. **S04 summary frontmatter incomplete** — provides/key_files/key_decisions sections left empty despite full implementation delivery (documentation gap only).

## Milestone Sequence

- [x] **M001:** Reliable media-to-transcript pipeline — **COMPLETE**
  - ✅ Durable queue and artifact state (S01)
  - ✅ Decoupled staged processing: status, download-all, transcribe-all (S02)
  - ✅ Trustworthy transcription with quality heuristics and failure surfacing (S03)
  - ✅ Ready-for-notes state and safe cleanup workflow (S04)
  - ✅ End-to-end operator flow proof with durable proof log (S05)

- [ ] **M002:** Notes and Ember memory workflow
  - Manual-first note generation with prompt/style choice (R007)
  - Ember memory persistence for selected outputs (R008)
  - Support/contradict analysis against memory context (R011)

- [ ] **M003:** Source expansion and workflow polish
  - Additional media sources beyond Twitch (R010)
  - Suspect transcription force-retry UX
  - Concurrent access safety for status.json
