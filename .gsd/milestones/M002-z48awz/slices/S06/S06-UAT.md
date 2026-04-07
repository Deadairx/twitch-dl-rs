# S06: Retry And Operational Hardening — UAT

**Milestone:** M002-z48awz
**Written:** 2026-04-07T04:36:07.867Z

# S06: UAT — Retry And Operational Hardening

**Objective:** Verify that `--force-suspect` enables operator-driven retry of suspect transcriptions and that concurrent writes to status.json do not corrupt artifact state.

**Prerequisites:**
- Working `vod-pipeline` binary built from source
- Test artifact directories with artifacts in various states
- Empty temporary directory for new test artifacts

---

## Test Suite 1: Force-Suspect Flag on Transcribe-All

### TC1.1: Help text displays --force-suspect flag

**Setup:** None (binary already built)

**Steps:**
1. Run: `cargo run --quiet -- transcribe-all --help 2>/dev/null | grep -A1 force-suspect`

**Expected Outcome:**
```
      --force-suspect              Re-transcribe artifacts that previously resulted in suspect transcriptions
```

**Verification:** Flag is visible with human-readable description.

---

### TC1.2: Suspect item is re-transcribed when --force-suspect is passed

**Setup:**
1. Create test artifact with suspect status (transcribed=false, outcome="suspect")
2. Place minimal audio file at artifact path

**Steps:**
1. Run force-suspect transcription
2. Verify transcription completed and status.json is updated

**Expected Outcome:**
- Suspect item was re-transcribed (not skipped); status.json reflects new outcome

---

### TC1.3: Suspect item is skipped when --force-suspect is NOT passed

**Setup:** Same artifact as TC1.2

**Steps:**
1. Run transcribe-all without --force-suspect flag

**Expected Outcome:**
- Output: "No artifacts pending transcription."

**Verification:** Suspect item excluded from run without flag.

---

### TC1.4: --force-suspect composes with --video-id for targeted retry

**Setup:** Multiple suspect artifacts

**Steps:**
1. Run: `transcribe-all --force-suspect --video-id <specific-id>`

**Expected Outcome:**
- Only specified artifact re-transcribed
- Other suspect artifacts unchanged

---

## Test Suite 2: File Locking and Concurrent Write Safety

### TC2.1: Concurrent writes do not corrupt status.json

**Steps:**
1. Spawn two transcribe-all processes writing to same artifact directory
2. Verify both complete without panic
3. Verify status.json is valid JSON

**Expected Outcome:**
- Lock serializes writes; no corruption

**Verification:** file deserializes successfully

---

### TC2.2: Lock file exists and is reused

**Steps:**
1. Run transcribe-all
2. Check for status.lock alongside status.json

**Expected Outcome:**
- status.lock file exists and is reused across invocations

---

### TC2.3: Single-process operation unaffected

**Steps:**
1. Run transcribe-all normally
2. Validate status.json before and after

**Expected Outcome:**
- Lock is transparent; no UX impact

---

## Test Suite 3: Integration

### TC3.1: Status shows updated outcome after force-suspect retry

**Steps:**
1. Note suspect item in status output
2. Run --force-suspect retry
3. Verify status shows new outcome (completed, suspect, or failed)

**Expected Outcome:**
- Status correctly reflects retry result

---

### TC3.2: Download and transcribe run concurrently without corruption

**Steps:**
1. Launch download-all and transcribe-all in parallel
2. Verify all status.json files are valid JSON

**Expected Outcome:**
- Concurrent writes safely serialize via lock

---

## Edge Cases

### EC1: --force-suspect with no suspect items
**Expected:** "No artifacts pending transcription." (exit 0)

### EC2: --force-suspect --video-id with wrong ID  
**Expected:** Error message "video ID X not found" (exit 1)

### EC3: Stale lock file from prior crash
**Expected:** Lock reused successfully; no cleanup needed

---

## Summary

All test cases verify --force-suspect retry works correctly, file locking prevents concurrent corruption, and integration points function as expected.
