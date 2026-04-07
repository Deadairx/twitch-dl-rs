---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T02: Implement src/ytdlp.rs with two-pass yt-dlp invocation and add from_ytdlp to ArtifactMetadata

Create `src/ytdlp.rs` containing: (1) `YtDlpMetadata` struct with fields `id: String`, `title: String`, `channel: String`, `uploaded_at: String`; (2) `pub fn download_youtube(url: &str, output_path: &Path) -> Result<YtDlpMetadata, Box<dyn std::error::Error>>` using two-pass invocation. Pass 1: `yt-dlp --dump-json --skip-download <url>` — parse stdout as JSON, extract `id`, `title`, `channel` (prefer the `channel` field, fall back to `uploader`), and `upload_date`. Normalize `upload_date` from YYYYMMDD to YYYY-MM-DD. Pass 2: `yt-dlp -x --audio-format m4a --no-part -o <output_path> <url>` — download audio to the specified path. If `yt-dlp` is not found in PATH (i.e., `std::io::ErrorKind::NotFound` from `Command::new`), return a clear error: `"yt-dlp not found; install with: brew install yt-dlp"`. Also register the module in `src/lib.rs` with `pub mod ytdlp;`. Finally, add `ArtifactMetadata::from_ytdlp(video_id: &str, source_url: &str, output_file: &Path, meta: &YtDlpMetadata) -> Result<Self, std::io::Error>` to `src/artifact.rs` — same schema as `from_download` but with `stream_name: None`, `selected_bandwidth: None`, `selected_resolution: None`, `selected_codecs: None`, `is_audio_only: true`, `used_auth_token: false`. Write unit tests in `src/ytdlp.rs` for: date normalization (`20050424` → `2005-04-24`), and the not-found error message containing `brew install yt-dlp`.

## Inputs

- ``src/artifact.rs``
- ``src/lib.rs``

## Expected Output

- ``src/ytdlp.rs``
- ``src/lib.rs``
- ``src/artifact.rs``

## Verification

cargo test 2>&1 | tail -10

## Observability Impact

yt-dlp subprocess errors (not found, non-zero exit, malformed JSON) return descriptive error strings that propagate to the caller and are printed to stderr by main.rs's error handling path.
