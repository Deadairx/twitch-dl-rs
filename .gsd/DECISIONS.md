# Decisions Register

<!-- Append-only. Never edit or remove existing rows.
     To reverse a decision, add a new row that supersedes it.
     Read this file at the start of any planning or research phase. -->

| # | When | Scope | Decision | Choice | Rationale | Revisable? | Made By |
|---|------|-------|----------|--------|-----------|------------|---------|
| D001 | M001 | scope | Product framing | Reframe from a Twitch-only downloader into a queue-first, artifact-first media ingestion pipeline | The real workflow is media-to-transcript processing, not isolated downloading | No | agent |
| D002 | M001 | pattern | Core workflow model | Center the CLI on durable per-item artifact/job state | The user wants unattended progress, resume behavior, and clear failure visibility | No | agent |
| D003 | M001 | quality-attribute | Transcript default bar | Bias toward trustworthy transcript output over raw speed | Downstream notes and memory work depend on transcript trustworthiness | Yes — if a later hybrid strategy proves equally trustworthy | agent |
| D004 | M001 | convention | Cleanup behavior | Cleanup is explicit operator action via candidate review, not automatic deletion | Source deletion is destructive and should remain under manual control | No | agent |
| D005 | M001 | scope | Milestone split | M001 proves reliable intake-to-transcript; M002 adds notes and Ember; M003 adds source expansion and polish | This sequences trust-critical backbone work before downstream and expansion work | Yes | agent |
| D006 |  | architecture | How to handle discrepancy between task summaries and actual slice delivery for S01 | Document the gap honestly in slice summary; record what was actually delivered vs claimed; mark incomplete work as blocking for downstream slices | Task summaries for T01/T02/T03 claim complex lifecycle types (JobLifecycleState, StageLifecycleState, etc.), three regression tests, and a status CLI command. Actual implementation uses simple boolean flags, zero tests, and no status command. Writing a dishonest slice summary would hide critical gaps (missing status CLI and test coverage) from downstream agents and from the human planner. Documenting the truth in the summary allows S02 to proceed using the durable queue/status files while flagging that status CLI is a critical blocker for human UAT and milestone sign-off." | Yes | agent |
