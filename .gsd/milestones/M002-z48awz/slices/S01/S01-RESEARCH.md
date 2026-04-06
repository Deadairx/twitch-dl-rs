# S01 — Metadata Durability: Research

**Date:** 2026-04-06

## Summary

S01's job is to extend `ArtifactMetadata` with `title`, `channel`, and `uploaded_at` fields, then thread the `VodEntry` context through every code path that creates a `metadata.json`. This is straightforward schema evolution — the data already exists in `VodEntry` and just isn't being written to the artifact.

`VodEntry` in `src/twitch.rs` already carries `title`, `channel`, `uploaded_at`, and `duration`. The problem is that `ArtifactMetadata::from_download` in `src/artifact.rs` takes only `video_id`, `source_url`, stream info, and auth flag — it discards the `VodEntry` context entirely. The fix is to add the three display fields to `ArtifactMetadata` and thread the `VodEntry` (or equivalent values) through the two call sites that construct it.

The `download` command (direct URL) is the harder path: it has no `VodEntry` at all. It calls `twitch::extract_video_id` and `fetch_vod_access_token` but never fetches video metadata. That path needs a new Twitch GQL call to retrieve title/channel/uploaded_at by video ID, or these fields must be left as `Option<String>` with `serde(default)` so old artifacts and direct-download artifacts both deserialize cleanly.

The `download_vod_to_artifact` helper (used by `download-all` and `process_channel`) already has a `VodEntry` in scope, so those paths are trivially fixed once the schema is extended. The `download_vod` function (used by the bare `download` command and as the inner download within `download_vod_to_artifact`) is the one that constructs `ArtifactMetadata::from_download` without access to `VodEntry` metadata.

## Recommendation

Add `title: Option<String>`, `channel: Option<String>`, and `uploaded_at: Option<String>` to `ArtifactMetadata` with `#[serde(default)]`. Extend `ArtifactMetadata::from_download` to accept an optional `VodEntry` reference (or the three fields directly). In the direct-`download` path, add a `fetch_vod_metadata_by_id` call to twitch.rs that returns title/channel/uploaded_at from the Twitch GQL API so the field is always populated when downloading via URL. In the queue-backed paths (`download_vod_to_artifact`), populate from `VodEntry` directly — no extra API call needed.

The `write_metadata` / `read_metadata` contract does not change for consumers. `scan_artifact_statuses` does not need changes — it reads `status.json`, not `metadata.json`. `show_status` (S02) will later need to read `metadata.json` per artifact, but that's out of S01 scope.

Old `metadata.json` files without the new fields will deserialize without error because `ArtifactMetadata` only uses `Serialize` currently, not `Deserialize`. If a `read_metadata` function is added (not currently present), it would need `#[serde(default)]` on new fields. Since metadata.json is currently write-only, no existing deserialization breaks.

## Implementation Landscape

### Key Files

- `src/artifact.rs` — `ArtifactMetadata` struct needs three new `Option<String>` fields (`title`, `channel`, `uploaded_at`). `ArtifactMetadata::from_download` signature must change to accept vod context. `write_metadata` is unchanged. A `read_metadata` function should be added here for S02 to use later (reading `metadata.json` per artifact for status display).
- `src/twitch.rs` — Add `fetch_vod_metadata_by_id(video_id: &str) -> Result<VodMeta, TwitchError>` for the bare `download` path. `VodEntry` already has all needed fields; no schema change needed.
- `src/main.rs` — Two sites construct `ArtifactMetadata::from_download`:
  1. `download_vod()` — the inner function called by both the bare `download` command and by `download_vod_to_artifact`. This is where the direct-URL path gets vod metadata via the new GQL call.
  2. `download_vod_to_artifact()` — has `vod: &VodEntry` in scope; can pass title/channel/uploaded_at directly instead of re-fetching.
  The cleanest approach: add an optional `Option<&VodEntry>` parameter to `download_vod()`, or factor the metadata fetch out of `download_vod` and let callers pass it in.

### Build Order

1. **Extend `ArtifactMetadata`** — add the three fields to the struct with `#[serde(default)]`, update `from_download` signature to accept optional vod context. Add backward-compat unit test confirming old metadata.json without new fields doesn't panic (if `Deserialize` is derived; otherwise add it now for the test).
2. **Add `fetch_vod_metadata_by_id` to `twitch.rs`** — single GQL query for `id`, `title`, `publishedAt`, `login` (channel). This is the only new external call.
3. **Wire up `download_vod` in `main.rs`** — after fetching the access token (where we already have video_id), call `fetch_vod_metadata_by_id`. Pass the result into `ArtifactMetadata::from_download`.
4. **Wire up `download_vod_to_artifact`** — pass `VodEntry` fields directly into `from_download`, skipping the GQL call since we already have the data.
5. **Add `read_metadata` to `artifact.rs`** — add `Deserialize` to `ArtifactMetadata` and a `read_metadata(artifact_dir)` function returning `Result<Option<ArtifactMetadata>>`. S02 will need this; better to land it in S01 while the schema is being touched.
6. **Unit tests** — backward compat test for old metadata.json (no new fields), roundtrip test for metadata with all fields populated.

### Verification Approach

- `cargo test` passes with new unit tests.
- `cargo build` succeeds — no compile errors.
- Manual: run `vod-pipeline download <twitch-url> --output-root /tmp/test-artifacts` and inspect the resulting `metadata.json` for `title`, `channel`, `uploaded_at` fields.
- Manual: verify an old-format `metadata.json` (missing the new fields) deserializes without error if `read_metadata` is used.

## Constraints

- All new `ArtifactMetadata` fields must be `Option<String>` with `#[serde(default)]` — old artifacts on disk must load without error.
- `ArtifactMetadata` currently derives only `Serialize`. Adding `Deserialize` is required for `read_metadata` and for backward-compat unit tests. This is safe since the struct is self-contained.
- The Twitch GQL call in `fetch_vod_metadata_by_id` should use the same unauthenticated client and Client-ID as the rest of `twitch.rs`. Public video metadata doesn't require auth.
- `hear` invocation and `ffmpeg` invocation are not touched. `transcribe.rs` is not touched.
- `ProcessStatus` / `status.json` is not touched — metadata.json is the home for display fields (D013).

## Common Pitfalls

- **`download_vod` is called from two places** — the bare `download` command path and from `download_vod_to_artifact`. The GQL fetch for vod metadata should only happen in the bare `download` path. In `download_vod_to_artifact`, pass the fields from `VodEntry` directly. The simplest approach is to thread an `Option<(&str, &str, &str)>` (title, channel, uploaded_at) into `download_vod` so callers can supply it or let it be fetched.
- **`ArtifactMetadata` only derives `Serialize` today** — adding `Deserialize` won't break anything but must be done before writing `read_metadata` or backward-compat tests that deserialize.
- **GQL API structure for single-video metadata** — the existing `fetch_channel_archive_vods` query operates on `user.videos`; single-video lookup uses `video(id: "...")` or the `videos` query with an `id` filter. A simpler approach: the GQL `video` top-level query: `{ video(id: "123456") { title publishedAt owner { login } } }`. This is undocumented but matches patterns in the existing codebase's unauthenticated Client-ID usage.
