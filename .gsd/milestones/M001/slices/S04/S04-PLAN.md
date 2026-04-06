# S04: Ready-for-notes and manual cleanup workflow

**Goal:** Surface a `ready_for_notes` lifecycle state automatically when transcription completes successfully, and provide a two-step cleanup command that lists audio and intermediate deletion candidates for operator review before any deletion occurs.
**Demo:** After this: Completed transcripts enter a clear ready-for-notes state, and a cleanup command shows only safe deletion candidates without auto-deleting anything.

## Tasks
- [x] **T01: Add ready_for_notes field to ProcessStatus with automatic state transition on transcription completion** — Add `ready_for_notes: bool` to `ProcessStatus` in `src/artifact.rs` with `#[serde(default)]` for backward compatibility. Set `status.ready_for_notes = true` in `transcribe_artifact()` in `src/main.rs` inside the `Completed` match arm, alongside the existing `status.transcribed = true`. Update `show_status()` to include a READY column (shows `yes` / `-`) after the OUTCOME column. Add two unit tests in `src/artifact.rs`: one confirming that old `status.json` JSON without the field deserializes as `false`, and one confirming `ready_for_notes = true` round-trips through write/read. Do NOT set `ready_for_notes` on `Suspect` or `Failed` outcomes.
  - Estimate: 30m
  - Files: src/artifact.rs, src/main.rs
  - Verify: cargo test artifact::tests 2>&1 | grep -E 'test result|FAILED' && cargo build 2>&1 | grep -E 'error|warning'
- [ ] **T02: Add cleanup CLI command with candidate listing and --delete flag** — Add a `cleanup` CLI subcommand that lists `ready_for_notes == true` artifacts as deletion candidates, showing per-item file sizes for `audio.m4a` and `transcript.srt`. With `--delete --all` or `--delete <video_id>`, remove those two files for the specified item(s). Never touch `transcript.vtt`, `metadata.json`, `status.json`, or `source_url.txt`. Eligibility is gated on `ready_for_notes == true` from the status field — NOT on file presence alone. Items with `suspect` or `failed` outcomes never appear as candidates.

CLI contract:
- `cleanup --output-root <dir>` — lists candidates with file sizes, no deletion
- `cleanup --output-root <dir> --delete <video_id>` — deletes audio.m4a and transcript.srt for that specific video_id (must be a ready_for_notes candidate)
- `cleanup --output-root <dir> --delete --all` — deletes audio.m4a and transcript.srt for ALL listed candidates
- Running `--delete` without either a video_id or `--all` is an error

Verification test: create a temp dir, write a synthetic `status.json` with `ready_for_notes: true, transcribed: true, transcription_outcome: 'completed'`, create dummy `audio.m4a` and `transcript.srt` files. Run the binary with `cleanup --output-root <tmpdir>` and verify the item appears. Run with `--delete --all` and verify both files are removed while `status.json` and `transcript.vtt` (if present) remain.
  - Estimate: 45m
  - Files: src/cli.rs, src/main.rs
  - Verify: cargo build 2>&1 | grep -c error | grep -q '^0' && ./target/debug/twitch-dl-rs cleanup --help | grep -q 'delete' && ./target/debug/twitch-dl-rs --help | grep -q cleanup
