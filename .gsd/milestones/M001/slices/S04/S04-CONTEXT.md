---
id: S04
milestone: M001
status: ready
---

# S04: Ready-for-notes and manual cleanup workflow — Context

## Goal

Surface a `ready_for_notes` lifecycle state automatically when transcript.vtt is produced, and provide a two-step cleanup command that lists audio and intermediate deletion candidates for operator review before any deletion occurs.

## Why this Slice

S03 produces a trusted transcript artifact and signals completion — S04 converts that signal into a durable lifecycle state and gives the operator a safe, deliberate workflow for reclaiming disk space. Without this slice, completed items have no first-class `ready` state and cleanup requires manual filesystem inspection. S05 (end-to-end proof) depends on this slice to demonstrate the full lifecycle from intake to ready-for-notes.

## Scope

### In Scope

- `ready_for_notes` field added to the artifact status model, set automatically when S03's transcript completion signal fires (i.e. `transcript.vtt` exists and transcription stage is complete)
- A `cleanup` CLI command that lists only `ready_for_notes` items as candidates, showing per-item artifact sizes for the files that would be deleted
- Cleanup candidates include `audio.m4a` and `transcript.srt` — the source audio and the intermediate SRT; `transcript.vtt`, `metadata.json`, `status.json`, and `source_url.txt` are never touched
- A `--delete` flag (or `--delete <video_id>`) on the cleanup command that executes the actual deletion after the operator has reviewed the list
- Items that are failed, partial, in-progress, or only partially transcribed never appear as cleanup candidates regardless of what files are present on disk

### Out of Scope

- Any automatic deletion — cleanup is always a two-step operator action
- Notes generation, LLM summarization, or any downstream notes workflow beyond surfacing the `ready_for_notes` state
- Cleanup of artifact directories that have not reached `ready_for_notes` state, even if `transcript.vtt` happens to exist without a matching status
- Interactive per-item confirm prompts or dry-run/apply flag variants — the UX is list-only by default, `--delete` to act
- Batch delete-all without specifying a target

## Constraints

- Cleanup eligibility is gated on `ready_for_notes` status flag, not on file existence alone — this keeps the safety contract tied to explicit lifecycle state rather than filesystem inference
- Originals (`audio.m4a`) are never auto-deleted; deletion only happens when the operator explicitly passes `--delete`
- `ready_for_notes` is set by the pipeline automatically — no `mark-ready` command is needed or in scope
- Must respect D004: cleanup is explicit operator action via candidate review, not automatic deletion

## Integration Points

### Consumes

- `src/artifact.rs` — existing `ProcessStatus` struct; S04 adds `ready_for_notes: bool` field and the transition logic that sets it
- S03's transcript completion signal — specifically: transcription stage marked complete and `transcript.vtt` present in the artifact directory
- Artifact directory layout established in S01 — `status.json`, `audio.m4a`, `transcript.srt`, `transcript.vtt`

### Produces

- `ready_for_notes: bool` field on `ProcessStatus` (written to `status.json`)
- `cleanup` CLI subcommand: default behavior lists candidates with file sizes; `--delete` flag executes removal of `audio.m4a` and `transcript.srt` for the listed (or specified) items
- Cleanup candidate discovery contract consumed by S05 for end-to-end proof

## Open Questions

- Should `cleanup --delete` require a specific video ID, or delete all listed candidates if none is specified? — current thinking is that requiring an explicit ID (or an `--all` flag) is safer and matches the "deliberate operator action" contract; default to requiring an argument rather than deleting everything silently
- Should the `status` inspection command (from S01/S02) surface `ready_for_notes` items distinctly in its output? — current thinking is yes, a simple marker in the status display is enough and belongs in this slice since it's tied to the new lifecycle state
