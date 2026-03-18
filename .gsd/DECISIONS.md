# Decisions Register

<!-- Append-only. Never edit or remove existing rows.
     To reverse a decision, add a new row that supersedes it.
     Read this file at the start of any planning or research phase. -->

| # | When | Scope | Decision | Choice | Rationale | Revisable? |
|---|------|-------|----------|--------|-----------|------------|
| D001 | M001 | scope | Product framing | Reframe from a Twitch-only downloader into a queue-first, artifact-first media ingestion pipeline | The real workflow is media-to-transcript processing, not isolated downloading | No |
| D002 | M001 | pattern | Core workflow model | Center the CLI on durable per-item artifact/job state | The user wants unattended progress, resume behavior, and clear failure visibility | No |
| D003 | M001 | quality-attribute | Transcript default bar | Bias toward trustworthy transcript output over raw speed | Downstream notes and memory work depend on transcript trustworthiness | Yes — if a later hybrid strategy proves equally trustworthy |
| D004 | M001 | convention | Cleanup behavior | Cleanup is explicit operator action via candidate review, not automatic deletion | Source deletion is destructive and should remain under manual control | No |
| D005 | M001 | scope | Milestone split | M001 proves reliable intake-to-transcript; M002 adds notes and Ember; M003 adds source expansion and polish | This sequences trust-critical backbone work before downstream and expansion work | Yes |
