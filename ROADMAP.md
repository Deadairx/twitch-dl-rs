# Development Roadmap

## Overview
This document outlines the planned steps and module responsibilities for building the `twitch-dl-rs` CLI tool.

---

## 1. CLI Argument Parsing
- Use `clap` to parse commands and options.
- Support the `download` subcommand and the `--auth-token` option.
- File: `src/cli.rs`

## 2. Validate and Process the Video Link
- Ensure the provided link is a valid Twitch video URL.
- Extract the video ID or relevant identifier.
- File: `src/twitch.rs`

## 3. Fetch Video Information
- Use Twitch's public API (or scrape if necessary) to get the video's playlist (m3u8) URL.
- If `--auth-token` is provided, use it for authenticated requests (for subscriber-only VODs).
- File: `src/twitch.rs`

## 4. Download the Video Segments
- Download the video using the m3u8 playlist (HLS stream).
- Optionally, use `ffmpeg` to download and convert the stream directly to mp4.
- File: `src/downloader.rs`, `src/ffmpeg.rs`

## 5. Handle Errors Gracefully
- Avoid panics and unwraps; use proper error handling and user-friendly messages.
- File: `src/error.rs`

## 6. Scaffold for Extensibility
- Structure the code so that adding new commands or features is easy.
- Keep modules decoupled and testable.

---

## Module Responsibilities

- **main.rs**: CLI entry point, argument parsing, command dispatch.
- **cli.rs**: CLI argument definitions and parsing logic.
- **twitch.rs**: Twitch API interaction, video info extraction.
- **downloader.rs**: Download logic for m3u8/HLS streams.
- **ffmpeg.rs**: ffmpeg invocation and video conversion.
- **error.rs**: Custom error types and handling.

---

## Next Steps
- Implement Twitch video ID extraction and m3u8 URL retrieval.
- Implement download logic for m3u8 streams.
- Integrate ffmpeg for conversion to mp4.
- Add robust error handling and user feedback. 