# Pre-registration refresh - M2 TRAIN-mask rung-0 sheet under D37 (`m2-pin-train-mask-sheet`)

- date: 2026-10-23; item #2851; base `8720c5d6d`
- milestone: M1 #2029 rung 0 on the compliant trunk (the design record this refresh re-pins); it is the M2 pre-registration source (`#2030`).
- mechanism: rank-margin-objective; mechanism brief `docs/mechanisms/rank-margin-objective.md`.
- old sheet: [`m1-rank-margin-rung0-2026-10-13.md`](m1-rank-margin-rung0-2026-10-13.md) (Amendments 1-7; Amendment 7 froze it under D32).
- **legal basis: D37 + CHARTER §4 rule 5** - floating point is legal in **offline training only**, every float candidate reports its **export gap on the same evaluator**, and **a gap above the two-seed noise band disqualifies it** (CHARTER §3.7). D37 revises D32's learning-path float ban; serving stays integer (§3.1-§3.3, §3.6).
- **this is a NEW sheet, not a revival.** CHARTER §4 rule 5: "Archived float sheets return only as new sheets with a fresh FIT." The old Amendment 6 sheet stays an archived design record; this page is the new D37-basis sheet and carries `TEST FITNESS: NOT_FIT` (`antigravity-33` non-author review; four prerequisites recorded below).
- status: **refresh; `TEST FITNESS: PENDING` (non-author re-review requested).** The owner answered the live/superseded question on 2026-10-24T03:42Z ([#2851](https://github.com/UOR-Foundation/uor-r4/issues/2851), [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6379430093)): *"I want both because we do not know which we will ultimately need."* So this D37-basis sheet is the successor to the Amendment 6 design record **and** a discrete-law rung-0 sheet is also wanted; deepseek covers the PM role meanwhile. This revision registers the export-gap statistic and numeric band below, pins the literal source SHAs, and pre-registers the fresh-transfer slice gate. It still runs nothing and releases no hold: the remaining `FIT` prerequisites are the separate D29 float module [#2823](https://github.com/UOR-Foundation/uor-r4/issues/2823), the D11 float-trunk exporter [#2820](https://github.com/UOR-Foundation/uor-r4/issues/2820), and a sealed fresh-transfer slice.
- scope: this refresh re-pins the old sheet's four Amendment 6 mechanical pins against current `main` and re-states its bars unchanged. No bar, hold or spend is changed here; a bar change is a decision card (CHARTER §5.2).

## Bar

The **decisive bars, the reference, the anti-vacuity guards, the dev evaluation policy, the noise floor and the diagnostics are unchanged from the old sheet's `## Bar` (Amendments 1-5) and are not restated to be reworded here** - restating a bar risks changing it by paraphrase. In full, they are:

- **(i) raw-score top-1** over the scored dev-panel set beats the count predictor's top-1 by **>= 5 percentage points on both seeds** (seeds 1 and 2).
- **(ii) recall@8** over `top_token_ids(8)` beats the count predictor's recall@8 by **>= 5 percentage points on both seeds**.
- **reference:** token-frequency count table over the **response-masked TRAIN target positions** (`mw-balp/train/`: 7,512,797 tokens, 2,425,925 response-masked targets; ties by smallest token identity), computed in-run on the GPU; the DEV-response oracle is a diagnostic only and never the reference.
- **anti-vacuity floors:** final dev token-margin hinge < 0.5 raw-score units and dev unviolated-target rate >= max(5x same-seed untrained, 1% absolute), on both seeds.
- **noise:** scored denominator n = 14,287 (mask sum over scored positions), Wilson 95% half-width < 0.83 points; the 5-point margin sits far above it; between-seed disagreement is reported.
- **fresh transfer:** the decisive bars are re-measured on a fresh held-out draw for a KEEP; the frozen reply panels are not scored by rung 0. **Pre-registered gate:** the fresh-transfer slice must be sealed and pinned in this sheet before any KEEP can be claimed - file path, SHA-256, and row count `>= 200` rows of `mw-balp` windows the run never trained on. No such slice is sealed yet (the frozen reply panels are not rung-0 windows), so a KEEP is blocked until the slice is sealed and pinned; the decisive bars are still re-measured on it.

Nothing above is narrower than the old sheet's registered bars; the old sheet's full text remains the authoritative wording.

## Base artifact

- **base: none** - trained from seed initialization at the pinned config (the historical 4.4M `rrarra` smoke config is context only, not a base artifact).
- **pinned config (unchanged), with the per-arm objective made explicit:** `arch=geometric pattern=rrrrrr width=256 heads=4 stack_mlp=384 context=384 gate=hamilton rec_gates=geometric projection=spiralcore_fbs head=spiralcore_product device=cuda precision=f32 data_parallel=1`; **`objective=hinge` for M1 (1 rival) and `objective=hinge8` for M8 (8 rivals)** - the old sheet's arm table (line 213) and the trainer's accepted values (`geometric-stack-native.rs:451/494`: `hinge|hinge8`).
- **D37 export gap and its outcome rule, stated:** the run trains in offline f32; the exported artifact is the integer artifact, and the run report must state the measured gap between the f32 training evaluations and the exported integer artifact's evaluations on the same evaluator (D37; CHARTER §3.7). **Outcome rule: a gap above the two-seed noise band disqualifies the configuration** - the **PM role (deepseek, covering while the PM is offline, 2026-10-24T13:10Z)** registers the statistic and numeric band here, so no reviewer has to set it (CHARTER section 5.2; D39 item 5): the statistic is `G_top1 = |top1_f32 - top1_int|` and `G_recall8 = |recall8_f32 - recall8_int|`, both computed on the same `14,287` eligible DEV rows with the same evaluator, read mode and greedy decode. **Disqualifying band:** the configuration is disqualified iff `max(G_top1, G_recall8)` exceeds `max(0.83 percentage points, the same-seed-pair two-seed spread of that metric)`. `0.83 pp` is `119` rows of `14,287`, the worst-case Wilson 95% half-width at `p = 0.5`; the measured two-seed spread is reported alongside and the larger of the two is the effective band. This band is pre-registered and no reviewer or run may widen it; the run reports the gap and the spread against it, and a float arm cannot KEEP above it. Serving is integer only.

## Data split and pins

- **TRAIN archive:** `data/ft-balp.tar`, 326,860,800 bytes, md5 `45fbd266ecab1e5c113e10ec7ae1e918`, sha256 `b1d2211377a2e99d5977cc95d46875027023b08867488b1372185b3f62a1473f`. The run reads only `mw-balp/train/`: 7,512,797 u16 tokens (tokens sha256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`), response mask `mw-balp/train/response_mask.u8` (**7,512,797 bytes, sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`**), **`expected_train_targets=2425925`**.
  - **re-verified this review:** the mask digest appears in the current code pin (`crates/uor-r4-discrete/src/attention_learning.rs:1367`, with 7,512,797/2,425,925) and the count reference archive carries the 2,425,925 count (`archive/labs/count-reference-2026-10-13/README.md:32,84,96`). The DEV tar hashes and lengths above are **restated** from the old sheet's Amendment 4 record, not re-computed (the DEV tar is not in this checkout).
- **DEV panel:** `data/ft-dev.tar`, 143,360 bytes, md5 `0a1d0a05020a5d80b5d008fd5b731134`, sha256 `45e93e32918f8ad5c977ad3a98926b561173049aacd8bfce2b6e2e49a0390b5c`; `ft/dev/tokens.u16` = 64-byte UORT header + **42,856** u16 payload tokens; `ft/dev/response_mask.u8` = 42,856 bytes with **14,287** eligible within-document targets across 400 documents; **`expected_dev_targets=14287`**; 111 global 384-token inventory blocks + 232 tail tokens.
- **tokenizer:** sha256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`, vocab 4096, bos 0 / eos 1 / unk 2.
- **dose:** 7,324 steps x batch 16 x context 384 = **44,998,656** tokens (~6 passes over the 7,512,797-token train split). (The old sheet's `44,957,696` is an arithmetic typo; the step/batch/context pins are unchanged, so the run is unaffected.)
- No corpus is regenerated, padded or changed by this refresh; nothing here is training data for the frozen reply panels.

## Steps

- **exact CLI (unchanged from Amendment 6; verified present in `crates/uor-r4-training/examples/geometric-stack-native.rs`). The CLI below is the M1 arm (`objective=hinge`); for the M8 arm substitute `objective=hinge8` and change nothing else:**

```text
geometric-stack train arch=geometric pattern=rrrrrr width=256 heads=4 stack_mlp=384 context=384 gate=hamilton rec_gates=geometric projection=spiralcore_fbs head=spiralcore_product objective=hinge device=cuda precision=f32 data_parallel=1 train=/workspace/rung0/data/mw-balp/train/tokens.u16 train_mask=/workspace/rung0/data/mw-balp/train/response_mask.u8 valid=/workspace/rung0/data/ft/dev/tokens.u16 valid_mask=/workspace/rung0/data/ft/dev/response_mask.u8 tokenizer=/workspace/rung0/data/tokenizer-d36d3e87.json expected_dev_targets=14287 expected_train_targets=2425925 batch=16 steps=7324 lr=0.002 warmup=200 min_lr=0.1 weight_decay=0.1 clip=1 eval_every=128 eval_windows=111 final_windows=111 checkpoint_every=128 sample_tokens=0 max_seconds=inf seed=SEED out=EXCLUSIVE_REPORT_ROOT
```

- four runs on one leased GPU, sequential: **M1** (`objective=hinge`, 1 rival) and **M8** (`objective=hinge8`, 8 rivals) at **seed 1 and seed 2**; paired seeds, same initializer and window RNG stream, only the objective/rival count differs.
- this refresh executes none of it: training and rental remain HELD until the owner releases them (AGENTS.md; CHARTER §4.5/§5.1), and a FIT releases nothing.

## Stop conditions

- **what the CLI actually enforces (verified in `crates/uor-r4-training/examples/geometric-stack-native.rs` and `native_rung0.rs`):** it refuses without `train_mask`; it binds the TRAIN input content (records bytes + sha256) and aborts when the masked TRAIN reference count differs from `expected_train_targets` (2,425,925); it checks the mask length equals the token count; it stores `expected_dev_targets` in the panel plan and **reports** any DEV denominator mismatch (14,287) as a diagnostic rather than aborting; on resume it requires the checkpoint's saved input binding to match.
- **manual pre-run verification this sheet requires (restored from the old sheet's step list):** before launching, the operator verifies the declared tar/member identities and mask inventory by hand - `data/ft-balp.tar` and `data/ft-dev.tar` sizes/md5/sha256 above, the mask member's 7,512,797 bytes and sha256 `b5e7352d…`, the 42,856 DEV tokens and the 14,287 mask sum - and records each result in the run report. A mismatch stops the launch; it is not absorbed.
- **lawful stops:** the enforced abort above, a failed manual hash/count check, a hashing/read error, a GPU/lease failure, or an owner stop. A self-set wall-clock or budget estimate is not a stop rule; a healthy run is extended and its lease renewed (AGENTS.md). The DEV denominator mismatch is reported, not absorbed, and the report must name it.
- **outcome rule (D37/§3.7):** the export gap on the same evaluator must be stated in the report, and **a gap above the two-seed noise band disqualifies the configuration**; that is a negative, and it closes this configuration, not the mechanism.
- **negative classification:** a failed bar closes this configuration (offline float, hinge objective/arm objectives M1 `hinge` / M8 `hinge8`, rival counts, pinned dose), never the mechanism; the old sheet's verdict wording and §4 rule 1 apply.
## Cost

- preparation: no GPU (source, sheet, pins; the data is already published).
- run: **one Vast pod, 1x RTX 5090 or the next rung, <= $4 total UNVERIFIED projection** for the four runs (history: <= $2 on a 1x4090), within the owner caps (**<= 4 running pods and <= $8/h across all labs**: CHARTER §5 rule 4, not D26 - D26 is only that Vast is the default provider).
- no separate control pod: the count predictor and the untrained same-seed guard are computed on the same GPU inside the run; retention: small checkpoints/report only, cloud-store round-trip, 128 MiB physical stop margin.

## Minimum test power

- **seeds:** 1 and 2, paired across M1/M8: 2 seeds satisfy D22 section 3's decisive-run requirement (>= 2 seeds **or** >= 200 fresh rows).
- **trained in, not bolted on:** the full pinned run (7,324 steps, ~6 passes) gives the base time to adapt; a short add-on would be a probe.
- **noise floor:** Wilson 95% half-width < 0.83 points at n = 14,287; between-seed disagreement is additionally reported; no bar narrower than the noise.
- **fresh-transfer slice (KEEP gate, not yet sealed):** before a KEEP, seal and pin a fresh held-out draw of `mw-balp` windows the run never trained on (`>= 200` rows; record path, SHA-256 and row count here), then re-measure top-1 and recall@8 on it with the anti-vacuity guards reported alongside. No slice is sealed at this revision, so a KEEP cannot be claimed; sealing it is a separate data step, not a run step.

## Literal frozen source SHAs

- count-reference implementation frozen at `2fd6358bbc52d2a202a872ad0954ebb12894606f` (11 finite controls: 9 metadata + 2 CUDA).
- protected PR [#2328](https://github.com/UOR-Foundation/uor-r4/pull/2328) merge commit `9e904a83448c451f738d5c1d45224dfdf73eb84e` (count-predictor scope fix; the CLI counts response-masked TRAIN targets).
- `9e904a83448c451f738d5c1d45224dfdf73eb84e` is an ancestor of the current `main`.
- **Re-verified 25 Oct 2026 (claude-99) at `53b40ecd6`:** `mw-balp/train/response_mask.u8`, extracted from the HF store's `data/ft-balp.tar` (sha256 `b1d2211377a2e99d5977cc95d46875027023b08867488b1372185b3f62a1473f`), is `7,512,797` bytes with sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`. `9e904a83448c451f738d5c1d45224dfdf73eb84e` is an ancestor of `main`. The count reference `crates/uor-r4-training/examples/geometric-stack/native_rung0.rs` is byte-identical between `2fd6358bb` and `main`; the `geometric-stack` trainer around it has changed, so the CLI line is re-checked when the float module wires it in. This was a host check only: no build, no training.
- `2fd6358bbc52d2a202a872ad0954ebb12894606f` is the frozen source commit whose change landed squashed in `9e904a834` (PR #2328); the commit object exists and is referenced by the retained count-reference archive, but it is **not** an ancestor of `main` itself, so a fresh checkout must use the merge commit for the landed source.

## TEST FITNESS line

```text
TEST FITNESS: NOT_FIT - M2 #2030 rank-margin-objective
sheet: docs/harness/prep/m2-pin-train-mask-sheet-2026-10-23.md
reviewer: antigravity (fresh-context subagent 40c9d939-5725-4a06-b791-5d7c7f3aa071)
note: Four Amendment 6 pins verified against main and archive; NOT_FIT for compute until (1) PM/owner live-vs-superseded decision (#2851) is recorded, (2) D37 export-gap statistic and numeric band are fixed in the sheet itself rather than delegated to the reviewer, (3) the separate D29-scanned float module (float-training-module) and D11 float-trunk exporter (d11-exporter-float-trunk) land and are wired into Steps, and (4) literal source SHAs (2fd6358bbc52d2a202a872ad0954ebb12894606f / 9e904a83448c451f738d5c1d45224dfdf73eb84e) and the fresh-transfer slice (>=200 rows, path + sha256) are pinned in the sheet
date: 2026-10-23
```

### Non-Author `TEST FITNESS` Audit Summary (`antigravity-33`, 2026-10-23)

1. **Four Amendment 6 mechanical pins — VERIFIED:**
   - `mw-balp/train/response_mask.u8` (`7,512,797` bytes, sha256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`) and `expected_train_targets=2425925` match `crates/uor-r4-discrete/src/attention_learning.rs:1367-1370` and `archive/labs/count-reference-2026-10-13/README.md:32,84,95-96`.
   - The `geometric-stack train` CLI and per-arm objectives (`objective=hinge` for M1, `objective=hinge8` for M8) match `crates/uor-r4-training/examples/geometric-stack-native.rs:361-406,449-456,492-496,535-543`, and the dose product (`7,324 × 16 × 384 = 44,998,656`) corrects the old sheet's arithmetic typo (`44,957,696`).
   - Frozen commits `2fd6358bbc52d2a202a872ad0954ebb12894606f` and `9e904a83448c451f738d5c1d45224dfdf73eb84e` verify in git history.
2. **Why `TEST FITNESS: NOT_FIT` for compute (four prerequisites before `FIT`):**
   - **(a) Unresolved PM/owner scope decision (`#2851` / `owner-decision-float-path-scope`):** The PM/owner answer on whether this D37-basis float rung-0 sheet is the active successor to Amendment 6 or superseded by the discrete learning law is still pending (lines 9, 75).
   - **(b) Unregistered D37 export-gap statistic and numeric band (Criterion 2 / CHARTER §3.7, §5.2, D39 item 5):** Line 29 delegates fixing the numeric export-gap band to the FIT reviewer (*"the non-author FIT fixes the numeric band for this sheet before any run"*). Under CHARTER §5.2 a reviewer cannot author or change a bar/band; the sheet itself must specify the exact gap statistic (e.g., `|top1_f32 - top1_int|` and `|recall8_f32 - recall8_int|` on the `14,287` DEV rows) and either a PM/owner-approved numeric band or an explicit pre-registered formula anchored by the Wilson 95% half-width (`0.83` pp / `119` rows) and two-seed spread.
   - **(c) Missing D11 float-trunk exporter and separate D29-scanned float module (Criteria 1 & 4 / CHARTER §3.7, D37 items 3 & 6):** `## Steps` executes `geometric-stack` inside `crates/uor-r4-training` (`model.scored_in_f32(...)`) with no D11 integer export or same-evaluator integer scoring step, because `d11-exporter-float-trunk` (`#2820`) and `float-training-module` (`#2823`) are not yet built. Without integer export + same-evaluator scoring, the CLI measures only the f32 proxy and cannot evaluate the D37 export-gap gate.
   - **(d) Missing literal source SHAs and fresh-transfer slice pin in the sheet body (Criterion 3):** The sheet body should state the literal 40-char frozen source SHAs (`2fd6358bbc52d2a202a872ad0954ebb12894606f` and `9e904a83448c451f738d5c1d45224dfdf73eb84e`, currently only in `docs/labs/m2-pin-train-mask-sheet/README.md:10`) and pin the file path, SHA-256, and `>= 200` row count for the fresh-transfer draw (lines 21, 69).