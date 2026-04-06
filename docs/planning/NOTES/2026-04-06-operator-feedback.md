# 2026-04-06 Operator Feedback

Source: `vod-pipeline` ramble and follow-up planning discussion.

Captured themes:

- `status` is useful structurally, but video ID alone is not enough for real operator use.
- Status should show a truncated title, a human-readable stream/upload date, and channel name.
- Queue metadata already contains useful context; that context should be preserved into durable artifact metadata instead of being lost.
- Queueing should support one explicit video, not only channel backlog intake.
- `download` should be verified against the full artifact contract and normalized if needed.
- Batch-only controls are too coarse for some real workflows; selective download/transcribe targeting is desirable.
- Queue-aware status/filter views are needed so queued-only and processed items can be inspected intentionally.

Planning impact:

- These additions belong to workflow polish, not the notes/memory milestone.
- They now inform M003, especially status legibility, metadata durability, intake flexibility, and queue-aware filtering.
