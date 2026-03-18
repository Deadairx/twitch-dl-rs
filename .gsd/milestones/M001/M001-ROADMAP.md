# M001: Reliable media-to-transcript pipeline

**Vision:** Turn the current Twitch-first downloader into a queue-first, artifact-first media pipeline that can ingest media, keep making progress without babysitting, produce trustworthy transcript artifacts, and leave completed items in a clear state for later notes work.

## Success Criteria

- The user can queue Twitch media and later inspect durable per-item stage state without inferring progress from loose files.
- Download work keeps moving even when transcription is pending, slow, or failed.
- Completed items produce transcript artifacts trustworthy enough for downstream use.
- Failed items remain visible with clear reasons and resumeable state.
- Completed transcripts surface a real `ready for notes` state.
- Cleanup is explicit and safe: the user can review deletion candidates, but originals are never auto-deleted.

## Key Risks / Unknowns

- Transcription reliability may still be too lossy for trustworthy downstream use — this could invalidate the whole promise of usable transcripts.
- Stage orchestration may stay confusing if artifact lifecycle state is too coarse or inconsistent — this could make resume behavior untrustworthy.
- Cleanup candidate logic could accidentally overreach without strong lifecycle and lock semantics — this would be a trust-breaking failure.

## Proof Strategy

- Transcription reliability may still be too lossy for trustworthy downstream use → retire in S03 by proving the chosen transcript path and artifact outputs are materially more trustworthy than the current fragile flow.
- Stage orchestration may stay confusing if artifact lifecycle state is too coarse or inconsistent → retire in S02 by proving download and transcription progress can proceed independently and resume from durable state.
- Cleanup candidate logic could accidentally overreach without strong lifecycle and lock semantics → retire in S04 by proving cleanup only surfaces explicit safe candidates from completed artifact state.

## Verification Classes

- Contract verification: shell commands, artifact/state file inspection, and targeted tests where useful
- Integration verification: real Twitch queue + local processing run using the CLI against live source intake and local transcript generation
- Operational verification: interrupted or partial work resumes without losing stage visibility or redoing completed stages unnecessarily
- UAT / human verification: operator judgment that status surfaces, ready-for-notes state, and cleanup candidate review feel legible and trustworthy

## Milestone Definition of Done

This milestone is complete only when all are true:

- All slice deliverables are complete
- Shared queue, artifact, and status components are actually wired together
- The real CLI entrypoint exists for queueing, staged progress, inspection, and cleanup candidate review
- Success criteria are re-checked against live behavior, not just artifact existence
- Final integrated acceptance scenarios pass

## Requirement Coverage

- Covers: R001, R002, R003, R004, R005, R006, R009, R012
- Partially covers: none
- Leaves for later: R007, R008, R010, R011, R020, R021
- Orphan risks: none

## Slices

- [ ] **S01: Durable artifact and queue state** `risk:high` `depends:[]`
  > After this: You can queue Twitch media into durable per-item artifact folders with explicit status, and inspect what exists without guessing from raw files.

- [ ] **S02: Decoupled staged processing** `risk:high` `depends:[S01]`
  > After this: Downloads can continue making progress while transcription work remains pending, running, or failed, and interrupted work can be resumed.

- [ ] **S03: Trustworthy transcription and failure surfacing** `risk:high` `depends:[S01,S02]`
  > After this: Finished items produce transcript artifacts you can trust more than the current fast path, and failed transcriptions show clear reasons and remain recoverable.

- [ ] **S04: Ready-for-notes and manual cleanup workflow** `risk:medium` `depends:[S01,S02,S03]`
  > After this: Completed transcripts enter a clear ready-for-notes state, and a cleanup command shows only safe deletion candidates without auto-deleting anything.

- [ ] **S05: End-to-end operator flow proof** `risk:medium` `depends:[S01,S02,S03,S04]`
  > After this: In one real CLI workflow, you can queue media, let staged processing run without babysitting, inspect failures, see ready-for-notes items, and review cleanup candidates.

## Boundary Map

### S01 → S02

Produces:
- Filesystem-backed per-item artifact record with stable identifiers and richer stage/status fields than the current `downloaded` / `transcribed` booleans
- Queue file format that distinguishes queued work from already-known artifact state
- CLI-visible status inspection surface for existing artifacts and queued items

Consumes:
- nothing (first slice)

### S02 → S03

Produces:
- Separate stage commands or scheduler behavior for download progress vs transcription progress
- Durable stage transitions for pending, running, failed, completed, and resumable work
- Resume semantics that prevent completed download work from being redone unnecessarily

Consumes from S01:
- artifact state model
- queue file format
- status inspection surface

### S03 → S04

Produces:
- Trusted transcript artifact contract for completed items
- Structured failure reason capture for transcription failures
- Transcript completion signal strong enough to drive downstream lifecycle state

Consumes from S01:
- artifact state model

Consumes from S02:
- staged execution and resume behavior

### S04 → S05

Produces:
- `ready for notes` lifecycle state and artifact marker
- Cleanup candidate discovery contract that excludes unsafe or incomplete items
- Manual cleanup review command surface

Consumes from S01:
- artifact metadata and status model

Consumes from S02:
- staged lifecycle behavior

Consumes from S03:
- transcript completion and failure semantics
