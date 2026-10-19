# BVG-K Stage T Learning-Law, Per-Family `BlockCursor`/`BlockReceipt` Schedule & `decode-bench` Consumer Verifier (19 October 2026)

- **Line:** `m1-bvgk-staget-and-decode-bench-consumer-verifier` — **Count:** 0/3 (verification tooling; does not count against any 3-negative mechanism line per D21) — **Headline:** `1.20` (`#2029` / `#2028`, unchanged)
- **Deliverable:** [`verify_staget_and_bvc_contracts.py`](verify_staget_and_bvc_contracts.py) (`012e6a74b9644f3446d3bb2fc66240a83142da039bf3f8b6be1a65e515c84e2f`, `93,457` B, `2,149` lines; `32/32` positive and negative self-tests `PASS`) — host-only consumer verifier for:
  1. [`docs/harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md`](../../harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md) (`0d2775e1aa197f0cff577f01f00fc80ae9072cd7cbe11bb79558e35fb0f32e32`, `9,884` B, `128` lines, `#2731` / `#2029` comment `6257754797`)
  2. [`docs/harness/prep/m1-batched-verified-coordinates-DRAFT-2026-10-18.md`](../../harness/prep/m1-batched-verified-coordinates-DRAFT-2026-10-18.md) (`c9f49291e57010769ec94678a95b29170c781947b756f9143086370616908e19`, `15,181` B, `153` lines, `#2696` / `#2715`)
  3. [`docs/harness/prep/m1-decode-tokens-per-second-DRAFT-2026-10-18.md`](../../harness/prep/m1-decode-tokens-per-second-DRAFT-2026-10-18.md) (`5b6e4cdcf30cbe5252af47afd926757bc4aca59ec08ccf3cf747ada729440527`, `4,118` B, `73` lines, `#2686`)
  4. [`crates/uor-r4-discrete/src/training.rs`](../../../crates/uor-r4-discrete/src/training.rs) (`280f4b9533f6686c87a3f8bc5e18ff839c7fc6cea3e39817d86c47c718e110f3`, `126,242` B, `2,969` lines, `#2724`)
- **Scope & Holds:** Host-only (`cuda_context_created: false`, `model_constructed: false`, `training: "HELD"`, `$0.00` GPU/rental spend). Does not register any sheet, release any D4 training hold, or dispatch a pod.

---

## 1. Pinned Repository & Sheet Contracts Verified (`verify_repo_and_sheet_contracts`)

| File | SHA-256 | Bytes | Lines | Status / Verified Surface |
| :--- | :--- | ---: | ---: | :--- |
| [`docs/harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md`](../../harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md) | `0d2775e1aa197f0cff577f01f00fc80ae9072cd7cbe11bb79558e35fb0f32e32` | `9,884` | `128` | `DRAFT_PENDING` (`TEST FITNESS: PENDING` top & bottom consistent); all `13` required headings and all `5` `#2029` `6257754797` Stage-T draft repairs verified (`PASS`) |
| [`docs/harness/prep/m1-batched-verified-coordinates-DRAFT-2026-10-18.md`](../../harness/prep/m1-batched-verified-coordinates-DRAFT-2026-10-18.md) | `c9f49291e57010769ec94678a95b29170c781947b756f9143086370616908e19` | `15,181` | `153` | `DRAFT_PENDING` (`TEST FITNESS is not requested` / bottom `TEST FITNESS: PENDING`); all `14` required headings verified (`PASS`) |
| [`docs/harness/prep/m1-decode-tokens-per-second-DRAFT-2026-10-18.md`](../../harness/prep/m1-decode-tokens-per-second-DRAFT-2026-10-18.md) | `5b6e4cdcf30cbe5252af47afd926757bc4aca59ec08ccf3cf747ada729440527` | `4,118` | `73` | `DRAFT_PENDING` (`TEST FITNESS is not requested` / bottom `TEST FITNESS: PENDING`); all `10` required headings verified (`PASS`) |
| [`crates/uor-r4-discrete/src/training.rs`](../../../crates/uor-r4-discrete/src/training.rs) | `280f4b9533f6686c87a3f8bc5e18ff839c7fc6cea3e39817d86c47c718e110f3` | `126,242` | `2,969` | All `7` BVG-K symbols (`BlockCursor`, `BlockReceipt`, `family_cursors`, `block_at`, `advance_block`, `transition_block`, `run_block_transitions`) and all `11` exact refusal strings verified (`PASS`) |

### 1.1 Stage T Draft Repairs (`#2029` Comment `6257754797`, Verified in `0d2775e1…`)

1. **`<= 4`-Pod Wave Plan (`repair_1_wave_plan_within_4_pod_cap`):** Enforces `"Using 2 pods × 2 GPUs (4 slots, about $3.8/h)"` with **Wave 1** (`T1` and `T16` on seeds `1` and `2`, four arm-seeds at once) and **Wave 2** (optional `T4` on seeds `1` and `2`, on `1` pod), and rejects any `"3 pod-pairs"` or `"6 pods"` cap violation.
2. **Base-Lineage Per-Batch Time Explicitly Labeled `"not measured"` (`repair_2_base_batch_time_not_measured_scoped_c1_c2`):** Enforces that `"The base lineage's per-batch time is **not measured**"` and `"The 1.31 s/batch figure is the injective sheet's planning assumption, not a measurement"`, while scoping measured per-call latencies to the **K16 Route shape** (`C1`: `125.5 ms` at `K = 1` vs `123.6 ms` at `K = 16` scoring-only; `C2 TP`: `129.7 ms` at `K = 1` vs `259 ms` at `K = 16` including joint verify).
3. **Precondition 3 Two-Seed Dispersion Requirement (`repair_3_precondition_3_two_seed_dispersion`):** Enforces that the non-inferiority margin is fixed from a **measured two-seed dispersion** of the chosen base at matched wall clock (running a second-seed `T1` at its own cost line if the base has only one seed, or adopting a named measured two-seed dispersion from the parent lineage with the reason stated; never set from a single seed or after any Stage T result is visible).
4. **Equal-Wall-Clock `W` Protocol (`repair_4_equal_wall_clock_w_protocol`):** Enforces exact clock start/stop (starts at the first training batch after admission and checkpoint-0 evaluation; build, table load, and pre-flight excluded; DEV evaluations and checkpoint I/O **included** in every arm's own clock; each arm read at its last completed checkpoint `<= W`; base lineage Q1 stop mapped to `W/4` on the same axis).
5. **Quality Metrics & `KEEP` / Non-Inferiority Bars (`repair_5_quality_metrics_and_keep_bars`):** Enforces **Primary** = DEV `P_succ` conditional top-1, **Co-primary** = full DEV loss, **Reported, not gating** = top-8 and B3; `KEEP` requires primary top-1 better than `T1` beyond the margin **and** full DEV loss non-inferior on **both seeds** with fresh transfer reported (D22), and margin never narrower than the D22 §3 2-SE row floors (`20` rows for top-1, `63` for top-8, `87` for B3).

---

## 2. Per-Family `BlockCursor` / `BlockReceipt` Schedule & Commit-Rule Verifier (`verify_family_cursors_and_block_receipts`)

Replicates the exact integer schedule and cursor validation arithmetic of [`crates/uor-r4-discrete/src/training.rs`](../../../crates/uor-r4-discrete/src/training.rs) (`lines 321-391, 595-717, 2534-2553`):
- **`family_cursors(completed_batches, block, schedule, vectors, reads)` & `block_at(...)`:**
  - Computes `vector_blocks = ceil(vectors / K)` and `read_blocks = ceil(reads / K)`.
  - Under `InterleavedReadEvery8V1` (`interleaved_read_every8_v1`), batch `b` is a read batch iff `b % 8 == 7` (`vector_batches = b - b // 8`, `read_batches = b // 8`), preserving the exact `1 read : 7 vector` batch ratio for every `K`.
  - Under `CanonicalVectorThenReadV1` (`canonical_vector_then_read_v1`), each cycle runs `vector_blocks` vector batches followed by `read_blocks` read batches.
  - Each family ordinal advances as `(batches // blocks) * size + (batches % blocks) * K`, and each batch visits `start..min(size, start + K)` where `start = ordinal % size`, so **no block ever straddles a family pass boundary** (verified on `K = 1` identity over `200` steps, `K = 3` pass-boundary truncation `[18, 19]` and `[3]` on `vectors = 20, reads = 4`, and Dose 1 `K = 16` `vectors = 16,384, reads = 45` where the third read batch visits `13` gates `[32..44]`).
- **`Cursor` / `BlockCursor` & `BlockReceipt` Invariants:**
  - Enforces absence of `block` at `K = 1` (`"coordinate_block 1 cursor carries no block fields"`) and presence of `block = {o_vec, o_read, committed_moves}` at `K > 1` (`"coordinate_block > 1 cursor requires block fields"`).
  - Enforces `(block.o_vec, block.o_read) == family_cursors(completed_batches, K, ...)` (`"resume o_vec/o_read differ from the per-family block schedule"`) and `accepted_updates <= committed_moves <= K * accepted_updates` (`"resume committed_moves outside accepted_updates..=K*accepted_updates"`).
  - Enforces the `S_t` / `best` / joint-verify commit rule (`§2` step 3):
    - `|S_t| == 0`: `joint == false`, `l_joint` absent, `moves == 0`, `committed_loss == l_before`;
    - `|S_t| == 1`: `joint == false`, `l_joint` absent, `moves == 1`, `committed_loss == l_best < l_before`;
    - `|S_t| >= 2`: `l_joint` required; commits the full set `S_t` (`joint == true`, `moves == s_len`, `committed_loss == l_joint < l_best < l_before`) **if and only if `L(S_t) < L({best})` strictly**, and falls back to the single move `{best}` (`joint == false`, `moves == 1`, `committed_loss == l_best < l_before`) whenever `L(S_t) >= L({best})` (including ties `L(S_t) == L({best})`).
  - Under `FrozenZeroV1`, refuses any `BlockReceipt` that commits a `Gate` coordinate with `index >= combine_gate_count` (`20`).

---

## 3. Stage T Preconditions, First-Batch Controls & 2-Seed Decision Verifier (`verify_staget_bundle`)

1. **Preconditions 1–3 & Registration Interlocks:**
   - **Precondition 1:** Requires `c2_qualification_pass == True` bound to `#2029` comment `6257768576` with `5/5` C2 controls passing.
   - **Precondition 2:** Checks whether the base lineage passed both pre-flight (`b3_init_psucc >= 500`) and Q1 (`q1_passed == True`, `q1_admission_collapse == False`). If either failed (as on historical `P8` `127 < 500`, `rows_v1` Q1 stop, or `A8` `Relation` Q1 stop `551 < 675.5`), returns `"STAGE_T_HELD_PRECONDITION_2_BASE_LINEAGE_NOT_ADMITTED"` (`mechanism_negative: false`). Enforces the single-variable rule across `T1` (`K = 1`), `T16` (`K = 16`), and optional `T4` (`K = 4`) with `counterfactual_path == "rows_v1"` and `objective_policy == "nearest_address_margin_v1"`.
   - **Precondition 3:** Requires `fixed_before_results == True`, `source_seeds_count >= 2`, and D22 §3 2-SE floors (`margin_top1_rows >= 20`, `margin_top8_rows >= 63`, `margin_b3_rows >= 87`, `margin_full_dev_loss > 0`).
   - **Registration & D40 / Resource Caps:** Requires `sheet_verdict == "FIT"`, fresh-context adversarial reviewer subagent findings on `#2029`, `pm_dispatch_id`, per-arm Casey `D40` release (`uor_lab_env_unset == True`), `pods_concurrent <= 4`, `rate_usd_per_hr <= 8.0`, and `disk_free_gib >= 30.0`.
2. **Pre-Registered First-Batch Controls (`V1d`, `KId`, `V8d`, `V9d`, `AKd`):**
   - Verified before any Stage T outcome is read; any failure returns `"STAGE_T_BLOCKED_FIRST_BATCH_CONTROL_FAIL_D31"` (`results_readable: false`, `mechanism_negative: false`).
3. **Equal-Wall-Clock `W` & 2-Seed Decision Table:**
   - Verifies Dose 1 data split pins (`TRAIN` `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`, `DEV` tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `DEV` mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`, `14,287` DEV rows, `3,847` `P_succ` rows), last completed checkpoint `<= W`, and Q1 at `W/4`.
   - Emits `"KEEP_STAGE_T_ARM"` (or `"DEV_KEEP_PENDING_FRESH_TRANSFER_D22"` if fresh transfer is not yet reported) when both seeds beat `T1` on DEV `P_succ` top-1 beyond `margin_top1_rows` and are non-inferior on full DEV loss; `"NON_INFERIOR_THROUGHPUT_GAIN"` when both seeds are non-inferior on both primary metrics with higher `target_tokens_per_s`; and `"STAGE_T_ARM_NOT_KEPT"` (`configuration_closed: true`, `mechanism_negative: false` per D22) otherwise.

---

## 4. `decode-bench` Tokens/s Baseline & Candidate Kernel Verifier (`verify_decode_bench_bundle`)

Enforces the optimization rung step-2 instrument contract in [`docs/harness/prep/m1-decode-tokens-per-second-DRAFT-2026-10-18.md`](../../harness/prep/m1-decode-tokens-per-second-DRAFT-2026-10-18.md) (`5b6e4cdc…`):
- Separate release-profile binary (`binary_name == "decode-bench"`, `driver_identity_unchanged == True`, `release_profile == True`), GPU-only (`host_model_arithmetic == False`, `d11_integer_discrete == True`, `gpu_only == True`), DEV-only unscored prompts, and resource caps (`pods_concurrent <= 4`, `rate_usd_per_hr <= 8.0`, `disk_free_gib >= 30.0`).
- **3 Finite Controls (`D31`):** (1) `generated_ids_equal_across_repeats`, (2) `batched_stream0_equals_single_stream` (cross-checked via `stream0_tokens_sha256 == single_stream.generated_tokens_sha256` at every `(checkpoint, c)`), and (3) `sliding_window_c243_refuses_nothing`. Any control failure returns `"DECODE_BENCH_CONTROL_FAIL_D31"`.
- **24-Cell Grid:** `6` context lengths (`c in {16, 32, 64, 128, 192, 243}`) × `2` batch shapes (`single_stream` `rows = 1`, `batched_64` `rows = 64`) × `2` checkpoints (`treatment`, `frozen_zero`), each generating `N = 256` tokens over `5` timed repeats after `>= 1` warmup in integer nanoseconds (`carry_ns + forward_ns + host_id_move_ns == p50_ns`).
- **Candidate Kernel Comparison Mode:** Requires every cell in `candidate_cells` to match `generated_tokens_sha256` byte-for-byte (`"CANDIDATE_REJECTED_TOKEN_PARITY_MISMATCH"` on mismatch) and strictly improve `aggregate_tokens_per_s` (`"CANDIDATE_REJECTED_NO_THROUGHPUT_GAIN"` otherwise) before returning `"CANDIDATE_KERNEL_PASS"`.

---

## 5. Verification Command & Self-Test Summary (`32/32` PASS)

```sh
python3 docs/labs/staget-bvc-consumer-verifier-2026-10-19/verify_staget_and_bvc_contracts.py --verify-repo-contract --self-test
```

All `32/32` positive and negative self-tests (`[1/32]..[32/32]`) pass on host CPU in `< 0.2 s` with `$0` GPU/pod spend.
