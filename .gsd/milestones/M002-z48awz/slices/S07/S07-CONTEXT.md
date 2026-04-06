---
id: S07
milestone: M002-z48awz
status: ready
---

# S07: Additional Source Support — Context

## Goal

Add YouTube as a second supported source by routing `download <url>` through yt-dlp when the URL is not a Twitch URL, producing an artifact that downstream stages (transcribe-all, status) treat identically to a Twitch artifact.

## Why this Slice

S07 is the milestone capstone. All prior slices hardened the pipeline and operator experience against Twitch artifacts. S07 proves the artifact model is source-agnostic by running a non-Twitch URL through it end-to-end. It is last because it introduces the highest integration risk (new subprocess, new metadata shape) and must not destabilize the Twitch path.

## Scope

### In Scope

- URL-based routing in the `download` command: if the URL hostname is `twitch.tv`, use the existing Twitch path; if it is `youtube.com` or `youtu.be`, use a new yt-dlp path
- yt-dlp invoked as a subprocess — not a Rust library
- yt-dlp downloads the best available audio stream (same intent as the Twitch audio-only path)
- `metadata.json` populated from yt-dlp output: `title` from yt-dlp metadata, `channel` from yt-dlp `uploader`/`channel` field, `uploaded_at` from yt-dlp `upload_date` (normalized to ISO 8601 `YYYY-MM-DD` format to match Twitch convention)
- `status.json` written with `downloaded` state after successful yt-dlp download (same as the normalized bare-download path from S01)
- If yt-dlp is not found in PATH: abort with a clear error message and install instructions, e.g. `"yt-dlp not found; install with: brew install yt-dlp"`
- The resulting artifact appears in `status` alongside Twitch artifacts with no special treatment
- `transcribe-all` picks up the YouTube artifact and transcribes it via the existing `transcribe_artifact` helper — no transcription changes needed
- `cargo test` passes; `cargo build` succeeds

### Out of Scope

- `queue-video` support for YouTube URLs — S03 queues Twitch only; YouTube intake is bare `download` only in this slice
- Quality selection for yt-dlp (no `--quality` flag passthrough) — audio-only, best available, no operator control
- Any source beyond YouTube in this slice — the routing layer should be extensible but only YouTube is wired up
- yt-dlp version pinning or auto-update — operator is responsible for having a working yt-dlp installation
- Metadata fetch failure handling for yt-dlp path — if yt-dlp fails to retrieve metadata, the whole download fails (same hard-failure policy as S01's GQL fetch; no `--skip-metadata` equivalent needed here since yt-dlp metadata and download are one call)

## Constraints

- The Twitch download path must be completely unchanged — URL routing is additive, not a refactor
- Source-specific logic must be isolated: yt-dlp invocation lives in a new `src/ytdlp.rs` (or equivalent) module, not inline in `main.rs` or `twitch.rs`
- The artifact directory structure produced by the yt-dlp path must be identical to the Twitch path: `<output_root>/<video_id>/audio.m4a`, `metadata.json`, `status.json`, `source_url.txt`
- Video ID for YouTube artifacts: extract from the YouTube URL (the `v=` parameter or youtu.be path segment) — this is the artifact directory name and must be stable and unique
- `hear` invocation is unchanged; the transcript path is the same regardless of source

## Integration Points

### Consumes

- `src/main.rs` — `download_vod` function (gets URL routing added at the top); `write_status` pattern from S01 for writing `status.json` after download
- `src/artifact.rs` — `ArtifactMetadata::from_download` (extended or paralleled for yt-dlp metadata shape); `write_metadata`, `write_status`, `prepare_artifact_dir`, `write_source_url`
- `yt-dlp` CLI subprocess — invoked for download and metadata extraction in one call

### Produces

- `src/ytdlp.rs` (new) — `download_youtube(url, output_path) -> Result<YtDlpMetadata, ...>` where `YtDlpMetadata` carries title, channel, uploaded_at, and video_id extracted from yt-dlp JSON output
- `src/main.rs` — `download_vod` updated with hostname-based URL routing: Twitch → existing path, YouTube → new yt-dlp path
- Artifact directory for YouTube VODs: identical layout to Twitch artifacts, visible in `status` and pickable by `transcribe-all`

## Open Questions

- **yt-dlp JSON output format**: yt-dlp's `--print-json` or `--dump-json` flag emits a large JSON blob. Executor should identify the minimum fields needed (`id`, `title`, `uploader`/`channel`, `upload_date`) and parse only those — do not deserialize the full schema.
- **audio format from yt-dlp**: yt-dlp may produce `.webm` or `.opus` rather than `.m4a` depending on availability. Executor should pass `--audio-format m4a` (or equivalent post-processing flag) to normalize the output to `.m4a` for `hear` compatibility. Document the flag choice in DECISIONS.md.
