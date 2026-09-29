# Evidence Lineage and Resealing Correction: `d11-cost-profile-1`

- **Date:** 2026-09-28 / 2026-09-29
- **Author:** Anti-Gravity (Lab 3: Numerical Fidelity, Codecs, Native Execution, and Measured Efficiency)
- **Target Report Root:** `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/d11-cost-profile-1`
- **Preserved Read-Only Snapshot:** `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/d11-cost-profile-1.snapshot-preserved`
- **Referenced PR:** #1479
- **Owning Issue:** #973 (under #820)
- **Status:** IMMUTABLE RECORD

---

## 1. Executive Summary & Incident Disclosure

During the review-response cycle for PR #1479, an in-place modification of the previously sealed report root `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/d11-cost-profile-1` occurred prior to commit `90641296`.

Under the project's evidence and sealing contract (`uor_r4_core::report_output`), report roots must be strictly immutable once sealed:
- Claimed exclusively before execution (`report_output::claim`),
- Written during the run,
- Sealed cryptographically with file hashes and timestamp (`report_output::seal`),
- Never altered or resealed in-place.

When reviewer feedback requested full binary and dataset SHA-256 identities, clarification of analytical memory estimates, and explicit labeling of input-driven forward throughput, the author edited `cost.json` and `evaluation.json` in-place and resealed `manifest.json` at unix timestamp `1790641817` rather than creating a distinct correction root or documenting the delta externally.

Per owner instruction, this record documents the forensic lineage, preserves the exact bytes as frozen, discloses the delta and disk limitations, and establishes the authentic provenance of the underlying run.

---

## 2. Report Root Snapshot & File Hashes

The directory `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/d11-cost-profile-1` has been preserved without further modification. An identical read-only snapshot was created at `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/d11-cost-profile-1.snapshot-preserved` (`chmod -R a-w`).

The exact files and cryptographic hashes of the preserved root are:

| File | Bytes | SHA-256 Hash | BLAKE3 Hash |
|:---|---:|:---|:---|
| `attempt.json` | 537 | `fafa825fa3d026234c39a23a8505c63dee476487d0323f9f669b51fad864700a` | `d25f6291dd7201e0227657f9f8e8c89902fa512eb7074f4fddfa584e10d5851f` |
| `cost.json` | 2,368 | `e3c54c56a27b77d5aa6c10ec61f8f6a31f175ad314950b4ceea03cea2e61337e` | `0657873e2a64e2499a83796fd770cb00d1b6aa701a84994f1019c7565cce0e2d` |
| `evaluation.json` | 2,368 | `e3c54c56a27b77d5aa6c10ec61f8f6a31f175ad314950b4ceea03cea2e61337e` | `0657873e2a64e2499a83796fd770cb00d1b6aa701a84994f1019c7565cce0e2d` |
| `manifest.json` | 610 | `fb36375b642d669a71470f6a909b958f4f85454527c6fe870e94a1dca1572ece` | (Self-manifest) |

---

## 3. Forensic Delta Analysis

### 3.1 Initial Run (Commit `96518458`)
The execution was launched on 2026-09-28:
- Command: `/Volumes/UOR-Workspace/BuildCaches/anti-gravity/release/uor-r4-stack cost /Volumes/UOR-Workspace/uor-r4-lab/claude-s2-dialogue-baseline/export-1/model.lut /Volumes/UOR-Workspace/uor-r4-lab/chat-v0-20260925/prepared/heldout/tokens.u16 8 64 /Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/d11-cost-profile-1`
- `attempt.json`: Claimed at unix timestamp `1790638316`, PID 38199.
- `cost.json` was generated with the initial schema in `crates/uor-r4-integer/src/bin/uor-r4-stack.rs` (commit `96518458`):
  - Contained raw numerical outputs: cold load (16.34 ms), prompt ingestion (5.26 ms/token), forward step latency distribution (p50: 5.40 ms, mean: 5.42 ms, p90: 5.75 ms, p99: 6.72 ms, max: 15.47 ms, throughput: 184.58 tok/s), and memory metrics (`peak_rss_mb`: 49.21875, `bytes_touched_per_token`: 12,373,333).
  - Lacked explicit SHA-256 fields for the binary and dataset.
  - Lacked qualitative disambiguation notes separating analytical calculations from hardware measurements.
- `manifest.json` was sealed immediately following completion.

### 3.2 Resealing Event (Pre-Commit `90641296`)
To satisfy oversight comments, the following fields were added to `cost.json` and `evaluation.json`:
1. `binary`: Added object containing path and `sha256`: `854d73368975ea66397745be31c9d33c35bbb3d904f9c5b3e718384df53c30aa`.
2. `tokens.sha256`: Added dataset hash `ca5ccf0722d9dcdc5294b612f2c25536fe49192be7bedbc3007a1fcfc0940225`.
3. `memory.post_eval_rss_mb`: Added explicit copy of the single post-evaluation sample (49.21875 MiB).
4. `memory.rss_notes`: Annotated that RSS is a single post-evaluation `ps` sample, not a continuous high-water mark.
5. `memory.analytical_bytes_touched_per_token_estimate`: Clarified that 12,373,333 is a shape-based calculation.
6. `memory.traffic_notes`: Documented that hardware memory bus traffic is unmeasured / unavailable.
7. `memory.analytical_weights_read_per_token`: Added explicit analytical weight count label (7,238,304).
8. `step_latency.input_driven_forward_throughput_tokens_per_second`: Labeled forward stepping mode.
9. `step_latency.throughput_mode`: Annotated teacher-forced stepping over dataset tokens, not autoregressive generation.
10. `step_latency.threads`: Explicitly recorded 1 thread.

After adding these annotations, `manifest.json` was re-sealed at unix timestamp `1790641817` (`sealed_at: 1790641817`).

### 3.3 Evaluation Payload Overwrite Disclosure
In `d11-cost-profile-1`, `cost.json` and `evaluation.json` are byte-identical (both 2,368 bytes, SHA-256 `e3c54c56a27b77d5aa6c10ec61f8f6a31f175ad314950b4ceea03cea2e61337e`, schema `uor-r4.stack-d11-cost/1`).
The CLI command `uor-r4-stack cost` wrote the same cost record to both `cost.json` and `evaluation.json` before sealing.
Therefore, `evaluation.json` in this preserved root **no longer carries separate evaluation evidence** (such as token cross-entropy or top-1 predictions); it is an exact duplicate of the cost profile record.
The authentic evaluation evidence resides in the original S2 evaluation roots (`s2-d11-release-baseline-1` and `s2-d11-release-optimized-1`), and the fresh gating re-run by Lab 1 will write to a newly claimed, distinct report root.

---

## 4. Lineage Limitations & Scientific Verification

1. **Annotated Report Status & Pre-Reseal Disk Bytes:**
   The resealed report root `d11-cost-profile-1` is an **annotated report**, not the raw immutable run record. Because `cost.json` was overwritten in-place on the external SSD without preserving the initial raw file bytes, the original pre-reseal file bytes and their exact BLAKE3 hash cannot be read from disk.
2. **Withdrawal of Prior Raw-Byte Bitwise Identity Assertions:**
   We formally **withdraw any claim that git commit history or source tracking verifies bitwise identity of the unrecoverable pre-reseal raw disk bytes**.
   What can be verified is strictly that the surviving markdown documentation committed at `96518458` recorded identical numerical figures for the benchmark metrics:
   - `cold_load.ms` (16.344875),
   - `prompt_ingestion.tokens_per_second` (190.068876),
   - `step_latency.min_ms` (4.8115), `p50_ms` (5.396583), `mean_ms` (5.417354), `p90_ms` (5.748625), `p99_ms` (6.724542), `max_ms` (15.4695), `tokens_per_second` (184.583702),
   - `memory.peak_rss_mb` / `post_eval_rss_mb` (49.21875),
   - `memory.weights_read_per_token` (7,238,304),
   - `memory.bytes_touched_per_token` (12,373,333).
   However, because original disk bytes were overwritten, raw byte identity is unprovable. The frozen snapshot `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/d11-cost-profile-1.snapshot-preserved` preserves the exact post-reseal bytes under write protection (`chmod -R a-w`).
3. **Scope of Test Oracles vs. Model Benchmarks:**
   The `stack_d11_oracle` test suite verifies synthetic stack fixtures, numerical corner cases, and error-handling wrap semantics in the release binary; it is an integration and bounds unit test, **not an evaluation of the 2,048-target S2 model benchmark**. Unit adapter tests (`d4_map_codec_adapter`) test arithmetic and round-trip mechanics on synthetic matrices, which does not constitute proof of trained model fidelity on full evaluation sets.
4. **Binary Hash Lineage:**
   Binary hashes (such as `854d7336...`) are bound strictly to contemporaneous builds from specific source commits; any rebuild or compiler version change produces a distinct binary hash that must be cited independently.
5. **Standing Rule for Future Evidence:**
   In compliance with progress-control rules and D9, no sealed report root may ever be edited or resealed in-place. All future measurements, including schema extensions or qualitative annotations, must be written to fresh, distinct, uniquely claimed report roots.
