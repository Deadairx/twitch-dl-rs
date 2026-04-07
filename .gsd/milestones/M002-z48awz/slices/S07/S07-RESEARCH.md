# S07: Additional Source Support — Research

**Gathered:** 2026-04-07  
**Complexity:** Medium — the yt-dlp subprocess pattern is well understood, but there is a critical blocker in `existing_artifact_ids` that must be fixed to make YouTube artifacts visible to `status` and `transcribe-all`.

---

## Summary

S07 adds YouTube as a second download source by routing `download <url>` through yt-dlp when the URL hostname is not `twitch.tv`. The new `src/ytdlp.rs` module handles yt-dlp subprocess invocation and metadata extraction. The `download_vod` function in `main.rs` gets URL-based routing at the top. The resulting artifact is structurally identical to a Twitch artifact and is visible to all downstream commands (`status`, `transcribe-all`).

**Critical blocker discovered:** `existing_artifact_ids()` in `artifact.rs` only discovers directories whose names are all ASCII digits (`name.chars().all(|char| char.is_ascii_digit())`). YouTube video IDs are 11-character alphanumeric strings (e.g., `jNQXAC9IVRw`). Without fixing this filter, YouTube artifacts will be invisible to `scan_artifact_statuses`, `show_status`, and `transcribe-all`. **This is the highest-risk piece of S07.**

---

## Implementation Landscape

### Files to modify

**`src/main.rs`**
- `download_vod()` is the primary entry point for the `download` command. It currently calls `twitch::extract_video_id()` at the top — this will become the routing decision point.
- URL routing logic: parse the URL hostname before calling `extract_video_id`. If the host contains `youtube.com` or `youtu.be`, branch to the new yt-dlp path; otherwise fall through to the existing Twitch path unchanged.
- The yt-dlp path needs to: call `ytdlp::download_youtube()`, then call the same artifact helpers (`prepare_artifact_dir`, `write_source_url`, `write_metadata`, `write_status`) as the Twitch path.
- `download_vod`'s existing signature and Twitch path must remain completely unchanged.

**`src/artifact.rs`**
- `existing_artifact_ids()` must be widened to include non-all-digit directory names that contain a `status.json` file. The current filter (`chars().all(|c| c.is_ascii_digit())`) is the gating blocker.
- Proposed fix: drop the all-digit check; instead, treat any subdirectory that contains `status.json` as a valid artifact. This is already the de-facto contract — all artifact dirs have `status.json` written by S01's normalization. The `queues/` subdirectory won't have a `status.json`, so it won't pollute results. The status.json-presence check is also backward-compatible.
- `ArtifactMetadata::from_download()` is Twitch-specific (takes `StreamInfo`). The yt-dlp path needs a parallel constructor or a distinct metadata write path. Recommend a new `ArtifactMetadata::from_ytdlp()` that takes `YtDlpMetadata` and writes the same JSON schema. Stream-specific fields (`stream_name`, `selected_bandwidth`, `selected_resolution`, `selected_codecs`, `is_audio_only`) can be set to `None`/`true` for the yt-dlp case.

**`src/ytdlp.rs`** (new file)
- Public function: `download_youtube(url: &str, output_path: &Path) -> Result<YtDlpMetadata, Box<dyn std::error::Error>>`
- `YtDlpMetadata` struct: `id: String`, `title: String`, `channel: String`, `uploaded_at: String`
- Subprocess invocation strategy:
  - Run yt-dlp with `--dump-json --skip-download` first to get metadata, extract the video ID for naming the output file
  - Then run yt-dlp with `-x --audio-format m4a -o <path>` to download
  - OR: run a single invocation using `--print-json` (alias for `--dump-json`, available in recent versions) combined with actual download, then parse stdout for the JSON blob. This is riskier because stdout may mix JSON with progress output.
  - **Recommended approach**: two-pass — metadata first (no download), then download with deterministic `-o` path. This gives clean JSON parsing and a known output filename.
- yt-dlp not found: check with `which yt-dlp` or catch `std::io::ErrorKind::NotFound` from `Command::new("yt-dlp")` and return a clear error: `"yt-dlp not found; install with: brew install yt-dlp"`.
- Audio format: use `-x --audio-format m4a`. This invokes ffmpeg post-processing to convert to m4a. Some YouTube videos return m4a natively (via `-f bestaudio[ext=m4a]`) but conversion is more reliable across formats.
- Output path: `-o <artifact_dir>/audio.m4a` — deterministic, matches `find_media_file()` expectations.
- `upload_date` normalization: yt-dlp returns `"20050424"` (YYYYMMDD). Convert to `"2005-04-24"` (YYYY-MM-DD) for consistency with Twitch convention.

**`src/lib.rs`**
- Add `pub mod ytdlp;` to expose the new module.

### URL routing logic

```rust
// In download_vod(), before the Twitch path:
let url_host = url::Url::parse(video_link)
    .ok()
    .and_then(|u| u.host_str().map(|h| h.to_string()))
    .unwrap_or_default();

if url_host.contains("youtube.com") || url_host.contains("youtu.be") {
    return download_youtube_vod(video_link, output_root).await;
}
// else fall through to Twitch path (unchanged)
```

`url` crate is already a dependency — no new crates needed for URL parsing.

### yt-dlp subprocess: two-pass approach

**Pass 1 — metadata + video ID:**
```bash
yt-dlp --dump-json --skip-download "<url>"
```
Parse stdout as JSON. Extract: `id`, `title`, `channel` (or `uploader`), `upload_date`.

**Pass 2 — download:**
```bash
yt-dlp -x --audio-format m4a -o "<artifact_dir>/audio.m4a" "<url>"
```

yt-dlp is available at `/opt/homebrew/bin/yt-dlp` on this machine (version 2025.06.09). `--dump-json` is confirmed working. Sample output from `Me at the zoo`:
- `id`: `'jNQXAC9IVRw'` (11-char alphanumeric)
- `title`: `'Me at the zoo'`
- `uploader`: `'jawed'`
- `channel`: `'jawed'`
- `upload_date`: `'20050424'` → converted to `'2005-04-24'`

Both `channel` and `uploader` fields are present. Use `channel` first, fall back to `uploader`.

### artifact_dir naming for YouTube

The artifact directory must be named after the YouTube video ID (e.g., `jNQXAC9IVRw`). Extract from the `id` field of yt-dlp JSON (most reliable). Alternatively, parse from the URL (`v=` query param or youtu.be path), but the JSON `id` field is authoritative and handles edge cases (shortened URLs, playlist params).

### `existing_artifact_ids` fix — status.json presence check

Current code (artifact.rs line 135):
```rust
if name.chars().all(|char| char.is_ascii_digit()) {
    ids.push(name.to_string());
}
```

Proposed replacement:
```rust
// Accept any subdirectory that has a status.json (the artifact contract)
let status_path = entry.path().join("status.json");
if status_path.exists() {
    ids.push(name.to_string());
}
```

This is backward-compatible: all existing Twitch artifact directories have `status.json` (guaranteed since S01's normalization). `queues/` has no `status.json`. This is a pure widening — no regressions expected.

**Implication for unit tests:** `test_scan_queue_dedup_with_artifact` creates artifact dir `"100"` (all-digit) and `"300"` (all-digit with no status.json). After the fix, `"300"` won't appear in scan results because it has no `status.json`. This test needs updating: either add a `status.json` to `"300"` or adjust the assertion. The test's intent (dedup logic) is still valid; the setup needs to match the new contract.

### Metadata schema — `from_ytdlp()` constructor

YouTube artifacts write identical `metadata.json` schema. Stream-specific fields are set as:
- `stream_name`: `None`
- `selected_bandwidth`: `None`
- `selected_resolution`: `None`
- `selected_codecs`: `None`
- `is_audio_only`: `true` (audio-only download)
- `used_auth_token`: `false` (no auth for YouTube)
- `schema_version`: `1` (same)

The `title`, `channel`, `uploaded_at` fields are populated from `YtDlpMetadata`. `output_file` is `"audio.m4a"`. `output_size_bytes` is read from the downloaded file (same as Twitch path).

### Artifact directory invariant check

`status.json` + `metadata.json` + `source_url.txt` + `audio.m4a` — same four files as Twitch. The yt-dlp path must write all four to maintain the invariant. `find_media_file()` checks for `audio.m4a` and `video.mp4` — the yt-dlp path produces `audio.m4a`, so no change needed there.

### No changes needed in `transcribe.rs`, `cli.rs`, or `show_status`

- `transcribe_artifact()` is called with `(video_id, artifact_dir, media_path, status)` — source-agnostic, works as-is.
- `transcribe_all()` uses `scan_artifact_statuses()` which uses `existing_artifact_ids()` — fixed above.
- `show_status()` uses `scan_artifact_statuses()` — fixed above.
- `download` subcommand in `cli.rs` already accepts a generic URL — no CLI changes needed.
- `--skip-metadata` flag does not apply to the YouTube path (yt-dlp metadata and download are coupled by design).

### No new crate dependencies

- `url` crate: already in Cargo.toml (used in `downloader.rs`)
- `serde_json`: already in Cargo.toml (used everywhere)
- `std::process::Command`: used in `ffmpeg.rs` and `transcribe.rs` — same pattern for yt-dlp subprocess

---

## Risks and Constraints

### Risk 1: `existing_artifact_ids` all-digit filter (CRITICAL)
Without fixing this, YouTube artifacts are invisible to all downstream commands. This is the first task and blocks everything else.

### Risk 2: yt-dlp audio format output
yt-dlp with `-x --audio-format m4a` may need ffmpeg installed to transcode. On this machine, ffmpeg is already available (used for Twitch downloads). If ffmpeg is absent, yt-dlp will error — same class of error as if yt-dlp itself is absent. Document the dependency.

### Risk 3: yt-dlp `-o` template and file extension
When using `-x --audio-format m4a`, yt-dlp downloads the best audio stream and then post-processes it to m4a. The `-o` output template should specify `audio.m4a` directly. However, yt-dlp may append `.m4a` again if the template already has an extension. Use `-o "%(id)s"` and let yt-dlp handle extension, then rename — OR use the two-pass approach where Pass 1 gives us the ID and Pass 2 uses a clean path. Safest: set output to a temp path and verify the file exists after download.

Actually, using `-o <full/path/audio.m4a>` with `--no-part` should work cleanly. yt-dlp respects the full output path when the extension matches the final format. Test case: `yt-dlp -x --audio-format m4a --no-part -o /tmp/test_audio.m4a <url>`.

### Risk 4: unit test for `test_scan_queue_dedup_with_artifact`
The artifact dir `"300"` in this test has `audio.m4a` but no `status.json`. After the `existing_artifact_ids` fix (status.json presence), `"300"` will no longer appear in scan results. The test assertion `assert_eq!(artifact_results.len(), 2)` will fail (only `"100"` appears). The test must be updated to add a `status.json` to `"300"` or adjust the count to 1.

---

## Task Decomposition for Planner

**T01 — Fix `existing_artifact_ids` and add `from_ytdlp` to ArtifactMetadata**
- Files: `src/artifact.rs`
- Work: Replace all-digit filter with status.json-presence check; add `ArtifactMetadata::from_ytdlp(video_id, source_url, output_file, metadata: &YtDlpMetadata)` constructor; update `test_scan_queue_dedup_with_artifact` to add status.json to the "300" artifact dir.
- Verify: `cargo test` passes; YouTube-named artifact dirs (alphanumeric IDs) appear in `scan_artifact_statuses`.

**T02 — Implement `src/ytdlp.rs`**
- Files: `src/ytdlp.rs` (new), `src/lib.rs`
- Work: `YtDlpMetadata` struct with `id`, `title`, `channel`, `uploaded_at`; `download_youtube(url, output_path) -> Result<YtDlpMetadata, Box<dyn std::error::Error>>` using two-pass yt-dlp invocation; yt-dlp-not-found error with install hint; `upload_date` normalization from YYYYMMDD to YYYY-MM-DD; unit tests for date normalization and not-found error handling.
- Verify: `cargo test` passes; unit tests cover date normalization and not-found detection.

**T03 — Wire URL routing into `download_vod` in `main.rs`**
- Files: `src/main.rs`
- Work: Add hostname-based routing at the top of `download_vod`; implement `download_youtube_vod(url, output_root)` helper that calls `ytdlp::download_youtube()`, then calls artifact helpers in the same sequence as the Twitch path; write `metadata.json` via `ArtifactMetadata::from_ytdlp()`; write `status.json` with `downloaded=true`.
- Verify: `cargo build` succeeds; `cargo test` passes; integration test: `download <youtube-url>` produces an artifact dir with `audio.m4a`, `metadata.json`, `status.json`, `source_url.txt`; `status` shows the YouTube artifact; `transcribe-all` picks it up.

---

## Decisions for Planner to Record

- **yt-dlp invocation strategy**: two-pass (metadata first with `--dump-json --skip-download`, then download with `-x --audio-format m4a`) vs single-pass. Recommend two-pass for clean stdout parsing.
- **Audio format flag**: `-x --audio-format m4a` (post-processing via ffmpeg) vs `-f bestaudio[ext=m4a]` (format selection, no post-processing). Recommend `-x --audio-format m4a` for reliability across all videos.
- **`existing_artifact_ids` expansion strategy**: status.json presence vs broader name validation. Recommend status.json presence as it's the artifact contract.

---

## Verification Commands

```bash
cargo build                         # must succeed
cargo test                          # all tests pass (currently 65)
vod-pipeline download <youtube-url> # produces artifact dir
ls <output-root>/<youtube-id>/      # audio.m4a metadata.json status.json source_url.txt
vod-pipeline status                 # YouTube artifact appears in table
vod-pipeline transcribe-all         # picks up the YouTube artifact
```
