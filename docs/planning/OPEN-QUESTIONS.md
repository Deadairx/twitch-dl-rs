# Open Questions

## Metadata Ownership

- Which display fields should live canonically in `metadata.json`?
- Which fields, if any, should be duplicated into `status.json` for fast status rendering?
- Is `output_file` still useful, or is it durable noise?

## Status View Shape

- Should queued-but-not-downloaded items appear in `status` by default or behind a flag?
- How wide can the status table become before title/date/channel columns hurt terminal readability?
- Should channel always be shown, or only when mixed-channel output roots are expected?

## Intake UX

- Should single-video intake extend `queue` or become a separate command?
- Should selective processing target video IDs, queue positions, or another selector model?

## Pipeline Behavior

- How should `download` and queue-driven download behavior be normalized if they diverge today?
- What is the right force-retry UX for suspect transcriptions?
- Do long VODs require chunking or another transcription strategy?

## Future Scope

- Which non-Twitch source should land first?
- When chat capture eventually exists, should it live in the same artifact directory or a related sibling artifact?
