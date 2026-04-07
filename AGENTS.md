# AGENTS

`vod-pipeline` is a queue-first, artifact-first CLI for ingesting Twitch VODs into durable local artifacts, transcribing them with `hear`, and preparing trustworthy transcript outputs for later notes and memory workflows.

Use this file as a fast routing map. Prefer linked docs for planning context and linked code for implementation details.

- If the task is about roadmap, milestone sequencing, or future scope, read `ROADMAP.md` first.
- If the task is about durable product constraints or artifact shape, read `docs/planning/PRINCIPLES.md`.
- If the task is about future milestone breakdowns or ordering, read `docs/planning/MILESTONES.md`.
- If the task is exploratory or ambiguous, check `docs/planning/OPEN-QUESTIONS.md` for unresolved design choices.
- If the task references recent operator feedback or planning discussion, read `docs/planning/NOTES/2026-04-06-operator-feedback.md`.

- If the task is about command behavior or CLI flags, open `src/cli.rs` and `src/main.rs`.
- If the task is about artifact layout, metadata, queue files, or `status.json`, open `src/artifact.rs`.
- If the task is about transcription behavior, subtitle artifacts, or quality heuristics, open `src/transcribe.rs`.
- If the task is about Twitch discovery, VOD metadata, or queue inputs, open `src/twitch.rs`.
- If the task is about media download or assembly, open `src/downloader.rs` and `src/ffmpeg.rs`.

- If the task is about install/config behavior, check `README.md`, `Cargo.toml`, and `src/cli.rs`.
- If the task is about current runtime orchestration, start in `src/main.rs`.

- Do not treat `.gsd/` as the default planning entrypoint unless explicitly asked; root docs are the intended agent-facing planning surface.

## GSD workarounds
Before calling `gsd_complete_slice`: Generate the JSON first, check it against jq, make any adjustments to make it valid, and then send it to gsd_complete_slice once you're sure it's valid
