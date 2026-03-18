Download a video from Twitch

inspired by twitch-dl

## Requirements
- [ffmpeg](https://ffmpeg.org/download.html)

Optional:
- `TWITCH_DL_AUTH` or `--auth-token` for subscriber-only VODs

## Usage
`twitch-dl-rs download <video-link>` downloads a Twitch VOD into an artifact directory

`twitch-dl-rs queue <channel>` builds a backlog queue from a Twitch channel's archive page

`twitch-dl-rs process <channel>` builds a queue, downloads each queued VOD, and transcribes it with `mlx-whisper`

### Options
`-a, --auth-token <TEXT>`	Authentication token, passed to Twitch to access
subscriber only VODs. Can be copied from the auth_token cookie in any browser
logged in on Twitch.

`--output-root <DIR>`	Root directory where artifact folders are created. Defaults to `artifacts`.

`--quality <audio-only|lowest|highest>`	Preferred stream type. Defaults to `audio-only` for transcription-oriented workflows.

## Output

For a VOD like `https://www.twitch.tv/videos/123456789`, the downloader creates:

```text
artifacts/123456789/
  metadata.json
  source_url.txt
  audio.m4a
```

If the selected stream is not audio-only, the artifact file will be `video.mp4` instead.

This is Phase 1 of the larger pipeline. Transcription, notes, and memory storage come next.

## Queueing A Backlog

Build a queue from a Twitch channel name:

```bash
cargo run -- queue theprimeagen --past-broadcasts-only --min-seconds 600 --limit 25
```

This fetches recent archive VODs, skips video IDs that already have artifact directories in `artifacts/`, and writes the queue to:

```text
artifacts/queues/theprimeagen.json
```

Useful flags:

- `--past-broadcasts-only` makes the queue intent explicit
- `--min-seconds 600` skips short entries under 10 minutes
- `--limit 25` caps how much backlog to fetch on one run

## Processing A Queue

Run the end-to-end backlog processor for a channel:

```bash
cargo run -- process theprimeagen --past-broadcasts-only --min-seconds 600 --limit 5 --continue-on-error
```

Current `process` behavior:

- builds the filtered queue
- downloads each queued VOD using the selected stream quality
- transcribes each artifact to `transcript.txt` using `mlx-whisper` / `mlx_whisper`
- writes per-VOD `status.json` so reruns can resume cleanly

This stage does not yet generate `notes.md`; it prepares artifact directories for that next step.
