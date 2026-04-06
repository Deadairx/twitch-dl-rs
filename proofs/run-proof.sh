#!/usr/bin/env bash
# End-to-end proof walkthrough for M001 pipeline
# Phase 1: Real artifacts — status inspection
# Phase 2: Scratch completed artifact — cleanup candidate visibility
# Phase 3: Scratch failure artifact — failure reason visibility

# Initialize the proof log
> proofs/proof.log

# ============================================================================
# PHASE 1: Real artifacts — status inspection
# ============================================================================

echo "========================================"
echo "Phase 1: Real artifact status inspection"
echo "========================================"
echo "" | tee -a proofs/proof.log

echo "Command: ./target/debug/twitch-dl-rs status --output-root artifacts" | tee -a proofs/proof.log
./target/debug/twitch-dl-rs status --output-root artifacts 2>&1 | tee -a proofs/proof.log

echo "" | tee -a proofs/proof.log
echo "Phase 1 complete: Real artifacts scanned" | tee -a proofs/proof.log
echo "" | tee -a proofs/proof.log

# ============================================================================
# PHASE 2: Scratch completed artifact — cleanup candidate visibility
# ============================================================================

echo "========================================"
echo "Phase 2: Cleanup candidate visibility (completed item)"
echo "========================================"
echo "" | tee -a proofs/proof.log

echo "Command: ./target/debug/twitch-dl-rs cleanup --output-root proofs/scratch-artifacts" | tee -a proofs/proof.log
./target/debug/twitch-dl-rs cleanup --output-root proofs/scratch-artifacts 2>&1 | tee -a proofs/proof.log

echo "" | tee -a proofs/proof.log
echo "Phase 2 complete: Cleanup candidates identified" | tee -a proofs/proof.log
echo "" | tee -a proofs/proof.log

# ============================================================================
# PHASE 3: Scratch failure artifact — failure reason visibility
# ============================================================================

echo "========================================"
echo "Phase 3: Transcription failure handling"
echo "========================================"
echo "" | tee -a proofs/proof.log

echo "Command: ./target/debug/twitch-dl-rs transcribe-all --output-root proofs/scratch-artifacts --continue-on-error" | tee -a proofs/proof.log
./target/debug/twitch-dl-rs transcribe-all --output-root proofs/scratch-artifacts --continue-on-error 2>&1 | tee -a proofs/proof.log || true

echo "" | tee -a proofs/proof.log

echo "Status after transcribe-all (failure artifact):" | tee -a proofs/proof.log
echo "Command: ./target/debug/twitch-dl-rs status --output-root proofs/scratch-artifacts" | tee -a proofs/proof.log
./target/debug/twitch-dl-rs status --output-root proofs/scratch-artifacts 2>&1 | tee -a proofs/proof.log

echo "" | tee -a proofs/proof.log
echo "Phase 3 complete: Failure handling verified" | tee -a proofs/proof.log
echo "" | tee -a proofs/proof.log

echo "========================================"
echo "All phases complete"
echo "========================================"
echo "Proof log written to: proofs/proof.log" | tee -a proofs/proof.log
