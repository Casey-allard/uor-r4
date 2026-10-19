# Distill-D1 Donor Corpus Host-Only Consumer Verifier (October 19, 2026)

- **Lab:** Antigravity (`UOR_LAB=antigravity`, session `ag1`)
- **Milestone:** M1 Grounded Conversation and Durable Memory ([#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029))
- **Governing Decision:** [D41 — One-Time Offline Donor-Model Generation for the `distill-d1` Corpus Lineage (`openai/gpt-oss-120b`), Scoped and Expiring](../../integration/DECISIONS.md#d41--one-time-offline-donor-model-generation-for-the-distill-d1-corpus-lineage-openaigpt-oss-120b-scoped-and-expiring)
- **Registration Sheet:** [`docs/harness/prep/m1-distill-d1-donor-corpus-DRAFT-2026-10-19.md`](../../harness/prep/m1-distill-d1-donor-corpus-DRAFT-2026-10-19.md) (drafted in `#2765` `4d3ae4f6b`, prior main tip `30e7445d3` `#2773`; advanced to `TEST FITNESS: FIT` in `#2774` `bc74e5970`)
- **Compute / Cost:** Host-only (`$0.00` GPU, `0` pods, `0` GPU hours). No model execution, no pod dispatch, and no hold release.

---

## 1. Purpose and Scope

Under [D41](../../integration/DECISIONS.md#d41--one-time-offline-donor-model-generation-for-the-distill-d1-corpus-lineage-openaigpt-oss-120b-scoped-and-expiring) and the `TEST FITNESS: FIT` registration sheet [`docs/harness/prep/m1-distill-d1-donor-corpus-DRAFT-2026-10-19.md`](../../harness/prep/m1-distill-d1-donor-corpus-DRAFT-2026-10-19.md), the `distill-d1` offline donor corpus (`25,000,000`–`35,000,000` student tokens, target `30,000,000`, generated once on a single `RTX PRO 6000` `96 GB` pod from `openai/gpt-oss-120b` `MXFP4`) is admitted to the private store only when its registration checklist (§8), `uor-r4.donor-corpus/1` 5-block generation receipt (§2.3), pilot tranche gates (`P1`–`P8`, §4), and production gates (`G1`–`G14`, §3) all pass.

This record delivers [`verify_distill_d1_donor_corpus.py`](verify_distill_d1_donor_corpus.py), a standalone, host-only (`$0` GPU) consumer verifier that validates every clause of the `distill-d1` registration sheet and enforces the 5 clarifications landed in `#2774` (`bc74e597068ac04da515307dc4dd08d135d30157`):

1. **Top-line and bottom-line `TEST FITNESS` consistency (`parse_sheet_fitness_status`):** Parses both the header `- **Status:** ... **TEST FITNESS: <VERDICT>**` line and the footer `TEST FITNESS: <VERDICT>` line of the registration sheet, raising `INSTRUMENT_UNREACHABLE_D31` (`sheet_status_contradiction`) if they disagree, and pins both the initial `#2765` (`4d3ae4f6b`; prior main tip `30e7445d3`, `#2773`) draft SHA-256 (`dcf3ec1e4913126dbae71fc8674acc98e3c9a31ecf129c5c874c645201d14e73`, `24,263` B, `293` lines) and the post-FIT `#2774` sheet SHA-256 (`51797f33cf13981529fe10f29433ff52020cf1cf354603d61d229a3cc30ad7d6`, `25,622` B, `295` lines).
2. **Explicit `merkle_mode` binding (`compute_shard_merkle_root`):** Requires `checklist.store.merkle_mode` (§8 item 10) and `receipt.content_pin.merkle_mode` (§2.3) to be explicitly bound (`"hex-colon"` for `SHA256(left_hex || ":" || right_hex)` or `"raw-concat"` for `SHA256(left_bytes || right_bytes)`, with odd-node promotion and ascending shard order), rejecting any mismatch between checklist and receipt or between `content_pin.merkle_root` and `readback.recomputed_merkle_root` as `INSTRUMENT_UNREACHABLE_D31`.
3. **`G13` / `P6` `C = 192` mandatory seal gate with `C = 512` diagnostic under `MAX_CONTEXT = 243`:** Enforces `PreparedCorpus::batch(0, 192, 64)` returning `64` non-empty windows with in-document predecessors on `train` and `dev` as the mandatory `G13` / `P6` seal gate, while requiring `context_512_status == "NOT_RUN_MAX_CONTEXT_243"` so the `C = 512` diagnostic is never silently omitted or falsely claimed under `MAX_CONTEXT = 243`.
4. **Scope & fallback donor interlocks (§1.1, §6, §7, §8 items 3 & 9):** Enforces `donor.repo == "openai/gpt-oss-120b"`, `quantization == "MXFP4"`, `license_spdx == "Apache-2.0"`, `budget.driver_math_optimization_completed == True` (owner step 2 prerequisite before steps 4–8), and `budget.student_training_authorized == False` (§6 student consumption rows are informational only), rejecting any un-amended fallback donor (such as `Qwen/Qwen3-30B-A3B` on RTX 5090) unless a new owner decision amends D41.
5. **D31 instrument fault vs. D41 corpus-defect pass accounting:** Strictly separates instrument/receipt/transfer/sandbox faults (`INSTRUMENT_UNREACHABLE_D31`, `d41_pass_consumed = False`, repairable without consuming the one-time D41 pass) from `G4`–`G11` corpus quality defects (`CORPUS_REJECTED_CONSUMES_D41_PASS` / `PILOT_REJECTED_CORPUS_DEFECT_CONSUMES_D41_PASS`, `d41_pass_consumed = True`, requiring a new owner decision before any second generation run), while setting `mechanism_negative = False` on all corpus/instrument outcomes per [D22](../../integration/DECISIONS.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened).

---

## 2. Pinned Repository and Artifact Identities

All SHA-256 digests, byte counts, and line counts below are measured directly from the repository files on `origin/main` (`bc74e597068ac04da515307dc4dd08d135d30157`):

| File / Artifact | SHA-256 | Bytes | Lines | Role Verified |
|---|---|---:|---:|---|
| [`docs/labs/distill-d1-consumer-verifier-2026-10-19/verify_distill_d1_donor_corpus.py`](verify_distill_d1_donor_corpus.py) | `2e0011c36d21ef6acf1c89ac8804d3a134e641b5fa3d2e344e346436d1c04d75` | `154,310` | `3,287` | Standalone host-only `distill-d1` consumer verifier (`32/32` self-tests `PASS`) |
| [`docs/harness/prep/m1-distill-d1-donor-corpus-DRAFT-2026-10-19.md`](../../harness/prep/m1-distill-d1-donor-corpus-DRAFT-2026-10-19.md) (post-FIT `#2774`) | `51797f33cf13981529fe10f29433ff52020cf1cf354603d61d229a3cc30ad7d6` | `25,622` | `295` | `TEST FITNESS: FIT` registration sheet (top & bottom status consistent; 15 required sections) |
| `docs/harness/prep/m1-distill-d1-donor-corpus-DRAFT-2026-10-19.md` (initial `#2765` `4d3ae4f6b` draft; prior main tip `30e7445d3`, `#2773`) | `dcf3ec1e4913126dbae71fc8674acc98e3c9a31ecf129c5c874c645201d14e73` | `24,263` | `293` | Pre-FIT historical draft SHA-256 (rejected by §8 item 1 if used as a dispatch checklist pin) |
| [`crates/uor-r4-discrete/src/metadata.rs`](../../../../crates/uor-r4-discrete/src/metadata.rs) | `27e53e4feecb391adb4a4db3462712a8d9375aed09c0e57086581134cb6e1e32` | `7,086` | `211` | `MAX_CONTEXT = 243` (`3^5` ternary power-of-3 bound) |
| [`crates/uor-r4-discrete/src/corpus.rs`](../../../../crates/uor-r4-discrete/src/corpus.rs) | `826bdaba978b3c54542147656c8aa3cf24d1aa39b6711528b02b545c4fda9f7e` | `14,777` | `401` | `UORT` v1 `64`-byte header (`"UORT"`, version `1`, vocab, count, reserved `0`), `WINDOW_POLICY`, `IDENTITY_POLICY`, `batch(0, context, rows)` |
| [`crates/uor-r4-training/src/knowledge_corpus.rs`](../../../../crates/uor-r4-training/src/knowledge_corpus.rs) | `d07b19c97905f137fcc6c6878e07ec4989e68bc5a31dc2b6fe85125e669a3c2c` | `13,727` | `382` | `PanelExclusion` (`PANEL_NGRAM = 8`, `MIN_SHORT_REQUEST_WORDS = 4`, short turns `4..7` words) |
| [`crates/uor-r4-training/src/milestone_world.rs`](../../../../crates/uor-r4-training/src/milestone_world.rs) | `caf82592999c88afc17a183bac0508c996495697527584f4824647f6cbd2dccc` | `67,087` | `1,976` | `normalize_words` (`ascii_alphanumeric` lowercased tokens) and `mw-balp` 12-word train-overlap check |
| [`docs/labs/rung0-pilot-release-2026-10-14/sheet.md`](../rung0-pilot-release-2026-10-14/sheet.md) | `1029c3b92771e0938d2ec393ce419db81cdc17125dad4814243c235cfb7463d5` | `31,512` | `393` | Pinned student tokenizer (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`, vocab `4096`, `bos_id = 0`, `eos_id = 1`, `unk_id = 2`) |
| [`docs/integration/DECISIONS.md`](../../integration/DECISIONS.md) | `dfa96143cd7262a6458668d9e8f41b9d2fd6ed9c467182a2686d33398a50eaae` | `140,806` | `1,820` | `D41` one-time offline donor-model generation decision record |

---

## 3. Gate and Checklist Coverage Summary

| Contract Section | Gates / Items | Verifier Function(s) | Failure Classification | Consumes D41 Pass? |
|---|---|---|---|---|
| Staged sheet & repo source contract | Top/bottom `TEST FITNESS`, 15 sections, `D41`, `metadata.rs`, `corpus.rs`, `knowledge_corpus.rs`, `milestone_world.rs`, `rung0` tokenizer pin | `parse_sheet_fitness_status`, `verify_sheet_and_repo_contract` | `INSTRUMENT_UNREACHABLE_D31` | `False` |
| §8 Registration checklist | Items `1`–`13` (post-FIT SHA, `D41` ref, `openai/gpt-oss-120b` commit/weights/MXFP4/Apache-2.0, vLLM digest, 5-slice prompt set & SPDX licenses, fixed sampling, student tokenizer, 1x `RTX PRO 6000` `<= $8/h` & `<= 4` pods, `driver_math_optimization_completed = True`, `student_training_authorized = False`, `merkle_mode`, `D40` personal release by `casey` with `UOR_LAB` unset, `distill-d1` hard-targets-only student lineage) | `verify_registration_checklist` | `INSTRUMENT_UNREACHABLE_D31` | `False` |
| §2.3 Generation receipt & §4.3 repair boundary | `uor-r4.donor-corpus/1` 5 blocks (`donor_pin`, `generation_pin`, `run_pin`, `content_pin`, `readback`), non-null/non-empty recursive check, ascending shards, `merkle_mode` (`hex-colon` / `raw-concat`) recomputation (`G1`–`G3`, `P2`–`P3`), shard repair vs. regeneration check | `verify_generation_receipt`, `compute_shard_merkle_root`, `verify_repair_vs_regeneration` | `INSTRUMENT_UNREACHABLE_D31` (or `REGENERATION_FORBIDDEN_REQUIRES_NEW_OWNER_DECISION_D41`) | `False` (`True` if unauthorized regeneration attempted) |
| §4 Pilot tranche (`1,000,000` tokens across 5 shards) | `P1` throughput (`>= 300` tok/s `GO`, `250..300` tok/s `HOLD`, `< 250` tok/s `STOP`), `P2` receipt, `P3` readback, `P4` Rust-only / zero-float student tokenization, `P5` `PreparedCorpus::load`, `P6` `batch(0, 192, 64)` (`64` windows) + `context_512_status = "NOT_RUN_MAX_CONTEXT_243"`, `P7` quality gates `G4`–`G11`, `P8` cost & license accounting | `verify_pilot_bundle` | `GO_FOR_PRODUCTION_TRANCHE` / `HOLD_BELOW_300_TOK_S_GO_BAR` / `STOP_THROUGHPUT_SHORTFALL_OWNER_REPLAN` / `STOP_COST_OR_LICENSE_BLOCKER` / `PILOT_REJECTED_CORPUS_DEFECT_CONSUMES_D41_PASS` / `INSTRUMENT_UNREACHABLE_D31` | `True` only on `P7` (`G4`–`G11`) corpus quality defect |
| §3 Production quality gates | `G4` exact SHA-256 dedup (`0` dupes), `G5` 5-word shingle Jaccard `>= 0.80` (`< 3.0%` residual), `G6` `PanelExclusion` 8-word window + 4–7-word short-turn (`0` hits), `G7` 12-word `mw-balp` `TRAIN` overlap (`0` hits), `G8` sandboxed code (`100%` pass, `0` unevaluated), `G9` deterministic math (`100%` pass, `0` unevaluated), `G10` 200-doc/slice spot audit (`<= 2.0%` malformed, `<= 5.0%` refusals, `<= 1` redraw, or slice drop + renormalization), `G11a` slice shares within `+-3.0 pp`, `G11b` truncation `< 1.0%`, `G11c` 8-word n-gram `> 3x` in `< 2.0%` docs, `G11d` length `>= 16`, median `40..120`, `p99 <= 200` | `verify_quality_gates_g4_to_g11`, `check_panel_exclusion_text`, `check_5word_jaccard_clusters`, `check_12word_train_overlap`, `check_8gram_repetition` | `CORPUS_REJECTED_CONSUMES_D41_PASS` (or `INSTRUMENT_UNREACHABLE_D31` if gate unevaluated or sandbox invalid) | `True` on quality defect (`False` on D31 instrument fault) |
| §3 / §5 Student UORT v1, window admission, and D41 seal | `G12` UORT v1 `64`-byte header + binary `.mask.bin` + `dev >= 250,000` eligible targets + sealed `heldout` (`Split::Fresh`), `G13` `batch(0, 192, 64)` returning `64` windows on `train` and `dev` + `context_512_status = "NOT_RUN_MAX_CONTEXT_243"`, `G14` manifest/receipt seal + `d41_exception_expired = True` | `verify_uort_and_mask_bytes`, `verify_production_bundle` | `PRODUCTION_CORPUS_SEALED_D41_EXPIRED` (or `INSTRUMENT_UNREACHABLE_D31`) | `False` |

---

## 4. Self-Test Suite (`32/32 PASS`)

Running:

```sh
python3 docs/labs/distill-d1-consumer-verifier-2026-10-19/verify_distill_d1_donor_corpus.py --self-test --verify-repo-contract
```

executes all 32 positive and negative self-tests in `< 0.2s` on host CPU (`$0` GPU):

| # | Self-Test Name | Verified Outcome |
|---:|---|---|
| 1 | `repo_and_sheet_contract` | Verifies post-FIT sheet `51797f33cf139815…` (`25,622` B, `FIT`), rejects top/bottom `TEST FITNESS` contradiction as `D31`, and verifies all 6 repository source pins |
| 2 | `valid_registration_checklist` | Verifies all 13 §8 checklist items against post-FIT sheet SHA-256 |
| 3 | `checklist_rejects_unpinned_or_unauthorized_bindings` | Rejects `"main"` revision, un-amended `Qwen/Qwen3-30B-A3B` fallback, pre-FIT draft SHA, lab-typed `D40`, `driver_math_optimization_completed=False`, `student_training_authorized=True`, and stored logits/probs |
| 4 | `valid_receipt_and_merkle_root` | Verifies 5-block receipt, `"hex-colon"` (`de6a79410542288d…`) and `"raw-concat"` Merkle roots, single-leaf hash, and rejects checklist/receipt `merkle_mode` mismatch |
| 5 | `receipt_missing_or_null_field_is_d31` | Rejects missing block, `null` field, whitespace-only string, and unsorted shards as `INSTRUMENT_UNREACHABLE_D31` (`d41_pass_consumed=False`) |
| 6 | `raw_uort_v1_and_c192_window_verifier` | Verifies `222`-token UORT v1 payload + binary mask (`fea435ffd8a9…`) + `batch(0, 192, 64)` returning `64` windows, and rejects bad magic, non-binary mask, non-BOS start, and out-of-vocab token ID |
| 7 | `raw_panel_exclusion_8gram_and_short_turn_verifier` | Verifies clean text, 8-word window hit, 5-word short-turn hit, and `< 4`-word uncovered turn counting |
| 8 | `raw_5word_jaccard_12word_overlap_and_8gram_repetition_helpers` | Verifies 5-word shingle Jaccard `>= 0.80` clustering, 12-word `mw-balp` `TRAIN` overlap detection, and `> 3x` 8-word n-gram repetition detection |
| 9 | `valid_pilot_bundle_go_decision` | Grades valid `1M`-token pilot at `405.0` tok/s as `GO_FOR_PRODUCTION_TRANCHE` (`d41_pass_consumed=False`) |
| 10 | `pilot_throughput_shortfall_below_250_stops_for_owner_replan` | Grades `242.0` tok/s (`< 250`) as `STOP_THROUGHPUT_SHORTFALL_OWNER_REPLAN` (`d41_pass_consumed=False`) |
| 11 | `pilot_throughput_gray_zone_250_to_300_holds` | Grades `285.0` tok/s (`250..300`) as `HOLD_BELOW_300_TOK_S_GO_BAR` (`d41_pass_consumed=False`) |
| 12 | `pilot_quality_gate_failure_rejects_and_consumes_d41_pass` | Rejects pilot `G6` contamination as `PILOT_REJECTED_CORPUS_DEFECT_CONSUMES_D41_PASS` (`d41_pass_consumed=True`) |
| 13 | `pilot_p4_python_or_float_in_tokenization_rejected_d31` | Rejects `python_used=True` or `float_used=True` in `P4` as `INSTRUMENT_UNREACHABLE_D31` |
| 14 | `pilot_p8_cost_or_rate_cap_violation_stops` | Rejects unresolved license or cost-cap violation in `P8` as `STOP_COST_OR_LICENSE_BLOCKER` |
| 15 | `repair_vs_regeneration_enforcement` | Admits same-seed/same-prompt shard repair (`PERMITTED_SHARD_REPAIR`) and rejects donor-revision change as `REGENERATION_FORBIDDEN_REQUIRES_NEW_OWNER_DECISION_D41` |
| 16 | `valid_production_bundle_all_g1_to_g14_pass_and_seal` | Seals valid `30M`-token production bundle (`PRODUCTION_CORPUS_SEALED_D41_EXPIRED`, `d41_exception_expired=True`, `context_512_status="NOT_RUN_MAX_CONTEXT_243"`) |
| 17 | `g1_donor_weight_or_revision_mismatch_is_d31` | Classifies `G1` weight SHA mismatch as `INSTRUMENT_UNREACHABLE_D31` (`d41_pass_consumed=False`) |
| 18 | `g2_receipt_incompleteness_is_d31` | Classifies `G2` missing `run_pin.lease_id` as `INSTRUMENT_UNREACHABLE_D31` (`d41_pass_consumed=False`) |
| 19 | `g3_readback_or_merkle_mismatch_is_d31` | Classifies `G3` shard transfer hash mismatch as `INSTRUMENT_UNREACHABLE_D31` (`d41_pass_consumed=False`) |
| 20 | `g4_exact_dedup_nonzero_rejects_corpus` | Rejects `G4` duplicate document SHA-256 (`CORPUS_REJECTED_CONSUMES_D41_PASS`, `d41_pass_consumed=True`) |
| 21 | `g5_near_dedup_rate_at_or_above_3pct_rejects_corpus` | Rejects `G5` `3.0%` residual near-dedup rate (`CORPUS_REJECTED_CONSUMES_D41_PASS`, `d41_pass_consumed=True`) |
| 22 | `g6_panel_contamination_nonzero_rejects_corpus` | Rejects `G6` panel contamination (`CORPUS_REJECTED_CONSUMES_D41_PASS`, `d41_pass_consumed=True`) |
| 23 | `g7_mw_balp_train_12word_overlap_nonzero_rejects_corpus` | Rejects `G7` 12-word `mw-balp` `TRAIN` overlap (`CORPUS_REJECTED_CONSUMES_D41_PASS`, `d41_pass_consumed=True`) |
| 24 | `g8_code_slice_execution_or_sandbox_failure` | Distinguishes `G8` failing code sample (`CORPUS_REJECTED_CONSUMES_D41_PASS`) from network-enabled sandbox fault (`INSTRUMENT_UNREACHABLE_D31`) |
| 25 | `g9_math_slice_answer_check_or_unevaluated_failure` | Rejects `G9` unevaluated math sample (`CORPUS_REJECTED_CONSUMES_D41_PASS`, `d41_pass_consumed=True`) |
| 26 | `g10_spot_audit_and_slice_drop_renormalization` | Rejects `G10` `> 2.0%` malformed or `> 1` redraw, and verifies 4-slice renormalization in `G11a` when `reasoning_prose` is dropped |
| 27 | `g11a_slice_share_drift_beyond_3pp_rejects_corpus` | Rejects `G11a` `+-3.5 pp` slice drift (`CORPUS_REJECTED_CONSUMES_D41_PASS`, `d41_pass_consumed=True`) |
| 28 | `g11b_truncation_rate_at_or_above_1pct_rejects_corpus` | Rejects `G11b` `1.1%` truncation rate (`CORPUS_REJECTED_CONSUMES_D41_PASS`, `d41_pass_consumed=True`) |
| 29 | `g11c_repetition_8gram_rate_at_or_above_2pct_rejects_corpus` | Rejects `G11c` `2.2%` repeated 8-gram doc rate (`CORPUS_REJECTED_CONSUMES_D41_PASS`, `d41_pass_consumed=True`) |
| 30 | `g11d_length_bounds_violation_rejects_corpus` | Rejects `G11d` `min < 16`, `median > 120`, and `p99 > 200` (`CORPUS_REJECTED_CONSUMES_D41_PASS`, `d41_pass_consumed=True`) |
| 31 | `g12_and_g13_uort_or_window_admission_or_dev_power_failure` | Rejects `G12` `dev < 250,000` eligible targets, opened `heldout` split, and `G13` `< 64` windows as `INSTRUMENT_UNREACHABLE_D31` |
| 32 | `g14_unsealed_or_unexpired_d41_exception_refused` | Refuses `G14` seal if `d41_exception_expired` is `False` (`INSTRUMENT_UNREACHABLE_D31`) |

---

## 5. Coordination Hygiene (`docs/harness/claims.json`)

In accordance with [#2029 comment `6262421374`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6262421374) item 4 and [#2029 comment `6262709553`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6262709553), all 5 finished `antigravity/*` branch claims have been released from [`docs/harness/claims.json`](../../harness/claims.json):

- `antigravity/w2-k16-consumer-verifiers-20261019` (merged in `#2727` `85446e96a`)
- `antigravity/address-cells-consumer-verifier-2026-10-19` (merged in `#2750` `232b67a6a`)
- `antigravity/address-cells-consumer-verifier-v2-2026-10-19` (merged in `#2758` `3f3c7360f`)
- `antigravity/a8-dose-census-attribution-verifier` (merged in `#2773` `30e7445d3`)
- `antigravity/distill-d1-consumer-verifier-20261019` (merged in `#2775` `babe5ca9f`)
