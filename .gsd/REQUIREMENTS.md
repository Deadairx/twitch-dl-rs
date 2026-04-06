# Requirements

This file is the explicit capability and coverage contract for the project.

## Active

### R006 — Finished transcripts must enter a clear ready-for-notes state that separates core pipeline completion from optional downstream note work.
- Class: continuity
- Status: active
- Description: Finished transcripts must enter a clear ready-for-notes state that separates core pipeline completion from optional downstream note work.
- Why it matters: The user wants transcript completion and note generation to be related but distinct states.
- Source: user
- Primary owning slice: M001/S04
- Supporting slices: M001/S05
- Validation: mapped
- Notes: This prepares M002 without forcing notes into M001.

### R007 — The system must support manual-first note generation where the user can choose the style or question lens for a transcript.
- Class: differentiator
- Status: active
- Description: The system must support manual-first note generation where the user can choose the style or question lens for a transcript.
- Why it matters: Different content calls for different downstream questions, not one generic summary every time.
- Source: user
- Primary owning slice: M002/S01
- Supporting slices: M002/S02, M002/S03
- Validation: mapped
- Notes: Recap/summary is the safe default lens, but not the only one.

### R008 — Selected downstream outputs must be persistable into Ember as memories.
- Class: integration
- Status: active
- Description: Selected downstream outputs must be persistable into Ember as memories.
- Why it matters: Part of the workflow value is turning processed media into durable memory context.
- Source: user
- Primary owning slice: M002/S02
- Supporting slices: M002/S03
- Validation: mapped
- Notes: Memory-affecting actions should stay more explicit than basic recap generation.

### R009 — The system must provide an explicit cleanup command that lists safe deletion candidates instead of automatically deleting source media.
- Class: operability
- Status: active
- Description: The system must provide an explicit cleanup command that lists safe deletion candidates instead of automatically deleting source media.
- Why it matters: Cleanup needs strong safeguards and operator control to avoid losing originals prematurely.
- Source: user
- Primary owning slice: M001/S04
- Supporting slices: M001/S05
- Validation: mapped
- Notes: Safe candidate detection still needs locking and lifecycle awareness.

### R010 — The architecture must support additional sources such as YouTube after Twitch-first stabilization.
- Class: core-capability
- Status: active
- Description: The architecture must support additional sources such as YouTube after Twitch-first stabilization.
- Why it matters: The tool is evolving into a broader media ingestion workflow, not a Twitch-only utility.
- Source: user
- Primary owning slice: M003/S01
- Supporting slices: none
- Validation: mapped
- Notes: Not part of M001 delivery.

### R011 — The notes layer must support prompts that look for content supporting or contradicting current views or existing memory context.
- Class: differentiator
- Status: active
- Description: The notes layer must support prompts that look for content supporting or contradicting current views or existing memory context.
- Why it matters: The downstream value is not just summarization but reflective analysis.
- Source: user
- Primary owning slice: M002/S03
- Supporting slices: M002/S02
- Validation: mapped
- Notes: This is a later memory-shaping capability, not a first milestone requirement.

### R012 — Interrupted or partial work must be resumable without redoing completed stages or losing operator understanding.
- Class: continuity
- Status: active
- Description: Interrupted or partial work must be resumable without redoing completed stages or losing operator understanding.
- Why it matters: The user wants to queue work overnight and continue later without confusion.
- Source: user
- Primary owning slice: M001/S02
- Supporting slices: M001/S03, M001/S05
- Validation: mapped
- Notes: Resume behavior depends on durable stage state and clear status semantics.

## Validated

### R001 — The CLI must treat each media item as a durable tracked job with explicit stage state instead of a one-off command side effect.
- Class: primary-user-loop
- Status: validated
- Description: The CLI must treat each media item as a durable tracked job with explicit stage state instead of a one-off command side effect.
- Why it matters: The workflow depends on coming back later and understanding what happened without babysitting the run.
- Source: user
- Primary owning slice: M001/S01
- Supporting slices: M001/S02, M001/S05
- Validation: S05 proof phase 1: 25 real artifacts each with per-item stage state (DOWNLOADED, OUTCOME, READY) in status table. Queue files and status.json persist durable job state across restarts.
- Notes: Artifact-first job state is the center of the product, not just an internal detail.

### R002 — The system must ingest Twitch media into durable local artifacts as the first supported source.
- Class: core-capability
- Status: validated
- Description: The system must ingest Twitch media into durable local artifacts as the first supported source.
- Why it matters: Twitch is the current real workflow and the foundation for broader source support later.
- Source: user
- Primary owning slice: M001/S01
- Supporting slices: M001/S05
- Validation: queue and process commands ingest Twitch VODs via twitch.rs; artifact directories created with source_url.txt, metadata.json, audio.m4a, and status.json. Confirmed functional in S05 proof run against real artifacts.
- Notes: Twitch-first now; broader source coverage comes later.

### R003 — Downloading must continue making progress even when transcription is pending, running slowly, or failing.
- Class: operability
- Status: validated
- Description: Downloading must continue making progress even when transcription is pending, running slowly, or failing.
- Why it matters: The user does not want transcription to block ingestion of other media.
- Source: user
- Primary owning slice: M001/S02
- Supporting slices: M001/S05
- Validation: download-all and transcribe-all operate as independent CLI commands. S05 proof shows artifacts in mixed staged states. Each command checks status.json before acting, enabling independent recovery.
- Notes: This is a product behavior requirement, not just a threading choice.

### R004 — Completed transcript artifacts must be trustworthy enough for downstream note generation and real use.
- Class: quality-attribute
- Status: validated
- Description: Completed transcript artifacts must be trustworthy enough for downstream note generation and real use.
- Why it matters: Notes, memory work, and operator trust all depend on transcript quality.
- Source: user
- Primary owning slice: M001/S03
- Supporting slices: M001/S05
- Validation: hear-backed transcription with word-count threshold (50 words/hour) and repetition detection (trigram >10x in 200-word window). Both SRT and VTT required for completed outcome. 7/7 transcribe unit tests confirm heuristic logic.
- Notes: Reliability matters more than raw speed for default behavior.

### R005 — Each item must surface clear stage status, failure reasons, and recoverable state through durable artifacts.
- Class: failure-visibility
- Status: validated
- Description: Each item must surface clear stage status, failure reasons, and recoverable state through durable artifacts.
- Why it matters: The user wants to return to the queue and immediately see what failed and why.
- Source: user
- Primary owning slice: M001/S01
- Supporting slices: M001/S02, M001/S03, M001/S05
- Validation: ProcessStatus persists transcription_outcome, transcription_reason, and last_error. status command surfaces these in OUTCOME and REASON columns. proof.log phase 1 shows prior failure with reason; phase 3 shows manufactured failure captured and item remaining recoverable.
- Notes: Status needs to survive interruptions and reruns.

## Deferred

### R020 — The system may eventually auto-generate a safe default recap after transcript completion.
- Class: differentiator
- Status: deferred
- Description: The system may eventually auto-generate a safe default recap after transcript completion.
- Why it matters: This could reduce friction for the most common downstream use.
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: Deferred because the auto/manual boundary is intentionally still flexible.

### R021 — The system could theoretically execute cleanup automatically once safety checks pass.
- Class: anti-feature
- Status: deferred
- Description: The system could theoretically execute cleanup automatically once safety checks pass.
- Why it matters: It might reduce manual effort later.
- Source: inferred
- Primary owning slice: none
- Supporting slices: none
- Validation: unmapped
- Notes: Deferred away from current product direction; manual operator action is preferred.

## Out of Scope

### R030 — The system must not automatically delete original media as part of normal pipeline completion.
- Class: anti-feature
- Status: out-of-scope
- Description: The system must not automatically delete original media as part of normal pipeline completion.
- Why it matters: This prevents trust-destroying data loss behavior.
- Source: user
- Primary owning slice: none
- Supporting slices: none
- Validation: n/a
- Notes: Cleanup is explicit operator action only.

### R031 — The project does not include a graphical transcript editing interface.
- Class: anti-feature
- Status: out-of-scope
- Description: The project does not include a graphical transcript editing interface.
- Why it matters: This keeps the project focused on CLI-driven ingestion and processing.
- Source: inferred
- Primary owning slice: none
- Supporting slices: none
- Validation: n/a
- Notes: Transcript artifacts remain file-based.

### R032 — The project will not handle publishing transcripts, clips, or derived content to external platforms.
- Class: anti-feature
- Status: out-of-scope
- Description: The project will not handle publishing transcripts, clips, or derived content to external platforms.
- Why it matters: This prevents the scope from drifting into unrelated media operations.
- Source: inferred
- Primary owning slice: none
- Supporting slices: none
- Validation: n/a
- Notes: The focus stays on ingestion, processing, and memory-facing outputs.

## Traceability

| ID | Class | Status | Primary owner | Supporting | Proof |
|---|---|---|---|---|---|
| R001 | primary-user-loop | validated | M001/S01 | M001/S02, M001/S05 | S05 proof phase 1: 25 real artifacts each with per-item stage state (DOWNLOADED, OUTCOME, READY) in status table. Queue files and status.json persist durable job state across restarts. |
| R002 | core-capability | validated | M001/S01 | M001/S05 | queue and process commands ingest Twitch VODs via twitch.rs; artifact directories created with source_url.txt, metadata.json, audio.m4a, and status.json. Confirmed functional in S05 proof run against real artifacts. |
| R003 | operability | validated | M001/S02 | M001/S05 | download-all and transcribe-all operate as independent CLI commands. S05 proof shows artifacts in mixed staged states. Each command checks status.json before acting, enabling independent recovery. |
| R004 | quality-attribute | validated | M001/S03 | M001/S05 | hear-backed transcription with word-count threshold (50 words/hour) and repetition detection (trigram >10x in 200-word window). Both SRT and VTT required for completed outcome. 7/7 transcribe unit tests confirm heuristic logic. |
| R005 | failure-visibility | validated | M001/S01 | M001/S02, M001/S03, M001/S05 | ProcessStatus persists transcription_outcome, transcription_reason, and last_error. status command surfaces these in OUTCOME and REASON columns. proof.log phase 1 shows prior failure with reason; phase 3 shows manufactured failure captured and item remaining recoverable. |
| R006 | continuity | active | M001/S04 | M001/S05 | mapped |
| R007 | differentiator | active | M002/S01 | M002/S02, M002/S03 | mapped |
| R008 | integration | active | M002/S02 | M002/S03 | mapped |
| R009 | operability | active | M001/S04 | M001/S05 | mapped |
| R010 | core-capability | active | M003/S01 | none | mapped |
| R011 | differentiator | active | M002/S03 | M002/S02 | mapped |
| R012 | continuity | active | M001/S02 | M001/S03, M001/S05 | mapped |
| R020 | differentiator | deferred | none | none | unmapped |
| R021 | anti-feature | deferred | none | none | unmapped |
| R030 | anti-feature | out-of-scope | none | none | n/a |
| R031 | anti-feature | out-of-scope | none | none | n/a |
| R032 | anti-feature | out-of-scope | none | none | n/a |

## Coverage Summary

- Active requirements: 7
- Mapped to slices: 7
- Validated: 5 (R001, R002, R003, R004, R005)
- Unmapped active requirements: 0
