# S06: Retry And Operational Hardening

**Goal:** Add force-retry for suspect transcriptions without re-downloading. Add file-level locking on status.json writes. Validate long-VOD transcription behavior.
**Demo:** After this: After this: run transcribe-all --force-suspect and watch a suspect item re-transcribe and update its outcome in status.

## Tasks
