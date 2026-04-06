---
estimated_steps: 7
estimated_files: 2
skills_used: []
---

# T02: Add cleanup CLI command with candidate listing and --delete flag

Add a `cleanup` CLI subcommand that lists `ready_for_notes == true` artifacts as deletion candidates, showing per-item file sizes for `audio.m4a` and `transcript.srt`. With `--delete --all` or `--delete <video_id>`, remove those two files for the specified item(s). Never touch `transcript.vtt`, `metadata.json`, `status.json`, or `source_url.txt`. Eligibility is gated on `ready_for_notes == true` from the status field — NOT on file presence alone. Items with `suspect` or `failed` outcomes never appear as candidates.

CLI contract:
- `cleanup --output-root <dir>` — lists candidates with file sizes, no deletion
- `cleanup --output-root <dir> --delete <video_id>` — deletes audio.m4a and transcript.srt for that specific video_id (must be a ready_for_notes candidate)
- `cleanup --output-root <dir> --delete --all` — deletes audio.m4a and transcript.srt for ALL listed candidates
- Running `--delete` without either a video_id or `--all` is an error

Verification test: create a temp dir, write a synthetic `status.json` with `ready_for_notes: true, transcribed: true, transcription_outcome: 'completed'`, create dummy `audio.m4a` and `transcript.srt` files. Run the binary with `cleanup --output-root <tmpdir>` and verify the item appears. Run with `--delete --all` and verify both files are removed while `status.json` and `transcript.vtt` (if present) remain.

## Inputs

- `src/cli.rs`
- `src/main.rs`
- `src/artifact.rs`

## Expected Output

- `src/cli.rs`
- `src/main.rs`

## Verification

cargo build 2>&1 | grep -c error | grep -q '^0' && ./target/debug/twitch-dl-rs cleanup --help | grep -q 'delete' && ./target/debug/twitch-dl-rs --help | grep -q cleanup

## Observability Impact

cleanup command provides candidate listing with file sizes; deletion confirms each removed file by name
