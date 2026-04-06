# Milestones

## M001: Reliable Media-to-Transcript Pipeline

Status: complete

Delivered:

- durable queue and artifact state
- decoupled staged processing
- trustworthy transcription with surfaced failures
- ready-for-notes state
- safe cleanup workflow
- end-to-end proof of the operator flow

## M002: Notes And Ember Memory Workflow

Status: future

Primary scope:

- manual-first note generation with prompt/style choice
- Ember memory persistence for selected outputs
- support/contradict analysis against existing memory context

Success shape:

- a completed transcript can be turned into notes intentionally, with a chosen lens
- selected note outputs can be persisted into Ember without storing the raw transcript itself

## M003: Source Expansion And Workflow Polish

Status: future

### Slice 1: Additional Source Support

- add at least one non-Twitch source
- preserve the same durable artifact/state model
- keep source-specific intake isolated from downstream stages

### Slice 2: Status Legibility And Metadata Durability

- preserve queue metadata like title and uploaded-at into durable artifact metadata
- add truncated title, human-readable date, and channel name columns to `status`
- keep video ID visible as a stable selector
- decide canonical ownership between `metadata.json` and `status.json`
- re-evaluate whether `output_file` remains useful metadata

### Slice 3: Intake Flexibility And Selective Processing

- support queueing a single explicit video, not only channel backlog intake
- verify and normalize `download` so it consistently produces full artifact-shaped output
- add selective download targeting for chosen queued items
- add selective transcribe targeting for chosen downloaded items
- keep `download-all` and `transcribe-all` as batch flows

### Slice 4: Queue-Aware Views And Filtering

- show queued-but-not-yet-downloaded items in an intentional status-oriented view
- add filters such as queued-only, processed-only, pending-only, or failed-only
- keep queue state and artifact state visible without conflating them

### Slice 5: Retry And Operational Hardening

- force-retry UX for suspect transcriptions
- validate long-VOD transcription behavior
- improve concurrent access safety for `status.json`

## Recommended Execution Order

1. Status legibility and metadata durability
2. Intake flexibility and selective processing
3. Queue-aware views and filtering
4. Retry and operational hardening
5. Additional source support
