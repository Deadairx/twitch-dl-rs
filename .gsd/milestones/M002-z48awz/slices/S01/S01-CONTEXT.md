---
id: S01
milestone: M002-z48awz
status: ready
---

# S01: Metadata Durability — Context

## Goal

Extend `ArtifactMetadata` with `title`, `channel`, and `uploaded_at` fields and thread them through every code path that writes `metadata.json`, so every new artifact carries human-readable context and old artifacts load cleanly.

## Why this Slice

S01 is the foundation that S02 (Status Legibility) depends on directly. S02 needs to read `metadata.json` per artifact to populate the title, date, and channel columns in the status table — that read path (`read_metadata`) must exist before S02 can be planned or executed. Doing the schema work first avoids retrofitting it under time pressure when S02 is in flight.

## Scope

### In Scope

- Add `title: Option<String>`, `channel: Option<String>`, `uploaded_at: Option<String>` to `ArtifactMetadata` with `#[serde(default)]`
- Add `Deserialize` to `ArtifactMetadata` so old artifacts can be deserialized without error
- Add `read_metadata(artifact_dir)` function to `artifact.rs` returning `Result<Option<ArtifactMetadata>>`
- Update `ArtifactMetadata::from_download` to accept optional vod context (title, channel, uploaded_at)
- In the queue-backed path (`download_vod_to_artifact`), populate from `VodEntry` directly — no extra API call
- In the bare `download <url>` path, add a new `fetch_vod_metadata_by_id` GQL call to retrieve title/channel/uploaded_at by video ID
- **The GQL metadata fetch in the bare `download` path is a hard requirement** — if it fails, the download aborts with a clear error message stating the reason and suggesting the operator retry or use `--skip-metadata`
- Add `--skip-metadata` flag to the bare `download` command — when passed, the GQL fetch is skipped entirely, those fields are null/absent in `metadata.json`, and the download proceeds
- **Write `status.json` for bare `download <url>` artifacts** — if media exists after download, status is recorded as `downloaded`
- Unit tests: backward-compat deserialization of old metadata.json, full-field roundtrip

### Out of Scope

- `status.json` schema changes beyond the bare-download normalization above — ProcessStatus variants are untouched
- Status command display changes — that is S02
- `queue-video` command — that is S03
- Any changes to `download-all`, `transcribe-all`, or `transcribe` command behavior
- Non-Twitch source support — that is S07
- `hear` or `ffmpeg` invocation changes

## Constraints

- All new `ArtifactMetadata` fields must be `Option<String>` with `#[serde(default)]` — old artifacts on disk must deserialize cleanly without migration
- The GQL call in `fetch_vod_metadata_by_id` must use the same unauthenticated client and `Client-ID` as the rest of `twitch.rs` — public video metadata does not require auth
- `hear` invocation is not touched; `transcribe.rs` is not touched
- `ProcessStatus` / `status.json` is the home for stage state (D013); display fields live in `metadata.json` only
- When `--skip-metadata` is passed, null/absent fields in `metadata.json` are correct and expected — no placeholder strings

## Error UX

When the bare `download` GQL fetch fails and no `--skip-metadata` flag was passed:
- Print the failure reason (e.g. `failed to fetch VOD metadata: <error>`)
- Suggest: re-run with `--skip-metadata` to download without metadata, or check network and retry
- Exit non-zero; no partial artifact directory left behind

## Integration Points

### Consumes

- `src/twitch.rs` — `VodEntry` struct (already has `title`, `channel`, `uploaded_at`); existing unauthenticated GQL client pattern for the new `fetch_vod_metadata_by_id` function
- `src/artifact.rs` — `ArtifactMetadata::from_download`, `write_metadata` (both get updated, not replaced)
- `src/main.rs` — `download_vod` and `download_vod_to_artifact` call sites (both updated to pass vod context)

### Produces

- `src/artifact.rs` — updated `ArtifactMetadata` struct (three new fields, `Deserialize` derived), updated `from_download` signature, new `read_metadata` function
- `src/twitch.rs` — new `pub async fn fetch_vod_metadata_by_id(video_id: &str) -> Result<(String, String, String), TwitchError>`
- `src/main.rs` — `download_vod` updated to call `fetch_vod_metadata_by_id` when no vod context is available; aborts on failure unless `--skip-metadata`; writes `status.json` with `downloaded` state after successful bare download; `download_vod_to_artifact` passes `VodEntry` fields directly
- `src/cli.rs` — `--skip-metadata` flag added to the `download` subcommand

## Open Questions

- None. All decisions resolved in discussion.
