---
estimated_steps: 6
estimated_files: 4
skills_used: []
---

# T02: Wire vod context into download paths and add --skip-metadata flag

**Slice:** S01 — Metadata Durability
**Milestone:** M002-z48awz

## Description

T01 produced the schema changes and `fetch_vod_metadata_by_id`. This task wires them into the two download call paths and adds the `--skip-metadata` CLI flag. After this task, every artifact produced by `vod-pipeline download` will have `title`, `channel`, and `uploaded_at` in `metadata.json` and a `status.json` reflecting download state.

**Call path summary after this task:**

- `download <url>` → `download_vod(url, auth, output_root, quality, vod_context=None, skip_metadata=false)` → calls `fetch_vod_metadata_by_id` → passes context to `from_download` → writes `metadata.json` + `status.json`
- `download <url> --skip-metadata` → same but skips GQL fetch; display fields are `None`/absent in `metadata.json`; `status.json` still written
- `download_vod_to_artifact(vod, ...)` → calls `download_vod` with `vod_context=Some((&vod.title, &vod.channel, &vod.uploaded_at))` and `skip_metadata=false`; no extra GQL call

**Critical ordering constraint:** The GQL fetch must happen *before* `artifact::prepare_artifact_dir` so that a GQL failure leaves no partial artifact directory behind. In the current `download_vod` body, `prepare_artifact_dir` is called before `write_source_url`. Reorder so the GQL fetch (if applicable) happens first, then `prepare_artifact_dir`, then `write_source_url`, then download, then `write_metadata`, then `write_status`.

## Failure Modes

| Dependency | On error | On timeout | On malformed response |
|------------|----------|-----------|----------------------|
| `fetch_vod_metadata_by_id` (bare download, no `--skip-metadata`) | Print reason + suggest `--skip-metadata`; return error before creating artifact dir | Same — reqwest timeout surfaces as error | Same |

## Negative Tests

- **Error paths**: bare `download` with a GQL failure and no `--skip-metadata` must exit non-zero and leave no artifact directory
- **Boundary conditions**: `download_vod_to_artifact` must not call `fetch_vod_metadata_by_id` — it passes `VodEntry` fields directly

## Steps

1. **`src/cli.rs`**: Add `skip_metadata: bool` to the `CliCommand::Download` variant. Add `--skip-metadata` to the `download` subcommand:
   ```rust
   Arg::new("skip-metadata")
       .long("skip-metadata")
       .help("Skip fetching VOD title/channel/date from Twitch API; those fields will be absent in metadata.json")
       .action(ArgAction::SetTrue),
   ```
   In `parse_args`, thread `skip_metadata: download_matches.get_flag("skip-metadata")` into the `CliCommand::Download` struct.

2. **`src/main.rs`** — update `download_vod` signature to:
   ```rust
   async fn download_vod(
       video_link: &str,
       auth_token: Option<&str>,
       output_root: &std::path::Path,
       quality: cli::QualityPreference,
       vod_context: Option<(&str, &str, &str)>,
       skip_metadata: bool,
   ) -> Result<std::path::PathBuf, Box<dyn std::error::Error>>
   ```

3. **In `download_vod` body**, after extracting `video_id` and resolving `auth_token`, but before `prepare_artifact_dir`:
   - Resolve vod context: if `vod_context.is_some()`, use it directly. If `vod_context.is_none() && !skip_metadata`, call `twitch::fetch_vod_metadata_by_id(&video_id).await`. On error, print the failure reason and the `--skip-metadata` suggestion to stderr, then propagate the error (returning before `prepare_artifact_dir`).
   - Store the resolved `(title, channel, uploaded_at)` as owned `String` values in locals so references live long enough. Pattern:
     ```rust
     let (title_opt, channel_opt, uploaded_at_opt): (Option<String>, Option<String>, Option<String>) = ...;
     let ctx: Option<(&str, &str, &str)> = match (&title_opt, &channel_opt, &uploaded_at_opt) {
         (Some(t), Some(c), Some(u)) => Some((t.as_str(), c.as_str(), u.as_str())),
         _ => None,
     };
     ```
   - If `skip_metadata`, the resolved context is `None`.

4. Pass `ctx` to `artifact::ArtifactMetadata::from_download(...)` as the last argument.

5. After `artifact::write_metadata(...)`, write `status.json` for the bare download path:
   ```rust
   let mut status = artifact::ProcessStatus::new(&video_id, video_link);
   status.downloaded = true;
   status.media_file = Some(output_name.to_string());
   artifact::write_status(&artifact_dir, &status)?;
   ```
   Note: `download_vod_to_artifact` overwrites this `status.json` with its own write after the call returns — that is correct. The key is that the bare `download` command now also writes one.

6. Update all callers of `download_vod`:
   - In the `Download` command handler in `main()`: pass `vod_context: None` and `skip_metadata` from the CLI struct. The GQL fetch happens inside `download_vod` when `vod_context` is `None` and `skip_metadata` is `false`.
   - In `download_vod_to_artifact`: pass `vod_context: Some((vod.title.as_str(), vod.channel.as_str(), vod.uploaded_at.as_str()))` and `skip_metadata: false`.

## Must-Haves

- [ ] `--skip-metadata` flag exists in `cli.rs` for the `download` subcommand and is plumbed into `CliCommand::Download`
- [ ] `download_vod` accepts `vod_context: Option<(&str, &str, &str)>` and `skip_metadata: bool`
- [ ] GQL fetch happens before `prepare_artifact_dir` — no partial artifact directory on GQL failure
- [ ] GQL failure without `--skip-metadata`: exits non-zero with message containing failure reason and `--skip-metadata` suggestion
- [ ] `download_vod_to_artifact` passes `VodEntry` fields as `vod_context` — no extra GQL call made
- [ ] Bare `download` path writes `status.json` with `downloaded: true` after successful media download
- [ ] `cargo build` succeeds with no errors
- [ ] `cargo test` passes

## Verification

- `cargo build 2>&1 | grep '^error'` — must produce no output
- `cargo test 2>&1 | grep -E 'test result|FAILED'` — must show `test result: ok` with no FAILED lines
- `./target/debug/vod-pipeline download --help 2>&1 | grep skip-metadata` — must output the flag

## Inputs

- `src/artifact.rs` — updated `ArtifactMetadata::from_download` signature (from T01 — now accepts `Option<(&str, &str, &str)>`); `read_metadata`; `write_status`; `ProcessStatus`
- `src/twitch.rs` — `fetch_vod_metadata_by_id` (from T01)
- `src/main.rs` — `download_vod` and `download_vod_to_artifact` to update; current `main()` Download handler
- `src/cli.rs` — `CliCommand::Download` variant and `parse_args` download subcommand to extend

## Expected Output

- `src/main.rs` — `download_vod` updated with new parameters, GQL fetch logic, and `status.json` write; `download_vod_to_artifact` passes `VodEntry` context; bare `Download` handler passes `skip_metadata`
- `src/cli.rs` — `CliCommand::Download` has `skip_metadata: bool`; `download` subcommand has `--skip-metadata` flag
