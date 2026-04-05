---
id: S05
milestone: M001
status: ready
---

# S05: End-to-end operator flow proof — Context

## Goal

Prove the full M001 pipeline works as an integrated whole by running a real operator walkthrough against existing artifacts and a manufactured failure scenario, capturing all CLI output to a durable proof log.

## Why this Slice

S01–S04 each prove their own contracts in isolation. S05 is the only slice that proves they actually wire together: that the status model, staged processing, transcript contract, ready-for-notes state, and cleanup candidate discovery all compose correctly in one real CLI workflow. Without this slice, M001 is a collection of individually-verified parts, not a proven pipeline. The proof log also provides a debugging artifact if anything looks wrong during the walkthrough.

## Scope

### In Scope

- Operator walkthrough using pre-existing artifact state — no re-downloading of long VODs required; the proof operates on items already present in the artifact store from prior sessions
- Manufactured failure scenario scoped to a scratch/test artifact directory — deliberately trigger a transcription or stage failure, confirm it shows up with a clear reason, and confirm the item remains in a recoverable state without corrupting any real artifacts
- Capture all CLI command output to a proof log file during the walkthrough — the log is the durable evidence that the pipeline ran and produced correct output at each stage
- Verify each stage of the pipeline is represented in the walkthrough: queue/status inspection, staged processing state, transcript artifact presence, ready-for-notes state visible in status, and cleanup candidate listing
- The proof log lives in the project directory (not in `.gsd/`) — it is an operator output, not a planning artifact

### Out of Scope

- Re-queuing or re-downloading real long-form VODs to manufacture a fresh happy-path run — pre-existing artifacts are sufficient proof
- A re-runnable automated test script — the proof is a manual operator walkthrough captured to a log, not a CI-safe script
- Proving behavior against live Twitch API during S05 itself — intake was proven in prior slices; S05 exercises the artifact-side pipeline
- Any new feature development — S05 is strictly verification and proof, not implementation

## Constraints

- Manufactured failures must be scoped to a dedicated scratch artifact directory — no real artifact state should be mutated or corrupted during failure scenario testing
- The proof log must be produced by actually running the CLI commands, not by constructing expected output by hand
- Pre-existing artifacts are the primary happy-path inputs; the walkthrough must find and use them as-is without requiring a specific prior state beyond what S01–S04 would have produced

## Integration Points

### Consumes

- Existing artifact directories (output of S01–S04 in prior sessions) — used as happy-path proof inputs
- `status` CLI command (S01) — inspected to confirm per-item stage state is legible
- Staged processing commands (S02) — confirmed to show download/transcription progress independently
- `transcript.vtt` and `transcript.srt` artifacts (S03) — confirmed present in completed items
- `ready_for_notes` status field (S04) — confirmed set on completed items
- `cleanup` command (S04) — confirmed to list only ready-for-notes items as candidates
- A scratch artifact directory with a manufactured failure — used to verify failure visibility and recovery state

### Produces

- `proof.log` (or similar) — durable CLI output log capturing the full operator walkthrough, written to the project root or a designated output path; serves as both milestone completion evidence and a debugging reference

## Open Questions

- Where should `proof.log` live? — current thinking is project root or a `proofs/` subdirectory; not inside `.gsd/` since it's operator output not a planning artifact
- Should the manufactured failure scenario be documented as a step in the proof script (so it's reproducible), or is it sufficient to capture the failure output in the log without a recipe? — current thinking is document the failure setup steps briefly alongside the log so the scenario is reproducible if needed
