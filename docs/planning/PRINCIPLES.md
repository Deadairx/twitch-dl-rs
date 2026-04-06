# Planning Principles

## Durable Product Constraints

- `hear` is the default transcription backend for the current product direction.
- Canonical pipeline transcription remains `hear -d -i <audio-file> -S`.
- `transcript.srt` and `transcript.vtt` are the trusted transcript artifacts.
- `transcript.txt` is an optional readable derivative, not the source of truth.
- `status.json` is the durable per-item stage-state record.
- Cleanup remains explicit and operator-controlled.
- Notes and memory work remain downstream of transcript completion, not fused into the core pipeline.
- Full transcripts stay on disk; only concise downstream notes belong in Ember.
- Chat capture and transcript/chat alignment remain future work.

## Artifact Model

```text
<output-root>/
  queues/
    <channel>.json
  <video_id>/
    metadata.json
    source_url.txt
    audio.m4a | video.mp4
    transcript.srt
    transcript.vtt
    transcript.txt
    notes.md
    memory.json
    status.json
```

Notes:

- `notes.md` and `memory.json` are downstream artifacts planned for future milestones, not M001 outputs.
- Legacy artifacts may contain `transcript.legacy.txt`; treat those as upgrade candidates, not canonical transcript outputs.

## Implementation Guardrails

- Keep per-stage behavior isolated so failures remain resumable.
- Prefer durable JSON artifact/state files over hidden ad hoc state.
- Treat transcript generation as expensive; do not rerun canonical transcription unless explicitly forced.
- Treat note generation and memory persistence as downstream stages after transcript trust is established.
- Keep the operating model local-first and single-user unless a milestone explicitly broadens it.
