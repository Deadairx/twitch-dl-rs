# M002-z48awz: Workflow Polish

## Vision
Make the operator workflow legible, flexible, and robust: status shows human-readable context, intake accepts single videos, download-all drains queues without a channel argument, suspect transcriptions are retryable, and at least one non-Twitch source works through the same artifact model.

## Slice Overview
| ID | Slice | Risk | Depends | Done | After this |
|----|-------|------|---------|------|------------|
| S01 | Metadata Durability | high | — | ⬜ | After this: run download on a Twitch URL and inspect metadata.json to see title, uploaded_at, channel alongside existing fields. Old artifact directories still load cleanly. |
| S02 | Status Legibility | medium | S01 | ⬜ | After this: run status against an output root with queued, downloaded, and transcribed items and see a readable table with title, date, and channel for every row. |
| S03 | Intake Flexibility | medium | S01 | ⬜ | After this: run queue-video on a Twitch URL, then run download-all with no arguments and watch the queued item download. |
| S04 | Selective Processing | low | S01, S03 | ⬜ | After this: run download-all --video-id 123456789 and watch only that one item download while others are skipped. |
| S05 | Queue-Aware Filtering | low | S02, S04 | ⬜ | After this: run status --filter failed and see only failed items; run status --filter queued and see only items waiting to download. |
| S06 | Retry And Operational Hardening | medium | S05 | ⬜ | After this: run transcribe-all --force-suspect and watch a suspect item re-transcribe and update its outcome in status. |
| S07 | Additional Source Support | high | S06 | ⬜ | After this: run download on a YouTube URL, then transcribe-all, and see the artifact appear in status alongside Twitch items. |
