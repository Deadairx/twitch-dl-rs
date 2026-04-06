## vod-pipeline

Queue-first media pipeline for Twitch VOD artifacts.

The binary name is `vod-pipeline`.

## Requirements

- [ffmpeg](https://ffmpeg.org/download.html)
- `hear` available on `PATH` for transcription

Optional:

- `TWITCH_DL_AUTH` or `--auth-token` for subscriber-only VODs

## Install

```bash
cargo install --path .
```

This installs `vod-pipeline` into Cargo's bin directory, usually `~/.cargo/bin`.

## Default Artifact Root

Set a canonical artifact root once so the CLI can run from anywhere:

```bash
export VOD_PIPELINE_OUTPUT_ROOT="$HOME/artifacts/twitch"
```

Output root precedence is:

1. `--output-root <DIR>`
2. `VOD_PIPELINE_OUTPUT_ROOT`
3. `artifacts`

## Commands

```bash
vod-pipeline download <video-link>
vod-pipeline queue <channel>
vod-pipeline process <channel>
vod-pipeline status
vod-pipeline download-all <channel>
vod-pipeline transcribe-all
vod-pipeline cleanup
```

Common options:

- `-a, --auth-token <TEXT>` authenticates subscriber-only VOD access
- `--output-root <DIR>` overrides `VOD_PIPELINE_OUTPUT_ROOT` for a single run
- `--quality <audio-only|lowest|highest>` defaults to `audio-only`

## Artifact Layout

For a VOD like `https://www.twitch.tv/videos/123456789`, the pipeline creates:

```text
<output-root>/123456789/
  metadata.json
  source_url.txt
  audio.m4a
  status.json
  transcript.srt
  transcript.vtt

<output-root>/queues/
  <channel>.json
```

If the selected stream is not audio-only, the media artifact is `video.mp4` instead of `audio.m4a`.

`status.json` is the durable per-artifact record used for resume behavior, failure visibility, and cleanup eligibility.

## Queueing A Backlog

```bash
vod-pipeline queue theprimeagen --past-broadcasts-only --min-seconds 600 --limit 25
```

This fetches recent archive VODs, skips IDs that already have artifact directories in the output root, and writes the queue to `queues/theprimeagen.json`.

Useful flags:

- `--past-broadcasts-only` makes the queue intent explicit
- `--min-seconds 600` skips short entries under 10 minutes
- `--limit 25` caps how much backlog to fetch on one run

## Processing

Run the end-to-end channel processor:

```bash
vod-pipeline process theprimeagen --past-broadcasts-only --min-seconds 600 --limit 5 --continue-on-error
```

Current `process` behavior:

- builds the filtered queue
- downloads each queued VOD using the selected stream quality
- transcribes each artifact with `hear`
- writes `transcript.srt` and `transcript.vtt`
- persists per-VOD `status.json` so reruns can resume cleanly

For staged operation, use:

```bash
vod-pipeline download-all theprimeagen
vod-pipeline transcribe-all --continue-on-error
vod-pipeline status
vod-pipeline cleanup
```

The pipeline does not generate notes yet. It prepares durable transcript artifacts for that next layer.
