# VOD Pipeline Roadmap

## Goal
Turn `twitch-dl-rs` into a repeatable pipeline that:

- detects new VODs for a chosen Twitch creator
- downloads each VOD as a durable local artifact
- transcribes it with a local transcription backend
- produces concise notes suitable for long-term recall
- stores only the concise notes in Ember memory while keeping the full transcript on disk

## Decisions Locked In

- Use `hear` as the default and only in-scope transcription backend for the current milestone; prioritize transcript trustworthiness over speed or backend comparison.
- When invoking `hear`, always use on-device mode with file input explicitly: `hear -d -i <audio-file> -S`; do not rely on microphone defaults or omit `-d`.
- Persist the trusted transcript path as `hear -> transcript.srt -> transcript.vtt`, with `transcript.txt` as an optional derived readable artifact.
- Defer chat capture and transcript/chat timeline alignment to a future milestone; treat them later as first-class timed artifacts rather than part of transcription itself.
- Keep the full transcript as a filesystem artifact.
- Store concise structured notes in memory, not the raw transcript.
- Build the first version around one creator/channel, then generalize.

## Current State

What already exists:

- `src/cli.rs` supports a basic `download` command.
- `src/twitch.rs` can extract a video ID and fetch a playback token.
- `src/downloader.rs` can walk an HLS playlist and download `.ts` segments.

What is missing for the full workflow:

- channel-level VOD discovery
- durable metadata and processed-state tracking
- ffmpeg-based assembly into a single media artifact
- transcription orchestration
- summary generation and memory write flow
- idempotent automation for "only process new VODs"

## Target Artifact Layout

Store each processed VOD under a stable directory such as:

```text
artifacts/
  <channel>/
    <video_id>/
      metadata.json
      source_url.txt
      video.mp4
      transcript.srt
      transcript.vtt
      transcript.txt
      notes.md
      memory.json
      status.json
```

Notes:

- `transcript.srt` is the raw trusted subtitle output from `hear`.
- `transcript.vtt` is the canonical timed transcript artifact for downstream use.
- `transcript.txt` is an optional readable derivative, not the canonical source of truth.
- `notes.md` is the concise human-readable summary.
- `memory.json` stores the exact concise payload intended for Ember.
- `status.json` tracks pipeline completion so reruns are safe.

## Phase Plan

### Phase 1: Harden Single-VOD Download

Objective: make one VOD download cleanly into a single usable artifact.

Tasks:

- improve CLI shape so a single command can target one VOD and an output directory
- remove debug playlist dumping and replace it with clear progress output
- select the desired stream variant deterministically instead of taking the first playlist entry
- assemble downloaded segments into a single media file with `ffmpeg`
- write `metadata.json` and `source_url.txt`

Exit criteria:

- given a Twitch VOD URL, the tool produces one finished local media artifact plus metadata

### Phase 2: Add Channel Polling for New VODs

Objective: support "watch this creator and process new uploads only."

Tasks:

- add a way to resolve a channel/creator to recent VODs
- choose a persistence mechanism for processed IDs (`state.json` is enough for v1)
- implement a `sync` or `poll` command that fetches recent VODs and skips known ones
- cap how far back to look so the first run is controllable

Exit criteria:

- rerunning the command processes only newly discovered VODs

### Phase 3: Transcription Stage

Objective: convert the downloaded media artifact into a stable transcript artifact.

Note: chat capture is intentionally out of scope for the current milestone. A future milestone should add durable chat capture, likely as `chat.json`, plus alignment against VOD/transcript timestamps.

Tasks:

- invoke `hear` with the explicit file-input, on-device subtitle command: `hear -d -i <audio-file> -S`
- do not rely on implicit stdin/microphone behavior; the file path must always be passed via `-i`
- treat `-d` as mandatory so transcription stays local-only and avoids server-side/API-selected behavior
- persist the raw subtitle output to `transcript.srt`
- convert the trusted subtitle output into canonical `transcript.vtt`
- optionally derive a readable `transcript.txt`
- capture transcription stderr/stdout summary into status metadata for debugging

Exit criteria:

- each processed VOD has a canonical timed transcript artifact on disk, with optional readable derivative output

### Phase 4: Summarization Stage

Objective: produce concise notes that are worth remembering later.

Tasks:

- define a stable notes format for creator VODs
- generate notes from the transcript with emphasis on topics, takeaways, updates, and memorable moments
- keep the notes short enough for recall but detailed enough to be useful
- write `notes.md` and `memory.json`

Suggested note schema:

- creator
- stream title/date/url
- main topics
- notable announcements or decisions
- useful tips / recurring bits
- concise takeaways
- uncertainty flag if transcript quality is weak

Exit criteria:

- each transcript yields a concise summary artifact ready for memory storage

### Phase 5: Memory Write Flow

Objective: persist only the durable, compact knowledge.

Tasks:

- map `memory.json` into an Ember memory payload
- store one memory per VOD summary, not per transcript chunk
- tag memories consistently, for example `twitch`, creator name, and `vod-summary`
- include artifact paths in the memory expansion so the full transcript can be found later

Exit criteria:

- a processed VOD produces one searchable memory containing concise notes and artifact references

### Phase 6: Automation and Ops

Objective: make the workflow easy to run repeatedly.

Tasks:

- add a top-level command that runs discovery -> download -> transcribe -> summarize
- make every step resumable from `status.json`
- document required dependencies: Twitch auth token, `ffmpeg`, and the chosen local transcription backend
- optionally add a launchd/cron example for periodic runs

Exit criteria:

- one command can be scheduled and rerun safely

## Proposed CLI Shape

This is a good v1 shape, not a final contract:

```text
twitch-dl-rs download <vod-url> [--output-root <dir>]
twitch-dl-rs sync <channel> [--limit <n>] [--since <date>]
twitch-dl-rs transcribe <artifact-dir>
twitch-dl-rs summarize <artifact-dir>
twitch-dl-rs process <channel> [--limit <n>]
```

`process` becomes the main automation entrypoint once the lower-level commands are stable.

## Implementation Notes

- Keep per-stage functions isolated so failures are resumable.
- Prefer JSON metadata files over hidden ad hoc state.
- Treat transcript generation as expensive; do not rerun it if `transcript.vtt` already exists unless forced.
- Treat memory writes as the final step after notes are validated.
- Keep the first implementation local-only and single-user.

## Immediate Next Steps

1. Finish Phase 1 so one VOD becomes a single local media artifact.
2. Add a small artifact/state model on disk.
3. Wire in the `hear -d` transcription step.
4. Add summary generation and Ember memory storage.
5. Expand from single-VOD mode to channel sync mode.
