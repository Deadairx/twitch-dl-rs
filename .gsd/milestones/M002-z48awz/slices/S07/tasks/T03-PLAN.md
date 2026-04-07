---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Wire YouTube URL routing into download_vod and produce a complete artifact

Add hostname-based URL routing to `download_vod` in `src/main.rs`. At the very top of `download_vod`, before calling `twitch::extract_video_id`, parse the URL hostname using the already-imported `url` crate. If the host contains `youtube.com` or `youtu.be`, call a new async helper `download_youtube_vod(video_link, output_root)` and return its result. Otherwise, fall through to the existing Twitch path unchanged. Implement `download_youtube_vod(url: &str, output_root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>>` as an async fn in the same file: (1) call `ytdlp::download_youtube(url, &audio_path)` where `audio_path` is a temporary path (use a temp file or the final artifact path — see note below); (2) extract `video_id` from `meta.id`; (3) build `artifact_dir = output_root.join(&video_id)`; (4) call `artifact::prepare_artifact_dir(&artifact_dir)`; (5) call `artifact::write_source_url(&artifact_dir, url)`; (6) move/verify `audio.m4a` is at `artifact_dir/audio.m4a` (yt-dlp was told to write to this path directly); (7) call `artifact::ArtifactMetadata::from_ytdlp(&video_id, url, &artifact_dir.join("audio.m4a"), &meta)`; (8) call `artifact::write_metadata(&artifact_dir, &metadata)`; (9) write `ProcessStatus` with `downloaded: true` via `artifact::write_status`. Note on output path: pass `artifact_dir.join("audio.m4a")` to `download_youtube` — yt-dlp will write directly there. The `artifact_dir` must be created via `prepare_artifact_dir` BEFORE calling `download_youtube` so the target directory exists. Adjust the order in `download_youtube_vod` accordingly: create dir first, then call download. `cargo build` must succeed and `cargo test` must pass. Verify end-to-end: `vod-pipeline download https://www.youtube.com/watch?v=jNQXAC9IVRw` should create `<output-root>/jNQXAC9IVRw/` with `audio.m4a`, `metadata.json`, `status.json`, `source_url.txt`.

## Inputs

- ``src/main.rs``
- ``src/ytdlp.rs``
- ``src/artifact.rs``

## Expected Output

- ``src/main.rs``

## Verification

cargo build 2>&1 | tail -5 && cargo test 2>&1 | tail -5

## Observability Impact

YouTube download errors (yt-dlp not found, network failure, format unavailable) propagate through the standard error chain and print to stderr. The resulting artifact directory contains `status.json` with `downloaded: true`, making the item visible in `vod-pipeline status`.
