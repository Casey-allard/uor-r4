# C2 qualification and the Stage T registration state (item #2835)

Item: [`m1-bvg-k-block-apply-c2`](../../roadmap/items/m1-bvg-k-block-apply-c2.md) (`#2835`, M1 W3 BVG-K). Lab: deepseek, Unit: the in-scope record while the Stage T sheet's preconditions are unresolved.

## What is already delivered

- **Stage C2 (block propose, verify and apply) PASS.** Fixture-only GPU controls from 2026-10-19, about 13 minutes for roughly $0.40. Receipt: [`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json`](../injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json), SHA-256 `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`.
- **Controls that passed:** V0 source identity, RC re-carry parity, UT untouched, JR joint reference, DC decision coverage, V3 block apply equals forced sequential apply, RF refusals and C1R (C1 parity rerun). All 3,419 updates were on synthetic fixture devices; `Device::apply` was never called.
- **The Stage T sheet exists as a DRAFT** and is machine-complete to shape: [`m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md`](../../harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md) (`TEST FITNESS: PENDING`; it covers arms T1/T16/T4, equal-wall-clock protocol, first-batch controls V1d/KId/V8d/V9d/AKd, cost waves and power).

## Why this item cannot be closed here (registration blockers)

1. **The Stage 2 predicate is unresolved.** D39 item 4 runs W3 only if W2 decides `H_law`, and the record is contradictory: `NOW.md` row 6 lists the W2 read-code capacity capture as **queued**, the roadmap item `m1-w2-capacity-capture` is **held**, while `project-track.md` says "W2 read-code capacity — DONE" (that sentence appears to describe the v5.1 consumer-verifier ACCEPT, not the capture). **PM answer needed:** the capture's status, and whether `H_law` is decided.
2. **No base lineage passes both its pre-flight and its registered Q1 admission gate.** The injective pre-flight failed B3_init (127 of 3,847 < 500) and was not dispatched; the A8 lineage *reached* B3_init (1,351/3,847, clearing the 500 floor) and then hit its registered Q1 admission-collapse stop (P_succ 1,351 → 551; FrozenZero 857/3,847) — a D31 diagnostic stop with no paired DEV decision. The Stage T sheet's precondition 2 therefore stays unmet; the census (Q1–Q5) narrows the question but does not supply a lineage that survives Q1.
3. **Registration numbers are not fixed.** The non-inferiority margin, the base-lineage choice, T4 inclusion and the AKd shadow-device admission are all registration-review items (the DRAFT lists them); the item's bar is the Stage 2 bar ("beat the unigram table, then the admission-matched next-word table … on two seeds"), with the unigram top-1 baseline 1,226 and the panel's 2-SE floors (20/63/87 rows) as the non-narrowness floor. **No owner release or FIT is requested here**, and training remains HELD.

## Notebook row (staged; the page is outside this item's scope)

The item's step 6 asks for one row in `docs/research/notebook/m1-discrete-attention.md`. That page is outside this item's declared `scope:`, so the row is staged here for the same backfill path used for the m3 items: after a `notebook:` field is added to this item (umbrella-scope PR, e.g. the pattern of #2997/#3002), the row can be appended there verbatim.

| Date | Item | Change / Finding | Verification |
|---|---|---|---|
| 2026-10-22 | `m1-bvg-k-block-apply-c2` (`#2835`) | BVG-K Stage C2 (block propose/verify/apply) qualified: V0, RC, UT, JR, DC, V3, RF and C1R pass on synthetic fixture devices (3,419 updates; `Device::apply` never called). Stage T sheet exists as a machine-complete DRAFT, but registration is blocked on the W2 `H_law` predicate, a base lineage that reaches admission, and the margin/base-lineage registration items; no compute is requested and training stays HELD. | C2 receipt SHA-256 `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`; `prep-check` on the Stage T DRAFT reports `complete: true`, `TEST FITNESS: PENDING`; the A8 dose Q1 stop (P_succ 1,351 → 551; FrozenZero 857) is in [the dose record](../../labs/a8-dose-2026-10-19/README.md), and the host census (Q1–Q5) in [the diagnosis record](../../labs/m1-a8-erosion-diagnosis/README.md). |

## Sources

- [`docs/harness/prep/m1-bvgk-stagec2-block-qualification-2026-10-18.md`](../../harness/prep/m1-bvgk-stagec2-block-qualification-2026-10-18.md) (C2 sheet and FIT).
- [`docs/harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md`](../../harness/prep/m1-bvgk-staget-learning-law-DRAFT-2026-10-19.md) (Stage T DRAFT).
- [`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md`](../injective-preflight-and-c2-qualify-2026-10-19/README.md) (C2 PASS record).
- [`docs/labs/a8-dose-2026-10-19/README.md`](../a8-dose-2026-10-19/README.md) (the A8 dose result and its registered Q1 admission-collapse stop).
- [`docs/labs/m1-a8-erosion-diagnosis/README.md`](../m1-a8-erosion-diagnosis/README.md) (the host placement census that narrows, but does not answer, the erosion question).
- [`docs/integration/NOW.md`](../../integration/NOW.md), [`docs/integration/project-track.md`](../../integration/project-track.md) (the W2 status contradiction).

---

## Stage T Pre-Registration Completion (`2026-10-25`, Lab: `antigravity`, PR `#3204`)

- **Status:** **COMPLETE (`status: done`) — Stage C2 finite CUDA controls PASS (`status = "PASS"`, `8/8` controls, `3,419` fixture updates, receipt `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`) and the one-page Stage T pre-registration sheet ([`../../harness/prep/m1-bvgk-stage-t-training-2026-10-25.md`](../../harness/prep/m1-bvgk-stage-t-training-2026-10-25.md)) is registered with a non-author `TEST FITNESS: FIT` review and D68 decision review ([`#2835` comment `6395412609`](https://github.com/UOR-Foundation/uor-r4/issues/2835#issuecomment-6395412609), reviewer `bb65a68d-8e8b-4e09-8d64-f624827d645f`).**
- **Stage C1 Qualification Receipt:** [`../bvgk-rowsk-scoring-2026-10-18/evidence/run1/clean/rowsk-qualification.json`](../bvgk-rowsk-scoring-2026-10-18/evidence/run1/clean/rowsk-qualification.json) (`sha256 = d5ec5348d64e2abd74a334319d4f78e6ed7b174b5a2fc0bb381505dc46e09e56`, `5,130 / 5,130` coordinate parity, `16.25x` scoring speedup)
- **Stage C2 Qualification Receipt:** [`../injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json`](../injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json) (`sha256 = 2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`, `8 / 8` controls `V0`, `RC`, `UT`, `JR`, `DC`, `V3`, `RF`, `C1R` pass on RTX 5090)

### 1. Stage C2 Exact Sub-Count Breakdown (`bvgk-qualification.json`)

| Control | What It Verifies | Measured Result in `bvgk-qualification.json` | Verdict |
| :--- | :--- | :--- | :--- |
| **`V0`** | Source, PTX, cubin, and loaded kernel register/memory resource parity | `0` register or local/shared/const memory delta | `PASS` |
| **`RC`** | Scratch re-carry window splitting (`split_windows`) reproduces original carried batch | `64` cases / `14,080` lanes across all 4 cells (`Route` 2/4 blocks, `OccupiedCellRankHingeV1`, `Relation`) | `PASS` |
| **`UT`** | `propose_block` leaves original device state (`table`, `routes`, `gates`) and `epoch` byte-identical | `804` proposals (`319` with `|S_t| >= 2`), `0` state/epoch mutations | `PASS` |
| **`JR`** | Scratch-clone joint loss `L(S_t)` matches independent sequential forced single-coordinate applies | `319 / 319` multi-coordinate proposals match exactly | `PASS` |
| **`DC` (`V4` + `V5`)** | Decision rule (`S_t` iff `L(S_t) < L({best})`, else `{best}`), commit mask (`V4`), and committed loss (`V5`) | `260` joint wins, `59` fallback-to-best (`joint_loses_or_ties`), `345` single, `140` empty (`804` total); all match rule | `PASS` |
| **`V3`** | One-epoch `apply_block` state matches sequential forced single-coordinate applies in storage, with `epoch` advancing by `1` | `664 / 664` commits (`260` joint commits; `3,418` applied updates) match byte-for-byte in `table`, `routes`, and `gates` | `PASS` |
| **`RF`** | Host and device refusals (empty/oversized block, repeated parameter, mixed kinds, stale epoch, out-of-range lane) | `9 / 9` typed refusals (`1` setup update = `3,419` total applied updates) with state and epoch unchanged | `PASS` |
| **`C1R`** | Stage C1 `V2` K-coordinate scoring regression (`5,130` coordinates across `2,028` blocks) | `5,130 / 5,130` exact parity (`0` mismatches) | `PASS` |

### 2. Resolved Prerequisites for Stage T (`#2831` and `#3179`)

1. **`m1-a8-full-dose-rerun` (`#2831`, merged as `dc5f409b0`) — Qualified Base Lineage & Controller**:
   - The A8 address-structured codebook (`address-structured-v1.json`, rebuilt deterministically from TRAIN via `address-cells-codebook` and pinned in `crates/uor-r4-discrete/src/attention_learning.rs` with `file_sha256 = 6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`, `table_sha256 = 4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74`) under active D40 64-batch whole-interval admission-preserving rollback (`AuditPolicy::registered()`: `cadence = 64`, `margin = 24`, `minimum_baseline = 500`, `reject_whole_interval = true`) passed `B3_init` (`1,351 / 3,847` DEV `P_succ`, `4,244 / 14,287` DEV Full, `1,211 / 3,847` TRAIN anchor) and passed `Q1` (`batch = 4,096`) on `FrozenZero` (`DEV b_3(P_succ) = 1,106 / 3,847`, `DEV b_3(Full) = 4,292 / 14,287`, `DEV top1_correct = 871 / 14,287`, `6.55x` over initial `133` and `+408` over the prior best trained discrete `463 / 14,287`).
   - `#2831` also isolated why `K = 1` single-coordinate updates subsequently hit `STOP_FOR_DIAGNOSIS` (`consecutive_rejections`): scoring a single coordinate per 64-row local batch while auditing the 3,847-row global anchor every 64 batches rolls back the entire 64-batch window whenever one move inside the window breaches the anchor floor. Scoring `K = 16` coordinates per batch and committing verified joint blocks (`L(S_t) < L({best})`) directly addresses this `K = 1` acceptance-and-credit bottleneck.
2. **`m1-w2-code-utilization` (`#3179`, merged in PR `#3193`) — Admission-Matched Next-Word Comparators**:
   - On the stored W2 captures (`caseyallard/uor-r4-store:results/claude/w2-capacity-2026-10-25/record.json`, `69717971a6f4e6f4a6dede6ad5eee630c4abdbda13e4c8dd434ef126a74c1abc`, summarized in `docs/labs/m1-w2-capacity-capture/README.md`, and `caseyallard/uor-r4-store:results/claude/w2-utilization-2026-10-25/record.json`, `d9aa6cd1a9a46876d4e50f1143d481e810fdcfb216beeccf6422ce1b7d70db90`, summarized in `docs/labs/m1-w2-code-utilization/README.md`), full-panel comparators are Unigram `U = 1,226 / 14,287`, Bigram `G = 6,300 / 14,287`, Code table `T = 3,414 / 14,287`, and Q4 Writer `W = 149 / 14,287`.
   - Restricted to the `1,920` writer-admitted rows (`A_W`), the **admission-matched code table (`restricted table`) scores `1,138 / 14,287` (`1,138 / 1,920`)** and the **admission-matched bigram (`restricted bigram`) scores `1,482 / 14,287` (`1,482 / 1,920`)** (`QA = A1a`, `QB = B2`, `QC = C1`).
   - This pins the Stage 2 bar for Stage T (`docs/roadmap/items/m1-bvg-k-block-apply-c2.md` & `m1-bvg-k-stage-t-training.md`): **beat the unigram table (`1,226 / 14,287`), then the admission-matched next-word table (the version W2 produces: `1,138 / 14,287` restricted code table and `1,482 / 14,287` restricted bigram on the baseline, plus the checkpoint-matched restricted tables on the checkpoint's admitted rows `A_W`, not the `6,300` full-panel table), on two seeds (`~$12` budget).**

### 3. How Stage T Covers the Three C2 Uncovered Items

The three items left outside Stage C2's fixture scope (`docs/labs/injective-preflight-and-c2-qualify-2026-10-19/README.md`) are registered in Section 4 of [`m1-bvgk-stage-t-training-2026-10-25.md`](../../harness/prep/m1-bvgk-stage-t-training-2026-10-25.md) as mandatory Step 0 GPU controls before any Stage T training result is read:

1. **`AKd` (BVG-K `apply_block` vs ordinary single-move `Device::apply` kernel)**: Across the first `32` single-move commits (`|commit_ordinals| == 1`), a shadow device restored from the pre-commit snapshot applies the same coordinate via ordinary `Device::apply` (`discrete.cu`); `table`, `routes`, `gates`, `epoch`, and snapshot SHA-256 match byte-for-byte.
2. **`V1d` (`K = 1` identity with today's `transition` law)**: Across the first `24` batches on the production A8 configuration, `transition_block` at `K = 1` on a shadow device matches the `T1` arm's `transition` bitwise in state hash, `epoch`, `accepted_updates`, and schedule cursor.
3. **Driver-level controls (`KId`, `V8d`, `V9d`)**: `KId` verifies `model.epoch() == accepted_updates`, `committed_moves == sum(moves)`, `|commit_ordinals| == moves`, fresh device score equals `committed_loss < l_before`, and all-ineligible batch no-op; `V8d` verifies `FrozenZeroV1` keeps all relation trits `0` with `0` relation ordinals committed; `V9d` verifies restoring the batch-`64` checkpoint and running to batch `128` matches an uninterrupted `128`-batch run bitwise.
