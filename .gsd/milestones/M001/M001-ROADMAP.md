# M001: Reliable media-to-transcript pipeline

## Vision
Turn the current Twitch-first downloader into a queue-first, artifact-first media pipeline that can ingest media, keep making progress without babysitting, produce trustworthy transcript artifacts, and leave completed items in a clear state for later notes work.

## Slice Overview
| ID | Slice | Risk | Depends | Done | After this |
|----|-------|------|---------|------|------------|
| S01 | Durable artifact and queue state | high | — | ✅ | You can queue Twitch media into durable per-item artifact folders with explicit status, and inspect what exists without guessing from raw files. |
| S02 | Decoupled staged processing | high | S01 | ✅ | Downloads can continue making progress while transcription work remains pending, running, or failed, and interrupted work can be resumed. |
| S03 | Trustworthy transcription and failure surfacing | high | — | ✅ | Finished items produce transcript artifacts you can trust more than the current fast path, and failed transcriptions show clear reasons and remain recoverable. |
| S04 | Ready-for-notes and manual cleanup workflow | medium | — | ⬜ | Completed transcripts enter a clear ready-for-notes state, and a cleanup command shows only safe deletion candidates without auto-deleting anything. |
| S05 | End-to-end operator flow proof | medium | S01, S02, S03, S04 | ⬜ | In one real CLI workflow, you can queue media, let staged processing run without babysitting, inspect failures, see ready-for-notes items, and review cleanup candidates. |
