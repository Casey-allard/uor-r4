# A8 address-cells learned-attention dose — Relation arm Q1 diagnostic stop (2026-10-19)

- **Line:** M1 learned attention (#2029). **Lab:** claude, as sole operator. **Count:** the headline is unchanged; this is a D31 diagnostic stop, not a negative.
- **Sheet:** [`m1-attention-a8-address-cells-learning-2026-10-19.md`](../../harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md), FIT, SHA `3e622beb…`, registered in #2755.
- **Runbook:** [`a8-dose-runbook-2026-10-19`](../a8-dose-runbook-2026-10-19/README.md).
- **Authority:**
  - Kimi PM dispatch #2029 `6259364299`, recording the owner's costed-plan approval, with the ceiling reconciled to $14 in `6259385236`.
  - Casey confirmed directly in Claude chat.
  - **Casey typed both D40 releases himself** on the pod, with `UOR_LAB` unset. Admission owner records: Relation `f392a260…`, FrozenZero `1e74a6e5…`.
- **Source freeze:** `2c015db9f01d3ba3a7d79889a052b1345d9da48f`. The ELF is `ea2e13933fbf61b43b42c948bef91eac70ac76352c24019c64913cf72475f4e5`, and the driver source `f673edb5…`.
- **Pod:** `vast-56670997`, 2 × RTX 5090, Vast Czechia, $1.816/h. It was created at about 04:38Z and deleted after archiving at about 05:40Z: roughly 62 minutes of pod life, **about $1.9**. That is a resource proxy, not an invoice; storage and transfer were not measured.
- **Archive:** HF store `caseyallard/uor-r4-store:vast-archive/vast-56670997/`, native ledger 2,301,552 KiB = **2,247.6 MiB** (2,357.6 MB decimal) for **442 payload files** (434 runs + 6 jobs + 2 pods) plus 3 manifest sidecars; the native proof is file-count + downloaded manifest-sidecar comparison, not a full 442-payload byte readback. See the post-merge correction below. It holds the gates, both arms' runs, and custody (plans, reviews, releases, state). Every directory carries a `FILES.sha256`. The sealed fresh V2 slice is not in the archive.

## Pre-run gates (zero updates): all PASS

[`evidence/a8-gates.sh`](evidence/a8-gates.sh) (`a11ae977…`) ran from 04:40 to 04:43Z; its log is [`evidence/gates-job.log`](evidence/gates-job.log). Receipt: #2029 `6259849156`.

- **Instrument:** build OK; 6 / 6 data pins; the A8 rebuild reproduces `6600bdb1…` / `4122cbab…`.
- **Both arms:**
  - attention-build-identity and G-Occ pass (256 cells, tie mass 0);
  - the G-Init table check passes, and INIT LOCK holds on the gates;
  - the **unpinned-initializer comparison matches on routes, gates, seed and epoch.** This was NOT_RUN in #2751.
  - The **reproduction gate** gives 1,351 / 4,244 / 133.
  - **Gate22 and Gate0 are sensitive.**
- **Cross-arm:** the two checkpoint-0 models are byte-equal.
- **Bar (i):** FrozenZero P_succ top-1 is 62, so Bar (i) = 86. Net G-Cap 1,351 > max(500, 86).
- **Negatives:** (a) refused on the host; (b) the A8 non-root table (`e35bb48d…`) refused on the GPU with status 6.
- **qualify-relation:** 9 / 9, `not_run` 0, applied 0. The control `active_leaf_initialization_and_matched_off_path` is PASS, and its INIT LOCK seeds 1 and 2 are bit-identical and reload-identical.
  - **Instrument correction:** the script's `INIT_LOCK_RECORD` line read FAIL because its grep also matched a summary object. The gate was re-graded from the canonical `relation-controls.json` field and passes.

## Plans and reviews

- The plans were built from the actual `attention-build-identity` output ([`evidence/a8-plans.sh`](evidence/a8-plans.sh)). The review payload comes from the crate's own `review_payload_sha256`, through a ten-line host tool ([`evidence/a8-plan-payload.rs`](evidence/a8-plan-payload.rs); it opens no FilePins and does no model work). Codex's rows_v1 helper refuses non-rows_v1 packets, so it could not be reused.
- **Fresh-context reviewer subagent, three rounds:**
  - **Round 1, REQUEST_CHANGES:** `base_source` must be `ATTENTION_BASE_SOURCE` `ef9d3099…`, not the freeze. The fix is applied in `a8-plans.sh`.
  - **Round 2, APPROVE** for both arms.
  - **Round 3:** `attention-check-plan` then refused FrozenZero. The control's `relation_off_dev_evaluations` must be 0 (`AttentionCostContract::expected`, attention_learning.rs:83-99), but it had been copied from the treatment as 1. The field was patched on the pod with `jq`; the committed `a8-plans.sh` still writes 1 for both arms, a known defect for any rerun. FrozenZero was re-reviewed and approved.
- Receipts: #2029 `6259900886` (Relation) and `6259913207` (FrozenZero). Both plans are `PLAN_METADATA_VALID`, with `compiled_identity_checked: true`: Relation `c97649e3…`, FrozenZero `faed2c0d…`.
- **A setup slip, fail-closed:** Casey's first release attempt was refused because the run-state parent directory did not exist. Nothing was written, and the retry succeeded.

## Outcome

Both arms started at 04:55Z, one per GPU.

| Arm | P_succ init | P_succ at Q1 (batch 4,096) | Full panel at Q1 | Q1 stop rule (Q1 < 500, or Q1 < ½ init = 675.5) | Status |
|---|---:|---:|---:|---|---|
| **Relation** (LearnV1) | 1,351 | **551** | 1,511 / 14,287 | **½-init criterion fired** (551 ≥ 500) | `Q1_STOP_FOR_DIAGNOSIS` at 4,096, 05:30Z |
| **FrozenZero** (control) | 1,351 | 857 | 2,519 / 14,287 | not fired | `COMPLETE_PENDING_REGISTERED_DISCRIMINATOR` at 16,404, 05:28Z |

**Q1-interval counters** (`evidence/*-q1-summary.json`; attempted / candidate-cost-sensitive / accepted):

| Arm | vector | combine gates | relation trits | refused relation |
|---|---|---|---|---:|
| Relation | 3,584 / 413 / 192 | 237 / 236 / 117 | **275 / 124 / 87** | 0 |
| FrozenZero | 3,584 / 469 / 207 | 237 / 236 / 120 | 275 / 140 / 0 | 275 |

**FrozenZero full dose** ([`evidence/frozen-zero-report.json`](evidence/frozen-zero-report.json)):
- it passes the §7.8 item 2 control-lock proof: gate_relation 1,130 attempted, 1,130 refused, 0 accepted; combine 920 attempted, 508 accepted;
- vector updates: 301 accepted;
- 1,049,856 training-target exposures;
- 1,967 s for 16,404 batches, **about 0.12 s per batch**, against the 1.31 s planning assumption, so cost estimates for this lineage should be revised down.

Its fresh transfer is `SEALED_PENDING_PAIRED_DEV_DECISION`, and `learned_attention_result` is `NOT_YET_SCORED`.

## Reading (scoped)

- This is the risk the sheet stated before measurement (§3 *Known risk*): learning moves carry tokens out of their predecessor-placed cells. **Both arms lost admission**: the control by 37% (1,351 → 857), and the Relation arm by 59% (1,351 → 551).
- With otherwise matched move budgets, the extra loss coincides with the Relation arm's 87 accepted relation-trit updates. This shows co-occurrence, not attribution: a cell-level ejection count would be needed to attribute it.
- Per the sheet this is a **D31 diagnostic stop, not a mechanism negative and not a KEEP**. It closes no mechanism (D22) and does not count as a headline negative.
- The sheet requires a paired DEV H_geom decision, and there is none, so `attention-transfer` does not run and the fresh V2 slice is unopened.

## Next (PM decision)

Diagnose why accepted updates eject P_succ targets, and why Relation does so faster. Candidates, each needing its own FIT'd sheet:
- **(i)** an admission-preserving acceptance criterion for codebook and relation moves, which would refuse a move that ejects an admitted target's cell;
- **(ii)** a per-checkpoint census of token-cell placement on the archived 64-batch checkpoints. Placement can be reconstructed from each checkpoint's table on the host; recomputing admission itself needs device evaluation. This is the cheapest first step;
- **(iii)** re-registering at a finer checkpoint cadence to locate where the erosion happens.

The archived 64-batch checkpoints support (ii) without any new training.

## Post-merge correction (2026-10-19)

The archive accounting above was re-worded after independent custody review: the native ledger records **2,301,552 KiB = 2,247.6 MiB** (2,357.6 MB decimal) for 442 payload files (434 runs + 6 jobs + 2 pods), with 3 manifest sidecars listed (445 total). Native `--to-store` verifies file counts and downloads/compares each manifest sidecar; it does not download/hash every payload byte. The custody `FILES.sha256` self-entry is stale (`e3b0...`); its raw file hash is `bc338d2c...` and its 27 other members verify. No result, bar, report or evidence byte changed.
