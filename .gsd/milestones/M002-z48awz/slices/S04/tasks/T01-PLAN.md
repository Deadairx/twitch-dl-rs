---
estimated_steps: 9
estimated_files: 1
skills_used: []
---

# T01: Extend CLI structs and register --video-id arg on download-all and transcribe-all

Add `video_id: Option<String>` to `CliCommand::DownloadAll` and `CliCommand::TranscribeAll` in `src/cli.rs`, register the `--video-id` arg on both subcommands, and populate the field in the two `parse_args()` match arms. This is purely additive — no handler logic changes yet.

## Steps

1. Open `src/cli.rs`. Locate the `DownloadAll` variant (line ~42). Add `video_id: Option<String>` field after `continue_on_error`.
2. Locate the `TranscribeAll` variant (line ~48). Add `video_id: Option<String>` field after `continue_on_error`.
3. Locate the `Command::new("download-all")` block (line ~224). Copy the `Arg::new("video-id")` registration from the `cleanup` subcommand (line ~281) and add it to the download-all args. Update the help text to: `"Process only the VOD with this video ID"`.
4. Locate the `Command::new("transcribe-all")` block (~line 250). Add the same `Arg::new("video-id")` arg with help text: `"Transcribe only the artifact with this video ID"`.
5. Locate the `Some(("download-all", download_all_matches))` match arm (line ~378). Add `video_id: download_all_matches.get_one::<String>("video-id").cloned()` to the `CliCommand::DownloadAll { ... }` struct literal.
6. Locate the `Some(("transcribe-all", transcribe_all_matches))` match arm (line ~396). Add `video_id: transcribe_all_matches.get_one::<String>("video-id").cloned()` to the struct literal.
7. Run `cargo build` to confirm no errors.

## Inputs

- `src/cli.rs`

## Expected Output

- `src/cli.rs`

## Verification

cargo build && ./target/debug/vod-pipeline download-all --help | grep -q 'video-id' && ./target/debug/vod-pipeline transcribe-all --help | grep -q 'video-id'
