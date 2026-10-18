## 2026-10-18 — A2 completed; R2 running; canonical docs synced (deepseek)

K16-A-seed2 completed **16,404 batches / 410 accepted**, native exit 0 at **22:39:16Z**; report raw
`3d4d17fe…`, final checkpoint `0a8b9556…`, final evaluation `d7976a0b…`; DEV 14,287 rows / 3,167 admitted /
463 top-1 / 1,295 top-8; status `COMPLETE_PENDING_REGISTERED_TWO_SEED_INTERPRETATION`; fresh
`UNAVAILABLE_NOT_RUN`. All four quarter-evaluation references were freshly rehashed. R2 launched once at
**22:40:35Z** (same GPU0, PID 178054) after the exact runtime/config/release/plan/qualification/PM/sheet/
packet pins and host/remote floors passed; lease renewed to 23:40:52Z; no release reissue, reallocation or
retry. Both arms are Route writer-admission objectives, not an attention comparison; Claude's registered
two-seed/conditional-fresh interpretation remains pending after actual terminal inputs.

Canonical NOW/ROADMAP/STATUS/README/docs-labs now state A2 COMPLETE / R2 RUNNING. Docs-only; claim-wording
PASS via the existing checker; no build, no compute, $0.
Record: [labs/a2-r2-docs-2026-10-18](../labs/a2-r2-docs-2026-10-18/README.md).

## 2026-10-18 — State-of-attention audit + PM framing correction (kimi)

Owner question: six months without demonstrated attention. PM-commissioned independent synthesis
maps every attention mechanism with its measurements and status: the bottleneck is the codebook
(16 distinct vectors at init, ties to lowest ID — nothing distinct to select); the read is starved
by construction (20 proposals / 16,404 updates); Flock never corpus-trained; Relation verdict
NOT_RUN; the injective PrimeZeta codebook (4,096 vectors, 0 tie mass) is the measured fix and the
next attention test. PM correction: K16 A/R are writer admission objectives on `Route`, not an
attention arm (#2029 6255963920). Also strikes the stale "zero code for Relation" sentence in
finite-relational-head.md (its 2026-10-17 correction stands).
Record: [labs/attention-state-2026-10-18](../labs/attention-state-2026-10-18/README.md).

## 2026-10-18 — Injective codebook init: sheet FIT after Claude subagent review; plan binds its own sheet (claude)

Under the owner's subagent-review direction, two fresh reviewers returned FIT-WITH-FIXES. All fixes are applied:
- **F1 (HIGH):** attention plans now bind the registered sheet by `codebook_initialization`, so an injective plan can no longer bind the parent's seeded-init table.
- **F3:** the injective scope certifies `distinct == V` and names its table.
- **Bar (i):** now the ranking bar, `top1_control(cp0) + 24`, gated as net G-Cap > max(500, Bar (i)).
- Sequencing, the G-Init controls and the re-pin rule are stated.

`driver_source_sha256` changes, because `attention_learning.rs` is in its include list. The running K16 build is frozen at `21224b44` and unaffected. Lib tests pass 107/107. Training HELD; dispatch waits for D39 W1. No compute, $0.

## 2026-10-18 — BVG-K Stage C is FIT after Claude subagent adversarial review; GPU qualification next (claude; W3)

Under the owner's direction in the Claude chat, three fresh-context reviewer subagents replaced cross-lab FIT for this lane. They returned FIT-WITH-FIXES, and every fix is applied:
- the latency curve now checks every coordinate's status and reads the 2× threshold against the spread;
- V2 must show aliasing power and must exercise partial blocks;
- a pre-registered V2-neg sabotage patch must make V2 FAIL on the pod.

The sheet is FIT. Next: one RTX 5090 via `uor-pod` (owner-approved in chat). It is finite controls only: no training, zero updates, about \$0.75. The driver hash and the rows image are unchanged.

## 2026-10-18 — R1 completed; A2 running; canonical docs synced (deepseek)

K16-R-seed1 completed **16,404 batches / 130 accepted**, native exit 0 at **22:02:38Z**; final report
`2c0ccad5…`, final evaluation `bb6885e8…`; DEV 14,287 rows / 12,089 admitted / 101 top-1 / 515 top-8.
Status `COMPLETE_PENDING_REGISTERED_TWO_SEED_INTERPRETATION`; no paired advantage, KEEP, fresh or mechanism
claim. After terminal/final-hash/GPU/lock/A2-absent/lease and host+remote reserve guards passed, **A-seed2
started once at 22:05:18Z** (same pod/GPU, PID 177458; 2,304/193 at 22:10:18Z), source failure marker absent;
R-seed2 is next serial under its existing release. Host free recovered to ~39.1 GiB (41,025,772 KiB); lease
expiry 23:06:18Z. R1 final cloud custody pending; A1 three-shard custody complete at its scope; four-role/final
late-OP custody still pending. Docs-only; claim-wording PASS via the existing checker; no build, no compute.
Record: [labs/r1-completion-docs-2026-10-18](../labs/r1-completion-docs-2026-10-18/README.md).

## 2026-10-18 — BVG-K Stage C: FIT fixes to the qualifier and both sheets (claude; W3)

This applies the DeepSeek and AGY FIT-WITH-FIXES items for piece 1 and the parent BVG-K sheet:
- **Latency curve:** 1 warmup + 20 timed launches per K, reporting median and nearest-rank p95 with all raw samples kept.
- **V0 preflight:** `rowsk.cu` must begin with `rows.cu`. The receipt records `driver_source_sha256` and the timing scope.
- **V7:** explicit D29 receipts.
- **Parent sheet:** per-family cursors, tie-breaks, and the Stage C split into C1 (scoring) and C2 (apply).

The rows image and the driver hash are unchanged. Host checks are listed in the PR. The **GPU run is still NOT_RUN** pending the FIT flip and the owner's release. No compute, $0.

## 2026-10-18 — R1 live documented; A1 partial DATA custody complete (deepseek)

R1 launched once at 21:26:32Z on the same pod/GPU with the existing admission and frozen 21224b446
source/ELF; PID 176300, source cursor 1,408 completed / 58 accepted / next target 90,112 at 21:29:48Z.
Canonical NOW/ROADMAP/STATUS/README/docs-labs now say R1 RUNNING (not NOT_STARTED) and the ROADMAP M1
Status cell no longer says training NOT_RUN; dated pre-R1 history is preserved. A1 partial DATA cloud
custody completed for the 2109Z snapshot (three compressed shards; 332 members/1,848,283,971 raw bytes;
root join `9576bc73…`); four-role/final late-OP not complete; `safe_to_down=false`. A1's one-arm DEV counts
were 14,287 rows / 2,237 admitted / 65 top-1 / 547 top-8; registered paired interpretation pending, no
KEEP/fresh/pair claim. Docs-only; claim-wording PASS via existing checker; no build, no compute.
Record: [labs/a1-completion-docs-2026-10-18](../labs/a1-completion-docs-2026-10-18/README.md).

## 2026-10-18 — A1 completion synced to canonical docs; #2700 docs claim corrected (deepseek)

K16-A/seed1 completed 16,404/407 (exit 0 20:58:35Z; report `fa863063`, final `6936cf99`, four evaluations
matched), status `COMPLETE_PENDING_REGISTERED_TWO_SEED_INTERPRETATION`. NOW/ROADMAP/STATUS/README/docs-labs
now state the completed-A1 state and R-seed1 host-unblocked/NOT_STARTED. Corrected the earlier #2700 record:
commit `1cf87485` only carried its record/current-state, not the canonical-page edits. No result/KEEP claim;
claim-wording PASS via the existing checker; docs-only, no compute.
Record: [labs/a1-completion-docs-2026-10-18](../labs/a1-completion-docs-2026-10-18/README.md).

## 2026-10-18 — D38 evidence and injective-draft planning-cost provenance (codex)

ROADMAP R3/R5 now name D5-a post-warmup hardware-counter cache/DRAM bytes per token
separately from D5-b measured selected parameter access when the model exceeds
cache, retaining J/token, RSS, tokens/s and D25. Section8 of the unregistered
injective attention draft labels its historical resource-lifetime extrapolations
as assumption-based planning estimates, not measured batch-rate/cost bounds,
under [timing clarification6253995798](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6253995798).
Changed-lineage throughput and all-in cost remain unmeasured. No scientific bar,
registered budget, permission or running-status change; no compute, $0 new work.
Record: [labs/d38-cost-provenance-docs-2026-10-18](../labs/d38-cost-provenance-docs-2026-10-18/README.md).

## 2026-10-18 — BVG-K Stage C piece 1: K-coordinate row-parallel scoring (claude; W3)

This adds opt-in `Device::score_rows_k`, which scores 1..=64 coordinates of one kind in a single launch. It is a separate `rowsk.cu` built from the unchanged rows text plus two kernels, with its own PTX admission and a D29 census.

It also adds `rowsk-qualify`, with:
- V2 per-coordinate parity with `propose_rows` over complete inventories at K ∈ {1, 4, 16};
- V6 refusals;
- a K latency curve that tests the measured occupancy gap (about 130 ms per proposal on 8 of 170 SMs).

The rows image and `driver_source_sha256` are unchanged. Host checks are green: 103/103 lib tests, the stub-CUDA check, clippy, fmt and D29. The **GPU run is NOT_RUN** pending FIT on `m1-bvgk-stagec-rowsk-qualification-2026-10-18.md` and the owner's release. [Record](../labs/bvgk-rowsk-scoring-2026-10-18/README.md). No compute, $0.

## 2026-10-18 — BVG-K (W3) bound to the measured K16 profile and split into construction and training stages (claude)

The batched-verified-coordinates draft's utilization hypothesis is now bound to measured data and the source:
- **Measured.** The accepted K16 timing record (`3efb4768…`) gives about **130 ms per proposal** at 64 rows (about 99.6% of each batch).
- **From source.** The heavy `rows_v1` evaluation launch is `rows × 16 = 1,024` threads in blocks of 128 (`launch_config`). That is **8 thread blocks, so at most 8 of an RTX 5090's 170 SMs** do work.

Scoring K coordinates per launch is hypothesized to raise proposal throughput by up to about K×. The construction stage measures that directly as a K ∈ {1, 4, 16} latency curve.

The sheet now has two separately registered stages:
- **Stage C:** opt-in source plus finite controls V1–V9 and the latency curve. It is configuration-agnostic, about $0.50, and can proceed while K16 runs.
- **Stage T:** the training arms, held until the K16 outcome fixes the configuration of record.

Still a DRAFT (TEST FITNESS: PENDING). No compute, $0.

## 2026-10-18 — D40 timestamp corrected (deepseek)

#2701 merged before review; its D40 text said the owner clarified at a garbled `20:2720:29Z`. Corrected
DECISIONS.md D40 and the D40 record to the verified chronology: operation 20:23–20:24Z; owner ratification
20:32:02Z (`6254801586`). Rule content unchanged; docs-only; no compute.
Record: [labs/d40-timestamp-2026-10-18](../labs/d40-timestamp-2026-10-18/README.md).

## 2026-10-18 — D40: owner-identity phrases are typed by the owner personally (kimi)

The owner ratified the four K16 owner-release/admission records Codex operated at 20:24Z under earlier
authorization (#2029 6254801586: "Yes, ratify; you type next time") — the records stand, the run continues.
Standing rule recorded in DECISIONS.md D40 and AGENTS.md: any owner-identity phrase (`owner-release`,
`completion-owner-release`, `attention-owner-release` and the like) is typed by Casey personally with
`UOR_LAB` unset; labs stage exact commands and verify records only. Docs-only; no compute.
Record: [labs/d40-owner-phrase-rule-2026-10-18](../labs/d40-owner-phrase-rule-2026-10-18/README.md).

## 2026-10-18 — K16 running state synced into canonical docs (deepseek)

#2698 merged pre-start wording; while A1 runs, NOW/ROADMAP/STATUS/README/docs-labs README said
training NOT_RUN. This corrects them to the observed state: four owner/admission pairs created; K16-A/seed1
RUNNING since 20:24:29Z (cursor 1,360 at 20:27Z); other three NOT_STARTED serially; no result implied.
Docs-only; claim-wording PASS; no compute.
Record: [labs/k16-running-docs-sync-2026-10-18](../labs/k16-running-docs-sync-2026-10-18/README.md).

## 2026-10-18 — K16 rows GPU qualification and two 100+1 timing cells observed; custody verified and Claude ACCEPT; owner release, PM dispatch and GO issued, native transport pending (Codex)

Protected source `21224b446db14efc8f9123cf8c43917a5337ea95` passed all 21 construction/identity phases, 25/25 finite controls plus `admit_ptx`, Relation 9/9, rows parity 7/7, G-Pre and C-R1..C-R6. The two owner-selected checkpoint-0 A/R timing cells each completed 100 timed samples plus one excluded warmup, with no recorded faults. Timing and a genuine fresh-init companion share the actual Rust library; the complete comparison against native G-Pre matches every state field/array except each graph's honest compiled-source identity. Original bccc E0063, 13b fixture-inventory and 452 stale-refusal failures remain retained. [Canonical qualification/timing record](../labs/k16-rows-qualification-2026-10-18/README.md).

The full four-root/all-job DATA custody and late operator payload/metadata have verified cloud readbacks. The sealed packet request[6254373730](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6254373730), manifest`e367301901499c125521b15cc05acfc0bc0fa7eaaa4888e072cbe8ac1b1a047a`, includes four DOCUMENT_ONLY A/R×seed1/2 configs and measured timing/budget. Claude's sole delegated [ACCEPT6254417234](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6254417234) followed sealing. The owner's [relayed gated release6254435225](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6254435225) covers exactly the four unchanged trajectories; [Kimi PM dispatch6254457591](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6254457591) is ISSUED_GATED_START; [Claude census PASS6254480425](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6254480425) posts raw/net1,986 on all four actual roles >1,941, clearing the scientific training-start gate. Each role reports16 distinct vectors,16 alias groups×256, tie mass4,096 tokens/12,301 DEV rows; cell metrics are UNAVAILABLE. Published reportSHA`19c053e9779d6ab1dd7b02a5403b6c10910131c5829b51dd7ca361d512f050cf` is verified against the retrieved3,119-byte report. Durable object`claude-k16-cp0-coverage-census-20261018.tar` is16,384bytes/MD5`6687ae6ad16d9a1b56836ec414a57467`; root native fetch83459 exited0, closing locator request6254490769 after Claude correction6254516409. No further census wait remains; root prepares native admission transport, with no training execution claimed. AGY's consumer source findings are closed at helper5c3bc562/README410fa4de under6254038861. The same lease is retained; normal close/final cost remain separate, with a dated rental-proxy cutoff in the canonical record. Training and fresh scoring are NOT_RUN. The 2.3691 GPU-hour checkpoint-0 proposal-work projection is not training-dose throughput; the remaining 5–8 GPU-hour range includes unmeasured allowances and is proposal-only, not measured all-in cost or release. Owner delegation6251237195 assigns acceptance to Claude alone. The actual relayed owner release6254435225 and PM dispatch6254457591 are issued; [Kimi GO6254507680](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6254507680) confirms both gates cleared. Native admission transport remains before start; training is NOT_RUN. D39/NOW is the current plan: interim M1 headline injective codebook + Q4 target admission ≥ 4,894; old reply37/232 and attention count0/3 retain their historical SUSPENDED/unchanged scope. No learned advantage, mechanism negative, KEEP, milestone closure or count reset follows. K16 Amendment7 is prospective for A/R and successors; the in-flight qualification is unchanged.

## 2026-10-18 — Core README and geometry front-door corrections (deepseek)

Fixes the core README's "Active native model" contradiction (now Historical, Cargo does not select it) and
replaces docs/geometry.md's repeated 13-line status paragraph with a concise pre-alpha + D37 sentence,
closing #2682 and #2683. No claims/numbers/mechanism labels changed; docs-only; core check and
claim-wording PASS; no compute.
Record: [labs/core-geometry-docs-2026-10-18](../labs/core-geometry-docs-2026-10-18/README.md).

## 2026-10-18 — ROADMAP D37 permission and M2/M3/M4 statuses corrected (deepseek)

Section 0 now states the D37 learning permission instead of the superseded total D32 float ban, and the
milestone table uses the README statuses (M2 held, M3 paused, M4 held). Closes #2680 and #2681; no other
cells or the SVG touched. Claim-wording PASS; docs-only; no compute.
Record: [labs/roadmap-d37-status-2026-10-18](../labs/roadmap-d37-status-2026-10-18/README.md).

## 2026-10-18 — PROJECT_MAP ops row and heading spacing fixed (deepseek)

Adds `ops/` to "Where work and evidence are stored" with its RAPL/D5-b/Braket caveats and fixes the two
missing spaces in the September headings, closing #2679. Docs-only; claim-wording PASS; no compute.
Record: [labs/project-map-ops-headings-2026-10-18](../labs/project-map-ops-headings-2026-10-18/README.md).

## 2026-10-18 — Mechanism briefs corrected for the aliased-key finding (claude; mechanism-brief lane)

- **`finite-relational-head.md`:** records that every trained Relation run so far started from the 16-vector xorshift codebook, so no run yet isolates the read. It adds a fair-test prerequisite: a distinguishable-key codebook, shown by artifact-level coverage gates (D39 item 2).
- **`prime-zeta-complex-routing.md`:** records the #2692 / #2694 wiring and the census numbers, plus the caveat that injectivity comes from the construction's quota/uniqueness pass (which the nulls share), not from the prime phases.
- **The mechanisms index rows** are updated to match.

Docs only. No compute, $0.

## 2026-10-18 — Injective-lineage attention-runner interlocks (claude)

Following AGY's FIT review (#2029 6253870519), this change adds:
- a distinct `injective-` attention scope for pinned-codebook configs (the seeded scopes are unchanged);
- **G-Init enforcement** in `validate_checkpoint`: before any update, a pinned lineage's checkpoint must carry exactly the pinned table;
- pin-format validation.

100/100 lib tests pass, including a G-Init negative gate, and the stub-CUDA check is clean. The sheet-SHA dispatch and the H1 title land with registration. [Record](../labs/injective-codebook-init-2026-10-18/README.md). No compute, $0.

## 2026-10-18 — Injective-init attention draft revised after DeepSeek's FIT review (claude)

DeepSeek's non-author FIT review (#2029 6253838178) returned FIT-WITH-FIXES, and every fix is applied:
- **New D22 scope and identity pins:** PrimeZeta `pinned_table_v1`, driver `a328e63f…`, with config, lineage, table and binary recorded at release. No parent receipt is inherited.
- **Census framing:** the census comparison is stated as instrument characterization only.
- **New pre-run gates:**
  - an artifact-level **G-Occ**, using the GPU token-cell export plus `coverage-census` (occupied cells > 16, tie mass 0);
  - **G-Cap raw and net of admission**, where net must be at least 500 P_succ rows;
  - an explicit **G-Init** check list with a status-6 negative control.
- **Pre-flight:** the inherited battery is re-run at the new head.
- **Cost on a measured basis:** the earlier 20 min / $2 used a microbenchmark. The fast Dose 1 rerun of this exact path gives at most 1.31 s per batch, so a full dose costs about $11.4 or less and a Q1 stop about $2.8.

The draft stays unregistered (TEST FITNESS: PENDING); re-review is requested from DeepSeek and AGY. No compute, $0.

## 2026-10-18 — Injective codebook initialization wired for new lineages; attention pre-registration drafted (claude; self-directed)

**Change.** `DriverConfig.codebook_initialization` can now pin an exact integer codebook (`pinned_table_v1`). The default is serde-skipped, so existing configs and the K16 packet are byte-identical. The pinned table is checked on the host (schema, spec, length, word digest, recounted distinct vectors), bound into the lineage, and loaded through `Device::from_snapshot`, whose GPU validation admits every word as a root. `prime-zeta-codebook` exports the qualified PrimeZeta table (#2433).

**Measured (host census, no model execution).** The default initializer gives **16 distinct token vectors**, a DEV alias ceiling of 1,986 / 14,287 rows, and 4 of 674 DEV targets that can win a tie. PrimeZeta V4096 B4 gives **4,096 vectors, 256/256 cells, a ceiling of 14,287 / 14,287, all 674 targets, and zero tie mass**. The learned read has never before had distinguishable keys at initialization.

**Pre-registration draft:** `m1-attention-injective-init-DRAFT-2026-10-18.md`. It is the parent FIT'd Relation-vs-FrozenZero attention sheet with exactly one change, the initial codebook, plus G-Cap / G-Init / B3_init pre-run gates. It is also the downstream sheet the PrimeZeta construction sheet deferred.

**Checks and status.** 98/98 lib tests pass, including a negative gate. The stub-CUDA type check is clean. A non-author adversarial review returned APPROVE-WITH-NITS, and all findings are applied or documented. `driver_source_sha256` changes on main. [Record](../labs/injective-codebook-init-2026-10-18/README.md). Training is HELD; the draft needs FIT, PM dispatch and owner release. No compute, $0.

## 2026-10-18  `ops/` evidence indexed in PROJECT_MAP (deepseek)

Adds a provenance paragraph for the `ops/` external-contributor artifacts (RAPL provisional synthetic-stack
receipt, D5-b QUBO memo with no solver, unverified Braket Bell-state receipt) after the current
compiled-navigation table, per the Yazan audit should-do. Docs-only; no code, model or evidence change.
Record: [labs/ops-index-2026-10-18](../labs/ops-index-2026-10-18/README.md).

## 2026-10-18 — Coverage census instrument for D39 item 2 (claude; self-directed)

A new host binary, `coverage-census` (`crates/uor-r4-discrete/src/coverage_census.rs`), reports the coverage metrics that D39 item 2 requires in every qualification packet:
- distinct vectors and the alias-group histogram;
- within-cell tie mass;
- the DEV alias ceiling that feeds the G-Cap gate.

**How it stays D24-clean.** It validates config, corpus and checkpoint through `PreparedDriver::load` and `validate_checkpoint`, then groups and counts stored data only. Cell metrics come solely from the GPU token-cell export receipt, bound to the exact checkpoint; it never reconstructs a cell on the host. This replaces the local Python W0 census, which re-derived cells on the CPU.

**Validation:**
- It reproduces W0 exactly on the same stored checkpoints and 14,287 DEV rows: Q4 has 264 vectors and an 8,948-row ceiling; the initial snapshot has 16 and 1,986.
- 6/6 unit tests pass, and a split-alias negative gate FAILs as required.
- A non-author adversarial review returned APPROVE-WITH-NITS; every finding is applied or resolved from the source.
- `driver_source_sha256` is unchanged.

**Limits.** The GPU exporter admits only attention-learning configs, so Route configs (the K16 redirect) have no cell receipt yet and report cell metrics as UNAVAILABLE. A full positive run awaits a native K16 checkpoint on the host. [Record](../labs/coverage-census-2026-10-18/README.md). No compute, $0.
*Rolling window, updated 2026-10-18 (course-correction PR-D): this page holds entries dated
2026-10-14 and newer. Entries for 13 October back to 3 October are in
[current-state-2026-10-03-to-2026-10-13-archive.md](../history/current-state-2026-10-03-to-2026-10-13-archive.md);
everything earlier is linked under [Older entries and authority](#older-entries-and-authority).*

## 2026-10-18 — Front door: README hero, figure badges, templates, good-first-issues (kimi PM; course-correction PR-C)

The README status block is now the front-door audit's ~110-word hero, followed by a "What works
today, and what does not" table and a contents line; pod, waiver and timestamp detail moved to
`STATUS.md`, where it already lived. `serving-pipeline.svg` is embedded under the M4 boundary, and
`assets/images/hero_banner.jpg` became `historical-concept-art-2077.jpg` so it cannot be mistaken
for a banner. The two dormant float figures now say `HISTORICAL · INACTIVE` instead of
`BUILT & TESTED`, `operators.svg` names "the earlier float trunk (inactive)", and
`milestones.svg` M1 reads "Dose2 stopped at Q1; admission collapse".
`.github/ISSUE_TEMPLATE/config.yml` plus a `question` form and an outside-contributor PR checklist
open the GitHub surface, and five `good first issue`s are seeded (#2679–#2683).

**Checks.** `python3 scripts/check_claim_wording.py` and the Rust claim-wording gate pass;
`cargo fmt --check -p xtask` passes; `cargo run -q --locked -p xtask -- figures` reports every
generated figure current.
Record: [labs/front-door-2026-10-18](../labs/front-door-2026-10-18/README.md).

**Next:** the newcomer queue is #2679–#2683; the README rules-table D35/D36 rows and the
`docs/README.md` / `docs/PROJECT_MAP.md` splits remain open from the audit.

Documentation and assets only — no model compute, $0.


## 2026-10-18 — Plan repair: project-track collapsed to one active plan, per-milestone DoD, rolling current-state window (kimi PM; course-correction PR-D)

`project-track.md` had grown to 1,458 lines carrying **three sections that each called itself the active plan**, a D8 rung ladder written for a substrate D32 froze, and 13 October text under an 18 October heading (audit `docs/labs/course-correction-2026-10-18/audits/roadmap-tracker-coherence.md`, findings C14–C20). It is now 219 lines: the K16-A/K16-R admission redirect as the single active line, the critical path with owners from `NOW.md`, **definitions of done for M1–M8**, the D39 item 4 W1–W4 sequence, and the rules that bind every row. The whole prior file is preserved verbatim in [project-track-archive-2026-10-18](../history/project-track-archive-2026-10-18.md) with five `Superseded` banners — including the pre-D23 Rung-1 text whose step 3 is `a = softmax(-energy / temperature)`, marked prohibited rather than deleted — and its three externally linked anchors kept as stubs.

M1's definition of done is **D39 item 1**: G-Pre ≥ 4,894; G-Adm on both seeds; G-Occ with > 16 occupied cells and an actual R > 0 row; ≤ 0.90 BPB on the chat held-out stream through the D11 export with the export gap reported; a fresh pre-registered transfer panel. The 116/232 reply bar is BLOCKED under R14 and is not a closing condition. M2–M8 quote the acceptance criteria already frozen on `#2030`–`#2036`; no bar is new, and amending one is an owner decision.

`current-state.md` is now a rolling window: the 355 entries for 13 October back to 3 October moved verbatim to [current-state-2026-10-03-to-2026-10-13-archive](../history/current-state-2026-10-03-to-2026-10-13-archive.md), and the page keeps only entries dated 2026-10-14 and newer (233 at this commit, including this entry and three merged the same day). Page lines 8,667 → 2,358. Nothing was deleted, and every moved entry keeps its anchor.

**Checks.** `python3 scripts/check_claim_wording.py` and `cargo run -q --locked -p uor-r4-claim-wording -- .` pass; the project-track archive reproduces its source text exactly once the five inserted banners are stripped and the link prefixes reversed; the current-state archive matches all 355 source entries the same way; `uor-harness check` reports no finding on any of the four files.

**Next:** `#2028`'s milestone table now matches `NOW.md` and D39 — M1 is "rows_v1 successor (Dose 2)" with the interim headline **injective codebook + Q4 admission ≥ 4,894** and the reply bars recorded as BLOCKED under R14. The remaining stale cells (the four status words for M3 and M4, audit C11/C12) belong to the owning milestone issues, not to this page.

Documentation only — no model compute, $0.
## 2026-10-18  README decision range and statistics refreshed (deepseek)

Updates the README owner-decision link range to D0–D39 and replaces the 13 October statistics snapshot
with current verifiable counts (merged PRs, Rust lines, test functions, open issues), noting the 18 October
history consolidation. Docs-only; no code, model or evidence change, no compute.
Record: [labs/readme-stats-2026-10-18](../labs/readme-stats-2026-10-18/README.md).

## 2026-10-18  ROADMAP stale rows corrected for D37, M7, and closed citations (deepseek)

Fixes four Phase 1 tracker-hygiene items in `ROADMAP.md`: the D32 learning paragraph now reflects D37's
offline-float allowance (serving still D11), the M7 row records the #2654 WASM runtime and that the model
bundle is not integrated, and closed `#820` / `#1542/#1476/#1718/#1738` citations are no longer presented
as open. Docs-only; no code, model or evidence change, no compute.
Record: [labs/roadmap-stale-rows-2026-10-18](../labs/roadmap-stale-rows-2026-10-18/README.md).

## 2026-10-18  One-page plan linked from README/STATUS/ROADMAP (deepseek)

Links `docs/integration/NOW.md` from the repository front door, status page and roadmap title block,
closing the D39 Phase 1 tracker-hygiene item. Docs-only; no code, model or evidence change, no compute.
Record: [labs/now-link-2026-10-18](../labs/now-link-2026-10-18/README.md).

## 2026-10-18 — D39: course-correction package approved; NOW.md is the current plan page (kimi PM; owner decision)

The owner approved the PM's consolidated course-correction package in full after six independent DeepSeek audits (preserved in `docs/labs/course-correction-2026-10-18/`). [D39](DECISIONS.md#d39--m1-closing-bar-k16-reachability-gate-post-qualification-sequence-and-lab-self-direction-owner-2026-10-18) records: the M1 definition-of-done (the 116/232 reply bar is BLOCKED under R14, not a closing condition); the G-Cap reachability gate in its corrected form (DEV-row alias ceiling vs Bar (i); the W0 census showed Bar (i) reachable — 16 distinct vectors at checkpoint 0 and 264 at Q4, but alias ceilings of 1,986–8,948 rows; token coverage is aliasing's real cost); injective initializers permitted for new lineages; the W1→W4 post-qualification sequence (K16 run → FIT'd read-code capture → BVG-K discrete speed law → one conditional float sheet); the export-gap disqualification rule (two-seed band, fixed at FIT); the interim M1 headline (injective codebook + Q4 admission ≥ 4,894, so D21 can fire again); lab self-direction with PM redirection; serving-float as reporting-only. New `docs/integration/NOW.md` is the 90-second current-plan page. The K16 sheet carries Amendment 7 (G-Cap + coverage metrics, prospective). The issue template that permitted training matmul is corrected (live D25/D30/D37 violation). GitHub milestones M1–M8 now exist with #2029–#2036 assigned. No hold, FIT or compute is released.

## 2026-10-18 — D38: D5 split into D5-a residency and D5-b sparsity ratified (kimi PM; owner decision on `#2657`)

The owner ratified the D5-a/D5-b split proposed by outside contributor Yazan Barakat in `#2657` ("yes amend the policy as you recommend", in Kimi's PM session, after the outside-collaborator audit recommended explicit ratify-or-reject). **D5-a** is residency — DRAM bytes/token → 0 after warm-up, measurable only with hardware counters; **D5-b** is the original per-token parameter sparsity, binding when the model exceeds cache. Claims must name the half they meet; the existing served artifact stays D5-b-non-compliant (611,814 inspections/step) with D5-a unmeasured. M5 (#2033) now reports D5-a residency counters and, past cache, D5-b selected access. Not adopted: the memo's QUBO formulation (unimplemented; co-access matrix and assignment solver do not exist; the codec SA solves a different problem), and the Braket "QAOA" receipt (a Bell-state parity check, unverified; external paid compute outside `uor-pod` remains unauthorized). Recorded in [DECISIONS.md D38](DECISIONS.md#d38--d5-split-d5-a-residency-and-d5-b-sparsity-owner-2026-10-18); D5 carries an amendment pointer, ROADMAP R3 cites the split, and the memo is annotated. No hold, FIT or compute is released.

## 2026-10-18 — Driver qualifier: stale resume-cursor refusal text repaired (claude; third K16 qualification blocker)

At protected `452ffdd2`, the first finite native job on the K16 qualification pod passed driver controls 1–7. Control 8, `corpus_schedule_resume`, then failed (Codex, #2029 6252236106). The driver correctly refused the corrupted cursor, but the qualifier compared an obsolete message: `driver_qualification.rs:662` expected `…canonical corpus/coordinate cycle`. #2575 had changed the actual text at `training.rs:393` to `…declared corpus/schedule cycle`. This change updates the expected string. Matching on `Invalid`, CUDA error propagation and refusal of an unexpected success are all unchanged.

**Audit.** Every `expect_invalid` message in the five CUDA qualifiers (`driver_`, `flock_`, `relation_`, `rows_`, base `qualification.rs`) now matches a literal or `format!` source string, so no further stale-text blocker is known.

**Effects.** `driver_qualification.rs` is not in `driver_source_sha256`'s file list (`training.rs:21–31`), so the driver hash `9a1b121b…` and the packet pins are unchanged. The stub-nvcc `--features cuda --all-targets` check passes, and fmt is clean.

This is a fixture error, not a CUDA, scientific or mechanism result (D31). **Next:** Codex reruns the qualification on the same retained pod at this head. Then comes Claude's delegated packet acceptance. No compute, $0.

## 2026-10-18 — Restore `docs/wasm-serving/wasm_serve_demo.html` & `ops/ENERGY_MEASUREMENT_PROTOCOL_V2.md`, align `#2654` symbol/path references, and release merged Antigravity claims (antigravity)

Restores [`docs/wasm-serving/wasm_serve_demo.html`](../wasm-serving/wasm_serve_demo.html), [`docs/wasm-serving/verify.mjs`](../wasm-serving/verify.mjs), and [`ops/ENERGY_MEASUREMENT_PROTOCOL_V2.md`](../../ops/ENERGY_MEASUREMENT_PROTOCOL_V2.md) from [PR #2654](https://github.com/UOR-Foundation/uor-r4/pull/2654) (`667db6749`), corrects symbol (`loadStack`/`step`/`saveSession`/`restoreSession`, `UR4SESS1`) and file path (`ops/D5B_ADDRESS_ASSIGNMENT_MEMO.md`, `ops/braket_qaoa_toy_receipt.json`) references in [`docs/labs/wasm-serving-runtime-2026-10-18/README.md`](../labs/wasm-serving-runtime-2026-10-18/README.md) and the `#2654` entry below, verifies `wasm32-unknown-unknown` `+simd128` execution of `swizzle_dot_arm_t`/`dot_dot_arm_m` in Node.js V8 (`rc = 0`, `mul=0, div=0, float=0`), and releases nine merged `antigravity/*` branch claims in `docs/harness/claims.json` and `docs/harness/ledger/antigravity-2026-10.jsonl`. Host-only, `$0` new paid resource; no model/scoring change.

## 2026-10-18 — Fixture-runner kernel inventory synced to the crate (claude; second CUDA-only build blocker)

A stub-nvcc `cargo check --features cuda` on the host found the next K16 qualification build blocker before it reached the pod. #2601 added the `discrete_export_token_cells` kernel to `discrete.cu` and to the crate's own 12-entry PTX inventory. The C-R fixture runner's private inventory (`admission-rank-fixture-runner-2026-10-15/instrument/build.rs:8`) still listed 11, so its build-time PTX admission would reject the real module, and C-R1..C-R5 would not build. This change adds that one entry, so the list matches the crate's exactly.

**Checks (stub-nvcc type check only, no CUDA or model execution):**
- The crate's `--features cuda --all-targets` build passes.
- The timing instrument passes.
- The fixture runner exits 101 before this change and 0 after it.
- The fixture runner's 11 host tests pass, and `fmt` is clean.

No kernel, objective, fixture data or bar change. This is an instrument build blocker (D31), not a result. **Next:** Codex's qualification at this head; then Claude's delegated packet acceptance. No compute, $0.

## 2026-10-18 — K16 CUDA-only Snapshot initializer repair after actual stage0 failure (Codex)

The selected protectedbccc source passed the native CUDA release build, then the standalone timing build failedE0063/exit101 at15:19:20Z before any model command. Two detached CUDA-only Route fixture constructors omitted the required `Snapshot.read_initialization`; this normal repair unit explicitly binds both to existing `ZeroGatesV1`, with no kernel, driver, schema, objective or bar change. Host metadata checks: timing49PASS/0FAIL/1ignored/1same inherited filtered test; fixture11PASS/0FAIL. Actual repaired CUDA builds/qualification remain NOT_RUN; host checks are not CUDA evidence. [Canonical source/failure limits](../labs/k16-rows-timing-source-2026-10-18/README.md). [Actual handoff6251622438](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6251622438). Failed-attempt custody/close and its actual elapsed cost remain pending; current single-pod total quote$0.908/h is not an invoice. No on-lease accepted-source patch, model replay or new allocation is made by this repair. [Owner6251237195](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6251237195) delegates this packet's acceptance to Claude alone, while training dispatch still needs actual owner release. AGY consumer repair stays parallel; no Kimi acknowledgment wait. Reply37/232 SUSPENDED, no model/scoring result or scientific negative.

## 2026-10-18 — WebAssembly D11 integer serving runtime and audit gate (#1172, PR #2654; Yazan Barakat, reviewed/hardened by antigravity)

Delivers the browser/Node.js WebAssembly D11 integer serving runtime (`crates/uor-r4-integer/src/wasm_serve.rs`, `serving-wasm` feature), deterministic `UR4SESS1` session state codec (`crates/uor-r4-integer/src/session_codec.rs`), 4-byte-plane WASM SIMD128 `i8x16_swizzle` dot-product kernels (`crates/uor-r4-integer/src/stack/simd.rs`), and call-graph WASM zero-matmul / zero-mul / zero-float audit gate (`scripts/audit_zero_matmul_wasm.py --serving --call-graph --tap`), plus the offline `xtask qubo-codec` solver (`xtask/src/qubo_codec.rs`) and `README.md` `## Acknowledgements` updates. Review hardening eliminated the remaining `i32.mul` instructions in `with_slot_mut` (`Vec<Option<Box<Slot>>>`) and `stack_recurrence` (`black_box(d)`), made `swizzle_dot_arm_t` exact across all 32-bit `i32` table entries, bounded `session_codec::Reader` slice counts before allocation, and removed generated artifacts from tracked git state. Verification: `cargo test -p uor-r4-integer --features serving-wasm` (165 passed), `cargo test -p xtask` (78 passed), `audit_zero_matmul_wasm.py --serving --call-graph --tap` (7/7 serving symbols PASS: `mul=0, div=0, float=0, unclassified=0`), and Node.js `loadStack`/`step`/`saveSession`/`restoreSession` round-trip (`reloadIdentical: true, run1MatchesRef: true`). Host-only, $0 new paid resource; no language model training or evaluation claim. Record: [labs/wasm-serving-runtime-2026-10-18](../labs/wasm-serving-runtime-2026-10-18/README.md).

## 2026-10-18 — Native SVG composition repaired; independent visual review pending

[Visual repair record](../labs/svg-visual-repair-2026-10-18/README.md) identifies Claude's original mathematical figures and landscape schema in #2057/#2059/#2090/#2298, with before/after embedded-page previews. The architecture returns to restrained horizontal lanes with a separate narrow SVG; mathematical geometry stays intact while zeta annotations and scoped quaternion/600-cell/E8 labels are corrected. This is documentation-only: no model result, scientific gate or owner hold changes, and #2662's timing waiver remains preserved. **Next:** independent inspection of the exact PR head and rendered pixels; keep the PR unmerged until that review, then follow the already-authorized protected delivery path.

## 2026-10-18  Laptop/reap archive verification before pod destruction (deepseek)

Closes the remaining pod-custody hole after #2629: the default laptop tar archive and reap paths
destroyed the pod without verifying the copy, and reap deleted even when archiving failed. Adds a
pod-vs-laptop count + sorted SHA-256 manifest digest check before delete; the ledger records
verified/files/manifest_sha256; reap now retains the pod on archive failure. Dry-run suite 373 passed /
0 failed, including a digest-mismatch retention case. Host-only, no model compute.
Record: [labs/pod-laptop-archive-verify-2026-10-18](../labs/pod-laptop-archive-verify-2026-10-18/README.md).
PR: (added after creation).

## 2026-10-18 — Explicit rows_v1 K16 timing adapter; existing A/R release sequence reconciled (Codex)

Adds the default-preserving counterfactual selector to the standalone K16 timing instrument. Warmup and measured proposals use the production cached rows entry; plan, sample and cell receipts bind the actual path and rows image inventory. Unknown variants and rows with the lanes-only subphase profiler refuse before CUDA.100timed+1excluded, both policies and source-selection validation, synchronization, zero-apply and forward-only rate scope are unchanged. Host checks:49PASS/0FAIL/1ignored with1explicitly filtered inherited reuse-identity test; unfiltered49PASS/1FAIL/1ignored, caused by the historical frozen ordinary-source guard after#2601; no guard relaxed. Formatting/host Clippy pass with inherited warnings; CUDA/model/provider/scorer NOT_RUN. [Canonical source and limits](../labs/k16-rows-timing-source-2026-10-18/README.md). [Owner relay6250410397](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6250410397) renews the existing FIT K16-A/R redirect on rows_v1, seeds1/2, Route/canonical16404each. [PM6250526855](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6250526855): preparation GO; actual qualification, owner-waived two-cell checkpoint-0 A/R100sampletiming and accepted measured D-Packet precede training under the existing conditional release. No repeated strategic ownerack or imported K8/B3 gate. Claude owns R/K16 qualifier; Codex timing/operator/packet; AGY consumer; Kimi acceptance/dispatch. The separately consumed attention Q1 outcome is now independently reviewed as configured admission collapse with no implementation/identity fault found; no mechanism verdict, KEEP, paired advantage, fresh result or reset. Reply37/232 SUSPENDED and attention count0/3 remain. $0 new paid resource.

**Later owner waiver:** [6251027125](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6251027125), delivered in#2662, removes the two Q4 timing cells and importer from the critical path. Qualification uses checkpoint-0 A/R100samples each; scientific gates, four training trajectories and fresh-before-KEEP are unchanged. Source supports that selection already; no new kernel or model execution follows.

## 2026-10-18 — Owner waives the two Q4 timing cells; Relation+R successor drafted (claude)

**Owner decision (in Claude's chat, 14:25Z; relayed at [#2029 6251027125](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6251027125)).** The owner waived the two Q4-occupancy timing cells of the K16 redirect's registered 100-sample timing slot. The slot now times only the checkpoint-0 K16-A and K16-R cells (100 samples each), and the Q4 importer is off the critical path. This is the explicit waiver that the "unless explicitly waived" clause of the immediate-priority entry below anticipated. Arms, bars, G-Pre/G-Adm/G-Occ/G-Guard, outcomes and the fresh seal are unchanged. Packet acceptance and training remain NO-GO until qualification.

**Successor draft.** [`m1-learned-attention-link-R-DRAFT-2026-10-18.md`](../harness/prep/m1-learned-attention-link-R-DRAFT-2026-10-18.md) is the Relation + R successor named by the parent §7.12.4. It is unregistered and has no FIT. A non-author adversarial review returned SOUND-WITH-FIXES, and all six findings are applied:
- an occupancy guard that makes a vacuous B3 pass INVALID;
- registration only on K16-R passing G-Adm + G-Occ, with named withdrawal branches including `K16_A_KEEP_R_NULL_AT_SCOPE`;
- no attribution across sheets or to R alone;
- trajectory rules that forbid selection and require zero-apply;
- per-arm scope strings;
- C-R1..C-R6 as a prerequisite.

A source check (`discrete.cu:125–247`) shows that the read width (8 context nodes) is independent of the writer's admitted cells (K), so no target is hidden from the read. **Next:** the redirect runs; this draft is registered or withdrawn by its outcome. No compute, $0.

## 2026-10-18 — Documentation #2643 protected-delivered; own claim closeout

[PR#2643](https://github.com/UOR-Foundation/uor-r4/pull/2643) squash-merged through the protected queue at `9fd1b8baec6a7586a2595cd4be94f6df77fd1c66`; fresh-main tree `90f743529b2e50ef72ec4120dd22869a3d0249aa` equals the revalidated delivery head. PR CI/harness and merge-group CI succeeded; the only post-merge Pages run is terminal/skipped by existing policy. [Closeout record](../labs/documentation-2643-closeout-2026-10-18/README.md) records exact checks, source identity and preservation. This unit removes only the delivered docs branch's own claim; peer state and research outcomes remain unchanged. **Next:** protected delivery and verification of that claim removal, then owned transport cleanup without deleting unique local evidence.

## 2026-10-18 — Documentation merge reconciliation after R/K16 qualifier delivery

[PR#2658](https://github.com/UOR-Foundation/uor-r4/pull/2658) is now merged at `43bca5e1`; its source-scoped active-R parity and five-sample diagnostic timings remain distinct from the complete new-head qualification, registered four-cell100-sample timing, measured budget and accepted redirect packet. Current STATUS/ROADMAP retain source/packet preparation GO and packet acceptance/training NO-GO under the [existing PM predicates](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6250526855). The documentation branch preserves both upstream qualifier entries and all prior Dose2/direction records; no source or scientific-gate change is authored here. **Next:** deliver the reviewed documentation through the protected merge queue, verify main and its applicable CI, then release this documentation branch's own claim after verified preservation.

## 2026-10-18 — Active rank-hinge (R>0) row-parallel parity PASS (claude; PM audit fix)

Kimi's audit found my K16 R parity vacuous: at ≤16 occupied cells, R = 0 everywhere. Attempt 5 adds a 64-cell fixture where **46/64 rows have R > 0**. Lanes-vs-rows parity under R holds on **134/134** Route/VectorBlock parameters, objective A differs on all 134 (the wrong-objective gate), and every earlier control passes again; the negative gate FAILs as required. At `3af49e029` this meets the R/K16 qualifier criteria. The final head adds only AGY P1/P2 error and state hardening, and it runs on GPU in the single new-head qualification session (PM item 4). The 5-sample timing is diagnostic; the registered 100-batch K16 slot is #2659. [Record](../labs/counterfactual-rows-parity-2026-10-18/README.md). About $0.20.

## 2026-10-18 — Row-parallel parity under the rank-hinge objective at K16 PASS; redirect budget ~1.2 h (claude)

Attempt 4 (RTX 5090) adds `rows_rank_objective_parity` (Route, K16, R and A, complete inventories): **1,368/1,368 PASS**, all earlier controls pass again, and the negative gate FAILs as required. K16 timing gives K16-A 7.40 s → 0.130 s and K16-R 7.40 s → 0.129 s (about 57×, equal losses). The owner-approved fast-path admission redirect (6250410397) therefore has GPU parity under its own objective and shape, and a measured budget of about 1.2 h for the four runs. [Record](../labs/counterfactual-rows-parity-2026-10-18/README.md). About $0.25. **Next:** Codex's redirect packet on `rows_v1`, then Claude's review and owner dispatch.

## 2026-10-18 — Immediate priority: prepare and qualify the existing K16 A/R redirect on rows_v1

The [owner direction6250410397](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6250410397) and [PM gate audit6250526855](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6250526855) prioritize the existing FIT'd K16-A/K16-R admission redirect on `rows_v1`, retaining Route/K16, seeds1/2, the registered16,404-batch schedule and scientific bars. **Source/packet preparation GO; packet acceptance and training NO-GO** pending exact-head qualification, measured timing/budget and an accepted regenerated D-Packet under the existing conditional release. Required evidence includes same-device/same-state/same-R-objective lanes/rows parity with **>16 occupied cells and an actual R>0 row**, 25/25 plus `admit_ptx`, G-Pre, C-R1..C-R6, refusals/terminals, negatives and zero apply; all four checkpoint-0/Q4 × A/R timing cells retain100 samples each plus one excluded warmup unless explicitly waived. Author [PR#2658](https://github.com/UOR-Foundation/uor-r4/pull/2658) results are not accepted readiness ([acknowledged limits and revised qualification6250595150](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6250595150)); the separate K8 Relation/R successor remains deferred planning, not an added K16 gate. **Next:** prepare and qualify the packet, then obtain measured packet acceptance before training; fresh execution remains before KEEP, not a training-start gate, and this documentation update launches nothing.

## 2026-10-18 — Dose2 scientific review: configured admission collapse in both arms

[Designated outcome-seat review6248429872](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6248429872) confirms the registered Q1 admission-collapse stop after **4,096 batches in each arm**: P_succ target admission **2,034 → 225 treatment /205 control**, not accuracy or a treatment win. No implementation or identity fault was found; no registered efficacy hypothesis fires, fresh remains sealed, and there is no KEEP or mechanism-negative verdict. A shared vector/combine pathway is source-supported and plausible, but its causal role is not established. Pod56510253 was released and down at **07:53:30 UTC**; [scorer compatibility#2652](https://github.com/UOR-Foundation/uor-r4/pull/2652) and [diagnostic publication#2653](https://github.com/UOR-Foundation/uor-r4/pull/2653) are merged. The [dated operational/custody record](../labs/attention-rows-q1-diagnosis-2026-10-18/README.md) retains its pre-review snapshot; this entry records the subsequent scientific review. **Next:** prospective next-configuration registration and owner decision under the existing sheet, with no resumed dose, retry, scoring or compute release implied. Count0/3 and reply37/232 SUSPENDED remain unchanged.

## 2026-10-18 — Both rows learning arms reached Q1 diagnostic stops; complete custody and own close verified (Codex)

At protected source`20d158ae` and actual RELEASE ELF`e7cb6197`, both paired arms reached **4,096 contiguous batches / Q1_STOP_FOR_DIAGNOSIS / exit0** under [owner release6246440910](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6246440910). P_succ target admission fell from2,034/3,847 initially to **225 treatment /205 frozen-zero**; both registered admission predicates fired with nonzero combine/relation sensitivity. These are admission counts, not accuracy or a paired advantage. Genuine [operational audit6247115698](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6247115698) verifies allthree registered native jobs terminal0, source/build/controller identities and empty GPUs; it does not replace the pending Claude scientific outcome review. Full DATA member SHA/same-reader/cloud MD5 and complete late-operator cloud readback preceded own `vast-56510253` down **07:53:30Z**; the separate after-down closure also passed full readback. [Canonical result and sanitized evidence](../labs/attention-rows-q1-diagnosis-2026-10-18/README.md); [actual sealed handoff6247444829](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6247444829). Both releases remain consumed; final16,404/map/224capture/fresh/scorer remain NOT_RUN, with no mechanism negative, KEEP or reset: **count0/3, reply37/232 SUSPENDED**. AGY's typed consumer repair is delivered in#2652 and source/metadata-reviewed at [6247409879](https://github.com/UOR-Foundation/uor-r4/pull/2652#issuecomment-6247409879); its diagnostic template does not license full scoring. **Next:** genuine scientific outcome review, then the existing prospective configuration/owner decision in sheet7.12.4; the conditional R source-dependency map is preparation, not a new objective, FIT or compute release. This documentary unit runs no model; the separately accounted resource proxy is$2.820481667, not an invoice.

## 2026-10-18 — Typed `rows_v1` `DriverConfigRecord` schema compatibility in `attention_reference.rs` (antigravity)

Adds `CounterfactualPath` (`LanesV1` default-skip, `RowsV1` explicit, `#[serde(rename_all = "snake_case")]`) and `#[serde(default, skip_serializing_if = "CounterfactualPath::is_lanes_v1")] pub counterfactual_path: CounterfactualPath` between `relation_learning_policy` and `seed` on `DriverConfigRecord` in `crates/uor-r4-harness/src/attention_reference.rs`, resolving the typed `rows_v1` lineage and Domain C `DriverCheckpointRecord` schema finding from [#2029 `6246867414`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6246867414) without weakening `#[serde(deny_unknown_fields)]` or altering legacy `lanes_v1` canonical bytes (`c446aa7f…` / `1435f016…`). Unit tests verify exact producer `rows_v1` `config_sha256` (`c194cd6e4f3e5b127e6ef5dbe3294aa893a205518ed62a632da7236e57c72228` / `15a10bac9eb04bb84daaa71f9c580cfaa2e28151f05616da768aa9fc784d866f`), `lineage_sha256` (`f1403a8894563c99706b69fe501e066f1ca62951ddef5d870bf8df409d491a53` / `480c5bafd3a07ddfafd02bde47f71bfbce3544e677da0c7e289158ac4a2210c9`) against `consumer-panel-rows-treatment-v1/report.json` (`7f59e7c4…`), 4-domain checkpoint verification (`verify_driver_checkpoint_domains`), `paired-dev` arm path consistency, and unknown variant/field rejection. Both `rows_v1` Dose 1 arms reached `Q1_STOP_FOR_DIAGNOSIS` at batch `4,096` ([#2029 `6247068871`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6247068871)); no full-dose scorer run, `KEEP`, mechanism-negative or count reset is claimed (`$0`, host-only). Record: [labs/rows-v1-consumer-config-schema-2026-10-18](../labs/rows-v1-consumer-config-schema-2026-10-18/README.md).

## 2026-10-18 — Wire route counters accumulate across connections (kimi)

Fixes the original #2613 P2: `WireGateway._handle` overwrote the persistent route bucket's `wire`
bytes with the latest completed connection while `plain_in`/`plain_out` accumulated, understating the
documented per-route ratio on reused routes. One-line change to `+=`; adds
`spiralcore-base/wire/tools/wire_socket_stats_smoke.py`, which completes two connections with
distinct wire counts on one route and asserts the bucket equals both sums and the aggregate ratio
(negative control against the original source fails with the last-only counter). The branch is
forward-merged with fresh `origin/main` (`58757e219`) without rebase; `WIRE_SOCKET_ROUTING_PASS`, the
new `WIRE_SOCKET_STATS_PASS`, py_compile, claim wording and `git diff --check` pass at the merged
head; host-only, no model compute, pods, training or evaluation. Claim:
`kimi/wire-counter-aggregation-2026-10-18`; protected PR [#2651](https://github.com/UOR-Foundation/uor-r4/pull/2651) for exact-head review, not queued or merged.
Record: [labs/wire-counter-aggregation-2026-10-18](../labs/wire-counter-aggregation-2026-10-18/README.md).

## 2026-10-18  Seventh DeepSeek claims cleanup (deepseek)

Removes the merged #2649 claim and the cleanup branch claim from docs/harness/claims.json. Coordination only; no code/model/evidence change.

## 2026-10-18  UOROBJ01 non-Rust conformance run (deepseek)

Runs the pure-Python port_agent.py peer against the Rust objpeer server: stale audit (exit 3), pinned
pull byte-identical to the 256 KiB artifact, warm audit verified (exit 0), all through the wire
protocol with no Rust client involved. Adds a reproducible smoke script and transcript, closing the
backlog ask for a non-Rust UOROBJ01 verifier proof. Host-only, no protocol/model change.
Record: [labs/uorobj01-conformance-2026-10-18](../labs/uorobj01-conformance-2026-10-18/README.md).
PR: (added after creation).

## 2026-10-18 — Original Dose 1 owner-stopped after 904/1,065 batches; custody verified; rows_v1 successor released separately (kimi)

The [dispatch record](../labs/attention-dose1-dispatch-2026-10-18/README.md) binds the actual PM receipt [#2029 `6244964455`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6244964455) and `DISPATCH_VALID` [#2029 `6244978507`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6244978507): private `dispatch.json` `4f7681690f0ded838f8793a64117f3fdd96f2bf5464b2803730eb12f4fa543bb`, static manifest `5d5481cd6c19513c316d1f7144257bcf623b0c660afb0fb9c34f9052c0fb7c44` (234/234 read-only `shasum -c` pass), AGY verdict bytes `e3209a2a…`, source `23d5b146`, sheet `84481400…`, driver `9538f5c3…`, FIT `6230317153`, owner resolutions `6236444507` and attempt `uor-cf1-dose1-ac60c495ec8d601b267319b8cf548910`. Allocation is **CONFIRMED** at `vast-56488188` (`runtime-allocation.json` `1f1ed8db…58515`, two GPUs `[0,1]`, quote USD `1.893688888888889`/h total, a quote not an invoice, storage/transfer `UNMEASURED`); the RELEASE build is **metadata verified** (`RELEASE_BUILT_METADATA_VERIFIED`, exit 0, shared `8,099,440`-byte ELF `1f7dd526…dbb3`, source `64e4846374…`, kernel `5cd0e732…`, geometry `46ddbb76…`, no CUDA context); AGY approved **both** arms (`6245205468`), finalize exited 0 with final typed plans `e8d25b21…0297`/`a8b76562…8de40`, and both `attention-check-plan` entries are `PLAN_METADATA_VALID` with `owner_release` then `STILL_REQUIRED`. The owner then delegated the launch ([`6245307092`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245307092)) and both arms started at 03:48:36Z/03:48:38Z ([`6245367341`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245367341)). The owner chose **“Stop it; rerun on the fast path”** ([`6245711750`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6245711750)); the run stopped after contiguous **904 treatment / 1,065 frozen-zero batches**, both below Q1 4,096, with native ends `04:58:38Z`/`04:58:40Z` and raw `COMMAND_FAILED_TERMINAL`/exit 143, independently attributed **OWNER_STOPPED_STRATEGIC** — not source failure, mechanism negative, Q1 diagnosis or KEEP. **Q1 NOT_REACHED_BOTH; dose NOT_COMPLETED; final maps/224 capture/fresh NOT_RUN; scientific verdict NOT_ISSUED**; the original dispatch is consumed. Custody was accepted ([`6246074493`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6246074493)): 198 regular members/237,872,396 bytes; data `25,345,536`/MD5 `ec029a201130ab41b3cd89b3cc6355de` and late operator `143,634,944`/MD5 `a75a23a70ab438e46f399a96df39ad8e` verified before teardown; down 05:16:47Z; after-down closure `4,026,880`/MD5 `fb4a4a264408c44ae61fbad5559a7e05` read back 05:23:37Z; local closure record `c5ca0f6a…8274e`. The separate `rows_v1` successor was owner-released ([`6246440910`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6246440910)) with packet `01833fa7…`, source `20d158ae…` and new pod `vast-56510253`; its outcome is not claimed here. No result, completed dose or count reset follows; `$0` for this record. **Next:** the successor proceeds under its own release; this attempt remains consumed.
## 2026-10-18  Sixth DeepSeek claims cleanup (deepseek)

Removes the merged #2646 claim and the cleanup branch claim from docs/harness/claims.json. Coordination only; no code/model/evidence change.

## 2026-10-18  Fleet maintenance demo: audit, failover sync, repair (deepseek)

Adds tools/fleet_maintenance.sh, the audit -> failover -> repair demo the backlog requested on real
artifacts: two loopback replicas with a partitioned target set; verify audit (4 checks, stale/unknown),
failover sync with a dead first replica (3 artifacts from the survivor), repair of the missing target
(current=3, repaired=1, unavailable=0), and a final union-coverage audit (every target verified on at
least one replica). Fail-closed JSON assertions; host-only, no protocol/model change.
Record: [labs/object-store-fleet-maintenance-2026-10-18](../labs/object-store-fleet-maintenance-2026-10-18/README.md).
PR: (added after creation).

## 2026-10-18 — D37 harness propagation: prep rules, PM rules, plan and optimization record (kimi)

Mechanically propagates the merged [D37](DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised) owner decision into the harness and current-authority documents; no new permission and no reinterpretation. `docs/harness/milestones.json` now scopes the D32 summary to the active discrete path/Dose 1 and adds an all-scope D37 rule sourced to `DECISIONS.md`; `crates/uor-r4-harness/src/prep.rs` and `scripts/harness/prep.py` both carry ten applicable rules including D37. `docs/harness/pm-rules.md` R5 pins policy schema `/7` and the serving float/transcendental/multiplier false commitments; R13 governs the active discrete path only, with a separate D37 offline float lane under R12/D31 that cannot weaken discrete exact instruments; that lane must remain geometric (SpiralCore/E8/FBS, icosian/quaternion, prime/zeta) and use its own scanned allowlist entry, added/reviewed with the pre-registration and never widening the discrete crate scan. `.kimi-code/AGENTS.md`, `docs/integration/project-track.md` and the [optimization path record](../labs/optimization-path-2026-10-18/README.md) now state offline float is legal only under D37 in such a separately scanned geometric module; dense matrix products, squashing, D11 serving, Dose 1 and all holds are unchanged. No model compute, pods or source-behavior change; $0. Checks: prep tests PASS, parity PASS under Python 3.12 (the system Python 3.9 reproduces only the pre-existing brief-receipts mismatch), `uor-harness check` exit 0 (0 errors; 87 + 1 at review, 86 + 1 pre-merge, 79 + 1 merged; live peer state), `prep-render` contains D37 and no stale D25-supersession sentence, policy check PASS, ratchets d23/d25/d32 at baseline, claim wording, fmt and `git diff --check` PASS. Record: [labs/d37-harness-propagation-2026-10-18](../labs/d37-harness-propagation-2026-10-18/README.md). **Next:** independent exact-head review and protected PM delivery; Dose 1 dispatch/decision remains the path dependency.

## 2026-10-18 — Documentation front door and geometry reconciled to current source (Codex)

The README, documentation hub, contributor navigation and active discrete-crate guide now distinguish selected M1 computation from retained research and separately qualified D11 serving. The [source-linked audit](../labs/documentation-reconciliation-2026-10-18/README.md) corrects dimensions, quaternion/norm and icosian/E8 explanations, K-versus-scan cost, figure labels and broken links, while preserving dated evidence. Live #2029 review/release and the 06:24 successor pod-up receipt supersede the earlier awaiting-review snapshot; no successor scientific outcome follows. Documentation/host checks only; no model behavior change, training or paid resource action. **Next:** independent draft-PR review, then refresh live status before delivery.

## 2026-10-18  Fifth DeepSeek claims cleanup (deepseek)

Removes the merged #2641 claim and the cleanup branch claim from docs/harness/claims.json. Coordination only; no code/model/evidence change.

## 2026-10-18  Cold-join benchmark: tools/cold_join.sh (deepseek)

Adds the end-to-end cold-join benchmark requested by the join record: cold store pull vs gzip
download + local decompress + pinned join, then a warm delta, with real PULL_STATS/JOIN fields and
fail-closed checks. Host-only loopback, 2 MiB single runs: text cold pull 33,777 B vs gzip 7,223 B
(warm delta 252 B / 0 nodes); random cold pull 5,734,720 B vs gzip 2,097,823 B (ratio 1.00032),
confirming the earlier incidental cold-pull figure. No protocol change, no model compute.
Record: [labs/object-store-join-2026-10-17](../labs/object-store-join-2026-10-17/README.md).
PR: (added after creation).

## 2026-10-18 — Post-dose drafts: batched verified coordinates (BVG-K) and decode tokens/s baseline (claude)

These two DRAFT prep sheets are not registered: no FIT is requested and no compute runs ($0). Each is to be registered after the fast-path Dose 1 decision. [BVG-K](../harness/prep/m1-batched-verified-coordinates-DRAFT-2026-10-18.md) is path step 3, the batched-coordinate learning-law change. Each batch scores K same-kind coordinates in one row-parallel launch and commits the better of the jointly verified set and the single best move, so each batch's loss can only fall. K = 1 is a bitwise rollback identity. The read share is preserved per coordinate, and the cursor gains a coordinate ordinal plus `committed_moves`. Arms are K = 1/4/16 at equal wall clock, with target-tokens/s as the lead metric. It was revised after an adversarial reviewer-subagent round (10 findings). [Decode tokens/s](../harness/prep/m1-decode-tokens-per-second-DRAFT-2026-10-18.md) is the optimization rung's missing baseline (owner metric: decode speed against context), drafted for Kimi's measurement-plan seat. **Next:** fast-path Dose 1 outcome; then register C1, BVG-K and decode against its decision.

## 2026-10-18 — K16 draft source preserved inactive; no timing prerequisite (Codex)

The unique draft2571 count-hardening source at `a3d8ca2d38f5fb2925e02ef7e7c09c8ead98323a` is retained as an exact six-file inactive patch plus metadata/INDEX row; isolated-index application to based325 restores the entire original tree `9e7208093cbd950a64ea2d31e37a07a760cc0f31`. Source-only review6229329805 remains distinct from construction/tests, the unresolved100-batch bridge and timing/model work, all NOT_RUN; historical disk shortage is not a current blocker or new programme. No active instrument, frozen timing3/binary, scientific bar or count changes: **count0/3, reply37/232 SUSPENDED**, no KEEP/negative/reset. [Canonical inactive record](../labs/k16-inactive-archive-2026-10-18/README.md). **Next:** protected archive delivery/restoration proof, then close2571 and clean only its preserved owned transport; this does not block the rows successor or issue a timing release.

## 2026-10-18  Fourth DeepSeek claims cleanup (deepseek)

Removes the merged #2638 claim and the cleanup branch claim from docs/harness/claims.json. Coordination only; no code/model/evidence change.

## 2026-10-18  Object-store pipelined FETCH window (deepseek)

Follow-up to #2607: `pull` was strict lock-step per chunk, so batch tuning still left one RTT per
chunk. `pull_one` now pipelines FETCH frames in a bounded window (default 4; window 1 is the old
lock-step path), exposed as `objpeer --fetch-window 1..=64` on pull/sync/repair and rejected
elsewhere. Host-only delay-proxy matrix: 3.706 s lock-step vs 1.902 s window 4 vs 1.324 s window 8 at
batch 128, byte-identical output; package 61 passed / 0 failed, clippy clean. No protocol change, no
model compute. Record: [labs/object-store-pipelined-fetch-2026-10-18](../labs/object-store-pipelined-fetch-2026-10-18/README.md).
PR: (added after creation).

## 2026-10-18 — Original Dose1 strategically stopped; partials and closure independently accepted (Codex)

After reviewed rows-driver delivery, the owner's strategic stop ended the original source23d5 paired dose at **904/1065 batches**, before Q1; both raw wrapper/native exits143 remain preserved with separate OWNER_STOPPED_STRATEGIC provenance. Full original-data and late-operator SHA/cloud custody preceded own down at05:16:47Z; the separate after-down closure is also verified, with genuine independent actual-outcome approval [6246074493](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6246074493). No completed dose, final device maps/224 captures, fresh scoring, KEEP, mechanism negative or count reset follows: **count0/3, reply37/232 SUSPENDED**. [Canonical result and sanitized evidence](../labs/attention-dose1-strategic-stop-2026-10-18/README.md). **Next, updated after the separate successor:** source20d's1376-member/01833fa7 packet was owner-released and reached [paired Q1 diagnostic stops](../labs/attention-rows-q1-diagnosis-2026-10-18/README.md), with its own custody/closure complete and scientific outcome review pending. This does not reinterpret the original904/1065 partials or reuse either release.

## 2026-10-18  Third DeepSeek claims cleanup (deepseek)

Removes the merged #2635 claim and the cleanup branch claim from docs/harness/claims.json. Coordination only; no code/model/evidence change.

## 2026-10-18  Object-store batch plumbing for sync/repair (deepseek)

Follow-up to #2607: objpeer --batch was silently ignored by sync and repair (pull_series hardcoded
the default). Added pull_series_with_batch, passed batch through sync/repair, rejected flags a
subcommand ignores, and listed the real subcommands in usage. Tests: sync --batch and repair --batch
end-to-end plus three rejection cases; package 60 passed / 0 failed, clippy clean. Host-only, no model
compute. Record: [labs/object-store-batch-plumbing-2026-10-18](../labs/object-store-batch-plumbing-2026-10-18/README.md).
PR: (added after creation).

## 2026-10-18  Final DeepSeek claims cleanup (deepseek)

Removes the two merged delivery claims (#2629, #2633) and the cleanup branch claim from docs/harness/claims.json. Coordination only; no code/model/evidence change.

## 2026-10-18  Wire matrix reproducibility and claim correction (deepseek)

Fixes evidence defects at the edge of the merged wire side exploration: the three adapter legs now
resolve objpeer repo-relatively (or via OBJPEER) and fail with the build command instead of hardcoding
the owner checkout; the matrix preflights objpeer and gains --list (18 legs); and the 15/15 vs 13/13 vs
18/18 counts plus the absent-tool VERIFY_ALL_PASS/EVIDENCE_OK claims are corrected. Re-measured full
matrix on main 03d82fb7b: WIRE_MATRIX_RESULT pass=18 fail=0. Host-only, no model compute.
Record: [labs/wire-matrix-repro-2026-10-18](../labs/wire-matrix-repro-2026-10-18/README.md).
PR: (added after creation).

## 2026-10-18  Pod store read-back before down (deepseek)

`uor-pod down --to-store` now reads every Vast result upload back before it may destroy the pod:
file count plus a SHA-256 manifest sidecar must match, otherwise down aborts with the pod retained.
Dry-run suite 368 passed / 0 failed, including an injected short-upload failure case. Host-only,
no pods or model compute. Record: [labs/pod-store-readback-2026-10-18](../labs/pod-store-readback-2026-10-18/README.md).
PR: #2629.

## 2026-10-18 — Row-parallel parity attempt 3: complete inventories PASS (claude; Codex findings fixed)

Codex found two problems in the parity suite: the racy-refusal retries discarded CUDA faults and other outcome classes, outside the state check, and the Route-mode sweep covered only indices `0..blocks`. Both are fixed. Attempt 3 on a fresh RTX 5090 passes **1,080/1,080** cases, including all 684 Route/VectorBlock inventory cases, plus refusals and forced terminals; the negative gate FAILs as required; timing reproduces at ~56×. Attempts 1–2 keep their subset scope, and the record notes that #2630 merged before the findings were read. [Record](../labs/counterfactual-rows-parity-2026-10-18/README.md). About $0.25. **Next:** Codex re-issues fast-path Dose 1 citing attempt 3.

## 2026-10-18 — Training driver: opt-in `counterfactual_path = rows_v1` (claude)

`DriverConfig.counterfactual_path` now routes `propose_next` and `transition` through the row-parallel exact counterfactual from #2630 (identical losses, ~56× faster). The default `lanes_v1` is never serialized, so existing config hashes are unchanged; driver source identity changes for new runs. This is the prerequisite for the owner's decision to stop Dose 1 and rerun it on the fast path. [Record](../labs/driver-rows-path-2026-10-18/README.md). No compute, $0. **Next:** Codex stops Dose 1 under its own lease and re-issues the packet with `rows_v1` on this head; Claude takes the review seat; the owner releases dispatch.

## 2026-10-18 — Row-parallel exact counterfactual: GPU parity PASS, ~56× faster proposals (claude)

The opt-in row-parallel `discrete_counterfactual` (owner: "Build now, Dose 1 runs on") passes finite GPU parity on an RTX 5090: 468/468 sweep cases (Route, Flock and Relation Gate), refusals, a forced status-8 batch and a forced lane break all match the ordinary kernel exactly, and the negative gate FAILs as required. At the Dose 1 shape (V4096/B4/C192/K8, 64 rows) a proposal takes 69–73 ms against 3.8–4.1 s, about **56–57×**, with identical losses. That derives to ~20 min per Dose 1 arm instead of ~19 h. The default path is unchanged and nothing is wired into training yet. About $0.30; [record](../labs/counterfactual-rows-parity-2026-10-18/README.md). Dose 1 is untouched. Training/M2/Stage B stay HELD. **Next:** opt-in `counterfactual_path` in the training driver (separate PR), then the owner decides whether to re-run Dose 1 on it.

## 2026-10-18  Delivered M1/M4/harness branch claims cleanup (antigravity)

Removes the stale committed branch claims for merged PRs #2565, #2573, #2604, #2605, #2611, #2617, #2618, #2620, and #2623 (`antigravity/*`, `kimi/optimization-path-2026-10-18`, `codex/cf1-attention-hash-domains`, `codex/cf1-attention-dose-packet`, and `claude/*`) from `docs/harness/claims.json` per `#2029` comments `6245248627` and `6245449740`. No code, model, or evidence change; coordination record only. Record: [labs/attention-cell-reference-scorer-2026-10-17](../labs/attention-cell-reference-scorer-2026-10-17/README.md).

## 2026-10-18  PM coordination hygiene: stale DeepSeek claims removed (deepseek)

While Kimi is rate-limited, DeepSeek is covering PM coordination. This removes 25 stale committed
claims for already-merged DeepSeek branches (object-store series and wire follow-ups), removes the
orphan local branch flagged by uor-harness check, and records the PM coordination note. No code,
model or evidence change; no other lab's claims or files touched.
Record: [labs/pm-coordination-2026-10-18](../labs/pm-coordination-2026-10-18/README.md).

## 2026-10-18  Fork-safe interposer claims cleanup (deepseek)

Removes the stale committed claim for the merged fork-safe interposer branch (#2625) and this cleanup branch from docs/harness/claims.json. No code, model or evidence change; coordination record only.

## 2026-10-18  Interposer fork-safe close (wire side exploration; deepseek)

Closes the Codex P1 residual on the merged wire side exploration: close no longer calls
shutdown(SHUT_RDWR) (which disturbed fork-shared sockets). Receive is non-blocking chunked
(MSG_DONTWAIT) with an atomic active re-check; recv/send/shutdown carry a per-fd generation guard;
atfork child lock re-init and child skip-free windows; fork test covers close-wake and reused-fd
connect, with an unsafe-build negative control that must exit 7. Checks: WIRE_INTERPOSER_PASS and
WIRE_INTERPOSER_LINUX_PASS, both fork_ok=true negative_control=true. Trade-off documented (idle
20 Hz polls; close can wait for current-frame decompression). No model compute.
Record: [labs/spiralcore-wire-interposer-fork-safe-2026-10-18](../labs/spiralcore-wire-interposer-fork-safe-2026-10-18/README.md).
PR: https://github.com/UOR-Foundation/uor-r4/pull/2625.

## 2026-10-18 — Claim-wording gate ported to Rust (claude)

The claim-wording gate is now the `uor-r4-claim-wording` crate (D27): run `cargo run -q --locked -p uor-r4-claim-wording -- .` or `uor-harness claim-wording`. It is a line-for-line port of `scripts/check_claim_wording.py`. Its stdout and exit code are byte-identical to the script's on this repository and on a violation probe; the crate has 13 tests. The CI steps, the PR template, AGENTS/CONTRIBUTING, pm-rules, formal vocabulary §6 and the prlint checklist label now call the Rust gate. CI coverage is unchanged, including docs-only diffs in `fast-gate`. The Python script is retired, not deleted. [Record](../labs/claim-wording-rust-port-2026-10-18/README.md). No compute, $0. **Next:** none on this line; the retired script stays as dated evidence.

## 2026-10-18 — C1 DRAFT sheet shape repaired (claude, Kimi's repair)

The C1 DRAFT (`docs/harness/prep/m4-c1-flock-coupling-DRAFT-2026-10-18.md`, #2620) had no D22 blocks and no verdict line. That failed `prep.sheet_shape` and `prep::tests::the_real_sheets_are_readable` on main. This applies Kimi's repair (`9c60bb154`, which Kimi had not pushed before going offline): the eight blocks are marked NOT_REGISTERED/NOT_RUN at $0, plus `TEST FITNESS: PENDING`. Nothing is registered, and no bar, control, source or hold changes. `uor-harness check` reports 0 errors, and the test passes. **Next:** merge-forward #2623 (claim-wording Rust port) onto this.

## 2026-10-18  Final wire claims cleanup (deepseek)

Removes the stale committed claims left by the merged wire side-exploration PRs (#2613, #2614, #2619, #2621) and this cleanup branch from docs/harness/claims.json. No code, model, evidence or dispatch change; coordination record only.

## 2026-10-18  Wire review fixes round 3 (Codex finding on #2619; `deepseek/spiralcore-wire-review-fixes-3`)

Codex reviewed #2619 at `5d6ecdd62` and reported one P1: a blocking `recv` can remain asleep on Linux
after a close, so a later connect on the reused fd could wait forever on `io_lock`. `close` now issues
`shutdown(fd, SHUT_RDWR)` after marking the fd inactive and before `sys_close`, waking the receive; buffer
reset still happens on the next connect. Checks: `WIRE_INTERPOSER_PASS`, `WIRE_INTERPOSER_LINUX_PASS`; no
model compute.
Record: [labs/spiralcore-wire-review-fixes-3-2026-10-18](../labs/spiralcore-wire-review-fixes-3-2026-10-18/README.md).
PR: https://github.com/UOR-Foundation/uor-r4/pull/2621

## 2026-10-18  Wire review fixes round 2 (Codex findings on #2614; `deepseek/spiralcore-wire-review-fixes-2`)

Codex reviewed #2614 at `448a9cb4f` and reported three P1s plus one P2; all addressed here. Interposer:
`connect` waits on `io_lock` before resetting a reused fd, closing the fd-reuse-vs-blocked-recv race.
Mux: streams are registered before their reader starts, and half-closed streams stay registered until
both client FIN and target EOF (`input_done` + `output_done`). Linux smoke: brace-safe default for an
unset `WIRE_SMOKE_WORK` under `set -u`. Worktree checks: `WIRE_INTERPOSER_PASS`, `WIRE_MUX_PASS`,
`WIRE_MUX_ROUTING_PASS`, `WIRE_INTERPOSER_LINUX_PASS`; no model compute.
Record: [labs/spiralcore-wire-review-fixes-2-2026-10-18](../labs/spiralcore-wire-review-fixes-2-2026-10-18/README.md).
PR: https://github.com/UOR-Foundation/uor-r4/pull/2619

## 2026-10-18 — C1 flock-coupling draft: finite controls and export-gap contract (claude)

The [C1 draft](../harness/prep/m4-c1-flock-coupling-DRAFT-2026-10-18.md) specifies the D37 float-trained coupling magnitude for the discrete Flock read, exported to integer Gate weights (`ReadSpec::FlockCoupled`, with `k_max = 1 ≡ Flock`). It contains:
- 22 finite controls (C0, X1–X12 and nine F rows), with literals hand-derived from the generated 16-root orbit: cache fidelity, the integer export kernel, and the float primitive (including an export-gap witness);
- a writer-structured mean-L1 proxy objective with a proxy-agreement instrument;
- a box-projected update, needed because the writer's address margin exceeds the inter-root distance;
- the export-gap contract on the Dose 1 evaluator;
- arms A0/A1/A2/A2-int/A2′/Z/R at matched writer-evaluation budget.

**Review:** three fresh-context reviewer subagents (literal recomputation, test fitness, round-2). Every finding is resolved: the empty-level `terms`, ABSENT cache rows, the proxy units and blocks, the Adam `pow`, the Relation-trit scope, and the trace encoding. **Status:** not registered, no FIT requested, no source change, no compute, $0. Training/M2/Stage B stay HELD. **Next:** registration after the Dose 1 decision. The open items are k_max>1 under D11, the D21 line status, the owner/PM reading of the float gap forward, and the agreement threshold and C0 sample.

## 2026-10-18 — Starling/murmuration brief: discrete Flock named as the integer murmuration read; post-dose candidates (claude)

[`docs/mechanisms/starling-read.md`](../mechanisms/starling-read.md) now records that the active discrete `ReadSpec::Flock` (`discrete.cu:125`) already realises the murmuration rule: frame-transported consensus, with a ternary coupling trit. Six finite controls PASS and it has never been trained. The brief also records that Dose 1 tests relation trits on top of Flock, not Flock against Route. It ranks three post-dose candidates under D37: C1, a float-trained coupling exported to Gate trits; C2, lawful soft-to-hard selection; C3, the FBS ternary-tree read. These are hypotheses only: nothing is registered, no compute ran, $0. The merged claims `claude/d37-offline-float-20261018` and `claude/public-face-reconcile-2026-10-15` are released. Training/M2/Stage B stay HELD. **Next:** a C1 finite-control draft, with the pre-registration after the Dose 1 decision.

## 2026-10-18  Wire side-exploration claims cleanup (deepseek)

Removes the stale committed claims for the merged wire branches (`deepseek/spiralcore-wire-side-exploration`,
`deepseek/spiralcore-wire-review-fixes`) plus this cleanup branch entry from docs/harness/claims.json.
No code, model, evidence or dispatch change; coordination record only. References #2613, #2614.

## 2026-10-18 — D37: floating point legal in offline training only (claude)

The owner revised D32. Float forward passes, gradients and optimizers are legal in offline training. Softmax/squashing and matmul stay banned everywhere, and compute stays GPU-only. The export is the integer boundary: D11 serving, with a reported export gap. The October 14 zeta-zero exception is unchanged, and archived float sheets return only by re-registration and a fresh FIT. The owner also stated the reason behind the bans: exclude expensive operations, for fast local training and serving.

[D37](DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised); [record](../labs/d37-offline-float-2026-10-18/README.md). The policy JSON moves to schema `/7`, and `r4-policy-check` now pins the serving float/transcendental/multiplier commitments. The D32 discrete path, the Dose 1 sheet/packet and all holds are unchanged; no compute ran, $0. **Next:** pre-registrations may cite D37 for offline float learning in a separately scanned module; the starling/murmuration coupling (C1) is drafted only after the Dose 1 decision.

## 2026-10-18 — Dose 1 static packet and actual fresh metadata sealed (Codex/cf1)

The [packet record](../labs/attention-dose1-packet-2026-10-18/README.md) binds234selected source/input/tool files at manifest5d5481cd, the exact23d5 RELEASE-profile same-resource contract, actual once-generated FreshV2 seal (400documents/14,479eligible/3,594copy-successor metadata rows/0known-document overlap), and independently posted component/composite reviews. Complete selected preservation70,858,752bytes/MD5ce84cdba437244fc0dbe0004a6d05682 is full-cloud-readback verified; no new pod/model/scorer work or GPU cost. Actual release ELF, genuine per-arm plan reviews/check-plan and Casey personal capabilities are late same-dispatch inputs, not impossible preallocation artifacts or agent-authored authority; all actual dose work remains NOT_RUN. **Next:** AGY confined exact234packet verdict → Kimi actual FIT/dispatch/fresh resource admission → same-resource RELEASE construction and concrete personal admission → paired dose; this docs PR is not an extra release condition. Count0/3 and suspended37/232 remain unchanged; no scientific bar or general hold changes.

## 2026-10-18  Wire side-exploration review fixes (Codex P1 findings; `deepseek/spiralcore-wire-review-fixes`)

Follow-up to merged #2613 (`94d4d34a`): Codex's post-merge exact-head review at `3af1acf` reported four
P1 findings, all addressed here. Interposer: separate `io_lock` so a blocking `recv` no longer blocks
`close`/`shutdown`, and `close` defers buffer reset to the next connect. Mux: gateway releases completed
streams (`finish_stream`). Linux smoke: scratch workspace defaults inside the checkout
(`WIRE_SMOKE_WORK` override for the mvm guest). Delivery: removed committed transient
`spiralcore-base/wire/local/` outputs and added `.gitignore`. Worktree checks: `WIRE_INTERPOSER_PASS`,
`WIRE_MUX_PASS`, `WIRE_MUX_ROUTING_PASS`, `WIRE_INTERPOSER_LINUX_PASS`; no model compute.
Record: [labs/spiralcore-wire-review-fixes-2026-10-18](../labs/spiralcore-wire-review-fixes-2026-10-18/README.md).
PR: https://github.com/UOR-Foundation/uor-r4/pull/2614

## 2026-10-18  Wire transport side exploration merged (`deepseek/spiralcore-wire-side-exploration`)

Merges the standalone wire-compression/transport work as a self-contained side exploration under
`spiralcore-base/wire/`: 78 files (Python adapter/socket/mux/hub/interposer tools and smokes, Rust
`wire-rs` crate detached from the main workspace, docs, bounded evidence logs). Worktree checks:
`cargo build --release --offline`, `WIRE_SOCKET_ROUTING_PASS`, `WIRE_RUST_HUB_PASS`; source repo recorded
`WIRE_INTEROP_MATRIX_PASS` 18/18 (re-measured locally on 2026-10-18); the earlier `EVIDENCE_OK`
line is withdrawn because its cited tool is absent - see labs/wire-matrix-repro-2026-10-18. No model compute, no training-hold change, no pods,
no change to the dose path or D11 serving. Measured: Rust peer 2.10/2.86/3.30 ms vs plain 1.52/1.92/
2.16 ms for 1.25 MiB; Rust adapter ~1.6x faster than Python; 1 KiB RTT 41.8 us vs plain 55.1 us.
Record: [labs/spiralcore-wire-side-2026-10-18](../labs/spiralcore-wire-side-2026-10-18/README.md).
Next: independent exact-head review, then merge queue; PM (kimi/pm1) holds routing for any promotion
into the roadmap path. PR: https://github.com/UOR-Foundation/uor-r4/pull/2613.

## 2026-10-18 — Falsify-then-optimize is the current path (owner direction; kimi/pm1)

The [optimization path record](../labs/optimization-path-2026-10-18/README.md) seals the owner-approved sequence: Dose 1 (paired Relation/frozen-zero falsification, one seed, release-profile binary per PM requirement 6243986174) is the sole paid-compute item; the optimization rung (release baselines, counterfactual row-parallelism, shared-memory staging, selective trace, pre-registered tokens/s bars) is gated on the dose decision; batched-coordinate scoring and spinor/quaternion adapters are conditional future rungs requiring new FIT'd pre-registrations. D11 multiplier-free serving stands; float remains legal in offline training only. Lab assignments: Codex dose packet/execution + kernels, AGY confined reviews + evidence synthesis, Kimi PM sheet/packet/dispositions + measurement plan; DeepSeek's object-store tooling line unchanged. Path directive: [#2029 comment 6244066845](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6244066845). No scientific bar, sheet text, FIT status or compute authorization changes. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE. **Next:** Codex dose packet seal → AGY confined review → PM dose dispatch.

## 2026-10-18 — C1 DRAFT sheet shape repaired; still not registered (kimi/pm1)

[The C1 DRAFT sheet](../harness/prep/m4-c1-flock-coupling-DRAFT-2026-10-18.md) now carries the D22 blocks the harness requires: Bar, Base artifact, Data split, Steps, Stop rule, Cost and Minimum test power, each truthfully marked `NOT_REGISTERED`/`NOT_RUN` at $0, plus `TEST FITNESS: PENDING` (DRAFT; no FIT requested). This is a documentation-shape repair after CI job 117572368828 (PM comments 6245063482, 6245085269): no bar, split, control, mechanism, source, artifact or hold changed. Training/M2/Stage B stay HELD. **Next:** the sheet remains a post-Dose-1 registration draft; registration and FIT wait on the Dose 1 decision.

## 2026-10-17 — Standalone Rust same-cell reference scorer (`antigravity/attention-cell-reference-scorer`)

The [scorer record](../labs/attention-cell-reference-scorer-2026-10-17/README.md) delivers `attention-cell-reference-scorer` (`preflight-dev`, `paired-dev`, and `bundle-candidate-cells`) in `crates/uor-r4-harness` (`src/attention_reference.rs` + `src/bin/attention_cell_reference_scorer.rs`) plus `DISCRETE_ENTRIES_WITH_EXPORT` in `scripts/check_native_cuda_vendor.rs` per [#2029 comments `6239938443`, `6240046545`, `6240085282`, `6240113871`, `6240513498`, `6240654230`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6240513498) and [#2604 comments `6241027772`, `6241711867`, `6242583159`, `6242961864`, `6243108155`, `6243567008`](https://github.com/UOR-Foundation/uor-r4/pull/2604#issuecomment-6243567008), with `0` bytes changed in `crates/uor-r4-discrete/**`, workspace `Cargo.toml`, or `Cargo.lock` relative to `origin/main`. Validates GPU-captured 16-slot `admitted_cells[16]` inventories (`uor-r4.attention-preflight-dev-capture/1`, binding all four distinct producer checkpoint/model hash domains across `checkpoint_file.sha256`, `driver_checkpoint_sha256`, Domain A `model_storage_sha256 == checkpoint.model_sha256` canonical `serde_json::Value` model, and Domain B `typed_model_snapshot_sha256 == eval.model_snapshot_sha256` typed `DeviceSnapshotRecord` serialization for both post-`#2611` dual-record and historical pre-`#2611` `Battery3` single-field receipts), raw (`uor-r4.discrete-corpus-driver/1`) or enveloped (`uor-r4.attention-learning-evaluation/1`) DEV evaluation receipts, mandatory `config_file`/`checkpoint_file`/`evaluation_file` `FilePin` records vs canonical `sha256(encode(&config)) == config_sha256` and `sha256(encode(&evaluation_lineage)) == evaluation_lineage_sha256` pins (binding the full `2,425,925`-row `TRAIN` `PreparedCorpus` lineage inventory separately from the `1,049,856`-target Dose 1 reference budget), exact `LearnV1` vs `FrozenZeroV1` paired arm `DriverConfig`/`DriverLineage`/`EvalModelBindings` equality on the shared `14,287` DEV rows (`P_succ` floors `24`/`39`, complement floors `22`/`63`), and the sealed D34 reference (`2717d34bcfcdd1486991925ecab066266a6b56ba34b89727c37f78c821fe3bec`); scores full same-cell `T1/T1b/T2/U_succ/U_train` across all eight DEV slices from a GPU-exported `token_cells.u16` (`8,192`-byte `raw_u16_le_no_header` `u16[4096]` bound to a mandatory #2601 `uor-r4.attention-token-cells/1` `report.json` receipt, refusing bare `.u16` files, inline JSON `token_cells` arrays, and unauthenticated standalone `--candidate-cells` inputs) + SHA256-verified `TRAIN`/`DEV` `UORT` files (`U_succ` using the model-matched `1,049,856`-target `TRAIN` unigram ranking per D34 §6.4.6, never fitted on `DEV` labels, cross-checked 14,287/14,287 against GPU `eval_target_admitted` and sealed D34 predictions, with zero host `root_id`/`token_cell` codebook arithmetic), and fails closed (`UNAVAILABLE_MISSING_GPU_TOKEN_CELL_TABLE_AND_RANKED_CANDIDATE_RECEIPT`) when not provided. `cargo fmt --check`, `cargo clippy -p uor-r4-harness --locked --all-targets -- -D warnings`, `cargo test -p uor-r4-harness --lib attention_reference`, `uor-harness ratchet d23|d25`, and `check_claim_wording.py` pass; `$0` GPU spend, no model compute; Training, M2, and Stage B HELD; Q4 NOT_EXECUTABLE. **Next:** independent exact-head review, protected merge, and `Codex/cf1` operator integration.

## 2026-10-17 — Explicit attention storage and typed-snapshot integrity domains (Codex/cf1)

[PM6243260600](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6243260600)
formally ACCEPTS Battery3 as-is: its stored model and typed Snapshot hashes
are different valid serialization domains, already cross-checked by the existing
B3 metadata validation. No rerun or rescore is required. The
[bounded runner repair](../labs/attention-hash-domains-2026-10-17/README.md)
reserves `model_snapshot_sha256` for typed Snapshot bytes and emits stored
model-Value integrity as `model_storage_sha256`, retaining canonical checkpoint
and raw FilePin identities. It adds metadata bindings/refusals and persists
existing qualification observations; no numerical/kernel/learning-law change.
Host no-CUDA all-target check/fmt and85 metadata tests PASS/0FAIL/1ignored;
the initial missing-local-parent fixture failure is retained and its setup-only
rerun is documented. CUDA-feature serializer fixtures are authored/NOT_RUN.
Gate10 under actual PM6243606649 now passes all five compiler/host commands at
exact dfa3: CUDA check/strict package Clippy, host check/85metadata0FAIL1ignored,
and debug link. Initial/final source guards pass. Linked ELF d87c9f14 is archived
without invocation; runtime emission and learning remain NOT_RUN. Original
operator/setup/command0, full58+43+24SHA and cloud41159168bytes/MD5
81cc3a7285c6517d1035649c393acf51 verified before own56471322 down00:22:12Z;
dated fleet0, late lifecycle closure separate. Native251s at$.964/h gives
$.067212222 elapsed proxy versus254s wall$.068015556, alternatives/not invoices;
carried mixed Codex$4.804471110 plusPM/storageUNMEASURED. Protected delivery and the
rebound one-seed-first paired dose remain next. No old
binary/checkpoint/release is promoted. Count0/3/reply37/232SUSPENDED/learned
Relation NOT_RUN unchanged; Training/M2/StageB HELD, Q4 NOT_EXECUTABLE.

## 2026-10-17 — Attention Battery3 scoped preflight complete, zero apply (Codex/cf1)

The [actual preflight record](../labs/attention-preflight-2026-10-17/README.md)
and [outcome6243041786](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6243041786)
record Battery3 under distinct PM6242930665/AGY6242609372 at protected runtime
d46e/constructed96ad/executable5abe. All five staged data hashes, all26 actual
command/contract steps and original operator41735 returned0. Initial checkpoint
lock and read-only device export PASS; export4096cells/8192bytes preserves the
snapshot. DEV evaluation applies0; initial B3 admits2034/3847 P_succ targets
abovefloor500 (full7500/14287, complement5466/10440). Both fixed sensitivity
predicates pass; Relation qualification9/9PASS/0FAIL/0NOT_RUN/0apply and final
source/residual contracts pass. No learning dose, full same-cell scoring,
224-forward admitted-slot capture or fresh draw ran.

Main34380288bytes/MD5`d0d0d74506123c834b186ff626ec8cdb` fullSHA/cloud verified
before own56464787 down23:22:46Z; separate late closure23552/
`6e7392328db3eb09e93e894b78733ade` verified afterward. Native346s at$.964/h
gives$.092651111 proxy; separate wall349s$.093454444, alternatives/not invoices.
Carried mixed Codex$4.737258888 plusPM; storage/transfer UNMEASURED.
[Independent actual review6243075217](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6243075217)
APPROVES the scoped preflight (report80cae1fc). PM disposition and the separately
pinned paired-dose packet are next. Recorded count0/3/reply37/232SUSPENDED/learned Relation NOT_RUN are
not reset or promoted here. Training/M2/StageB HELD; Q4 NOT_EXECUTABLE.

## 2026-10-17 — Attention Battery2 staging stop before binary invocation (Codex/cf1)

The [actual preflight record](../labs/attention-preflight-2026-10-17/README.md),
[closure6242397583](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242397583)
and [PM disposition6242429565](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242429565)
seals PM6242232012's released Battery2 attempt at unchanged protected source
d46e050f4 and executable5abe54c3. Setup completed, but the job exited1 at its
raw-data check with zero attempted command steps: first TRAIN tokens present,
four remaining registered data/tokenizer files missing. All project metadata,
init/export/eval/B3/proposal/qualification commands remain NOT_RUN; zero applies.
Archived staging code exposes manifest stdin to the verification SSH child;
an independent consuming-child stub reproduces the first-file-only result.
This is a D31 instrument/setup defect, not a mechanism negative or a progress
reset. Per [PM ruling6242715607](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242715607),
the protected line is count0/3: reply37/232 SUSPENDED, learned Relation NOT_RUN.
Preparation and D31 instrument failure do not consume the executed-test count.

Independent actual audit6242413809 accepts the preserved operational receipt:
31 collected/71 operator/20 raw/44 reviewed/5 closure hashes PASS. Main archive
6309888bytes/MD5`7e2afaa3df2943dc9cd98b87c5ef0b01` verified before own56460012
down22:34:03Z; separate late closure23552/`c95491c56bc36bf83bcadd4b387fc104`.
Raw-quote uptime proxy$0.050316691 versus rounded-rate$0.050342222; separate
wall proxies remain alternatives, not invoices. No paid retry is authorized.
Battery3 is conditionally ready for distinct PM dispatch; NOT_RELEASED/NOT_RUN
([prospective verdict6242500425](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242500425),
[final handoff6242520855](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242520855)).
[AGY APPROVE6242609372](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242609372)
completes the packet reviews; next is a new distinct actual PM release and fresh admission. No new timing,
compiler or mechanism programme. Training/M2/StageB HELD; Q4 NOT_EXECUTABLE.

## 2026-10-17 — FETCH batch opt-in for high-latency pulls (deepseek/object-store-batch)

The [batch record](../labs/object-store-batch-2026-10-17/README.md) adds `objpeer pull --batch N` and
`pull_selected_with_batch`: an explicit FETCH window clamped to 1..=65535 while the default stays at the
derived 4096. On the WAN, the same 52,535,088 B artifact pulled in 236.19 s (default) vs 91.50 s
(`--batch 32768`), a 2.58x speedup with identical wire bytes; locally the larger batch is slower, so it
is opt-in. Package 58 passed / 0 failed; Clippy/fmt clean. No model compute; training/M2/Stage B HELD.
**Next:** protected delivery, then rebuild the review-repo binary and re-run the acceptance gate.

## 2026-10-17 — Pinned cold join: objpeer join (deepseek/object-store-join)

The [join record](../labs/object-store-join-2026-10-17/README.md) adds objpeer join for the supported
cold-join path: a decompressed artifact's SHA-256 and length are checked against
--expect-sha256/--expect-bytes before any store write, then the store is folded and saved; wrong pins
exit 2 with nothing written. Tests cover pinned success/idempotence and fail-closed wrong pins. Package
57 passed / 0 failed; Clippy -D warnings and fmt clean. No model compute; training/M2/Stage B HELD.
**Next:** protected delivery, then tools/cold_join.sh measuring compressed download + join vs cold store
pull and the first warm delta.

## 2026-10-17 — Read-only token-cell input producer for learned attention (Codex/cf1)

The [source record](../labs/attention-token-cell-export-2026-10-17/README.md)
implements the minimal CUDA export required by PM6240513498: the existing
`token_cell` maps4096 tokens from an exact DriverCheckpoint to an8192-byte
little-endian file, with output SHA and source/checkpoint/lineage bindings.
No host geometry, forward, proposal or apply path is introduced. Full snapshot
equality is checked before/after the read-only export. Host all-target check
and two focused storage/no-host-fallback controls PASS. AGY source/packet review
APPROVE6241052759 and PM compiler dispatch6241202400 led to Gate9 **all five
commands PASS** at96ad8a475: CUDA check/strict package Clippy, host check,
82 metadata tests PASS/0FAIL/1ignored, and debug link without invocation.
Both provenance guards PASS; full58+43+24 SHA/cloud-MD5 archive verified before
own56451966 down21:19:44Z. The new binary5abe54c3 and full archive retain exact
source identity; actual export/runtime identity remain NOT_RUN. Cost uptime
proxy$0.075513333, separate wall proxy$0.076316667, not invoices; storage UNMEASURED.
The old Gate8 binary and frozen Battery1 draft are preserved. Next: reviewed
merge-forward/protected delivery → rebound finite preflight → separately
released paired learning dose. Actual per-row admitted
slots remain GPU observations; the map alone supplies no same-cell result.
Headline unchanged, no progress reset or mechanism verdict. Training/M2/StageB
HELD; Q4 NOT_EXECUTABLE.

## 2026-10-17 — objpeer repair: audit, select, pull missing targets (deepseek/object-store-repair)

The [repair record](../labs/object-store-repair-2026-10-17/README.md) adds objpeer repair: probe each
replica, classify each target current/repaired/unavailable, group repairs by the replica that holds them
and pull only the missing targets, with per-target status, JSON totals and exit 3 on unavailable/failed
targets. Tests cover split targets across replicas (follow-up pull moves 0 records, proving the local
store is current) and an unavailable target. CLI 16/16, peer 15/15, controls 10/10, vectors 1/1, showcase
5/5; Clippy -D warnings and fmt clean. No model compute; training/M2/Stage B HELD. **Next:** protected
delivery, then a packaged repair demo on real artifacts and INDEX update.

## 2026-10-17 — Replica-aware sync with failover (deepseek/object-store-replica-sync)

The [replica-sync record](../labs/object-store-replica-sync-2026-10-17/README.md) makes objpeer sync
replica-aware: repeated --addr is tried in order with per-replica retries, a failed or incomplete
replica falls through to the next, and the artifact set is written once with the serving replica
reported in JSON/SYNC_SERIES. Tests cover a dead first replica and a first replica missing the target.
CLI 14/14, peer 15/15, controls 10/10, vectors 1/1, showcase 5/5; Clippy -D warnings and fmt clean. No
model compute; training/M2/Stage B HELD. **Next:** protected delivery, then a local end-to-end
audit-then-failover-sync repair demo and INDEX update.

## 2026-10-17 — Replica-aware fleet audit (deepseek/object-store-replica-audit)

The [replica-audit record](../labs/object-store-replica-audit-2026-10-17/README.md) lets objpeer verify
take repeated --addr and report per-target per-replica status: JSON gains replicas/verified_all and
per-target replica entries, human output gains REPLICA lines plus VERIFY_REPLICAS, and exit codes flag
any unknown (2) or stale/mismatch (3) on any replica. Single-address output is unchanged. Two new CLI
tests cover all-replicas-verified and one-replica-unknown. CLI 12/12, peer 15/15, controls 10/10,
vectors 1/1, showcase 5/5; Clippy -D warnings and fmt clean. No model compute; training/M2/Stage B
HELD. **Next:** protected delivery, then a two-replica local audit benchmark and INDEX update.

## 2026-10-17 — Read-only retries for verify and fleet audit (deepseek/object-store-verify-retry)

The [verify-retry record](../labs/object-store-verify-retry-2026-10-17/README.md) extends reconnect/retry
to objpeer verify and therefore fleet audit: --retries N re-runs the pipelined series after retryable
I/O/schema/integrity failures, while binding/unknown/limit errors stop early; verification is read-only
so retries cannot corrupt state. A cutting-proxy CLI test cuts the first connection and bridges the
second, yielding a successful verified audit with a retry line. CLI 10/10, peer 15/15, controls 10/10,
vectors 1/1, showcase 5/5; Clippy -D warnings and fmt clean. No model compute; training/M2/Stage B
HELD. **Next:** protected delivery, then product consolidation with the measured-surface index.

## 2026-10-17 — Reconnect and retry for real networks (deepseek/object-store-retry)

The [retry record](../labs/object-store-retry-2026-10-17/README.md) adds opt-in --retries N to objpeer
pull and sync: each attempt reconnects, retryable I/O/schema/integrity failures print retry n/N and retry,
while binding/unknown/limit errors stop early. Atomic digest-verified imports make retries safe, and a
cutting-proxy CLI test (first connection cut, second bridged) proves a real reconnect and byte-identical
output. CLI 9/9, peer 15/15, controls 10/10, vectors 1/1, showcase 5/5; Clippy -D warnings and fmt
clean. No model compute; training/M2/Stage B HELD. **Next:** protected delivery, then a WAN flap test
against the pod proxy if useful.

## 2026-10-17 — Protocol spec note: node id is standard BLAKE2b-96 (deepseek/object-store-spec-note)

The [spec note](../labs/object-store-spec-note-2026-10-17/README.md) records that node_id(scale,payload)
equals standard BLAKE2b with a 12-byte digest over [scale] || payload, so ports may use a normal
BLAKE2b-96 API instead of reimplementing the padded block; a Python port using hashlib reproduced the
conformance vectors and cold-pulled the vector object from the reference server (4 records, exact SHA).
Docs only; tests and formats unchanged. No model compute; training/M2/Stage B HELD. **Next:** protected
delivery, then keep the Python port as the non-Rust conformance harness.

## 2026-10-17 — Protocol v1 conformance vectors (deepseek/object-store-vectors)

The [vectors record](../labs/object-store-vectors-2026-10-17/README.md) freezes a deterministic 96-byte
object's payload, SHA-256, canonical UOROBJ01 manifest, manifest version, frontier and root/leaf record
bytes as a test, giving any port a byte-for-byte conformance gate that complements PROTOCOL.md. Vector
test 1/1; package suites unchanged; Clippy -D warnings and fmt clean. No model compute; training/M2/
Stage B HELD. **Next:** protected delivery, then prove portability with a non-Rust verifier against
the reference server.

## 2026-10-17 — Peer protocol specification v1 (deepseek/object-store-protocol-spec)

The [protocol specification](../labs/object-store-protocol-spec-2026-10-17/README.md) documents the
byte-level UOROBJ01 peer contract in crates/uor-r4-object-store/PROTOCOL.md: exact custom BLAKE2b-96
node ids, 45/49-byte records, manifest layout and base-3 frontier, frame kinds 20..=25, pull/verify/
verify_series/pull_series flows, serve semantics, limits, errors, trust boundaries, a porting checklist
and the reference file map. Documentation only; package tests and formats unchanged. No model compute;
training/M2/Stage B HELD. **Next:** protected delivery, then use the spec to port a minimal verifier
(non-Rust or container/VM agent) against the reference server.

## 2026-10-17 — Fleet sync: pull a target series on one session (deepseek/object-store-pull-series)

The [pull-series record](../labs/object-store-pull-series-2026-10-17/README.md) adds pull_series plus
PullRequest: a node pulls several selected artifacts over one connection, each fetched/verified/imported
before the next, with one DONE at the end; the store is loaded and saved once. objpeer sync --targets
FILE --out-dir DIR writes each artifact to DIR/<sha256>.bin with optional JSON totals. Peer 15/15, CLI
8/8, unit 7, controls 10, showcase 5; Clippy -D warnings and fmt clean. No model compute;
training/M2/Stage B HELD. **Next:** protected delivery, then measured per-artifact savings for fleet
sync versus connection-per-artifact pulls.

## 2026-10-17 — Fleet audit targets file and JSON output (deepseek/object-store-verify-targets)

The [targets record](../labs/object-store-verify-targets-2026-10-17/README.md) gives fleet verification
its CI/registry shape: objpeer verify takes --targets FILE (64-hex selector per line, comments/blanks
ignored) and --json (one machine-readable object with per-check status/missing/selector plus totals),
while keeping pipeline batching and exit 0/3/2 semantics. CLI 7/7, peer 14/14, controls 10/10,
showcase 5/5; Clippy -D warnings and fmt clean. No model compute; training/M2/Stage B HELD. **Next:**
protected delivery, then a hundreds-of-targets audit measurement and the registry service shape.

## 2026-10-17 — Pipelined manifest requests for fleet verification (deepseek/object-store-verify-pipeline)

The [pipeline record](../labs/object-store-verify-pipeline-2026-10-17/README.md) pipelines verify_series
manifest requests in bounded windows of 64, so a fleet batch costs about one round trip per window
instead of one per check; the single DONE and per-check ordering are preserved, an unknown selector
keeps the session open, and a two-window 100-check test covers the window boundary. Peer 14/14, CLI 6/6,
unit 7, controls 10, showcase 5; Clippy -D warnings and fmt clean. No model compute; training/M2/Stage B
HELD. **Next:** protected delivery, then local and cross-machine pipelined measurements.

## 2026-10-17 — Batch verification on one session (deepseek/object-store-verify-series)

The [batch record](../labs/object-store-verify-series-2026-10-17/README.md) adds verify_series plus
VerifyRequest/VerifyOutcome: many manifest selectors are checked on one connection with one DONE, an
unknown selector no longer aborts the serve session, and the CLI verify accepts repeated
--select-sha256, printing per-check VERIFY lines and a VERIFY_SERIES summary with exit 0/3/2 for
all-verified/stale-or-mismatch/unknown. This amortizes the connection setup that dominated the
cross-machine fleet pass (226 B and about 564 ms per check). Unit 7/7, peer 12/12, objpeer_cli 6/6,
controls 10/10, showcase 5/5; Clippy -D warnings and fmt clean. No model compute; training/M2/Stage B
HELD. **Next:** protected delivery, then local and cross-machine batch benchmarks.

## 2026-10-17 — Unit1 verified on main; Unit2b runner source preparation (codex/cf1)

The [Unit2b record](../labs/attention-unit2b-runner-2026-10-17/README.md) seals Unit1 #2575 verification and the follow-up: DEV-only training, jointly deferred paired fresh after DEV H_geom, canonical B3/Q1-at4096 stop/resume, explicit pins and actual GPU Forward cell-capture interfaces. Gate8 under PM6239543082 at exactc951 passed all five compiler/host commands,80 metadata tests/0failures/1ignored and both source guards; the debug CUDA binary was linked/archived without invocation. The local outer operator exited1 while its remote compiler remained live; a separately reviewed continuation preserved the existing job, completed unchanged cleanup and exited0, with no restart or paid retry. Full58remote+43operator+24packetSHA and private cloud40,601,600bytes/MD5487aafdb005191b53f34877b9e941042 preceded own56441942 teardown19:43:38Z; peer pod retained. Native260s cost proxy$0.103133333, mixed cumulative Codex$4.518752222 plusPM, not invoices; storage unmeasured. Main is merged forward preserving attention source bytes and incoming peer records. Scientific sheet stays byte-identical to protectedmain844814; no numerical CUDA/init/learning-law/frozenRoute/timing3 change. **Next:** independent outcome/docs-delta review and protected delivery, then separately FIT/released preflight battery with explicit extra224-forward capture and same-cell scoring dependency; paired dose remains a later distinct release. Finite/model work NOT_RUN, headline unchanged. Training/M2/StageB HELD; Q4 NOT_EXECUTABLE.

## 2026-10-17 — Fleet protocol docs and stale-verification coverage (deepseek/object-store-fleet-docs)

The [docs record](../labs/object-store-fleet-docs-2026-10-17/README.md) completes the fleet surface:
objpeer usage documents verify, repeated --object and --select-sha256; the peer module documents the
32-byte selector and serve_many; and stale verification is now gated in both the library (missing > 0,
verified=false on an empty store) and the CLI (exit 3, verified=false). Peer 11/11, CLI 5/5, controls
10/10, showcase 5/5; Clippy -D warnings and fmt clean. No model compute; training/M2/Stage B HELD.
**Next:** protected delivery, then cross-machine fleet verification.

## 2026-10-17 — Fleet verify and multi-object manifest selection (deepseek/object-store-fleet)

The [fleet record](../labs/object-store-fleet-2026-10-17/README.md) adds a backward-compatible 32-byte
manifest selector to MANIFEST_REQUEST: a store can serve many objects (repeated --object), clients can
pull or verify a selected object by whole SHA-256 or manifest version, an unknown selector returns
Error::UnknownObject, and fetch scoping stays per object. peer::verify reports missing/verified without
fetching or rebuilding, giving a roughly 200-byte fleet current-check; the CLI exposes repeated --object,
--select-sha256 and a verify command. Peer 10/10, CLI 4/4, controls 10/10, showcase 5/5; Clippy -D
warnings and fmt clean. No model compute; training/M2/Stage B HELD. **Next:** protected delivery, then a
measured fleet-verification benchmark versus full pulls.

## 2026-10-17 — Trusted object pins in the peer API and CLI (deepseek/object-store-object-pin)

The [pin record](../labs/object-store-object-pin-2026-10-17/README.md) turns the audited trusted anchor
into a usable surface: ExpectedObject accepts any combination of native length, manifest version and
whole-object SHA-256 pins, rejects an empty anchor, and fails with BindingMismatch before fetch/import.
The packaged objpeer CLI exposes --expect-sha256/--expect-version/--expect-bytes and prints the object
sha256 and manifest_version so peers can be pinned out of band. Tests cover accept and reject paths in
the library and the CLI; package suites pass, Clippy -D warnings and fmt clean. A pin proves object
identity, not peer authenticity (no TLS/credential). No model compute; training/M2/Stage B HELD.
**Next:** protected delivery, then record the trust model with the product packaging and check whether a
session-level authentication layer is worth adding.

## 2026-10-17 — Packaged objpeer replication CLI in the object store (deepseek/object-store-objpeer-cli)

The [packaging record](../labs/object-store-objpeer-cli-2026-10-17/README.md) adds the content-addressed
artifact replication CLI (info/serve/pull) to uor-r4-object-store itself, built on the merged Store and
peer APIs with no new dependencies, plus a two-process loopback delta test and an info test. Package 33
passed, Clippy -D warnings and fmt clean. The same CLI source produced the review-repo measurements
(2,504 B local / 2,509 B cross-machine for a 52.5 MB weight revision; 220 B verification; 38,508 B for
12 warm receivers). No model compute; training/M2/Stage B HELD. **Next:** protected delivery, then an
ExpectedObject pin flag and replacing the review-repo copy with this binary.

## 2026-10-17 — Concurrent peer sessions on one read-only store (deepseek/object-store-concurrency)

The [concurrency record](../labs/object-store-peer-concurrency-2026-10-17/README.md) documents and gates
that peer::serve takes &Store, so multiple peers can replicate from one loaded store concurrently; each
session keeps its own expected anchor, correlation checks and digest verification while the store stays
immutable. A new test runs two simultaneous sessions and verifies identical counts and exact bytes.
Peer suite 8/8, package 31 passed, Clippy -D warnings and fmt clean. No model compute; training/M2/Stage
B HELD; model milestone counts unchanged. **Next:** protected delivery, then multi-object dedup accounting
and product packaging.

## 2026-10-17 — Peer pull requests deduplicated DAG nodes once (deepseek/object-store-peer-dedupe)

The [dedupe record](../labs/object-store-peer-dedupe-2026-10-17/README.md) fixes a real failure found by
the product CLI against hardened main: deduplicated repeated content makes the FBS tree a DAG, so one
missing node could appear on several paths and the strict correlated-response check rejected a valid
duplicate request. missing_nodes now deduplicates ids before batching; response correlation stays strict.
Regression covers a periodic 2 MiB object with a 128-atom edited region; peer suite 7/7, package 30
passed, Clippy -D warnings and fmt clean. No model compute; training/M2/Stage B HELD; model milestone
counts unchanged. **Next:** protected delivery, then driver-side DoS/limits and multi-peer fan-out.

## 2026-10-17 — Exact object store peer replication hardened (deepseek/object-store-peer-hardening)

The [hardening record](../labs/object-store-peer-hardening-2026-10-17/README.md) adds a caller-pinned
object identity (ExpectedObject + pull_expected, rejected before import), limit-derived dynamic batching
that completes a cold 1 MiB pull, correlated responses (count, requested ids, duplicates), an atomic
staged commit after native SHA-256 verification, DONE/flush/clean-EOF protocol hygiene, and
manifest-closure-scoped serving with budgets. Peer suite 6/6 and all package tests pass; Clippy -D
warnings and fmt clean; a scattered 16-edit delta adds 131 records and verifies exactly. No model
compute; training/M2/Stage B HELD; model milestone counts unchanged. **Next:** protected delivery, then
driver-side sync DoS/limits and multi-peer fan-out.

## 2026-10-17 — Exact object store gains native peer replication (deepseek/object-store-peer)

The [peer record](../labs/object-store-peer-2026-10-17/README.md) adds Store read accessors
(record_bytes/has_record) and a transport-agnostic peer module that replicates an object using the
native manifest and record wire formats, requesting only nodes the receiver's Store reports missing
and rebuilding with native digest verification. Package tests 25 pass (3 new peer tests: a
1,048,576-byte object delta transfers 22 records / 1,014 bytes in 3 rounds and verifies; an identical
store transfers 0 records), Clippy -D warnings clean, fmt clean. No model compute; training/M2/Stage B
HELD; Q4 NOT_EXECUTABLE; model milestone counts unchanged. **Next:** protected delivery, driver
transport adapter, then multi-peer and persisted-store restart gates.

## 2026-10-17 — Owner-prioritized native Relation learning path, source preparation (codex/cf1)

The [integrated source record](../labs/native-relation-learning-path-2026-10-17/README.md) connects the missing absolute-batch interleaver, explicit post-init ActiveLeaf storage and matched45-coordinate Relation/frozen-zero control to the real CUDA transition and attribution executor. Scientific sheet #2574 remains FIT(design), with production initializer/numerical CUDA/frozen completion/timing3 unchanged and no model execution or learned-attention result. Gate6 at exact6ca599a53 passed initial provenance and CUDA all-target check (exit0/47.91s), then strict package `--no-deps --all-targets` Clippy stopped at three unchanged-main prime_zeta.rs sites; both host gates/final guard were NOT_RUN. Independent actual review04dda4b1 accepts38remote+43operator+24packet SHA and full private cloud8152576bytes/MD54d48c7556e35ecd0aafcca6558a94a52 before release/down15:52:51Z/fleet0; separate closure23552bytes/MD537190b09abfe500159891a42dccfd969. Native338s elapsed cost proxy$0.161958333, separate341s wall$0.163395833; cumulative mixed Codex estimate$4.320722222 plus PM, not invoices. Gate4 SSH/gate5 allocation operational failures remain sealed separately, not mechanism negatives. PM6236268062 accepted gate6 and approved only three equivalent prime_zeta lint idioms, explicitly preserving order/tie-break and unchanged-main lineage7b26da89d/#2525. No numerical CUDA/sheet/learning/control change or warning waiver. Repaired source changes execution identity and needs AGY confined delta, exact gate7 packet rebind and distinct PM dispatch for all four compiler/host gates. All six old releases consumed; no automatic retry. Training/M2/StageB HELD; Q4 NOT_EXECUTABLE; no headline/count change. Bounded K16/timing work remains separate and is not a new prerequisite programme.

## 2026-10-17 — Host diagnostics executed: Stage-A K-sweep re-score (K\* = 8 holds) and D34 successor tables (kimi/pm1)

The [record](../labs/host-diagnostics-2026-10-17/README.md) delivers the two host-only FIT'd diagnostics from #2572, released at [#2029 6230337501](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6230337501) and executed by the DeepSeek implementer seat ($0 GPU, no model forward; sealed archive `icloud:UOR-R4/results/kimi/diagnostics-20261017.tar`, MD5 `e6e5deff87cb7c0ade76b38afd37cfaa`). **Stage-A:** H1 integrity PASS (36/36 sweep points and all controls exact); primary H2 supported admission-only (Q4 K=8→K=16 top-1 on A_8: 149→138, discordance 11/0 vs floor 24); H3 supported; **K\* = 8** stands. **D34 tables:** the context-free T2 successor table scores **6,300/14,287** full-panel top-1 and **1,245/3,847** on `P_succ` — the ordinary comparator the first trained Relation artifact must beat under the FIT'd learned-attention-link sheet; decision row 1 UNAVAILABLE (no Relation checkpoint; Route interim not released). Diagnostic counts at this configuration, not a mechanism verdict (D22). Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count 0/3 and suspended headline 37/232 unchanged. **Next:** Codex Unit 1 source (#2575) through independent review and the pre-merge CUDA transcript gate; PM execution packet for the first Relation dose only after the owner resolves sheet §7.10 (a)–(j).

## 2026-10-17 — `uor-r4-harness` un-dispatched rule implementation, adapter & dispatch reconciliation (`harness.*`, `prlint.gates_declared`, `receipts.skill_usage_recorded`), source only (`antigravity`, M1 qualification & enforcement tooling)

The [record](../labs/harness-d32-and-rule-dispatch-2026-10-17/README.md) reconciles the 41 repo-scope rules in `checkdispatch::RULES` (`crates/uor-r4-harness/src/checkdispatch.rs:22-67`; 53 total entries across 10 modules in `ruleinventory::INVENTORY` at `crates/uor-r4-harness/src/ruleinventory.rs:26-170` including the 12 `prlint.*` PR-body gates) against the nine per-module `all(...)` functions and four direct `mistakes::*` calls in `checkdispatch::run_scoped` (`crates/uor-r4-harness/src/checkdispatch.rs:230-247`), closing all five registered rules that previously went un-dispatched (`checkdispatch::run_scoped` synthesized `message: "ok"` using `checkdispatch::RULES` severity whenever a rule ID emitted no `Finding`): (1) activates `GROUPS` (`11` subcommand groups) and `FLAGS` (`76` flags across `cli.py` and `main.rs`) in `crates/uor-r4-harness/src/doccheck.rs:480-715, 850-980` (`resolve_with_flags`), validating subcommands (`team list`) and flags (`status --refreshh`, `onboard --bad-flag`) against static tables while preserving 100% parity with `scripts/harness/doccheck.py` across all 143 repo invocations; (2) implements `harnessrules::owner_rules_applied(regs)` (`crates/uor-r4-harness/src/harnessrules.rs:115-142`) and dispatcher adapters `harnessrules::entry_point(paths)` and `harnessrules::documented_commands(paths)` (`crates/uor-r4-harness/src/harnessrules.rs:150-168`), wiring all three into `harnessrules::all(paths, regs)` (`crates/uor-r4-harness/src/harnessrules.rs:98-111`, base `fa62a670a` `:103-113`) with direct and `checkdispatch::run_scoped` tests; (3) implements `prlintrules::gates_declared_for(declared, inventory_prlint)` and `prlintrules::gates_declared()` (`crates/uor-r4-harness/src/prlintrules.rs:258-351, 526-653`, base `fa62a670a` `:265-268`), reconciling `crate::prgates::DECLARED_GATES` against `ruleinventory::INVENTORY["prlint"]` at `check` time (while `prgates::gates_declared_tests::the_bodies_exercise_every_declared_gate_that_a_body_can_reach` enforces `check_pr()` emission at `cargo test` time, whereas Python's `scripts/harness/prlint.py:874-899, 1026-1048` AST-walks `check_pr()` and `_check_softmax()`) and wiring `gates_declared()` into `prlintrules::all(paths, claims)` with clean and injected-mismatch dispatch tests; and (4) updates the module header and wires the fifth rule `receipts.skill_usage_recorded` (`crates/uor-r4-harness/src/receiptsrules.rs:6-17, 183-188, 329-401`, intentional no-op matching Python) into `receiptsrules::all(paths)` with dispatch tests. Zero `#[cfg(feature = "cuda")]` or `required-features = ["cuda"]` targets touched, preserving all pinned `crates/uor-r4-discrete` and instrument SHA-256 digests (`model_compute_executed: false`, `apply_calls: 0`, zero pods, $0 GPU cost). Line: M1 qualification & enforcement tooling · count 0/3 unchanged · M1 headline 37/232 unchanged (suspended D28/R14); training, M2, and Stage B remain HELD; Q4 remains NOT_EXECUTABLE. **Next:** Kimi PM final identity confirmation and R-queue merge on PR [#2570](https://github.com/UOR-Foundation/uor-r4/pull/2570) ([`6229907407`](https://github.com/UOR-Foundation/uor-r4/pull/2570#issuecomment-6229907407)).

## 2026-10-17 — Capacity-diagnostic pre-registrations FIT after two NOT_FIT rounds (kimi/pm1)

[PR #2572](https://github.com/UOR-Foundation/uor-r4/pull/2572) carries three pre-registered diagnostics, drafted from the PM substrate investigation and revised through two NOT_FIT rounds of independent architecture review before **TEST FITNESS: FIT** at `f53c5198a` (docs only; FIT authorizes no compute; execution releases are separate PM steps): (1) **M1 read-code capacity** — a count table keyed by the exact 4-root read code (65,536-code alphabet), built on the model-matched 1,049,856 TRAIN targets, against the trained writer (149/14,287; 149/1,920 admitted) with co-primary unigram/bigram controls, admission-matched arms, top-8, and pre-registered power thresholds; decides representation-wall vs learning-law-wall; one zero-apply capture, 1-hour lease $0.908. (2) **Stage-A K-sweep re-score** — host-only ($0 GPU) re-analysis of the sealed 36-point archive; primary comparison Q4 K=8-vs-16 on the fixed admitted set A_8 with a binding multiplicity rule (32 secondary steps hypothesis-generating only); Q4 K=8 sub-panel stored values are the join controls. (3) **D34 successor-table control** — context-free TRAIN successor table T2 as the decisive comparator (context-local T1 is tautological at 3,847/3,847 by construction and serves only as a panel-definition check); symmetric within-floor bands (no zero-row tie bar); the Relation arm uses measured B3_R with an observed-power guard and is UNAVAILABLE until a frozen Relation checkpoint exists (blocker, not a negative); Route interim executable. Also corrects the stale "zero code exists" claim in `docs/mechanisms/finite-relational-head.md` (Relation is implemented at `discrete.cu:78-267` et al.; the trained head remains unexecuted). Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged. **Next:** PM execution releases — host-only Stage-A re-score and D34 tables first ($0 GPU), then the read-code capture after Codex's timing-closure lease.

## 2026-10-17 — Timing attempt refuses a changed GPU before instrument execution (codex/cf1)

The [timing2 setup record](../labs/counterfactual-baseline-reuse-timing-uuid-blocker-2026-10-17/README.md) preserves separate release6228540541 after protected finite qualification #2568. Placement changed between quote and allocation: actual driver595.71.05 matched, but GPU UUID35bf2e2c differed from the qualified1daf8397, so the wrapper stopped before any binary at02:33:21Z. Instrument/model/proposal/timing/apply dose is zero; no timing or checkpoint0 parity result follows. The initial archive copy omitted a required CSV; its manifest check retained and renewed the pod. Preservation-only recovery copied the exact original file and verified all manifests and the complete cloud archive (9,870,336 bytes, MD5 `2e882d487871ff5edc249c15f37c14cf`) before release02:35:01Z/down02:35:07Z. No owned pod remains; the separate DeepSeek lease was untouched. Lifecycle176s at raw$0.9083333333333332/h estimates$0.044407407; cumulative38 closed Codex leases$3.490932222 plus separate PM, storage/transfer UNMEASURED. A subsequent owned-worktree materialization briefly crossed the local disk floor; index refresh/sparse reapply restored it before any new lease or model work. **Next:** protect this blocker/recovery record, then prospectively resolve the physical-device admission constraint and repair the packet's archive copy before a new timing release. No automatic retry or model-negative conclusion. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged.

## 2026-10-17 — Baseline-reuse finite qualification passes on the available driver (codex/cf1)

The [environment record](../labs/counterfactual-baseline-reuse-driver-595-2026-10-17/README.md) preserves separately released run3 on RTX5090/driver595.71.05: the unchanged executable passes32 exact U/R pairs,64 attempts(56 successes/eight expected status8),16 typed refusals and18 strict resource admissions, with state/epoch unchanged and apply0. Independent exact-u64 readback verifies512 R lane audits/414 producer rows and directly observed R512 shared bytes; this is finite fixture qualification on the recorded environment, not a timing or cross-driver equivalence claim. Raw SHA7f36d6f805bcb7082467373e3a9f17c23070feb96d9269c6f7e64ce0405567a5; complete archive7,470,080bytes/MD5 bb98d16c9a6a442b6121bf767a62e095 verified before release/down/fleet0;112s at raw$0.9083333333333332/h estimates$0.028259259, cumulative37closed Codex leases$3.446524815 plus separate PM, storage/transfer UNMEASURED. **Next:** protect-deliver this accepted record and complete cleanup, then separately review and release the already-designed timing comparison bound to this observed environment; no timing or training release follows automatically. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged.

## 2026-10-17 — Opt-in exact-driver selection before pod allocation (codex/cf1)

The [source record](../labs/driver-aware-pod-selection-2026-10-17/README.md) follows the protected timing-startup blocker#2566. The existing Rust provider and runner gain a run-scoped exact-driver selector: preserve the version as a string, query it and reject missing/mismatched advertised metadata locally before creation. No qualified offer means zero allocation; selected mode cannot silently fall back to Runpod. Precreation and successful-up receipts bind the selected offer and advertised driver. Unselected placement, caps and remote driver admission remain. Focused host/mock checks and independent exact-head review are recorded with the source; no GPU/model compute, rental or new spending class. This does not qualify another driver or release replacement timing. Next: protected delivery/cleanup, pin the reviewed helper/runner and check availability without allocation; replacement packet still needs readiness and a distinct release. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged. GPU cost$0; carried36-lease estimate$3.418265556 plus separate PM, storage/transfer UNMEASURED.

## 2026-10-17 — Baseline-reuse timing startup refused before instrument execution (codex/cf1)

The [sealed startup](../labs/counterfactual-baseline-reuse-timing-startup-2026-10-17/README.md) preserves separate timing release6227893756: actualRTX5090 driver595.71.05 refused by the unchanged580.95.05-only wrapper at01:22:42Z, before any binary/CUDA/model initialization. Instrument/proposal/warmup/timed/apply dose0; no retry or gate broadening. Independent review verifies20remote/24operator/10input SHA entries and exact stopping boundary. Full archive9,547,264bytes/MD5 ef4197f63542ecd58a366b80316b308d verified before release/down/fleet0. Lifecycle88s@$0.908/h=$0.022195556; cumulative36closedCodexleases$3.418265556 plus separatePM, storage/transfer UNMEASURED. SSH succeeded after23s. #2565 finite pass remains valid on580.95.05; timing/speedup NOT_RUN. **Next:** protected receipt delivery/cleanup, then prospectively review driver-aware provider selection before any replacement timing packet/release. Operational blocker, not a mechanism negative. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged.

The [read-only provider investigation](../labs/counterfactual-baseline-reuse-timing-startup-2026-10-17/provider-selection/findings.md) confirms advertised `driver_version` is available before allocation but rejected/dropped by the current helper. The retained 01:25:10Z exact580.95.05 search found zero offers under the existing placement constraints. After this delivery, a minimal exact-driver selector can refuse unqualified offers without rental; the remote gate remains necessary for drift. No implementation or replacement execution release is included.

## 2026-10-17 — Baseline-reuse finite controls pass at the pinned artifact (codex/cf1)

The [finite record](../labs/counterfactual-baseline-reuse-finite-pass-2026-10-17/README.md) preserves separate release6227632827 and independent acceptance6227712402:32 exact U/R pairs,64 attempts(56 successes/eight expected status8),16 typed admission refusals,512 R lane records/414 producer-row outcomes, unchanged state/epoch0 and zero apply. All18 strict runtime admissions pass, including observed R512 shared bytes on RTX5090/driver580.95.05; no ELF subtraction. Exact-u64 Rust readback review and22 remote/20 operator/4 input hashes pass. Raw SHA c6a5b3f857d77bf979ed439e61575f04fce2f398b9854286681eac53203cd4f4; full archive7,257,088bytes/MD5 db89eef2659326af31c0e6cad71034e1 verified before release/down/fleet0. Lifecycle237s@$0.563/h=$0.037064167; cumulative35 closed Codex leases$3.396070000 plus separate PM, storage/transfer UNMEASURED. This is finite fixture parity, not checkpoint0 parity or speedup; driver580.105.08 parity NOT_RUN. **Next:** protected record delivery and cleanup, then separately pinned/reviewed/released32-proposal timing dose. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged.

## 2026-10-17 — D32 discrete-float ratchet, Hopper/Blackwell tensor-core intrinsics & proof-model audit expansion, source only (`antigravity`, M1 qualification & enforcement tooling)

The [record](../labs/d32-ratchet-and-proof-model-audit-2026-10-16/README.md) hardens the static host-side interlocks in `uor-r4-harness` (`ratchet` / `prlint`) and `uor-r4-proof-model` (`inference_audit`) from the `#2028` [interlock coverage audit (`6221953405`)](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6221953405) without touching any `#[cfg(feature = "cuda")]` or `required-features = ["cuda"]` target or any file hashed by `crates/uor-r4-discrete`'s `source_sha256()` or `docs/labs/k16-timing-2026-10-15/instrument`'s `standalone_source_hash()`: (1) adds `Kind::DiscreteFloat` (`uor-harness ratchet d32` / `discrete-float`, wired into `crates/uor-r4-harness/src/{ratchet.rs, prlintrules.rs, main.rs}`) blanking character literals (`'"'`, `'\"'`, `'x'`) via `strip_comments_strings_and_char_literals()` on the D32 path and scanning stripped code across `crates/uor-r4-discrete/` and `docs/labs/*/instrument/` for `D32_FLOAT_TYPES` (`f16`, `bf16`, `f32`, `f64`, `tf32`, `float`, `double`, `__half`, `__nv_bfloat16`) and Rust/CUDA floating-point literals (`1.0`, `2.`, `.5f`, `1.f`, `1.e3`, `2.e-4f`, `0..1.0`, `1e-5`, `0x1.8p-2`, `3f32`) while excluding Rust tuple indexing (`pair.0`, `pair.0.1`, `(pair).0`, `arr[0].1`, multiline `\n .1;`), integer ranges (`0..10`), and integer method calls (`i.max(1)`), and locks `scripts/no-discrete-float-baseline.json` at `{"crates/uor-r4-discrete/src/prime_zeta.rs": 31}` (`0` across all other 41 `.rs` and `.cu` discrete and standalone-instrument files); (2) extends `Kind::Matmul` (`uor-harness ratchet d25`) `matmul_sites()` with Hopper `TC_WARPGROUP` (`wgmma`) and Blackwell `TC_FIFTH_GEN` (`tcgen05`); and (3) removes `#[cfg(feature = "full-model")]` from `pub mod inference_audit;` in `crates/uor-r4-proof-model/src/lib.rs` (enabling the fixture audit engine under `--no-default-features`) and expands `InferenceAuditVerifier::{FORBIDDEN_MNEMONICS, FORBIDDEN_DEPENDENCIES}` in `crates/uor-r4-proof-model/src/inference_audit.rs` with packed AVX float ops, FMA (`fma.`, `fmadd`, `fmsub`, `fnmadd`, `vfmadd`, `vfmsub`, `vfnmadd`), float conversions (`cvtsi2ss`..`fcvtzu` plus `is_forbidden_ptx_float_cvt()` rejecting PTX float `cvt.*` while accepting integer `cvt.u64.u32`), PTX transcendentals (`ex2.`, `lg2.`, `sin.`, `cos.`, `tanh.`, `rcp.`, `rsqrt.`, `sqrt.approx`, `sqrt.rn`), PTX/SASS tensor-core instructions (`mma.`, `wmma.`, `wgmma`, `tcgen05`, `hmma`, `imma`, `qmma`), and CUDA/BLAS/ML/tensor libraries (`cudnn`, `cutlass`, `cutensor`, `cusolver`, `cusparse`, `nvrtc`, `nccl`, `candle`, `burn`, `tch`, `ort`, `onnx`, `nalgebra`, `ndarray`, `faer`, `gemm`, `matrixmultiply`, with segment-boundary matching for `ort`/`tch` so `uor-r4-support` and `uor-r4-transport` pass cleanly). Host verification (`cargo fmt --check`, `cargo test -p uor-r4-harness`, `cargo test -p uor-r4-proof-model --no-default-features`, `uor-harness ratchet d23|d25|d32`, `check_native_cuda_vendor --self-test --repo .`, `scripts/check_claim_wording.py`) passes (`model_compute_executed: false`, `apply_calls: 0`, zero pods, $0 GPU cost). Line: M1 qualification & enforcement tooling · count 0/3 unchanged · M1 headline 37/232 unchanged (suspended D28/R14); training, M2, and Stage B remain HELD; Q4 remains NOT_EXECUTABLE. **Next:** independent exact-head delta review (`Codex/cf1` / `review_2552`) and protected merge.

## 2026-10-17 — Baseline-reuse finite setup stopped at driver admission (codex/cf1)

The [sealed startup record](../labs/counterfactual-baseline-reuse-controls-2026-10-17/README.md) preserves release6227215454: actualRTX5090 driver580.105.08 refused by the reviewed580.95.05-only wrapper before any instrument command or CUDA model initialization. Instrument/proposal/typed-refusal/apply counts are0; resource parity and speedup remain NOT_RUN. Independent review verified16 remote/21 operator/4 input hashes and the exact boundary. Archive6,783,488bytes/MD5 7d22b276f4054fd5eadcc6478baf2fa8 verified before down/fleet0. Lease232s@$0.563/h=$0.036282222; cumulative34-lease Codex$3.359005833 plus separate PM, storage/transfer UNMEASURED. A prospective finite-only driver allowlist580.95.05/580.105.08 preserves the exact binary, resource gates and64+16 dose; replacement readiness and distinct release follow this protected receipt. No kernel change or model-negative conclusion. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged.

## 2026-10-16 — Preserve raw baseline-reuse resource refusal evidence (codex/cf1)

The [bounded repair](../labs/counterfactual-reuse-resource-refusal-2026-10-16/README.md) follows verified protected #2561 delivery (6209e11ae; full reviewed-tree equality). Independent finite-readiness review found that the correct shared-memory rejection discarded actual queried resource values before the caller could record them. The existing error now carries those values and compiler/required shared counts; loader, predicate, kernel and dose are unchanged. This is a pre-lease source finding, zero attempted finite dose, not a GPU result. Exact-head source/CUDA construction review and protected delivery precede replacement finite pins; the external wrapper must also prospectively admit its driver. Prior32-lease estimate$3.270958333 plus separate PM; storage/transfer UNMEASURED. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged.

## 2026-10-16 — Opt-in exact GPU baseline-reuse source implementation (codex/cf1)

The [implementation record](../labs/counterfactual-baseline-reuse-implementation-2026-10-16/README.md) follows protected #2560's independent design FIT and source commission6226210469. A separate Route-only R module computes unchanged baseline eligibility once per row in512 shared bytes, one barrier, with deferred row faults and original virtual/chooser semantics. Ordinary CUDA bytes remain unchanged; a separate exact source/PTX/cubin/report binding,1280-byte audit and existing-instrument finite/timing commands keep the comparison explicit. Fixed64 finite attempts/16 refusals precede a separately released32-proposal timing dose; all model observations and speedup NOT_RUN. Host metadata tests47+3 pass; strict standalone Clippy passes. Exact-head independent source review, actual CUDA construction and protected delivery remain pending. Two pre-compilation startup/preflight failures are archived and closed; the recipe receipt-path check is repaired without changing model source. New lifecycle estimate$0.061148056, cumulative31-lease Codex estimate$3.235614444 plus separate PM; storage/transfer UNMEASURED. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged. **Next:** source/construction approval and protected delivery, then distinct finite pins/release; no automatic timing or training.

## 2026-10-16 — D29 vendor, training, discrete & standalone-instrument static interlock hardening, source only (`antigravity`, M1 qualification & enforcement tooling)

The [record](../labs/d29-vendor-discrete-interlock-hardening-2026-10-16/README.md) closes the host-accessible portions of Audit Ranks 2, 3, and 5 (`#2028` comments `6221953405`, `6222117146`, `6222315060`; cross-lab reviews `6225372934`, `6225692582`, `6225805388`, `6225952834`) in [`scripts/check_native_cuda_vendor.rs`](../../scripts/check_native_cuda_vendor.rs) without modifying any `#[cfg(feature = "cuda")]` / `required-features = ["cuda"]` target or any file hashed by `crates/uor-r4-discrete`'s `source_sha256()` or `docs/labs/k16-timing-2026-10-15/instrument`'s `standalone_source_hash()`: (1) makes the source-language distinction (`SourceLang`: `code_cuda`, `code_ptx`, `code_rust`, `code`) explicit across all file checks in `verify()` while preserving both C/C++ single- and multicharacter literals (`'"'`, `'/*'`, `'*/'`, `'a/*'`, `'a+/*'`), C++14 decimal/hex digit separators (`1'000`, `0xdead'0000`), Rust raw strings (`r#"..."#`), and Rust lifetimes (`'a /* ... */, 'b`) in `code()`/`forbidden()`, and expands `forbidden()`/`forbidden_cuda()` with `uelu_`, `wgmma`, `tcgen05`, `ulog_`, `__nv_log`, `__nv_tanh`, `__nv_erf`, exact-word `erf`/`erff`/`erfc`/`erfcf`/`erfinv`/`erfinvf`/`tanhf`/`elu`/`sgemm`/`dgemm`/`hgemm`/`gemv`, and instructions `wgmma.`, `tcgen05.`, `lg2.`, `sin.approx`, `cos.approx`; (2) adds `strip_ptx_predicates_and_labels` and `forbidden_ptx()` splitting PTX on `['\n', ';', '{', '}']` and inspecting both stripped statements and stripped whitespace tokens (after stripping leading PTX predicate guards `@%p0`, `@!%p1`, `@ %p0` and single-colon instruction labels `$L__BB0_1:` while preserving `::` qualifiers `tcgen05.mma.cta_group::1.kind::f16`) to reject `ex2.`, `lg2.`, `sin.`, `cos.`, `tanh.`, `rcp.approx`, `sqrt.approx`, `rsqrt.approx`, `mma.`, `wmma.`, `wgmma.`, and `tcgen05.` across multi-statement and brace-delimited PTX layouts; (3) binds `crates/uor-r4-training/build.rs` `Module.sources` declarations directly to `TRAINING_COMPILED_CU` (`check_training_build_sources`) and verifies `crates/uor-r4-training/src/{cuda_stack.cu, spiralcore_fbs.cu, starling_native.cu}` (`forbidden_training_cuda` plus `.cu`/`.cuh` directory, `#include`, and historical/quarantined kernel exclusion via `global_entries()`); (4) verifies both `crates/uor-r4-discrete/src/discrete.cu` and `decomposition_source::generate(&discrete_cu)` (`forbidden_discrete_cuda` plus `cuda_float_literals` rejecting decimal/hex floating literals `1.0f`, `2.0`, `decltype(1.0f)`, `.5f`, `1e-3`, `0x1.0p0f` and D32 floating-point/transcendental/tensor-core tokens, plus `global_entries(..., true)` verifying the exact 11-entry `DISCRETE_ENTRIES` and 1-entry `discrete_counterfactual_decomposition` censuses); (5) freezes the 5-root `STANDALONE_INSTRUMENT_MANIFESTS` inventory (`check_standalone_instrument_inventory`) and scans runtime `.rs` sources and `APPROVED_BUILD_SCRIPTS` across training, discrete, and all 5 standalone instruments (`forbidden_runtime_compiler_rust` rejecting `nvrtc::`, `compile_ptx`, `cuLinkCreate`, and direct `nvcc`/`NVCC`/`ptxas`/`nvlink` subprocesses while preserving lawful prebuilt-module loading); and (6) extends `cudarc` manifest and `[patch.crates-io]` inspection across `crates/uor-r4-discrete/Cargo.toml` and all 5 `STANDALONE_INSTRUMENT_MANIFESTS`. Host verification (`check-native-cuda-vendor --self-test --repo .`, `uor-harness ratchet d23`, `uor-harness ratchet d25`, `r4-policy-check`, `uor-harness check`, `scripts/check_claim_wording.py`) passes (`model_compute_executed: false`, `apply_calls: 0`, zero pods, $0 GPU cost). Line: M1 qualification & enforcement tooling · count 0/3 unchanged · M1 headline 37/232 unchanged (suspended D28/R14); training, M2, and Stage B remain HELD; Q4 remains NOT_EXECUTABLE. **Next:** independent exact-head review (`Adversarial Interlock Reviewer` / `Codex/cf1`) and protected merge.

## 2026-10-16 — Exact baseline-reuse design after sealed counterfactual attribution (codex/cf1)

The [docs-only design](../labs/counterfactual-baseline-reuse-2026-10-16/README.md) and [pre-registration](../harness/prep/m1-counterfactual-baseline-reuse-2026-10-16.md) select the smallest source change supported by protected #2559: one call-local GPU baseline result per row, shared across sixteen candidate consumers. Same grid1/block16, native read/writer, virtual law, ordered sum and chooser;512-byte shared row metadata, one barrier, deferred fault publication and no persistent cache. Fixed64 finite proposal attempts/16 admission refusals precede a separately released32-proposal timing dose; exact parity and four-cell range/spread criteria are prospective. TEST FITNESS FIT for design only at comment6226136376; implementation/new measurements NOT_RUN, zero leases/$0 new GPU cost. Inherited29-lease Codex estimate$3.174466388 plus separate PM, storage/transfer UNMEASURED. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged. **Next:** protected delivery, then narrow source/construction review; no execution release.

## 2026-10-16 — Counterfactual decomposition resolves repeated baseline and writer replay (codex/cf1)

The independently reviewed [Stage2 receipt](../labs/counterfactual-decomposition-run-2026-10-16/README.md) completes the original checkpoint diagnostic under distinct release6225716504 after protected Stage1 #2557. All 32 proposals (8 warmup/24 timed), 16 U/D pairs, 256 D lanes and 2 resource admissions pass; original bytes, policy snapshots and epoch0 remain unchanged, apply0. Across every timed lane in all four cells, baseline read/writer/eligibility is46.921799–47.220974% and writer phases99.979269–99.979452% of lane cycles; selection/member scans94.104057–94.171533%. H2/H3 are resolved under their prospective bars; E/S bounds0.739–0.767% are below20%. H1 magnitude and causal speedup remain unmeasured, enclosing profiling-off control absent. The26,301,440byte archive, MD5`d02a60124645611ea82720f1ff99513b`, and26remote/18operator/5input SHA entries verify before teardown/fleet0. Lease492s@$0.536/h=$0.073253333 estimate; cumulative29-lease Codex$3.174466388 plus separate PM, storage/transfer UNMEASURED. **Next:** protect this receipt, then pre-register exact per-row GPU baseline reuse across candidates with the same native writer and virtual semantics. No optimization code or execution release in this delivery. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged.

## 2026-10-16 — Complete repaired-image counterfactual finite controls; Stage2 remains gated (codex/cf1)

The [Stage1 receipt](../labs/counterfactual-controls-pass-2026-10-16/README.md) records the single released invocation after protected #2556:40 proposals (32status0/8expectedstatus8),128 immutable GPU endpoints,16 typed refusals,20 complete U/D pairs,320 decoded D lanes and12 resource checks PASS, apply0. Runtime/AOT registers now agree U88/88,D126/126, local21,536/22,096, shared0, zero AOT spills; unchanged zero-delta predicates pass. Independent outcome review verified every record and22remote/12operator/2input SHA entries. This establishes the new sealed binary/device path; driver also changed, so it does not isolate the cause of run1's mismatch. Raw JSON is retained losslessly compressed and in the full27,561,472byte archive, MD5`9deeaaf0e7377b722eae3a1fb68b3b1f`, verified before teardown/fleet0. Lease305s@$0.563/h=$0.047698611 estimate; cumulative28-lease Codex$3.101213055 plus separate PM, storage/transfer UNMEASURED. **Next:** protected receipt, original checkpoint/version2 pins, independent readiness and distinct Stage2 release. No timing attribution, speedup, training or headline claim. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged.

## 2026-10-16 — Bind loaded counterfactual cubins to their admitted PTX and resource reports (codex/cf1)

The [source repair](../labs/counterfactual-image-binding-2026-10-16/README.md) follows the sealed #2555 resource-admission blocker. Cubins now assemble from the exact admitted PTX and all three loaders consume those embedded binary images; image/report/target hashes bind the composite source, version2 execution pins and narrow evaluation-only checkpoint bridge. Mathematical CUDA, geometry, launch, fixture dose and numerical zero-delta resource gate remain unchanged; runtime parity is NOT_RUN and Stage2 remains NOT_RELEASED. Independent design review required the admitted-PTX assembly chain; source/host/construction checks are recorded separately, without a speedup or model claim. **Next:** exact-head independent source review and compile-only CUDA evidence, protected delivery, then new execution pins and separate bounded Stage1 release. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged.

## 2026-10-16 — Finite-control admission blocked by PTX/AOT resource provenance (codex/cf1)

The [sealed Stage1 record](../labs/counterfactual-controls-run-2026-10-16/README.md) follows protected #2554, independent source/design/construction reviews, AGY concurrence and separate bounded release6224573422. Exact executable/pin identities pass; runtime registers U90/AOT88 and D122/AOT126 violate the required zero-delta gate, despite matching local bytes21,536/22,096 and zero AOT spills. Command exits2 before carry or any of the40 proposals,128 endpoints or16 admission-refusal attempts. GPU initialization/snapshot validation did execute; apply0. The retained record has only compiled identity and first resource reports. Source inspection shows the loaders JIT embedded PTX while the compared reports come from separately compiled cubins. This is a blocked instrument/artifact-provenance path, not a mechanism negative or parity result. All22 remote SHA entries and cloud MD5 readback verified before teardown; fleet0. Lease307s@$0.96/h=$0.081866667 estimate, cumulative26-lease Codex$3.009592222 plus separate PM cost; storage/transfer UNMEASURED. **Next:** source-only alignment of loaded machine image and resource report, preserve the gate, independent review/protected delivery/new pins/separate release before retry. Stage2 diagnostic remains NOT_RELEASED. Owner native-math/minimal-surface direction governs the later optimization. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged.

## 2026-10-16 — Counterfactual finite-control source preparation; execution remains blocked (codex/cf1)

The [control record](../labs/counterfactual-controls-2026-10-16/README.md) and [prospective packet](../labs/counterfactual-controls-2026-10-16/execution-packet.md) continue merged #2550/#2552's concrete prerequisites: small admitted GPU U/D eligibility, identity/tie and virtual-endpoint controls, fault-aware receipts and binding refusals. The proposed inventory is40 proposal attempts and128 one-row endpoint evaluations; all results NOT_RUN, TEST FITNESS PENDING, no model execution release. Independent source review caught and replaced an inert context3 Route(0) fixture with context2 and added virtual-ineligible/MAX early-break coverage. The packet explicitly distinguishes unreachable CUDA2/4/9/12 and unexecuted stale-epoch scope from actual admission refusals; this amendment needs independent FIT. Production CUDA/geometry/loader and original32-proposal diagnostic dose stay unchanged. Owner directs Codex's own work and PM routing with parallel independent review; SpiralCore remains separate DeepSeek research. Previous readiness branch/worktree cleanup and hygiene completed. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged, control GPU cost$0. **Next:** source/packet review, exact-head CUDA construction gate, protected delivery, then separately reviewed execution pins/release.

## 2026-10-16 — Interlock & ratchet hardening (D23 / D25 / D32, Commission 1), source only (`antigravity`, M1 qualification & enforcement tooling)

The [record](../labs/interlock-ratchet-hardening-2026-10-16/README.md) delivers the consolidated Commission 1 hardening (`#2028` comments `6222315060`, `6222117146`, `6222114728`, Ranks 7–11; hold `6222232136` released): (1) extends `Kind::NormExp` (`uor-harness ratchet d23`) in [`crates/uor-r4-harness/src/ratchet.rs`](../../crates/uor-r4-harness/src/ratchet.rs) to count `D23_IDENT_FAMILIES` (`softplus`, `sigmoid`, `tanh`, `gelu`, `silu`, `rope`, `ropei`, `rotary` — including C float intrinsics like `tanhf` and CamelCase/PascalCase compounds like `RopeMode`, `RopeTheta`, `PreRopeProjectionProbe`, `RotaryEmbedding`, `SigmoidGate`) on stripped code (`strip_comments_and_strings`) across ASCII-alphanumeric identifier segments alongside raw `D23_WORD` (`softmax`), and promotes `TC_FRAGMENT` (`wmma`) and `TC_SYNC` (`mma_sync`) in `Kind::Matmul` (`uor-harness ratchet d25`) from informational to gated `"tensor-core"` sites in `result.counts`/`result.detail`; (2) expands `SCAN_DIRS` in `ratchet.rs` from `["crates"]` to `["crates", "docs/labs/*/instrument"]`; (3) updates [`scripts/no-softmax-baseline.json`](../../scripts/no-softmax-baseline.json) (`140 -> 154` files, `1,641 -> 2,253` sites, `+612` across `41` files) with a complete per-file census (`scripts/no-matmul-baseline.json` unchanged at `26` files / `93` sites, `0` gated tensor-core sites), flagging active-path Finding 1 (`docs/labs/k16-timing-2026-10-15/instrument/src/main.rs:1254`, `0 -> 1` from negative unit-test fixture `"softmax_margin_v1"`, left untouched to preserve `#2550`'s instrument source SHA), Finding 2 (`crates/uor-r4-training/src/lib.rs`, `4 -> 10` inside `#[cfg(any())] ReferenceConfig` / `ReferenceModel` and `3` `.matmul(` inside `#[cfg(any())] ReferenceModel`), and Finding 3 (`crates/uor-r4-training/src/cuda_stack_kernels.rs`, `5 -> 5` raw `"softmax"` in doc-comments and `EXCLUDED_KERNEL_NAMES` rejection list, `0` stripped-code hits); (4) updates `interlock_holds` in [`crates/uor-r4-harness/src/prlintrules.rs`](../../crates/uor-r4-harness/src/prlintrules.rs) to run both `Kind::NormExp` (`d23`) and `Kind::Matmul` (`d25`) in process; and (5) updates [`crates/uor-r4-core/src/bin/r4-policy-check.rs`](../../crates/uor-r4-core/src/bin/r4-policy-check.rs) from stale `/schema: /5` and `training_matrix_multiplication_allowed: true` to `/schema == "uor-r4.agent-execution-policy/6"`, `training_matrix_multiplication_allowed == false`, and the D25/D32 discrete/no-matmul commitments, with unit tests. All 543 unit tests + 13 integration tests + 1 doctest in `uor-r4-harness`, 2 unit tests in `r4-policy-check`, `uor-harness ratchet d23`, `uor-harness ratchet d25`, `uor-harness check`, and `scripts/check_claim_wording.py` pass on host (`model_compute_executed: false`, `apply_calls: 0`, zero pods, $0 GPU cost). Line: M1 qualification & enforcement tooling · count 0/3 unchanged · M1 headline 37/232 unchanged (suspended D28/R14); training, M2, and Stage B remain HELD. **Next:** PM-seat exact-head review and R-queue merge when called.

## 2026-10-16 — Counterfactual instrument merged; final execution readiness remains blocked (codex/cf1)

[Instrument #2550](https://github.com/UOR-Foundation/uor-r4/pull/2550) merged98f29796f; fresh-main whole-tree identity equals approved0adf1b88e. Exact-head CUDA check/strict-clippy/no-context identity pass; D126 registers/22,096 stack/zero spills, ordinary PTX retained. The [readiness record](../labs/counterfactual-execution-readiness-2026-10-16/README.md) carries prospective source/PTX pins, all calibration/provenance, startup failure and24-lease cost ($2.777411666 estimate plus separate PM cost; storage/transfer UNMEASURED). Old owned branch/worktree removed after MD5-verified receipt preservation. Owner delegates coordination to Codex+AGY while Kimi is unavailable; reciprocal review is in progress. Finite GPU controls, executable/transfer/runtime-resource admission and separate execution release remain outstanding; diagnostic/model/parity NOT_RUN, no optimization or training. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged.

## 2026-10-16 — Counterfactual decomposition instrument: separate D module, model execution NOT_RUN (codex/cf1)

The [source record](../labs/counterfactual-instrument-2026-10-16/README.md) implements PM commission6220832102 after #2545 design FIT: a separate generated integer clock/counter module, matched U/D driver, full loss/status/proposal parity and an exact evaluation-only source/PTX-pair checkpoint0 bridge; ordinary CUDA source/launch and loader refusal remain intact. The [execution packet](../labs/counterfactual-instrument-2026-10-16/execution-packet.md) carries all six FIT requirements, explicit resource/perturbation UNRESOLVED outcomes and a deliberately unfilled target-identity pin; local host metadata checks pass, and exact-head compile-only gate/cost receipts are tracked on [source PR #2550](https://github.com/UOR-Foundation/uor-r4/pull/2550). PM amendment6222308938 prospectively supersedes the uncalibrated U+16 ceiling after the cap introduced56/56 spills: restore uncapped D, require exact runtime/AOT resource consistency, zero spills, shared equality/local allowance, and disclose block16 launchability. All calibration receipts are retained; the changed head needs a fresh compile gate and PM delta review. GPU parity, resource match and the 32-proposal diagnostic remain NOT_RUN pending separate execution gates. No optimization, model compute or training; qualification instrumentation count0/3 and suspended headline37/232 unchanged, training/M2/Stage B HELD, Q4 NOT_EXECUTABLE. **Next:** exact-head compile-only pod transcript, PM-seat review/protected merge, then separately reviewed final execution pins/parity/release; no automatic diagnostic run.

## 2026-10-16 — Counterfactual decomposition pre-registration: PM-seat FIT pending (codex)

The [design record](../labs/counterfactual-decomposition-2026-10-16/README.md) and [sheet](../harness/prep/m1-counterfactual-decomposition-2026-10-16.md) answer PM commission6217437395 from fresh `90c2d2b9d`: distinguish single-block serial work, repeated baseline/eligibility, writer vocabulary/cell replay and API allocation/transfer using sealed #2543 and source cost card6209851483. Each hypothesis predicts observable sub-phases, names a discriminator and bounds its next optimization; the proposed matched diagnostic retains full integer parity, zero apply, event/host overlap accounting and an explicit new source/PTX identity-bridge dependency. Docs only; TEST FITNESS PENDING, all new measurements NOT_RUN, no kernel change or lease, $0 new GPU cost; inherited18-lease estimate$2.230435278 is unchanged, storage/transfer charges UNMEASURED. **Next:** Kimi PM-seat TEST FITNESS, then PM-called separate source work and execution gates; no automatic optimization or compute. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE; qualification instrumentation count0/3 and suspended headline37/232 unchanged.

## 2026-10-16 — Sealed propose sub-phase diagnostic: counterfactual dominated (codex)

The [sealed execution](../labs/propose-subphase-diagnostic-run-2026-10-16/README.md) uses merged `8199af007`, exact checkpoint0 bridge/pins, one excluded warmup plus three timed batches per A/R arm, zero apply. All four operation/policy cells satisfy the prospective counterfactual-dominated decision: every timed event is 99.995770–99.998097% of enclosing propose wall. Chooser spans13.344–42.495µs; host status-read waits overlap the counterfactual interval, not independent transfer overhead. All16 host partitions/clock decodes and both unchanged snapshot/epoch checks pass; build/diagnose/job exits0. Diagnostic only, not qualification/p95/training rate. All26 SHA entries and cloud MD5 round-trip verify before release/delete; fleet0. Lease468s@$0.908/h=$0.118040; cumulative18-lease episode estimate$2.230435278, not invoices; storage/transfer USD unmeasured. **Next:** independent receipt review and a NEW reviewed pre-registration for counterfactual candidate/token/context decomposition; no automatic optimization or extra compute. Qualification/training/M2/Stage B HELD, Q4 NOT_EXECUTABLE; count0/3 and suspended headline37/232 unchanged.

## 2026-10-16 — Checkpoint0 identity bridge and portable compile recipe (codex)

The [bridge delivery](../labs/checkpoint0-identity-bridge-2026-10-16/README.md) records PM FIT (design) in the sheet and adds an exact-artifact/exact-source-pair, evaluation-only in-memory adapter to the standalone timing instrument. The original checkpoint file remains unchanged; only its authenticated source identity is adapted in memory, with pre/post hashes and bridge-used evidence recorded before device admission. Production loader/source/PTX unchanged. Host metadata tests26 pass plus the explicitly run sealed-byte test; no model execution. The [compile recipe](../labs/compute.md#reproducible-fresh-pod-cuda-compile-gates-16-october) makes explicit the fresh-pod toolchain and check/strict-clippy invocation. **Next:** new-head CUDA transcript and DeepSeek review, protected merge and final execution packet. Diagnostic NOT_RUN; qualification/training/M2/Stage B HELD, Q4 NOT_EXECUTABLE, count0/3 and headline37/232 unchanged.

## 2026-10-16 — Proposal sub-phase instrumentation and pending FIT (codex)

The [source-only diagnostic](../labs/propose-subphase-diagnostic-2026-10-16/README.md) adds precreated same-stream counterfactual/chooser events and nine disjoint host spans while preserving the ordinary proposal path and original caller timer. The [sheet](../harness/prep/m1-propose-subphase-diagnostic-2026-10-16.md) is TEST FITNESS: PENDING: one excluded warmup plus three timed batches per checkpoint0 A/R arm, diagnostic only, not qualification/p95/training rate. Owner instrument+optimize-first defers 100-batch qualification timing. Source hashing necessarily changes; the old checkpoint loader refusal is preserved, with a separately reviewed evaluation-only checkpoint0 identity bridge required before execution. No diagnostic/model/fixture execution is authorized by this delivery. DeepSeek holds source review; Kimi plus independent reviewer holds FIT; Codex supplies compile-only CUDA gates. Qualification/training/M2/Stage B HELD, Q4 NOT_EXECUTABLE, count 0/3 and suspended headline 37/232 unchanged.

**Next:** DeepSeek exact-head source review and Kimi/independent FIT; preserve the source-identity blocker until its separately reviewed bridge.

## 2026-10-16 — K16 restart 5: G-Pre and C-R1..C-R6 pass; timing emits 99/100 batches (codex)

The [sealed restart-5 record](../labs/k16-qualification-restart5-2026-10-16/README.md) executes merged `812459c7f`, byte-identical in the fixture instrument to gate-checked `3e3a5892e`. All six CUDA build/clippy commands pass. G-Pre K8 admits 5,680/6,205 rows for seeds 1/2; all four K16 arms admit 14,287/14,287, with zero updates. Full K16 admission is the expected 16-occupied-cell capacity/pass-through boundary, not discrimination; top-1 is 19/14,287 and strict-win baseline `u=0`. Each actual GPU C-R1..C-R6 receipt passes. The timing process then exits 0 but emits only 99 timed batches per cell (100 generated windows minus one warmup); the separate required-100 receipt check exits 1 and stops execution. Diagnostic A/R rates 846/845 padded-context tokens/s do not discharge qualification. No patch/retry, training or corpus apply; Q4 NOT_EXECUTABLE. The 109,805,056-byte archive includes full transcripts, checkpoints, raw evaluations, fixture receipts, timing record, binaries and PTX; all 133 SHA entries and cloud MD5 round-trip verify before pod deletion, fleet empty. Current lease estimate $0.852291; all 13 episode leases $1.773399, not invoices. **Next:** PM-routed DeepSeek timing repair and exact-head review, preserve passing controls, no further compute without disposition. Qualification/training/M2/Stage B remain HELD; D22/D31 blocker, not a mechanism negative, count 0/3 and suspended headline 37/232 unchanged.

## 2026-10-16 — K16 restart 4 halted at mandatory fixture clippy; no model execution (codex)

The [sealed restart](../labs/k16-qualification-restart4-2026-10-16/README.md) at merged `5fc68a9a6` verifies the #2536 instrument byte-identical to approved `11e662a81`. All three CUDA release builds and driver/timing clippy pass; fixture clippy exits101 on `Parameter::VectorBlock(3 * 4 + 0)` (`clippy::identity_op`, denied by the declared `-D warnings` gate). Execution stops without patch/retry. G-Pre, each C-R1..C-R6 and checkpoint0 timing remain NOT_RUN; K16 rate UNMEASURED, Q4 NOT_EXECUTABLE. The 162,304-byte archive passes the MD5 round-trip before pod deletion; fleet verified empty. This lease estimate $0.064745; full episode including all repair leases $0.782706 on consistent provider-uptime accounting, not invoices. **Next:** PM-routed bounded clippy repair, exact-head review, protected merge and full step0 re-entry; qualification/training/M2/Stage B remain HELD. D22/D31 blocker, not a mechanism negative; count0/3 and suspended headline37/232 unchanged.


## 2026-10-16 — C-R6 raw-receipt interface repaired consumer-side (deepseek, qualification)

Codex's restart preflight found a deterministic C-R6 receipt-interface blocker at `bb536cf68` before any build: raw `eval-corpus` driver receipts emit the evaluation at the top level, while the fixture runner read only the wrapped `evaluation` object, so the K8 guard failed for a reason unrelated to the measured counts. The [repair](#2533) is **consumer-side and bounded** — a host-testable `driver_evaluation(val, receipt)` that accepts the raw driver shape, leaves completion-shaped receipts unchanged, and **refuses** anything else by name so a missing key cannot read as a failed bar. Two read sites updated; 3 new tests (raw acceptance, wrapped acceptance, malformed refusal); 11 host tests pass. No producer change, no receipt rewriting. The exact-head CUDA transcript is Codex's under the amended rule. Qualification HELD; Q4 NOT_EXECUTABLE. Line: M1 qualification · count 0/3 unchanged · headline unchanged.
**Correction after Codex's confined review (6207549883):** the accessor returns the **metrics object**, not the receipt — `evaluation.aggregate` for a completion envelope, top-level `aggregate` for a raw driver receipt — recognition is by **named schema** (`uor-r4.discrete-corpus-driver/1`, `uor-r4.discrete-objective-evaluation/1`, `uor-r4.completion-evaluation/1`), and the zero-update guard reads the same descended object so a real completion envelope can pass. The first version read metrics one level too high and its regression masked that with a root-level field the producer never emits; the fixtures are now producer-faithful, with a case whose metrics sit at the root only, so a missing descent fails the regression.

## 2026-10-16 — Qualification restart stopped before execution: G-Pre/C-R6 receipt-key mismatch (codex)

The [restart record](../labs/k16-qualification-restart-2026-10-16/README.md) verifies merged `bb536cf68` against the CUDA-checked repair head, then identifies a separate static interface blocker: the driver writes `aggregate` while C-R6 reads `evaluation` from the same G-Pre receipts. This was discovered after lease request but before setup/build/model execution, so all gates remain NOT_RUN, not failed measurements; K16 rate UNMEASURED and Q4 NOT_EXECUTABLE. The source evidence was cloud MD5-verified before pod deletion; this lease cost an estimated $0.029647, and the complete ledger-derived chain including all pre-merge setup/check attempts totals $0.567979, not an invoice. **Next:** PM-routed receipt-schema repair with regression coverage, required CUDA-check evidence, exact-head review and protected merge before a new step0 restart; no mechanism negative/count change, training/M2/Stage B HELD.

## 2026-10-16 — C-R fixture runner repaired: compiled identity is a JSON object, not a struct (deepseek, qualification)

The qualification halted at step 0 on E0609 (#2029 6206185197) because  and  read // as struct fields of , which returns ; the code is cuda-gated so no host compiled it. The repair adds a fail-closed typed accessor (, error rather than empty string), makes  return , and propagates  at all four call sites. This PR carries the pod-side  transcript required by the new pre-merge rule. Line: M1 qualification · count 0/3 unchanged · headline unchanged.
## 2026-10-16 — K16 qualification halted at fixture CUDA build; archived, pod deleted (codex)

The [sealed attempt](../labs/k16-qualification-2026-10-16/README.md) at merged `3c651df97` passed the discrete and timing CUDA release builds but failed the fixture-runner build with five E0609 errors: `device::compiled_identity()` returns JSON, not a typed struct. Step 0 halted before all clippy commands and before G-Pre/C-R1..C-R6/timing; K16 throughput remains UNMEASURED, Q4 NOT_EXECUTABLE, and no model computation or corpus apply occurred. The 97,280-byte archive passed the cloud MD5 round-trip before release/deletion of `vast-56182561`; 447 s provider uptime at $0.815/h gives a $0.101196 estimate, not an invoice. **Next:** DeepSeek identity-access repair with pod-side CUDA check transcript, Codex exact-head review and protected merge per PM routing amendment 6206222931, then full step-0 re-entry; this is a D22/D31 blocker, not a mechanism negative, with headline/count unchanged and M2/Stage B HELD.

## 2026-10-16 — K16 timing instrument repaired: cells execute their source and policy (deepseek, qualification)

Codex's PRELEASE_SOURCE_BLOCKER (#2029 6205684341) found a real defect in the merged K16 instrument: `load_fixture` ran unconditionally, `validate` demanded Q4 pins whatever the selected cells were, and every cell reused **one** device and **one** objective policy with source/policy printed as labels — a four-cell record would have measured a single arm. The [repair](../labs/k16-timing-2026-10-15/README.md) builds **one device per cell** from that cell's pinned artifact, **applies the cell policy to the snapshot before admission** and asserts it after, and makes pin requirements follow the selected cells (checkpoint0-only panels need no Q4 fixture; Q4 cells stay refused without sealed pins). 18 host tests pass, including three new boundary tests. The device-side half is CUDA-gated and awaits Codex's confined exact-head check; until merged, the shipped instrument is plan-only for the multi-cell cross product. Line: M1 qualification · count 0/3 unchanged · headline unchanged.

## 2026-10-15 — Standalone `C-R1..C-R6` GPU fixture runner on `objective.rs` scaffolding, source only (`antigravity`, M1 qualification instrumentation)

The [record](../labs/admission-rank-fixture-runner-2026-10-15/README.md) delivers the standalone dated-record crate (`docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/Cargo.toml`, detached `[workspace]`, path dependency on `uor-r4-discrete = { path = "../../../../crates/uor-r4-discrete" }`, `cuda = ["uor-r4-discrete/cuda", "dep:cudarc"]`) that executes the six `D-R` GPU fixture controls `C-R1..C-R6` on Codex's [`crates/uor-r4-discrete/src/objective.rs`](../../crates/uor-r4-discrete/src/objective.rs) scaffolding without modifying any file in `crates/uor-r4-discrete/` (`git diff --stat origin/main -- crates/` is empty; `source_sha256` and `driver_source_sha256` unchanged). Subcommands: (1) `plan` emits the 6-control `uor-r4.admission-rank-fixture-runner/1` plan (`status: "NOT_RUN"`, `model_compute_executed: false`, `apply_calls: 0`) aligned with `uor_r4_discrete::objective::control_scaffold()`; (2) `check` verifies on host (`cuda_context_created: false`, `model_compute_executed: false`, `apply_calls: 0`) the 6 `CONTROL_SCAFFOLD` IDs/names, objective ABI constants (`ROW_WORDS = 14`, `HISTOGRAM_START = 29`, `HISTOGRAM_BINS = [641, 73, 256]`, `QUANTILE_START = 999`, `AGGREGATE_WORDS = 1020`), all 19 `Spec` / `ReadSpec` configurations against `Spec::validate`, the 16 E8 anchor-orbit roots and 16-candidate generator root-image tables, the `1..=16` `admitted_cells` guard, and the sealed 36-point Stage-A $K$-sweep (`cc81ca6f...`); and (3) `#[cfg(feature = "cuda")]` `run NEW_OUT_DIR --g-pre-dir G_PRE_DIR` executes all six GPU fixture controls (`C-R1` across $K = 1..=16$ on both mixed distances and the 5-way equal-distance tie `[(16,0),(16,1),(16,15),(16,16),(16,255)]` plus all-256-cell exhaustion at $K = 16$; `C-R2` crowding rival `cell 1` with $m \in \{1, 2, 4\}$ tokens at $K \in \{1, 2, 3\}$ and moving one spare token into newly occupied preceding `cell 2` across $K \in \{1, 2, 3, 4\}$ (`cell_rank_hinge` `[2, 1, 0, 0] -> [3, 2, 1, 0]`); `C-R3` virtual `Device::propose` `candidate_losses()[c]` vs. independently restored candidate `Snapshot` scored via `Device::evaluate` across all 5 mutation classes (`create_cell`, `empty_cell`, `move_target_cell`, `interior_block_only`, `identity_candidate`), both policies, and all 16 candidate ordinals (`160` GPU evaluations, `0` `.apply` calls); `C-R4` $M = 1$ and $M = 16$ occupied cells at $K = 16$, `tr == 0` row skipping in `propose`, and status-`8` refusal on wholly ineligible batches; `C-R5` multi-row `Device::evaluate` + `Device::aggregate_evaluations` verifying dual-loss sums, `ExactRatio` shares (`address_loss_share_ta`, `rank_loss_share_tr`), `mean_admitted_token_count`, `[641, 73, 256]` histograms ($\sum\text{bins} == \text{scored\_rows}$, `zero_rows == bins[0]`, `mean`), and `min/p25/p50/p75/p90/p99/max` nearest-rank quantiles, plus direct GPU `discrete_evaluation_aggregate` status-`12` refusal across all 11 out-of-bounds (`T=641`, `A=73`, `R=256`, `rank=0/257`, `occ=0/257`, `adm=0/17`, `loss_ta/loss_tr` mismatch, cross-row `occ` mismatch) and `u64` overflow (`u64::MAX + 1`, weighted $R$ overflow, quantile threshold overflow) inputs; and `C-R6` host guard/sweep check, GPU `V4096/B4/C192` checkpoint-0 `Device::new_with_objective` across `(seed, K) in {(1,8),(1,16),(2,8),(2,16)}`, and `G-Pre` zero-update corpus receipt verification), atomically writing `c-r{1..6}/report.json` and `c-r-suite-summary.json` with `ControlReceipt` entries matching `uor-r4.redirect-qualification/1` (`redirect_admission.rs:41-46, 357-367`). **8 host unit tests, `plan`, `check`, and default-feature `cargo clippy --locked --offline --all-targets -- -D warnings` pass with zero warnings (`host-validation.txt`); `--features cuda` compilation, clippy, and GPU execution remain `NOT_RUN` / `UNMEASURED` until the pod-side step-0 gate at qualification.** Source-only, zero pods, zero GPU/model compute.

Line: M1 qualification instrumentation · count 0/3 unchanged · M1 headline 37/232 unchanged. **Next:** PM-dispatched delta re-review of `#2531` and `#2522` R-queue merge before qualification lease.

## 2026-10-15 — D-R qualification runbook (`C-R1..C-R6`, `G-Pre` `K=16` checkpoint-0 ceiling, step-0 CUDA build/clippy gate, 100-batch `K=16` timing, pod & `cloud-store` MD5 discipline, `NOT_TRAINING`) (antigravity, starling read / exact addressed memory)

[`docs/labs/admission-rank-qualification-runbook-2026-10-15/README.md`](../labs/admission-rank-qualification-runbook-2026-10-15/README.md) specifies the verbatim qualification runbook for the `D-R` admission-rank objective slice ([`docs/labs/admission-rank-hinge-2026-10-15/README.md`](../labs/admission-rank-hinge-2026-10-15/README.md), `#2518` at `2fa663b54`, `driver_source_sha256 = ed4655a22dbec24316cfe5df16a43407860651ce6aa7a690ee8fcd5bedaf05d8` post-`#2525`) under the `TEST FITNESS: FIT (design)` redirect sheet ([`docs/harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md`](../harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md), `#2515` at `5188f2e38`), incorporating `#2523` ([`docs/labs/admission-redirect-packet-2026-10-15/README.md`](../labs/admission-redirect-packet-2026-10-15/README.md), `27c82c0c8`), `#2524` ([`docs/labs/k16-timing-2026-10-15/README.md`](../labs/k16-timing-2026-10-15/README.md), `f8af41057`), `#2525` ([`docs/labs/k16-admission-validators-2026-10-15/README.md`](../labs/k16-admission-validators-2026-10-15/README.md), `7b26da89d`), and Codex's exact-head review deltas (`#2522` comments `6202481992`, `6202519677`, `6203506028`): (0) mandatory **step-0 pod-side CUDA build and clippy gate** (`cargo build --release --locked --features cuda` and `cargo clippy --locked --features cuda --all-targets` on both `crates/uor-r4-discrete/Cargo.toml` and `docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml` under `set +e` with command tracing so each command's non-zero exit status and transcript are preserved in `cuda_build_exit.txt`/`cuda_clippy_exit.txt` and `cuda-build.log`/`cuda-clippy.log` before halting on failure); (1) GPU fixture specifications for `C-R1..C-R6` with complete `Spec::validate`-compliant 16-token/256-token snapshots (`lib.rs:47-58`, unscaled L1 `root_cost` at `discrete.cu:37-40`), hand-derived analytic / sealed stored integer expected values (zero CPU model reference, D24/D32), and explicit separation between host `objective::tests` contract scaffolds and the required GPU fixture receipts (`uor-r4.redirect-qualification/1`, `redirect_admission.rs:18, 357-367`); (2) `G-Pre` zero-update (`init-corpus` + `eval-corpus`, `accepted_updates == 0`) checkpoint-0 evaluations against the exact `DriverConfig` contract (`training.rs:36-84`, `batch_rows: u32` in `1..=64`, `discrete_init` XOR/xorshift identity at `discrete.cu:353-355`, `#2523` `configs/k16-{A,R}-seed{1,2}.json` with both raw file SHA-256 and compact `serde_json::to_vec` `config_sha256` pins, plus derived $K=8$ control configs) writing `NEW_REPORT_DIR/report.json` (`main.rs:9-28, 71-93`), verifying the `seed = 1, K = 8` pilot reproduction (`17 / 173 / 105 / 0 / 5,680 / 261,824 / 16`), enforcing `target_admitted_rows >= 4,894` on both seeds at $K = 16$, and re-measuring each lineage's $K = 16$ checkpoint-0 strict-win baseline $u$; (3) the 100-batch $K = 16$ synchronized counterfactual timing benchmark (`6,400` target rows, `1,228,800` context tokens, zero `.apply`), bound to the merged `#2524` timing instrument (`docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml`), documenting its per-batch `carry_windows + propose(route) + observation + propose(vector_block) + observation` unit and the two-slice `80/20` canonical coordinate schedule (`0..80` `Parameter::VectorBlock(flat_index)` + `16384..16404` `Parameter::Route(flat_index)`, `training.rs:229-248`, `discrete.cu:49-52, 376-399`, evaluating all `16` candidate ordinals `0..16` via `read_vector` + `writer` on every eligible row), and documenting the `CompletionCheckpoint` wrapper structure of `checkpoint-16404.json` (`4477eb5c...`, inner `.checkpoint.model_sha256 = 29ba5517...`, historical `source_sha256 = e1836149...`, `kernel_sha256 = 931a511b...`) with the `sealed_seed1_q4` timing leg gated / `NOT_EXECUTABLE` pending the reviewed cross-version evaluation-only rebind/import dependency (never rewriting `source_sha256`/`kernel_sha256` via `jq` or weakening `Device::from_snapshot`); (4) `uor-pod` `status -> lease/up -> run (flock /root/gpu0.lock, --features cuda, NEW_REPORT_DIR arguments) -> renew -> scp + SHA256SUMS -> cloud-store put` with verified MD5 download round-trip before `uor-pod release` + `uor-pod down`; and (5) explicit `NOT_TRAINING` scope (`CorpusUpdateAdmission` in `training.rs:658-664` and `redirect_admission.rs`) and 8-point post-run zero-update audit checklist. Docs-only, zero pods/compute; training, M2, and Stage B remain HELD.

Line: starling read / exact addressed memory · count 1/3 unchanged (runbook specification only) · M1 reply acceptable 37/232 → 116/232 unchanged (suspended D28/R14). **Next:** Codex delta re-review of `#2522` → GPU `C-R1..C-R6` fixture executor review/merge → pod-side step-0 CUDA build/clippy gate + `C-R1..C-R6` + `G-Pre` + 100-batch $K=16$ timing and `D-Packet` completion → owner personal training-hold release.

## 2026-10-15 — R-queue: one PR in the merge queue at a time (deepseek, harness process)

The [rule](../harness/pm-rules.md) codifies what #2527's ejection showed is the general case: every PR touches `claims.json` and `current-state.md`, so **every merge invalidates every queued sibling**. A PR enters the queue only when the PM calls it next; merge-forward happens only when called (speculative merge-forwards race the queue and burn shared CI); after an ejection the author re-queues on green checks and **verifies `mergeQueueEntry` through GraphQL**, because `gh pr merge --auto` prints its strategy line whether or not the entry was taken (a `null` entry is an ejection, not a position). Approvals survive merge-forwards on content identity. Evidence: #2527 admitted 22:16:24Z, ejected 22:17:26Z. References #2028 (6203500744, 6203507894). Line: M1 qualification tooling · count 0/3 unchanged · headline unchanged.
## 2026-10-15 — Public-face reconciliation: NVRTC, Hamming figure wording, starling status blocked (claude, docs)

The [record](../labs/public-face-reconcile-2026-10-15/README.md) annotates the public face against current decisions, docs only, no compute. The README D29 row now records that NVRTC was removed by #2344. The Hamming-rank figure's "today's read" wording now says the exp-table softmax read is the historical pre-D23 float read. Pages (D36), the other figures and the README mechanism tables were checked and already carry the correct status; no edit was needed. `check_claim_wording.py` passes on the edited files.

**Resolved by PM ruling** [#2028 comment 6202844646](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6202844646): the starling status needs no annotation, because the D21 pivot already happened and the murmuration read is the active mechanism.

**Found, not fixed:** `docs/figures/geometry/generate.py` no longer reproduces the committed SVGs and drops the next-token historical banner.

Line: public-face docs · count unchanged · M1 headline 37/232 unchanged.


## 2026-10-15 — A check-in surfaces unread notes (deepseek, harness tooling)

The [record](../labs/unread-notes-check-in-2026-10-15/README.md) is the durable fix for the missed-CONTEST problem: `heartbeat` now prints one line per issue holding notes the lab has not seen, read from the live-tier cache (no network call in the five-minute path), reported as a **floor** ("at least N") because a cached read cannot know what it missed, and it **never advances the marker** — showing a floor is honest, marking unseen notes as seen is not. Silent when there is no cache or nothing unread; `--no-notes` silences it deliberately; JSON output unchanged. Two integration tests run the real binary against a scratch live tier. Claims housekeeping folded in (PM-authorized): released `deepseek/k16-timing-instrument` (merged `f8af41057`) and removed `kimi/stage-a-sweep-record-20261015` (merged `c17271e82`). Line: M1 qualification tooling · count 0/3 unchanged · headline unchanged.
## 2026-10-15 — Fresh-scoring eval-only source; pinned data verified, GPU scoring NOT_RUN (codex)

The [fresh-scoring instrument](../labs/fresh-scoring-eval-2026-10-15/README.md)
consumes the packet's fixed 2,000 rows/200 documents and authenticated K16
completion checkpoints through the production CUDA evaluator, with ordered row
correspondence, T/A/R/rank/admission/occupancy receipts and snapshot/epoch equality.
Seven host controls pass and actual frozen data correspondence is verified;
CUDA-feature compilation and model scoring remain NOT_RUN. No production crate
or training-driver identity changes. Codex owns delivery; Kimi PM dispatches the
exact-head review. This is required before KEEP promotion, not training start.
No pod/training/model compute; M1 37/232 and line count unchanged, M2/Stage B HELD.

## 2026-10-15 — Studio shell cleanup per D36: dead `wasm_*` removal, `native_geometric_finish_generation` guard, engine-status banner, and provenance note (`antigravity`, source-only)

- **Scope & Canonical Record ([`docs/labs/studio-shell-d36-cleanup-2026-10-15/README.md`](../labs/studio-shell-d36-cleanup-2026-10-15/README.md)):** Per [D36](DECISIONS.md#d36--webassembly-and-github-pages-scope-deployment-stays-frozen-cleanup-only) and Kimi PM assignment, cleaned up the Studio shell in [`index.html`](../../index.html) and [`assets/js/uor_model_worker.js`](../../assets/js/uor_model_worker.js):
  1. **Removed the five permanently-dead `window.wasmModule.wasm_*` branches in `index.html`**: removed `wasm_index_file`, `wasm_clear_workspace_index`, `wasm_query_workspace`, `wasm_get_workspace_stats`, and `wasm_myers_diff` (plus `indexWorkspaceFilesGeometric` and `queryWorkspaceGeometric`), none of which exist in the repository's Rust source and where `window.wasmModule` was never assigned (`0` remaining matches in `index.html`).
  2. **Guarded `native_geometric_finish_generation` in `assets/js/uor_model_worker.js`**: guarded `wasmModule?.native_geometric_finish_generation` (and `wasmModule?.native_geometric_cancel`) with `typeof ... === 'function'` so that when the shipped WASM bundle lacks `native_geometric_finish_generation`, generation completion/cancellation finishes cleanly and reports an explicit D36 status in `stoppedBy` / `finishStatus` (`#nativeStopReason`) instead of throwing a false `generate_error`.
  3. **Added D36 engine-status banner (`#d36EngineStatusBanner`)**: states plainly beneath the top navbar (and in `.sidebar-footer` / `#sidecarD36Status`) that **no D11 served wasm engine exists yet, GitHub Pages deploy is frozen per D36, and the live site is the last good deploy (13 Oct 2026, pre-#2345)**.
  4. **Added provenance note (`#studioProvenanceNote`)**: states plainly in the top banner, sidebar footer, telemetry sidecar, and file headers of `index.html` and `assets/js/uor_model_worker.js` that the Studio source is an import of the companion repository `Casey-allard/uor-r4-wasm-chat`.
- **D36 Invariants & Verification:** `.github/workflows/deploy.yml` is untouched (stays disabled), no WASM rebuild performed, `node --check assets/js/uor_model_worker.js` passes, all four prohibited strings in `src/lib.rs:307-313` remain absent (`0` matches), `python3 scripts/check_claim_wording.py` passes, and `uor-harness ratchet d23|d25` pass (`0` violations). Zero pods, zero GPU/model compute.
- **Line:** no model line — D36 Studio shell cleanup · count 0/3 unchanged · M1/M4 headlines unchanged.

## 2026-10-15 — K16-scoped admission source and fresh-scoring owner; no execution

Codex/Sol's [separate validator record](../labs/k16-admission-validators-2026-10-15/README.md)
implements four explicit K16-A/R seed1/2 scopes, fresh checkpoint-zero admission,
qualified source/config/lineage/receipt bindings, and fail-closed cross-scope resume.
Historical K8 pilot/completion releases remain unchanged. Qualification, pod-side
CUDA build/clippy, packet acceptance and R15 owner-release transport resolution
remain open gates; no training/GPU/model execution occurred. Codex owns the
fresh-scoring eval-only path before KEEP promotion, a separate deliverable.
The packet now says exactly 200 documents and the owner's ~30-minute/<$2
qualification envelope. M1 37/232 and D21/D22 count unchanged; M2/Stage B HELD.

## 2026-10-15 — K16 synchronized 100-batch timing instrument, source only (deepseek, tooling/M1 qualification)

The [record](../labs/k16-timing-2026-10-15/README.md) delivers the runner Codex delegated ([#2029 6201508191](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201508191)) under the PM ruling ([6201569519](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201569519)): a **standalone dated-record crate** that times the carry + propose + observe path over a frozen snapshot at V4096/B4/C192/**K16**, Route, 64 rows, **100 distinct batches**, both objective policies, one excluded warmup per cell, **zero apply** — each cell asserts an identical snapshot digest and epoch before and after. `measurement.rs` and every production kernel are untouched (`git diff --stat origin/main -- crates/` is empty). It **fails closed** on absent/malformed fixture pins, a shape that is not K16, a read policy that is not Route, a resume dressed as an evaluation-only rebind, or a missing policy arm; it reports target-rows/s and, separately, physical padded-context-tokens/s (64 x 192 = 12,288 per batch), with per-batch receipts and the zero-apply exclusion in every record; and a fault **preserves the partial receipts** rather than discarding the measurement. **14 host tests pass** and the plan command runs on any host. **No pod was leased: G-Pre and the timing benchmark stay NOT_RUN/UNMEASURED until this source is reviewed, merged and executed at the qualification stage.**

Line: M1 qualification instrumentation · count 0/3 unchanged · M1 headline unchanged. **Next:** PM-dispatched exact-head review, then the qualification lease (not mine to take until merged).

## 2026-10-15 — Admission redirect D-Packet data/config draft; qualification blocked on missing executors (codex)

The [packet](../labs/admission-redirect-packet-2026-10-15/README.md) binds four K16
seed/objective configurations and a deterministic fresh-transfer slice of **2,000
eligible targets across 200 documents**, with SHA-pinned private source, train/dev
exact causal-row disjointness, native `Split::Fresh` metadata validation, and
archive/receipt layout. Four data-only tests pass. The source is the historical
`chat-v0-p2` component, held out from this D32 contrast, not globally unseen data
or an independent 2,000-conversation panel. K16 timing remains **UNMEASURED**.

[Owner release 6201365250](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201365250)
authorizes the gated qualification/packet/training chain, not a bypass of missing
executables. [PM disposition 6201569519](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6201569519)
confirms C-R scaffolding is not executable qualification: AGY owns the actual GPU
fixture/G-Pre runner; DeepSeek owns K16 100-batch timing; Codex owns packet pins.
The missing local Vast CLI was built from existing source and live status now
works. **No pod leased, $0 GPU rental, no model compute, no corpus updates.**

Line: exact addressed memory / admission redirect · count unchanged · M1 headline
37/232 unchanged. **Next:** packet exact-head review and reviewed executor merges,
then qualification and measured packet completion. Training cannot start while
those gates are incomplete; M2 and Stage B remain HELD.

## 2026-10-15 — The merge-side inversion, written down (deepseek, tooling)

Follow-up approved on [#2520](https://github.com/UOR-Foundation/uor-r4/pull/2520): the merge-forward bullet in `docs/harness/pm-rules.md` now states the fact that bit me while merging that very PR — **in a `git merge`, `HEAD` is *your* branch and the incoming side is `origin/main`**, the opposite of a rebase, so `current-state.md` newest-first means *your* entry lands first. The [pattern record](../labs/merge-forward-pattern-2026-10-15/README.md) carries the same note. The inversion is silent: both sides are valid Markdown and the file still parses, so review does not catch it. Also released here: `deepseek/merge-forward`'s claim, released by `clean-merged --apply` proving the branch merged on its own — the #2516 fix, second production use.

Line: no model line - tooling · count 0/3 unchanged · M1 headline unchanged. **Next:** standby as backup executor for the qualification stage (Codex holds it; offer posted on #2029).

## 2026-10-15 — Merge forward, do not rebase: the pattern that needs no force (deepseek, tooling)

The [record](../labs/merge-forward-pattern-2026-10-15/README.md) answers the hook follow-up from [#2028 6200456329](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6200456329). A routine rebase was blocked because the push needed `--force-with-lease` and the agent-side pre-command hook rejects it with no approval path — leaving "abandon the rebase" or "work around the hook", the second of which turns a guard into decoration. The better answer is the one already working in production: **merge forward** — `git merge origin/main` into the delivery branch, resolve `docs/harness/claims.json` by union and `docs/integration/current-state.md` newest-first, plain `git push`; the merge queue squashes so the merge commit never reaches `main`, and `git diff origin/main..<new-head> -- <delivered paths>` keeps the reviewed diff exact. Now stated in `docs/harness/pm-rules.md` and the `AGENTS.md` post-merge section, both pointing at [#2519](https://github.com/UOR-Foundation/uor-r4/issues/2519), which files the narrow approval-gated exception for the one case merge-forward cannot cover (a stacked child branch).

Line: no model line - tooling · count 0/3 unchanged · M1 headline unchanged. **Next:** #2519 is a proposal for the fixer of the agent-side hook; nothing else is queued on this line.

## 2026-10-15 — D-R rank-objective source and occupancy/scale telemetry; qualification NOT_RUN (codex)

The [source record](../labs/admission-rank-hinge-2026-10-15/README.md) implements artifact-bound `NearestAddressMarginV1` / `OccupiedCellRankHingeV1`, exact occupied-cell `(address_distance, cell_id)` rank, GPU dual-loss/occupancy/histogram reductions, and the K16 stored-baseline bound against redirect sheet #2515 at `ab047f4c6`. C-R1..C-R6 are **authored fixture scaffolding, not an executable qualification suite**; 37 host/static source tests passed with one authenticated-artifact test ignored, and no model/GPU/pod computation ran. Existing owner-release pins are not broadened; Codex is author, so the review seat belongs to **Kimi PM's dispatched non-author reviewer**.

Line: exact addressed memory (discrete rung0 admission redirect) · count 0/3 unchanged · M1 headline 37/232 → 37/232 unchanged. **Next:** exact-head review, separately authorized fixture/qualification completion and D-Packet, then explicit owner release; training/M2/Stage B remain HELD.

## 2026-10-15 — Rung-0 G-A-FAIL admission-objective redirect pre-registered (`TEST FITNESS: FIT (design)`): matched K=16 cell-rank hinge `8*T + 32*R` vs `K16-A` control (antigravity + codex, starling read / exact addressed memory)

[`docs/harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md`](../harness/prep/m1-discrete-rung0-admission-redirect-2026-10-15.md) (canonical record [`docs/labs/rung0-admission-redirect-prep-2026-10-15/README.md`](../labs/rung0-admission-redirect-prep-2026-10-15/README.md)) registers the G-A-FAIL successor commissioned by Kimi PM ([#2028 6195270941](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6195270941)), integrating Codex/Sol's matched `K=16` contrast ([#2029 6196365768](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6196365768)), Kimi PM's six measured audit facts ([#2029 6197008827](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6197008827)) from the sealed 36-point Stage-A sweep (`report.json` SHA-256 `cc81ca6f76f3a38ac61ffe1d4665a1442d1e42961f4451691327b91b438f472c`, [#2511](https://github.com/UOR-Foundation/uor-r4/pull/2511)), all six amendments from Kimi PM's conditional `TEST FITNESS` review ([#2515 6198221127](https://github.com/UOR-Foundation/uor-r4/pull/2515#issuecomment-6198221127)), the owner's `G-Pre` confirmation ([#2515 6198442894](https://github.com/UOR-Foundation/uor-r4/pull/2515#issuecomment-6198442894)), and Kimi PM's `TEST FITNESS: FIT (design)` verdict at `8a4388424` ([#2515 6199263447](https://github.com/UOR-Foundation/uor-r4/pull/2515#issuecomment-6199263447)): (1) `address_hinge_sum` is identically `K`-invariant across `K ∈ {8..16}` at all four cursors (`261,344 / 262,360 / 270,500 / 276,608`) and accounts for `60.7%–62.8%` of weighted loss, so upweighting `A` cannot move the `K` boundary; (2) `token_unviolated_eligible_rows = 0` and `eligible_unviolated_rows = 0` across all 36 cells (`address_unviolated_eligible_rows = 2 / 1 / 0 / 0` at Q1→Q4 — no token/total satisfied regime; ≤2 address-only unviolated rows at Q1/Q2); (3) admission (`G-Adm ≥ 4,894`, where the `+119`-row margin above `Bar (ii)` equals `1.0×` the `119`-row Wilson 95% half-width and `1.37×` the `87`-row `2-SE` admission floor) and ranking (`Bar (i) ≥ 1,941`, `Bar (ii) ≥ 4,775`, both `+715` rows `≈ 6.0×` Wilson half-width) are separated because `K=16` admission (`3,242 / 3,206 / 2,509 / 2,400`) already exceeds `1,941` while `top1` sits at `136 / 138 / 88 / 145` (`1,796–1,853`-row pure ranking gap); (4) `strict_wins` (`max = 214` vs `≈ 525`) is non-increasing at all `32/32` adjacent `K` steps (`4 cursors × 8`, while `target_admitted_rows` rises at `32/32` adjacent `K` steps and falls at `27/27` adjacent cursor steps `9 K × 3`), requiring guard baseline `u` to be re-measured at each new `K=16` lineage's checkpoint `0` and registering a `G-Guard`-ONLY failure when `G-Adm + G-Occ + Bar (i) + Bar (ii)` pass as a `RANK_BOTTLENECK_ISOLATED`-adjacent diagnosis path (`GUARD_STRUCTURAL_BIAS_DIAGNOSIS`, not a `KEEP`-blocker or mechanism negative); (5) capacity was trained out (`5,680` at `K=8` checkpoint `0` → `2,180` → `1,920`), not absent, so owner-confirmed `G-Pre` (`6198442894`) binds a zero-update checkpoint-0 `K=16` ceiling check (`≥ 4,894` release prerequisite, `< $0.35`) before training; and (6) `K`-widening gain collapses across training (`+1,041 / +1,075 / +555 / +480`), anchoring sizing and cost at Q4 (`16,404`) rather than Q1/Q2 slopes while rejecting the false universals that `admitted_strict` or `top8` always fall with `K` (both rise at Q1/Q2). The sheet pairs `K16-A` (`8·T + 32·A`) against `K16-R` (`8·T + 32·R`, $R = \max(0, r - K)$ counting each occupied cell once, $R = 0 \iff \text{target\_cell admitted}$) across `2 seeds × 16,404 batches` (`65,616` total batches, fresh init at `V4096/B4/C192/K16`, `ReadSpec::Route`, `CanonicalVectorThenReadV1`), registers explicit boundary dispositions (`K16-A` passing with `≤ Δ_noise` `K16-R` gain = configuration `KEEP` on `K16-A` + registered negative on `R` at this exact scope; numeric occupancy-collapse trigger `G-Occ` if $\text{occ}_R < \max(32, \lceil 0.50 \times \text{occ}_A \rceil)$ or $\bar{n}_R > 2.0 \times \bar{n}_A$, where only `t = 16,404` decides, earlier-cursor triggers are diagnostic-only, and `K16-A`'s occupancy is reported as the control baseline, not gated), requires per-checkpoint $\sum 8T / \sum 32A / \sum 32R$ and per-row `T / A / R` hinge-distribution reporting to separate hinge shape from effective scale (`R ∈ 0..240` vs `A ≈ 18–19/row`), and binds the `≥ 2,000`-row fresh-transfer slice in `D-Packet` before training release (no `KEEP` promoted to a capability claim without fresh transfer scored and published, D22 §4). Docs-only, zero compute, training/M2/Stage B remain HELD.

Line: starling read / exact addressed memory · count 1/3 unchanged (registration only) · M1 reply acceptable 37/232 → 116/232 unchanged (suspended D28/R14). **Next:** bounded `D-R` objective/control source diff (author: Codex) → exact-head review by the PM-dispatched non-author seat ([#2028 6196807003](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6196807003), [#2029 6199294252](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6199294252)) → `C-R1..C-R6` + `G-Pre` qualification & `D-Packet` → owner personal hold release.

## 2026-10-15 — Two superseded DeepSeek parked branches archived, then deleted (deepseek, tooling)

The [record](../labs/deepseek-branch-archive-2026-10-15/README.md) closes the parked pair. `deepseek/comment-guard` (#2488) and `deepseek/recon-dag` (#2489) were held open until Codex's takeovers landed — **#2500 merged `5c4a2c867`, #2501 merged `fd01c98cf`**, both original PRs closed as superseded on 15 October. They are **not content-merged** (`clean-merged` reports both as not merged: the successors continued the work with different trees), so the October 9 rule applies: a patch plus an INDEX row in [docs/history/branch-archive](../history/branch-archive/INDEX.md), verified by isolated-index application, **including the recon-dag worktree's uncommitted tracked changes** (preserving only its committed head would have dropped that work). Branches and worktrees are deleted after this PR merges — deletion before the archive lands would be the one irreversible mistake available here.

Line: no model line - tooling · count 0/3 unchanged · M1 headline unchanged. **Next:** #2514's re-review and #2516's review, then the branch/claim cleanup they unblock.
## 2026-10-15 — `clean-merged` can prove a squash merge: cumulative patch-id instead of per-commit (deepseek, tooling)

The [record](../labs/merge-state-proof-2026-10-15/README.md) fixes the defect found while dogfooding #2514. **`githelp::merge_state` fell back to `git cherry`, which compares per-commit patch ids, and a squash merge lands the branch as ONE commit** — so no commit of a multi-commit branch ever matched, and merged work read as unmerged forever. `clean-merged` therefore kept dead branches ("its owner decides"), and after #2513 the claim release could never fire for the branches it was written for. Measured on `deepseek/merged-claim-release`: five `+` lines while the branch tree was byte-identical to the merge commit and its cumulative patch-id matched exactly (`122650e7...`). The proof now runs ancestry → `git cherry` fast lane → **the branch's cumulative patch-id against the commits `origin/main` gained since the merge base**; a step that cannot run returns `Unknown`, never `Unmerged`. 540 unit tests pass, the squash-through-multi-commit case is pinned, and the genuinely-unmerged case is pinned in the same probe as the control. `AGENTS.md` now describes the proof the code actually performs.

Line: no model line - tooling · count 0/3 unchanged · M1 headline unchanged. **Next:** the branch archive for the two parked deepseek branches, then #2514's re-review.
## 2026-10-15 — Claims writes are atomic, and two call sites stop deleting peers' live claims (deepseek, tooling)

The [record](../labs/claims-write-integrity-2026-10-15/README.md) closes the follow-up recorded on #2513. **`publish_claims_live` and `save_claims` wrote with `fs::write`, which truncates first** — a peer publishing in that window saw an EMPTY claims file, not a stale one, and a crash mid-write left a half-file that parses as nothing. Both now write a temp file and rename it into place, and `claim_branch`/`release_branch` publish the **merged view** (minus the released branch) instead of the committed store, which was deleting every claim that lives only in the live tier. 536 unit + integration tests pass, and both fixes were proved by removing them (the mirror's **inode** must change across a rewrite; a seeded `peer/live-only` claim must survive a claim and a release). **Dogfooded in production**: three of my own merged claims released through the installed binary — inode `127369871` → `135284752`, and the seeded peer claim survived. Also released here: the stale `deepseek/harness-write-path`, and the merged `deepseek/receipt-kind` / `deepseek/merged-claim-release` claims.

**Found while dogfooding, reported not fixed:** `clean-merged` cannot prove a MULTI-COMMIT branch merged — `githelp::merge_state` falls back to `git cherry`, whose per-commit patch ids never match a squash commit. Measured on `deepseek/merged-claim-release`: `git cherry` reported 5 `+` (unmerged) while the branch tree was byte-identical to the merge commit and its cumulative patch-id matched exactly. Every merge-cleanup therefore keeps its branch and (since #2513) its claim. The branch was deleted by hand on that evidence; the proof fix is its own unit.

Line: no model line - tooling · count 0/3 unchanged · M1 headline unchanged. **Next:** the `merge_state` proof fix (PM's call on ordering), then the dispatched review seat for the admission redirect sheet.
## 2026-10-15 — Stage M2 relation sub-panel, interleaved schedule reference, and zero-trit attribution receipt tooling (antigravity, D34 reranker-first attention sequence)

[`docs/labs/completion-scoring-readiness-2026-10-14/scoring-audit/src/bin/relation-subpanel-audit.rs`](../labs/completion-scoring-readiness-2026-10-14/scoring-audit/src/bin/relation-subpanel-audit.rs) (canonical lab record [`docs/labs/relation-subpanel-audit-2026-10-15/README.md`](../labs/relation-subpanel-audit-2026-10-15/README.md), unit test log [`synthetic-tests.txt`](../labs/relation-subpanel-audit-2026-10-15/synthetic-tests.txt), real-corpus seed-1 sub-panel receipts [`seed1-cursor0-subpanel.json`](../labs/relation-subpanel-audit-2026-10-15/seed1-cursor0-subpanel.json) and [`seed1-cursor16404-subpanel.json`](../labs/relation-subpanel-audit-2026-10-15/seed1-cursor16404-subpanel.json)) adds the standalone host-only receipt, schedule-reference, and checkpoint-attribution sidecar binary for the pre-registered Stage M2 `ReadSpec::Relation` vs. `ReadSpec::Route` discriminator ([`docs/harness/prep/m4-finite-relational-head-2026-10-15.md`](../harness/prep/m4-finite-relational-head-2026-10-15.md), [#2502](https://github.com/UOR-Foundation/uor-r4/pull/2502); [`docs/mechanisms/finite-relational-head.md`](../mechanisms/finite-relational-head.md) §4.3, §7.1–§7.2, §9, [#2486](https://github.com/UOR-Foundation/uor-r4/pull/2486)) without touching `crates/uor-r4-discrete/` (`driver_source_sha256()` unchanged) or existing `scoring-audit` binaries:

1. **Three host-only CLI modes (`subpanel`, `prepare-zero-trits`, `discriminator`) with `128 MiB` input guards and exclusive output creation:**
   - `subpanel DEV_TOKENS_U16 DEV_MASK_U8 EVALUATION_JSON NEW_OUTPUT_JSON`: verifies pinned `ft/dev/tokens.u16` (`0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `42,856` payload tokens, `400` documents, `UORT` v1) and `ft/dev/response_mask.u8` (`a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`, `14,287` eligible targets at $C = 192$), reproduces the exact `3,847 / 14,287` (`26.9266%`) causal-window copy-successor sub-panel $\mathcal{P}_{\text{succ}}$ (`[50, 2872, 900, 25]` across distance buckets `1..=8`, `9..=32`, `33..=96`, `97..=192`, `0` rows at $d = 1$, `8,648` query-recurrence rows, `5,182` any-occurrence successor rows $\mathcal{P}_{\text{any\_succ}}$, `8,248` copy-ceiling rows, `10,440` complement rows $\mathcal{P}_{\text{comp}}$), binds `ordered_row_binding_sha256` (`e4ef44fb81d6c3a833212e4857117db71cafb8259b81c286e5b1391129d74446` on `ft/dev`), enforces `validate_evaluation_population_binding` (`evaluation_lineage` bound to partition `tokens.u16` and `response_mask.u8` digests and `model_bindings`) and `verify_row_correspondence` (`top1_correct` / `top8_correct` vs `top8` target membership plus explicit `target_position` and `target_token` required on all unsealed receipts and Stage M2 discriminator receipts), emits integer `conditional_in_admitted_b3` numerators/denominators (`top1_correct / B3`, `top8_correct / B3`, `admitted_strict_wins / B3`), and checks bit-exact additive conservation $\mathcal{P}_{\text{succ}} + \mathcal{P}_{\text{comp}} == \mathcal{P}_{\text{full}}$ and bucket sums across all 18 aggregate fields (`E1`, `E2`). Executed on sealed seed-1 `Route-canonical` receipts at cursor `0` (`pilot-report.json` `e4bcf43859fa0645db2ca0d5fe620f18620990cbb7efb4f28e6873f5a57f90c3`: Full `17 / 173 / 5,680` = $\mathcal{P}_{\text{succ}}$ `1 / 68 / 1,658` + $\mathcal{P}_{\text{comp}}$ `16 / 105 / 4,022`) and cursor `16,404` (`evaluation-16404.json` `4fff1d3c04ff6107666b2b8281ef2bbedf3019eeff6bff682bbbb8687bf888c8`: Full `149 / 747 / 1,920` = $\mathcal{P}_{\text{succ}}$ `29 / 170 / 435` [`d_1_8: 0/3/5`, `d_9_32: 25/144/364`, `d_33_96: 4/23/62`, `d_97_192: 0/0/4`] + $\mathcal{P}_{\text{comp}}$ `120 / 577 / 1,485`).
   - `prepare-zero-trits RELATION_CHECKPOINT_JSON NEW_ZERO_TRIT_CHECKPOINT_JSON`: implements the host-side `E3` checkpoint transform across `uor-r4.completion-checkpoint/1`, `uor-r4.discrete-driver-checkpoint/1`, and `uor-r4.discrete-spiralcore-fbs/2`, verifying `routes.len() == blocks * stages` (entries in `0..64`), `table.len() == vocabulary * blocks * 8` (entries in `{-2, 0, 2}`), and `gates.len() == (blocks + 5) * stages` (`45` trits at `C192 / B4`, `14` trits at `C6 / B2`), zeroing only `gates[blocks * stages .. (blocks + 5) * stages]` while preserving `table`, `routes`, `gates[0..blocks * stages]` (`GateCombine`), `spec`, `read_spec`, `seed`, and `epoch` byte-for-byte; CUDA `discrete_validate` and zero-trit GPU evaluation receipt invariance are owed for `G3`.
   - `discriminator`: supports single-seed (`8` args, labeled `spread_mode: "floor_only_not_sheet_b1"`) and paired two-seed (`11` args, computing `DiscriminatorSpreads::from_seed_pair` across distinct seeds `seed1 != seed2` with `spread_mode: "two_seed_measured_with_39_floor_top8_and_0_floor_side_guards"`) evaluation of `B1` (`top8_correct` gain on $\mathcal{P}_{\text{succ}}$ strictly exceeding $\Delta_{\text{seed}}(\mathcal{P}_{\text{succ}}) = \max(|\text{seed } 1 - \text{seed } 2|, 39)$ with non-decreasing sub-panel `top1_correct`/`B3` relative to `Route-interleaved` minus `spread.max(0)`, where the `B1` side-guards use floor `0` since the sheet defines a non-zero single-seed floor `39` only for copy-successor `top8_correct`), `B2` (full-panel non-regression within $\Delta_{\text{seed}}(\mathcal{P}_{\text{full}})$ floored at `20 / 63 / 87` rows on `top1_correct / top8_correct / B3`), and `B3` (positive zero-trit attribution on $\mathcal{P}_{\text{succ}}$ + `gate_relation.accepted_updates >= 1`), unconditionally enforcing `validate_evaluation_population_binding`, `model_bindings`, `ScheduleKind::InterleavedReadEvery8V1`, decision cursor `completed_batches == 16,404`, `model_epoch == accepted_updates_in_model`, `zero_trits.source_relation_model_sha256 == Relation.model_snapshot_sha256`, `zero_trits.source_relation_checkpoint_sha256 == Relation.driver_checkpoint_sha256`, and `verify_split_family_counters` on both `Route` and `Relation` `sensitivity` blocks at `DECISION_CURSOR_BATCHES = 16,404`.
2. **Host reference for `S1`–`S4` verified (`6/6` offline tests); driver `S1`–`S4` owed for `G3`:** `cargo test --locked --offline --bin relation-subpanel-audit` passes **6/6** unit tests covering the `S1` 24-step fixture (`0..=48` batch round-trip), `S2` `C192` quarter `(vector, read)` counts and per-coordinate frequencies (`920` `GateCombine` + `1,130` `GateRelation` = `2,050`), `S3` `validate_checkpoint_resume` cross-schedule/cross-`read_spec` refusals, `S4` 4-family counter conservation, synthetic `E1`–`E2` (including negative `token_hinge`/`address_hinge`, mismatched `evaluation_lineage`, and per-row `target_position`/`target_token` correspondence refusals), `E3` (`C6/B2` and `C192/B4`), `B1`–`B3`, and CLI/resource guards; driver-side `S1`–`S4` (`cargo test -p uor-r4-discrete`) is owed for `G3`.

Line: starling read / exact addressed memory · count 0/3 unchanged (M1) / 3/3 unchanged (M4) · headline 37/232 → 37/232 (unchanged). **Next:** Codex re-review of `#2509`; driver `S1`–`S4` for `G3`; Stage M1 CUDA construction qualification and Stage M2 corpus execution remain `HELD` behind explicit owner release.

## 2026-10-15 — A claim dies with its branch: `clean-merged --apply` releases merged claims (deepseek, tooling)

The [record](../labs/merged-claim-release-2026-10-15/README.md) closes the second half of Kimi's follow-up [#2028 6195505734](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6195505734). **A claim outlived the branch it named**: `deepseek/harness-write-path` merged as `b72f904c4`, its branch was deleted, and the entry stayed in `docs/harness/claims.json`, so `team claim` would refuse that name with "already held by lab …" for work already in main — a conflict in the file, not in the work. `git clean-merged` now **reports** the claims on merged branches and `--apply` **releases** them (in the caller's own store and the live mirror, which is republished so the release cannot be resurrected) in the same pass that deletes the branches. The proof is the one `clean-merged` already makes — mergedness by content, because a squash merge leaves no ancestry link — and nothing else is touched: an open or unprovable branch keeps its claim. A refusal (R10 in the shared checkout) is reported, not fatal. 534 unit + 1 integration test pass, and each half was proved by removing it and watching its test fail.

**Reviewed by Kimi PM (CHANGES at `a98b2aa0d`); both findings fixed.** The release no longer deletes other
labs' live-only claims — the mirror is republished from the merged VIEW minus the released branches, not
from the caller's committed store — and the residue the mechanism cannot prove (a claim whose branch was
already deleted) is now **reported** as `claims whose branch is gone` with the exact `team release` line,
never released on a guess, because "gone" is not "merged". The docs state that reach instead of implying
more.

Line: no model line - tooling · count 0/3 unchanged · M1 headline unchanged. **Next:** exact-head review of this PR and of the `receipt` contract fix (#2512), then the dispatched review seat for the admission redirect sheet.
## 2026-10-15 — `receipt` refuses what it cannot honour: frozen kind list, `-m`, and unread flags (deepseek, tooling)

The [record](../labs/receipt-contract-2026-10-15/README.md) closes Kimi's follow-up [#2028 6195505734](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6195505734). **An unknown `--kind` was written to the shard and `check` then hard-errored on it at parse** — the harness accepting a record its own reader rejects, repaired only by deleting the line by hand. The kind is now validated against `receiptsrules::KINDS` in `Receipt::validate`, the one place every writer passes through, so `receipt`, `skill use`, `prep fit` and `team dispatch` are all covered. **`-m` was parsed, forwarded and dropped** (a receipt recorded as `-m m1` carried no milestone): it is read as the milestone short form now. And **any other flag the command does not read is refused** (exit 2, naming the flag and the list) instead of vanishing — `RECEIPT_FLAGS` declares the reads. 534 unit + 3 CLI contract tests pass, and each fix was proved by removing it and watching its test fail.

Line: no model line - tooling · count 0/3 unchanged · M1 headline 37/232 -> 116/232 unchanged. **Next:** the second follow-up on this queue — auto-releasing a branch's claim when its PR merges (the stale-claim mechanical conflict class) — then the dispatched review seat for the admission redirect sheet.
## 2026-10-15 — Stage M1 `ReadSpec::Relation` Rust + CUDA construction source slice implemented (antigravity, exact addressed memory)

[`docs/labs/readspec-relation-m1-construction-2026-10-15/README.md`](../labs/readspec-relation-m1-construction-2026-10-15/README.md) implements the Stage M1 `ReadSpec::Relation { kept, sweeps }` bounded exact-relation reranker inside `crates/uor-r4-discrete` (`lib.rs`, `device.rs`, `discrete.cu`, `training.rs`, `main.rs`, `relation_qualification.rs`, and `prime_zeta.rs`) against the merged [D34](DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate) design card ([`docs/mechanisms/finite-relational-head.md`](../mechanisms/finite-relational-head.md), `#2486`) and the FIT-reviewed Stage M1 pre-registration sheet ([`docs/harness/prep/m1-finite-relational-head-2026-10-14.md`](../harness/prep/m1-finite-relational-head-2026-10-14.md), `#2494`):

1. **Explicit Artifact Identity & Gate Trit Inventory (`lib.rs`, `device.rs`, `training.rs`, `main.rs`):** Adds `RELATION_EDGE_TYPES = 5` and `ReadSpec::Relation { kept, sweeps }` (`kept in 1..=8`, `sweeps in 1..=5`), encodes `s[8..=10] = [2, kept, sweeps]` in `spec_gpu`, allocates `blocks * stages + 5 * stages` `Parameter::Gate` (`kind = 3`) trits (`45` trits at `C192` where `blocks = 4, stages = 5`; `14` trits at `context = 6` where `blocks = 2, stages = 2`), maps all relation read coordinates in `PreparedDriver` to `CoordinateKind::Gate`, and refuses `Parameter::Route(_)` on `ReadSpec::Relation` snapshots with `Invalid("route parameters are inactive in a relation model")`.
2. **Integer-Only CUDA Relational Reranker (`discrete.cu`):** Adds `MODE_RELATION = 2u`, `EDGE_TYPES = 5u`, `MAX_SWEEPS = 5u`, and `gate_count(s)` across `flock_vector`, `read_vector`, `discrete_init`, `discrete_forward`, `discrete_counterfactual`, `discrete_apply`, and `discrete_validate`. Tracks `first[MAX_NODES]` alongside `level[MAX_NODES]` and `last[MAX_NODES]`, computes exact anchor-orbit base match scores $m_j^{(0)} = 8 - d_{\text{L1}}(f_q, f_j) \in \{0, 4, 8\}$, runs $S = \text{sweeps} \in \{1..5\}$ synchronous integer sweeps across the 5 retained-structure edge families (`ParentChild`, `Sibling`, `SequentialSuccessor`, `SameToken`, `SameFrame`, all enforcing `last < length` on both endpoints, with `SameToken` `e=3` and `SameFrame` `e=4` selecting only the most-recent prior matching node `largest j < i` at the same level via `esum = prev[j]` so total degree stays $\le 8$) via sign-conditioned integer addition/subtraction (`gt > 0 ? esum : -esum`, zero multiply or float instructions), ranks causally eligible level-$l$ nodes by `(rel[node] desc, distance asc, hamming asc, last[node] desc, node asc)` (`relation-off ≡ Flock` bit-for-bit when $r_i^{(S)} = 0$), and writes per-node relation scores (`Forward::relation_scores()`) after the unchanged `Flock` trace region while leaving `BFS` transport, `snap_K` consensus, level-combine residual, and `writer` (`discrete.cu:229-253`) untouched.
3. **Stage M1 Qualification Suite (`relation_qualification.rs`, `main.rs`):** Implements `discrete-stack list-relation-controls` and `discrete-stack qualify-relation NEW_REPORT_DIR` (`relation-controls.json`, `applied_updates: 0`, zero `.apply` calls) covering all 8 registered functions (`relation_off_flock_parity`, `relation_copy_successor_fixture`, `relation_irrelevant_change_invariance`, `relation_conflicting_successors`, `relation_suffix_and_target_isolation`, `relation_endpoint_causal_boundary`, `relation_deterministic_tie_order`, `relation_bounded_sweeps_and_refusals`, including subcase `8b2` pinning the 3-occurrence `SameToken` `e=3` and `SameFrame` `e=4` most-recent-prior rule on distinct token IDs across `sweeps = 1..=3`) with every pinned fixture and hand-derived integer literal from `m1-finite-relational-head-2026-10-14.md`. Host metadata/unit tests (`cargo test -p uor-r4-discrete --no-default-features`, `34/34` passing) and `d23`/`d25` ratchets pass; `0` GPU hours, training and M2 remain `HELD`.

Line: exact addressed memory · count 0/3 unchanged (finite construction source slice) · headline 37/232 → 37/232 (unchanged). **Next:** peer exact-head review of this Stage M1 source PR; finite CUDA execution of `qualify`, `qualify-flock`, and `qualify-relation` runs only after explicit owner release on `#2029`.

## 2026-10-15 — Stage-A K-sweep executed: G-A FAIL closes eval-time widening of the K=8-trained codebook (kimi PM, starling read / exact addressed memory)

The [Stage-A sweep record](../labs/rung0-stage-a-sweep-2026-10-15/README.md) reports the executed [pre-registered](../harness/prep/m1-discrete-rung0-admission-2026-10-15.md) kept-cells sweep run with the reviewed D-A instrument on pod vast-56109378 (1× RTX 5090, 913 s, est. $0.343 under the < $0.50 ceiling, released + downed; source `abdf431ed`, toolchain nvcc 12.8.93 / rustc 1.97.1 / compute_120 verified, compiled identity reproduces the checkpoint-bound pins): **G-A FAIL (`FAIL_EVAL_TIME_WIDENING_ONLY`), `smallest_passing_k: null`, no instrument fault.** All four K=8 controls reproduced the sealed quarter counts exactly (`ALL_FOUR_EXACT`), qualifying the instrument; at cursor 16,404 the target-admitted ceiling rises monotonically with K (1,920 → 2,400) but saturates at the ABI-max K=16 **2,494 rows below the 4,894 gate** (every row's full 16-slot budget consumed; strict wins fall as K widens, 105 → 99), and the earlier checkpoints reach higher K=16 ceilings than the most-trained one (3,242 / 3,206 / 2,509). The full 36-point table is sealed (38/38 blake3, per-row monotone admission independently re-verified, zero-update flags everywhere; `derived_record_bytes` 33,627,092) and archived at `icloud:UOR-R4/results/kimi/stage-a-sweep-1-20261015.tar` (md5 `b274beb3…`, round-trip verified; laptop copy `~/uor-r4-local/stage-a-sweep-1-20261015/`). Scope (R1): this closes **eval-time widening of the K=8-trained codebook only** — K>8-trained lineages remain the open named successor (Stage B territory); D22 configuration closure, not a mechanism negative. Pre-registered successor: redirect sheet on the admission policy/objective (address metric / selection rule / objective weights), commissioned to Antigravity + Codex ([#2028 6195270941](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6195270941)); verdict posted at [#2029 6195261236](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6195261236). Training, M2 and Stage B remain HELD; nothing in this result releases them.

Line: starling read / exact addressed memory · count 1/3 (admission line: eval-time widening closed without headline movement; mechanism open) · M1 reply acceptable 37/232 → 116/232 unchanged (blocked D28). **Next:** merge this record; redirect sheet draft (AGY+Codex) → Claude TEST FITNESS.

## 2026-10-15 — Codex post-merge cleanup and inactive worktree preservation

#2508 is merged as `abdf431ed`; its instrument and discrete crate match Claude-approved `8ec3bd7c8`, and its branch/worktree are removed after verified receipt preservation. The [cleanup/preservation record](../labs/codex-worktree-preservation-2026-10-15/README.md) and [indexed patches](../history/branch-archive/INDEX.md#october-15--stale-codex-worktrees-retained-inactive) retain the 12-file unfinished October 12 repair and the October 14 stale review claim without activating either. Both patches reproduce the original dirty tracked trees exactly. Kimi owns the released Stage-A evaluation; this delivery contains no model compute, no quality result and no change to training/M2 holds or milestone headlines.

**Next:** after this preservation merges, remove the named stale Codex worktrees; Codex remains D-A fault standby on #2029 while Kimi operates the sweep.

## 2026-10-15 — Harness write path: R10 enforced on the tool, the `unknown` fallback, the tracker watch, the claims split brain (deepseek, tooling)

The [record](../labs/harness-write-path-2026-10-15/README.md) closes Kimi's five-item harness queue. **R10 is now enforced by the harness instead of broken by it**: `receipt`, `skill use` and `prep fit` write the live shard like a check-in, every write the harness makes into a checkout — `team claim`/`release`, the ledger append and `ledger-sync`, `multilab sync-live`, `prep new`, `mistakes capture`, `ratchet --update-baseline` and `git install-hooks` — targets the caller's own checkout and refuses the shared one with the rule and the fix, and the end-to-end run from `~/uor-r4` leaves the checkout clean (`ledger-sync` exit 1, `team claim` exit 2, both refused). An unset lab now lands in the `unknown` shard with Python's loud warning instead of a nameless `.json`. The check-in watches the tracker #2028 as well as the milestones, so the PM's assignment comments surface in the unread report. The shared claims tier is read from `origin/main` through git rather than from the frozen checkout's working copy, which removes the split brain that reported a released `codex/d19-object-store-impl` claim as held. **And one defect found on the delivery run itself:** `ledger-sync --dry-run` was documented and ignored — the dry run appended 101 receipts — so the write now sits behind a read-only `plan_sync` and the flag is honoured (committed shard sha256 unchanged across a dry run, then the real sync appends exactly the planned line). 529 harness tests pass; source-only, no model/GPU/pod work and no headline change.

Line: no model line - tooling · count 0/3 unchanged · M1 headline 37/232 -> 116/232 unchanged. **Next:** merge this PR, refresh the PATH install to the merged build and repoint the stale `~/.local/share/uor-r4/bin/uor-harness` symlink (a build inside the owner checkout is R10-forbidden), then resume the DAG-reader/comment-guard parked branches.

## 2026-10-15 — Stage-A admission instrument D-A implemented, CUDA evaluation pending (codex, exact addressed memory)

The [D-A record and runbook](../labs/stage-a-admission-instrument-2026-10-15/README.md) supplies a standalone Rust evaluation instrument for the [Stage-A sheet](../harness/prep/m1-discrete-rung0-admission-2026-10-15.md): four exact pinned seed-1 checkpoints, K=8..16, all four K8 reproduction controls before wider K, unchanged parameter/source/PTX bindings, zero updates, exclusive sealed output and complete compact row receipts. Existing model, kernel and training/driver files are unchanged; the record discloses the existing operational driver-hash difference and the evaluator's masked-TRAIN reference read. Eight host metadata/output tests and D23/D25 ratchets pass; Q4 artifact metadata authenticated without CUDA. CUDA build and the 36-point sweep are **NOT_RUN**. Claude design FIT and Stage-A-only owner GO are recorded; exact-source review and Kimi's instrument packet remain before evaluation. Training/M2 HELD.

Line: starling read / exact addressed memory · count 0/3 unchanged · headline unchanged. **Next:** Claude reviews D-A and the TRAIN-reference clarification; Kimi binds the executable/PTX/source/artifacts and executes only the admitted Stage-A instrument after clearance.


## 2026-10-15 — D19 typed I/O follow-up and seed-2 checkpoint-zero guard appendix (codex, durable memory / evaluation integrity)

The [follow-up record](../labs/d19-object-store-review-nits-2026-10-15/README.md) addresses #2495's four nits: dated title style, approved-source claim pin, classifiable I/O errors, and the documented distinction between ingestion and quotation limits. The [guard appendix](../labs/seed2-guard-integration-2026-10-15/README.md) gives Claude the actual checkpoint0 invocation, all25 finite controls plus emitted-PTX admission, exact receipt locations and refusal semantics; it explicitly notes that high-u flags invalid instrumentation without automatically stopping the current CLI. Storage-byte tests only; no model/GPU/pod activity, no new FIT or release, no headline change, seed2/M2 HELD and released seed1 untouched.

Line: D19 infrastructure / evaluation integrity · count0/3 unchanged · retained M2 headline437/512 unchanged. **Next:** parent-first protected delivery, Claude's re-issued sheet after16,404, then guard support only on Casey's GO.
## 2026-10-15 — Rung-0 admission-objective successor pre-registered: Stage A kept-cells sweep ceiling instrument (kimi PM, owner decision 6187197075)

The verified seed-1 closeout (#2029 6186067850: top-1 149, top-8 747, target-admitted rows 1,920 vs bars 1,941/4,775 — arithmetically unreachable, iteration evidence under D22) led the owner to HOLD seed-2 as registered and iterate the admission objective: **the admitted-row ceiling must exceed the decisive bar counts before any run is released**. [`docs/harness/prep/m1-discrete-rung0-admission-2026-10-15.md`](../harness/prep/m1-discrete-rung0-admission-2026-10-15.md) (canonical record [`docs/labs/rung0-admission-prep-2026-10-15/README.md`](../labs/rung0-admission-prep-2026-10-15/README.md)) pre-registers **Stage A**: a zero-update K ∈ 8..16 sweep re-scoring the four preserved seed-1 quarter checkpoints (shas pinned) on the frozen 14,287-row panel — an exact-integer ceiling measurement, qualified by exact reproduction of the sealed K=8 quarter metrics. Gate G-A: some K reaches ≥ 4,894 target-admitted rows at 16,404 (bar 4,775 + one Wilson half-width). PASS → Stage-B training amendment at the smallest passing K (fresh lineage, re-based guard; Claude FIT, owner release — training HELD until both). FAIL → closes eval-time widening of the K=8-trained codebook (D22 configuration closure; K>8-trained lineages stay open as a named successor); redirect to the admission policy/objective sheet; no training under this sheet. Docs-only, TEST FITNESS: PENDING, zero compute, zero hold changes.

Line: starling read / exact addressed memory · count 0/3 unchanged · headline unchanged. **Update:** design FIT confirmed by Claude [6192687624](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6192687624), Stage-A-only owner GO recorded by Kimi [6192566334](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6192566334); the PENDING label above describes the original registration. **Next:** exact-source D-A review, then Kimi's bound instrument packet and evaluation. Training/M2 remain HELD.

## 2026-10-15 — Router repaired against the current core; Pages deploy paused to keep the working site live (claude, tooling, covering DeepSeek)

The Pages deploy has failed since #2345, which retired the core modules both browser engines were built on: the legacy R4G1/router engine (whose `transformerless::compiler` is floating-point with squashing code, D23/D32) and the old native-geometric model API. `uor-r4-router` now builds again natively and on wasm32. Its self-contained `session_signature`, `fallback` and `host_bounds` are compiled; the historical engine is preserved in place behind a non-default `legacy-engine` feature. The router dependency is dropped from the **library** targets of `api`, `graph-certify` and `graph-runtime` (none use it), and the `graph-runtime` lib builds again. **Stranded non-lib targets** (they still import the router or the retired core and do not compile under default features): `uor-r4-api/tests/production_prefix_memory_canary_944.rs`, `uor-r4-graph-certify/tests/router_reconnect.rs`, `uor-r4-graph-runtime/benches/session_signature.rs`, the `graph-runtime` lib tests (through its `graph-compiler` dev-dependency), and the router's 18 legacy integration tests (`crates/uor-r4-router/tests/`, `UorR4Router` behind `legacy-engine`). All were already blocked by the router before this change. **Owner decision:** keep the last good site (13 Oct 18:28Z) live, so the deploy job is skipped, not failing, until M4 lands a D11 served wasm engine (browser serving is CPU model compute under D24). Still broken and out of scope: native `graph-compiler`/`graph-certify`/`api`/`proof-model` and the root crate, all on the retired core compiler/runtime.

## 2026-10-15 — DECISIONS.md gains D35 (seed-2 re-issue requirement + admission-iteration hold) and D36 (Pages deploy skip until a D11 served wasm engine) (kimi PM, records)

Two owner decisions are now filed in [`DECISIONS.md`](DECISIONS.md): **D35** — any seed-2 GO requires a re-issued packet with the checkpoint-zero guard in the binary (new qualified build from a named main commit, new pins, `completion-check-plan` to `PLAN_METADATA_VALID`, owner-personal release), with §4 recording the same-day admission-iteration hold (6187197075): the re-issue applies only if the owner later GOs the registered seed-2; **D36** — the live Pages site stays as the last good deploy and the deploy job is skipped (`if: ${{ false }}`) until M4 lands a D11 served wasm engine, superseding the same-morning "drop R4G1 from wasm" relay (6184702464 → 6185876280); the native `graph-compiler`/`graph-certify`/`api`/`proof-model`/root-crate breakage on the retired core modules is filed for its own owner decision. Canonical record: [`docs/labs/decisions-d35-d36-2026-10-15/README.md`](../labs/decisions-d35-d36-2026-10-15/README.md). Docs-only, no compute, no hold changes.

Line: PM / records · count 0/3 unchanged · headline unchanged. **Next:** Claude's TEST FITNESS verdict on #2504 (rung-0 admission Stage A); Claude's #2503 records commit.

## 2026-10-15 — Post-D30 mechanism status labels and geometry/figure cross-reference audit (antigravity, mechanisms line)

[`docs/labs/post-d30-mechanism-labels-2026-10-14/README.md`](../labs/post-d30-mechanism-labels-2026-10-14/README.md) audits [`README.md`](../../README.md), [`docs/mechanisms/README.md`](../mechanisms/README.md), [`docs/geometry.md`](../geometry.md), and [`assets/images/README.md`](../../assets/images/README.md) for the post-[D29](DECISIONS.md#d29--no-banned-mechanism-in-any-compiled-or-loaded-module-vendor-runtime-content-counts)–[D34](DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate) discrete substrate, labeling every mechanism and historical result as `active`, `dormant`, or `archived-negative (D22 scope)` with direct links to preserved evidence:

1. **[`README.md`](../../README.md) and [`docs/mechanisms/README.md`](../mechanisms/README.md):** Defined the three-label vocabulary (`active` on the D32 discrete geometric substrate `crates/uor-r4-discrete`, the D32 Oct 14 offline Prime-Zeta `Snapshot.table` constructor, the D34 `ReadSpec::Relation` reranker design card, or the separate D19 `uor-r4-object-store` line; `dormant` for open geometric mechanisms under D12/D22 or pre-D32 floating-point / held M2 source preserved inactive under D29/D30/D32; `archived-negative (D22 scope)` for specific tested configurations closed or prohibited under D18/D21/D22/D23/D24/D25/D30 without closing the underlying mechanism). Added the missing [`prime-zeta-complex-routing.md`](../mechanisms/prime-zeta-complex-routing.md) and [`finite-relational-head.md`](../mechanisms/finite-relational-head.md) rows to `README.md` so all 17 briefs in `docs/mechanisms/` are indexed, extended the rules table through D28–D34, and labeled every row across the architecture summary, earlier operator-phase table, mechanism tables, and historical results/negatives tables.
2. **[`docs/geometry.md`](../geometry.md) and [`assets/images/README.md`](../../assets/images/README.md):** Preserved all existing prose, tables, and `<img src="...">` figure references with no content removed (in-place label additions/rewrites only) and zero SVG regeneration. Added a post-D29–D34 scope note to the top banner of `docs/geometry.md`, annotated `## The compliant next-token path` and `## The operators of the compliant trunk` with `dormant` (pre-D32 float source in `crates/uor-r4-training`) vs `active` (discrete substrate in `crates/uor-r4-discrete`, offline Prime-Zeta constructor, and D34 `ReadSpec::Relation` design card) and `archived-negative (D22 scope)` (`G1` gate), and added explicit **Post-D30/D32/D34 label** bullets and table annotations with preserved-evidence pointers across Sections `## 1`–`## 10`, `## Designs not yet built`, `## Serving without multiplication`, and `assets/images/README.md`. Docs-only, no content removed, zero SVG changes, zero compute (`$0.00`), training and M2 `HELD`.

Line: mechanisms design / documentation · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged). **Next:** Stage M1 finite CUDA construction qualification ([#2494](https://github.com/UOR-Foundation/uor-r4/pull/2494), `TEST FITNESS: FIT` by Claude at `4f0181b2c`) runs only after explicit owner release on `#2029`.

## 2026-10-15 — Stage M1 finite CUDA construction qualification pre-registered for `ReadSpec::Relation` (antigravity, mechanisms line)

[`docs/harness/prep/m1-finite-relational-head-2026-10-14.md`](../harness/prep/m1-finite-relational-head-2026-10-14.md) pre-registers the Stage M1 finite CUDA construction qualification for `ReadSpec::Relation { kept, sweeps }` (`uor-harness prep-check` `complete: true`, `missing: []`, `TEST FITNESS: FIT`) against the merged D34 design card ([`docs/mechanisms/finite-relational-head.md`](../mechanisms/finite-relational-head.md), `a6cd1d32d`, `#2486`) and incorporates all six source-derived indexing/literal corrections (`e30b7c029`) plus both final pin conditions from Claude's `TEST FITNESS` reviews ([#2029 comment 6182784893](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6182784893) / [#2494 comment 6182784657](https://github.com/UOR-Foundation/uor-r4/pull/2494#issuecomment-6182784657), [#2029 comment 6183518952](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6183518952) / [#2494 comment 6183518764](https://github.com/UOR-Foundation/uor-r4/pull/2494#issuecomment-6183518764), and confirmed `TEST FITNESS: FIT` at `4f0181b2c` in [#2029 comment 6184281299](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6184281299) / [#2494 comment 6184281042](https://github.com/UOR-Foundation/uor-r4/pull/2494#issuecomment-6184281042)):

1. **All 10 card controls pinned in `qualify-relation` (`C1`–`C8`, `8` control functions, `0` `.apply` updates):** Specifies exact `BinderPlan::new(6)` layout (`8` nodes, `stages = 2`, `14` Gate trits at `B = 2, stages = 2`, block-1 level-0 combine gate `gates[2]`, `SequentialSuccessor` level-0 edge trit `gates[6]`), exact `relation-off ≡ Flock` rollback across all `8` sweep×kept cells (`C1`, Controls 1 & 9), the 5-token `[x, A, B, y, z, A]` single-sweep `SequentialSuccessor` fixture (`C2`, Control 2) with exact level-0 score vector $r^{(1)}_{[0,1,2,4,5]} = [0, 0, 8, 0, 4]$, antipode payload `NEG_B = root 3` transporting under `BFS[1][0] = 16` ($x \mapsto -x$) to `carried[0] = 14` (`B`), outer residual `snap_2(root 15 + root 14) = 14` flipping output from `[A, D_ROOT]` to `[A, B]`, and exact counterfactual writer loss reduction `640 -> 448` (`[640, 640, 448]` across ordinals `[0, -1, +1]`) on `propose(Parameter::Gate(6))`; plus `ParentChild` level-1 promotion (`C3`, `gates[1] = 0, gates[3] = 0`), `SameToken` vs. `SameFrame` separation with `tok_x = [B, A]` (`m = 0, r_4 = 0`) and `top[0..2] = [5, 1]` / `[1, 5]` (`C4`), negative-trit suppression (`C5`), strict causal bound `last <= length - 1` (`C6`, eligible `node 3` vs. excluded `node 7`), deterministic comparator cascade `[1,4,0,2]` / `[0,1,2,4]` / `[0,4,1,2]` with `node asc` documented as a defensive guard (`C7`), and hand-derived multi-sweep score vectors for `sweeps = 1..=5` (`[2,1]`, `[2,4]`, `[4,5]`, `[5,4]`, `[5,4]`; `r^{(5)}_{0..8} = [1344, 2176, 2072, 1848, 4088, 5880, 5952, 5424]` at all-`+1`) plus `admit_ptx` and refusal checks (`C8`).
2. **Governance boundary (`TEST FITNESS: FIT` confirmed by Claude at `4f0181b2c`):** Docs-only pre-registration (`0` GPU hours, `0` `.apply` updates, `0` corpus rows); training and M2 remain `HELD`; `C6` vs `G64` remains an unselected contingent arm; D19 exact object store (`uor-r4-object-store`) remains separate; and any trained-in `Relation`-vs-`Route` corpus comparison waits for seed-1's `Route` rows at cursor `16,404` with a declared interleaved coordinate schedule ([#2502](https://github.com/UOR-Foundation/uor-r4/pull/2502)).

Line: exact addressed memory · count 0/3 unchanged (finite construction pre-registration) · headline 37/232 → 37/232 (unchanged). **Next:** finite CUDA qualification runs only after explicit owner release on `#2029`.

## 2026-10-15 — PM hygiene: stale tracked branch claims cleared (kimi, process)

`docs/harness/claims.json` is back to `{}`: it carried four claims whose remote branches are all deleted and delivered — `claude/completion-pins` and `claude/spiralcore-attention-review-20261014` (seed-1 pins family; the #2482 independent review at `618e422ce`), `kimi/completion-release-plan` (merged), and `claude/harness-notes-surfacing` (resurrected by a stale-base regeneration in #2497, which then merged at `231bc500c`). Live-tier claims are unaffected; labs claim fresh branches as usual. Docs-only, no compute.

## 2026-10-15 — Heartbeat surfaces unread notes itself; open-PR seats watched; board flags unread-but-live (claude, tooling, covering DeepSeek)

`uor-harness heartbeat` now refreshes the lab's notes cache from GitHub when it is absent or older than 20 minutes, so a lab that only checks in still sees comments addressed to it. The network step is bounded: 20 s per call, 45 s overall, presence written first, and a failure falls back to the cache. Freshness comes from a stamp written only after a COMPLETE read; a partial read retries. The watched set adds open PRs where the lab has a `<lab>/` branch or a whole-word mention. `multilab presence` shows ` · N unread >15m` (JSON `unread_over_15m`). `notes` and `heartbeat` create a lab's cache and marker on first use, so a new lab joins with one command (verified with `UOR_LAB=antigravity` on a fresh live tier). Rust-only; exact-head self-review including two Claude-internal fresh-context review passes (CHANGES, then APPROVE with nits; summarized on the PR), and Kimi PM's independent review ([#2497 6183406458](https://github.com/UOR-Foundation/uor-r4/pull/2497#issuecomment-6183406458)); 517 tests pass. Low-severity follow-ups are listed in the PR.

## 2026-10-15 — Stage M2 trained-in `ReadSpec::Relation` vs. `ReadSpec::Route` discriminator pre-registered on M4 #2032 (antigravity, D34 reranker-first attention sequence)

[`docs/harness/prep/m4-finite-relational-head-2026-10-15.md`](../harness/prep/m4-finite-relational-head-2026-10-15.md) (with canonical lab record [`docs/labs/m4-relation-discriminator-prep-2026-10-15/README.md`](../labs/m4-relation-discriminator-prep-2026-10-15/README.md) and aligned pointers in [`docs/mechanisms/finite-relational-head.md`](../mechanisms/finite-relational-head.md) §4.3, §7.1–§7.2, §9) drafts the next pre-registration in the [D34](DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate) reranker-first attention sequence while Stage M1 finite CUDA construction qualification ([#2494](https://github.com/UOR-Foundation/uor-r4/pull/2494), `m1-finite-relational-head-2026-10-14.md`, `TEST FITNESS: FIT` by Claude at `4f0181b2c`) is in the merge queue:

1. **Symmetric Interleaved Coordinate Schedule (`SchedulePolicy::InterleavedReadEvery8V1`, `training.rs:30-35, 225-286`):** Pre-registers `InterleavedReadEvery8V1` (`"interleaved_read_every_8_v1"`) across the `schedule_cycle = V * B + B * S = 16,404`-batch pass (`1,049,856` masked targets, quarter checkpoints `0 / 4,101 / 8,202 / 12,303 / 16,404`, `next_coordinate = (completed_batches % schedule_cycle) as u32` while `parameter_count` is `16,404` for `Route` and `16,429` for `Relation`): every 8th batch ($s \bmod 8 == 7$) proposes the next read coordinate $r = \lfloor s / 8 \rfloor \bmod N_{\text{read}}$ (`2,050` read proposals total: `512 / 1,025 / 1,537 / 2,050` at Q1–Q4; `103`/`102` proposals per `Route(0..20)` in `Route-interleaved`; `46`/`45` proposals per `Gate(0..45)` = `920` combine-gate + `1,130` relation-trit proposals in `Relation-interleaved`) while the remaining `14,354` batches propose `VectorBlock` (`3,589 / 7,177 / 10,766 / 14,354` at Q1–Q4), alongside the noted `18,432`-batch (`s % 9 == 8`) full-vector coverage option (`#2482` §6, §8).
2. **Pre-Run Schedule & Sub-Panel Instrument Controls (`S1`–`S4`, `E1`–`E3`, `0` updates):** Pins 4 schedule controls (`S1` 24-step fixture with `schedule_cycle = 24`, `S2` `C192` quarter/pass counts and `16,404 -> 0` wrap, `S3` cross-schedule/cross-`read_spec` checkpoint refusals, `S4` disjoint `VectorBlock` / `Route` / `GateCombine` / `GateRelation` counters in `steps.jsonl` / `report.json`) and 3 sub-panel evaluator controls (`E1` reproducing the exact `3,847` copy-successor DEV rows and `[50, 2872, 900, 25]` distance buckets on `ft/dev` with `0` rows at $d = 1$, `E2` bit-exact additive partition $\mathcal{P}_{\text{succ}} \sqcup \mathcal{P}_{\text{comp}} = \mathcal{P}_{\text{full}}$ verified across seed-1 Q0–Q3 receipts, `E3` in-memory `gates[20..45] = 0` attribution copy).
3. **Matched Corpus Discriminator Bar (`B1`–`B3` at Cursor `16,404`, `HELD` behind `G1`–`G5`):** Requires `Relation-interleaved` (`ReadSpec::Relation { kept: 8, sweeps: 1 }`) to beat `Route-interleaved` (`ReadSpec::Route`) on `top8_correct` over the `3,847`-row copy-successor DEV sub-panel $\mathcal{P}_{\text{succ}}$ by strictly more than the measured seed-to-seed spread $\Delta_{\text{seed}}(\mathcal{P}_{\text{succ}}) = \max(|\text{seed } 1 - \text{seed } 2|, 39\text{ rows})$ (`B1`), with no full-panel (`14,287`-row) `top1_correct`, `top8_correct`, or `B3` regression beyond full-panel seed noise (`B2`), and strict positive zero-trit attribution at cursor `16,404` when `gates[20..45]` are zeroed at evaluation time (`B3`, plus `>= 2,000`-target fresh held-out transfer via `discrete-stack eval-corpus` on `KEEP` per D22 §4). Execution remains **`HELD`** behind `G1` (Stage M1 `PASS`), `G2` (seed-1 cursor `16,404` `Route` learning-slot observation `#2492`), `G3` (`S1`–`S4` & `E1`–`E3`), `G4` (Claude `TEST FITNESS: FIT`), and `G5` (explicit owner release). Docs-only, `TEST FITNESS: PENDING`, zero compute, zero GPU, zero hold changes.

Line: starling read / exact addressed memory · count 3/3 unchanged (M4) / 0/3 unchanged (M1) · headline unchanged. **Next:** Claude `TEST FITNESS` review on `#2032`; await Stage M1 construction `PASS` (`#2494`), seed-1 cursor `16,404` `Route` rows (`#2492`), and explicit owner release before any Stage M2 GPU execution.

## 2026-10-15 — Parked comment-guard continuation: accurate scan scope and source provenance (codex, tooling)

The [continuation record](../labs/comment-guard-2026-10-14/README.md#codex-continuation--15-october-2026)
preserves DeepSeek's #2488 source (`2b4256d6e` and `c0f03685d`) through attributed
cherry-picks `3c5a25e32` and `5c12dfa9f` on fresh main `5fd44f578`.
Owner-authorized Codex continuation addresses PM's four remaining text nits:
the scan covers `scripts/harness/**` and `.github/workflows/**`, the historical
re-review passed 504 Rust unit tests plus 2 integration tests, the footer has its
blank line, and the docstring explicitly distinguishes full-line from inline comments.
Fresh local checks pass: 504 Rust unit tests (6 ignored), 2 integration tests and
1 doc-test; the explicitly selected ignored Rust/Python doccheck parity test also
passes. Doccheck resolves 146 commands with 0 problems, its 14 Python tests pass,
and formatting/Clippy are clean. These are current observations, separate from the historical review.
No runtime behavior changes, live comment editing, model/GPU work or hold release.

Line: no model line — tooling · count 0/3 unchanged · headline unchanged. **Next:** protected replacement delivery; preserve the parked original source.

## 2026-10-15 — Parked reconstruction reader continuation: deterministic records and authenticated fetches (codex, D19 tooling)

The [continuation record](../labs/recon-dag-reader-2026-10-14/README.md#codex-continuation--15-october-2026)
preserves DeepSeek #2489 source `b32651b82` as attributed cherry-pick `8b6dae504` on main
`5fd44f578`. DFS-post first-insertion serialization reproduces all four retained Showcase
source/record SHA-256 pairs. Corruption is explicitly outside the [0,64) fetched-ID set,
replacing the random last-record assumption (700 independent-process checks passed).
Malformed public payloads and non-ASCII IDs refuse without slicing panics; every fetch is
hashed, including repeated IDs. The helper `id_for` is now fallible; Backend::node/stored and
record bytes remain unchanged. Thirteen reader tests and formatting/Clippy pass; impossible output capacity returns a typed error. The record
states transient whole-file indexing and actual HashMap overhead, and links the provisional
object-store adapter settlement. Source-data checks only; training/M2 HELD.

Line: no model line — tooling · count 0/3 unchanged · headline unchanged. **Next:** protected replacement PR referencing #2489; exact-head review.

## 2026-10-15 — Complete GitHub issue/PR/comment corpus archived into the repository (kimi PM, owner-requested insurance)

[`docs/history/github-archive-2026-10-15/`](../history/github-archive-2026-10-15/INDEX.md) preserves the full public GitHub evidence trail inside git, where no permission change can remove it: 462 issues, 2,033 PRs, 7,878 conversation comments, 311 inline review comments on 62 PRs, 185 PR reviews, 30 labels, 21 milestones — snapshot 2026-10-15T02:31:23Z, reconciliation 2,495/2,495 exact against live counters (every row "match" in INDEX.md), five high-citation comments (6181921625, 6181460874, 6174007797, 6174496690, 6159308818) spot-verified byte-identical to the live API. Every comment keeps its GitHub-global numeric id as a markdown heading (`### comment <id> — <created_at> — <login>`), so citations in DECISIONS.md, prep sheets, and this file resolve with grep. `export.sh` in the archive is re-runnable (`--fresh`) for future snapshots. Motivation: the owner holds org-member push-only access to UOR-Foundation/uor-r4; this is insurance, not a migration — work continues on the canonical repo. Docs-only, no compute.

Line: PM / evidence preservation · count 0/3 unchanged · headline unchanged. **Next:** periodic re-export on major state changes; repo-governance ladder with the owner (request repo admin from the org owners) unchanged.

## 2026-10-15 — Final Route learning-slot receipt tool ready; actual analysis NOT_RUN (Codex)

The [tooling record](../labs/route-learning-slot-2026-10-15/README.md) delivers `route-learning-slot`, a standalone Rust metadata analyzer invoked by the Q4 runbook after the audited scorer succeeds. It cross-checks sealed cursor-16,404 pins and cumulative VectorBlock/Route counters, distinguishes cost sensitivity from accepted updates, and labels Gate as excluded by the frozen executor rather than inventing measured Gate counters. Four synthetic metadata tests pass; no actual quarter analysis, model/GPU execution, pod contact or hold change occurred, and the existing scorer arithmetic is unchanged. **Next:** after Claude seals Q4, run the documented audit and post the per-kind evidence for the later Relation learning sheet; schedule adoption and seed-2/M2 release remain separate decisions.

## 2026-10-15 — D19 exact byte object-store prototype: original Showcase compatibility and bounded reconstruction (codex, durable memory)

The [implementation record](../labs/d19-object-store-2026-10-15/README.md) delivers `uor-r4-object-store` under approved #2487: immutable exact records, deterministic exchange, transactional import, bounded persistence/ranges/rebuild, and shared-tokenizer occurrence bindings. Twenty byte-store controls and thirteen existing tokenizer tests pass, including original hash-pinned Showcase answers; exact unique bytes still expand (32,768 → 70,972 record bytes), while repeated content deduplicates (7,776 → 380). The actual #2489 reader adapter compiles and passes exact byte/integrity checks; PM has provisionally ratified the seam while DeepSeek is offline, with its confirmation still owed on return. Source/data-structure work only: no model, GPU, training, head adoption or headline change; seed-2/M2 HELD and released seed-1 untouched.

Line: D19 durable-memory infrastructure · count0/3 unchanged · headline unchanged. **Next:** protected implementation delivery, DeepSeek seam confirmation, and separately scoped durable-session integration.

## 2026-10-14 — D19 exact object-store prototype contract (Codex, design only)

The [design note](../labs/d19-object-store-design-2026-10-14/README.md) specifies `uor-r4-object-store`: Showcase-compatible ternary records, bounded exact fold/rebuild/range/missing operations, distinct partial-path and whole-object integrity types, persistence, and pinned occurrence-to-byte-span bindings. It names the durable-memory consumer seams without connecting model code or changing attention; no implementation or measured capability is claimed. Source-only review; no model/GPU compute or hold release. **Next:** protected delivery of this contract, then the Rust implementation and byte-only controls. Seed-2 authentication stays queued for the owner's decision; training/M2 remain HELD.

## 2026-10-14 — a comment-edit guard, because the safe path did not exist (deepseek, tooling)

The [record](../labs/comment-guard-2026-10-14/README.md) carries the three parts and the live verification.
**THE INCIDENT IT ANSWERS:** fixing a typo in my own status comment on #2028, I ran `gh issue comment 2028
--edit-last --body-file ...` — which edits the **last comment on the issue**, not the last comment *by me*, and
every lab posts through one shared account, so there is no author filter; it replaced the entire body of
**comment `6145450989` (13 Oct 12:23Z)**, `--body-file` replaces the whole body, GitHub keeps no comment edit
history, and the text was unrecoverable. **THE GUARD, IN THREE PARTS:** (1) **the safe path exists** — the
harness had **no command that edits a comment at all**, so `gh` was the only path and the hazardous flag was one
keystroke away; a warning would not have prevented what happened, so `uor-harness comment edit --id
<comment-id> --body-file <path>` now edits exactly the comment named and **cannot express "last"**; (2) **the
lesson surfaces one step before the mistake** —
`docs/harness/mistakes/edit-last-overwrites-another-labs-comment.md`, severity `block`, with the trigger words a
lab would be typing, so `mistakes find -f` raises it before the command; (3) **a machine check on the executable
workflow roots covered by CI** — the `no_position_based_edits` test scans `scripts/harness/**` and `.github/workflows/**` and fails with the file,
line and the id-addressed replacement, and states its own limit: **it cannot intercept a hand-typed `gh`**, which
is exactly why part 1 had to be built. **VERIFIED:** the refusal names the hazard and why; an empty body file is
refused **before** `gh` runs; **a real edit by id against the live tracker succeeded** (`edited comment
6173738543`); the lesson's four required sections are present and it loads in the registry, which the
mistakes-registry tests confirm. **Historical PM re-review at `c0f03685d`: 504 Rust unit tests passed,
plus 2 integration tests**; fmt and clippy `-D warnings`
clean; `unittest` exit 0 with 0 failure lines; `parity.py` exit 0; interlock OK — 1641, baseline 1641. **Stated
rather than hidden:** the overwritten comment's text is gone — this prevents the next one and cannot restore
that one; and **the machine check is narrow on purpose**, guarding what executes rather than what a lab types.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

## 2026-10-14 — `ReadSpec::Relation` bounded exact-relation reranker design card revised per D34 (antigravity, mechanisms line)

[`docs/mechanisms/finite-relational-head.md`](../mechanisms/finite-relational-head.md) (and [`docs/labs/finite-relational-head-design-2026-10-14/README.md`](../labs/finite-relational-head-design-2026-10-14/README.md) §3) is revised in place per Owner Decision [D34](DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate) (`4cef409fc`), Claude's independent adversarial review ([#2482](https://github.com/UOR-Foundation/uor-r4/pull/2482) at `618e422ce`), and Kimi PM's assignment ([#2028 comment 6175624248](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6175624248), [`6175886770`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6175886770), [`6178356343`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6178356343)):

1. **Re-cast first falsifiable artifact as `ReadSpec::Relation { kept, sweeps }`:** Drops the energy-controller class of machinery (`C6` / `G64` strict-descent settling) as the first artifact because a sweep-capped update over the bounded `BinderPlan` graph ($\le 286$ nodes, depth $5$ at $C=192$) is already a deterministic $S$-sweep neighborhood aggregate whose energy and descent proof are never consumed by the readout. Specifies `ReadSpec::Relation` as a third `ReadSpec` mode sharing `Flock`'s node construction (`discrete.cu:123-155`), edges from retained structure only (`ParentChild`, `Sibling`, `SequentialSuccessor`, `SameToken` by retained token ID `ids`, `SameFrame` by orbit root, with `last <= length - 1` enforced on both endpoints), and one ternary trit per `(edge_type, level)` on the **existing `Parameter::Gate` kind** (`lib.rs:125-131`; `discrete.cu:88-94, 315-317, 329-335`, $+25$ Gate trits at `C192`), with **`relation-off ≡ Flock`** ($g[e,l] = 0$) as the exact rollback identity.
2. **Capability target and pool/writer invariants:** Adopts the **copy-successor (induction) read** (`3,847 / 14,287 = 26.9%` of pinned DEV targets following the most recent earlier in-window occurrence of the query token, a capability neither `Route` at `discrete.cu:56-71` nor `Flock` at `discrete.cu:167-189` can express) as the primary capability target; keeps the candidate pool as all causally eligible `C192` `BinderPlan` nodes ($\le 16K$ Flock shortlist excluded); leaves the shared integer vector writer (`discrete.cu:229-253`) untouched with target-cell admission (`B3`, governed solely by endpoint blocks `0` and `B-1` at `discrete.cu:208-210`) reported separately; and notes that exact compressed-data reads belong to the separate D19 durable-memory Rust crate (`uor-r4-object-store`).
3. **Contingent arms and later learning-slot prerequisite:** Demotes `C6`-vs-`G64` to a contingent arm (explicitly unselected, not needed for the first test); specifies all 10 M1 construction controls from `#2482` §9 (with exact orbit-root $d_{\text{L1}} \in \{0, 4, 8\}$, unclamped `i32`-safe match scores $m_i^{(0)} = 8 - d_{\text{L1}} \in \{0, 4, 8\}$, and conflicting-successor recency tie condition under equal `(d_L1, d_Hamming)`); and records that any trained-in `Relation`-vs-`Route` comparison **waits for seed-1's `Route` rows at cursor `16,404`** (`training.rs:224-243`) and requires a **declared interleaved coordinate schedule on both arms** as a later sheet (not pre-registered now). Docs-only, zero compute, zero GPU, zero head adoption, training and M2 `HELD`.

Line: mechanisms design / documentation · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged). **Next:** await Kimi PM review of the revised `ReadSpec::Relation` card; M0/M1 prototype and construction qualification require their own pre-registration sheet, `TEST FITNESS: FIT`, and owner release, while any trained-in comparison waits for seed-1 cursor `16,404`.


## 2026-10-14 — the reconstruction-DAG production reader: on-demand backend, and RangeVerified vs WholeObject (deepseek, instrument)

The [record](../labs/recon-dag-reader-2026-10-14/README.md) carries the tests and the two claims. **WHAT IT
REPLACES:** the byte-DAG experiment at
`docs/labs/spiralcore-native-attention-review-2026-10-14/evidence/cs/range_reader.rs` loads **every** record into a
`HashMap<Id, Node>` before answering any range query — its own README says *"the experiment is not an end-to-end
low-memory reader"* — and its final line is the other half of the gap: **`whole_object_sha256_in_rust=NOT_CHECKED`**. **THE
TWO HALVES:** (1) **an on-demand backend** — the records file is a fixed-stride sequence of `1-byte scale ||
12-byte id || 32-or-36-byte data`. `IndexedBackend::open` transiently reads the whole file and retains
a header-derived `id -> offset` HashMap (13-byte on-disk headers, plus resident keys/offsets/bucket overhead);
traversal fetches and authenticates visited records, with the experiment's
`EagerBackend` kept as the reference; (2) **two digest claims kept apart** — a node id authenticates **the nodes a
range touched** and says nothing about the nodes outside it or about the object, so `RangeVerified { nodes,
unverified_nodes }` **reports its unverified remainder** instead of letting the weaker claim pass as the stronger,
while `WholeObject { digest }` is the only variant carrying object integrity. **THE OWNER'S RULE SHAPES THE API:**
*on the main track a read must always be verified*, so the strong claim is a **separate entry point**
(`read_whole_object`) rather than a flag on the range read — a caller cannot reach integrity by forgetting to ask,
and `Read::whole_object_verified` is the single question a main-track consumer asks. **Historical initial observation (6 tests passing in one run; isolation later found flaky by PM):** 203 ranges produce **byte-identical** output from both backends and equal the source; **the original last-record corruption check was not reliable because record order was randomized**
(corrected in the Codex continuation linked above); and the decisive one, **two DAGs differing in a single bit are both internally
consistent, so node ids cannot tell them apart and only the declared whole-object digest refuses the wrong one**
(while it verifies against its own). Fixtures are **built in Rust by the test file** because the experiment's
fixtures come from a JS generator over pinned sources and are not committed. **Stated rather than hidden:**
**nothing consumes this crate yet** — it is the reader, delivered with its tests, and wiring it into a
reconstruction path is a separate step; it is **shaped to consume Codex's D19 object store rather than compete
with it** (the `Backend` trait is the seam, flagged on #2028 so the digest distinction can be settled once in that
store's read contract); the whole-object digest is **SHA-256** while node ids are the crate's own 12-byte digest
over `scale || data`; and `IndexedBackend` currently **reopens the file per node** — correct and simple, with mmap
or a held handle the obvious next optimisation and not claimed here.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

## 2026-10-14 — D34: owner accepts the revised attention/memory sequence; dead target symlink removed (kimi PM, owner decision)

Delivered via a persistent decision dialog and recorded at #2028 comment 6176107062: the owner **accepts Claude's revised sequence** from the #2482 independent review — reranker-first (bounded exact-relation reranker as a third `ReadSpec`, relation-off ≡ Flock, copy-successor read at 3,847/14,287 = 26.9% of pinned DEV targets as the capability target) plus a separate D19 object-store crate for exact compressed reads on the durable-memory line. Direction only: no compute, adoption or hold release without pre-registration + FIT + explicit owner release; a trained-in comparison also waits for seed-1's Route rows at cursor 16,404 and an interleaved schedule on both arms. Antigravity's relational-head card addendum proceeds on this basis. The same decision authorized removing the dead `target/` symlink in the owner checkout (done). PM claims cleanup rides this delivery: the stale `kimi/d29-nvrtc-amendment` and `kimi/discrete-rung0-amendment1-20261014` claims are released (both long-delivered work). See [D34](DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate).

## 2026-10-14 — Independent adversarial review of the SpiralCore reconstruction-native attention proposal (Claude, owner-requested)

The [review](../labs/spiralcore-attention-independent-review-2026-10-14/README.md) (Claude Fable 5.1, max effort) read the six original attachments, both Codex reports (#2478, #2480), the eight specialist reviews and the migration decision brief, confirmed the discrete crate has no source drift since Codex's snapshot `88c413599`, and reran the three retained byte/data-structure checks on the original attachments: all byte-identical to the retained outputs (`results.json`, `golden-results.json`, `results.txt`; `construction.json` identical except timings). **Verdict on the migration as posed: REVISE — it is four separable items.** Exact compressed-data reads: PROCEED TO PROTOTYPE as a Rust object store on the D19 memory line (the model path has no transfer step, `corpus.rs:143-186`; unique content expands 2.17× in records, so the benefit is warm/edit reuse). Root packing and the ≤16K Flock index: INSUFFICIENT EVIDENCE for any change (correct derivations; no payoff at C192/B4, context capped at 243; the index is per frame block, and payload blocks 1-3 already share one selection, `discrete.cu:107`). Relational attention: REVISE then PROCEED TO PROTOTYPE of a bounded exact-relation reranker as a third `ReadSpec` using the existing Gate kind — the energy controller is unconsumed machinery, and the source shows neither Route nor Flock can express a copy-successor read (`discrete.cu:56-71`, `167-189`), which is the concrete capability the hypothesis should target, and a data statistic on the pinned DEV corpus shows it is large: 3,847 of 14,287 targets (26.9%) follow the most recent earlier occurrence of the query token (model top-8 at Q1: 1,090; count reference: 4,060). Shared writer: no change; admission (Q1 2,201/14,287) is the measured ceiling and must be reported separately. Learning-credit correction: the registered schedule proposes read parameters only in the last 20 of 16,404 batches (`training.rs:224-241`), so a trained-in comparison needs an interleaved schedule on both arms and should wait for seed-1's Route rows at cursor 16,404. Parallel versus replacement: neither; a third read law under the existing dispatch is the whole migration, rollback is `read_spec: route`, and no evidence proposes replacing transport, binder or writer. No model/CUDA/training execution, rental, adoption or hold release; not FIT; the active seed-1 run is untouched.

Line: owner-requested independent architecture review · count 0/3 unchanged · milestone headline unchanged. **Next:** the owner's decision on the revised sequence; if accepted, a source-only design card for `ReadSpec::Relation` (edges, trits, sweeps, controls) and a separate Rust object-store crate on the D19 line, each with its own prep sheet before any GPU work.

## 2026-10-14 — Q2–Q4 scoring runbook and seed-2 baseline authentication source (codex, evaluation integrity)

The [quarter runbook](../labs/completion-quarter-runbook-2026-10-14/README.md) now
provides the exact sealed-pod pull/verify/score sequence. Its Q1 rehearsal freshly
pulls the original evaluation/pin/checkpoint and reproduces Kimi's complete audit
JSON byte-for-byte: top-1 96, top-8 1,090, strict wins 177, admitted strict wins 41,
B3 2,201. Only the scorer's local checkpoint-path resolver changes; scoring and
validation logic are unchanged. Q2/Q3 remain diagnostic, final cursor is 16,404.

The [seed-2 authentication source](../labs/seed2-baseline-authentication-2026-10-14/README.md)
binds checkpoint zero, its own row-derived u and the complete frozen count reference
before update admission, with immutable receipt/pin checks on resume. Metadata
controls pass; the actual CUDA integration is not yet qualified. New operational
driver identity requires a new exact-source review/build/seed-2 packet before use.
The live seed-1 executable and all released plans remain untouched. No model/GPU
compute, seed-2 release, mechanism adoption or headline change occurred.

**Next:** Kimi/Claude use the Q2–Q4 runbook after each sealed quarter. Seed-2 source
is available for review; qualification and personal release remain separate gates
after the owner's seed-1 decision. Seed-2 and M2 stay HELD.

## 2026-10-14 — Prime-zeta mechanism doc correction and source-only finite relational head design card (antigravity, mechanisms line)

[`docs/labs/finite-relational-head-design-2026-10-14/README.md`](../labs/finite-relational-head-design-2026-10-14/README.md) completes both source-only items assigned by Kimi PM in [#2028 comment 6174783178](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6174783178) (with coordination note [#2029 comment 6175149631](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6175149631)):

1. **Corrected [`docs/mechanisms/prime-zeta-complex-routing.md`](../mechanisms/prime-zeta-complex-routing.md):** Removed the stale claim that the constructor replaced the runtime xorshift initializer and removed the nonexistent `metadata.rs:344-367` citation (`metadata.rs:1-211` is `BinderPlan` forest metadata, and `discrete_init` in `discrete.cu:257-267` remains the untouched xorshift PRNG default initializer preserving all 10 frozen D5 runtime/driver hashes). Aligned the brief with the delivered standalone host-side offline constructor (`crates/uor-r4-discrete/src/prime_zeta.rs` and `crates/uor-r4-discrete/src/bin/prime-zeta-qualify.rs`, [#2473](https://github.com/UOR-Foundation/uor-r4/pull/2473)/[#2477](https://github.com/UOR-Foundation/uor-r4/pull/2477)), explicitly stating that `V4096 B4` zero-collision (`4096/4096`) and 256-cell balance (quota `16`) are enforced by the post-scoring quota/uniqueness step shared with the nulls, that primary `PrimeZeta` seeds `0, 1, 2` verify deterministic repeatability (`b4_seed0 == b4_seed1 == b4_seed2`) rather than distinct random initializations, and that downstream learned attribution remains `NOT_RUN` / `HELD`.
2. **Authored [`docs/mechanisms/finite-relational-head.md`](../mechanisms/finite-relational-head.md) (and updated [`docs/mechanisms/README.md`](../mechanisms/README.md)):** Specifies the source-only finite relational head design card synthesized from [#2480](https://github.com/UOR-Foundation/uor-r4/pull/2480) (`reviews/integrator.md`, `reviews/math-cross-review.md`, `reviews/ml-cross-review.md`, `reviews/physics.md`, `reviews/cs.md`), covering: (i) the causal candidate pool $\mathcal{V}_q$ over the canonical token-occurrence `BinderPlan` (`< 286` nodes at $C=192$), with the **$\le 16K$ Flock shortlist explicitly excluded as the head's pool**; (ii) an $O(|\mathcal{V}_q|)$ prefix-local sparse edge generator (`QueryAnchor`, `BinderTree`, `OccurrenceChain`, `FrameBucket`, $D_{\max} \le 10$, canonical single-edge orientation, zero all-pairs scan, zero oracle hints); (iii) strict separation of immutable evidence frames/payloads from temporary controller states $s_i \in \mathcal{S}$, input/query anchors, and both candidate finite state realizations (**Option A: `G64` / `R16` native geometric states** with the order-4 stabilizer quotient and no order-3 subgroup vs. **Option B: separately typed `C6` = $\mathbb{Z}/6\mathbb{Z}$ phase controller** with exact integer table `[2, 1, -1, -2, -1, 1]` and frustrated $120^\circ$ triad minimum $-3$, **with `C6` vs `G64` explicitly UNSELECTED**); (iv) deterministic role-conditioned readout `(d_rel, d_L1, d_Hamming, -last, node_id)` that rejects unconditional synchronization / `PAS >= 0.82` filtering, preserves contradictory evidence under conflict regimes, and reduces bit-for-bit to `ReadSpec::Flock` at zero coupling; and (v) outer shared-writer learning slots (`RelCoupling`, `RelOffset`, `RelReadout`), active schedule exposure requirements, virtual-proposal cache/writer-admission invalidation rules, and Stage-1/Stage-2 qualification controls. Zero model/CUDA compute (`$0.00`), zero head adoption or replacement, training `HELD`.

Line: mechanisms design / documentation · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged). **Next:** incorporate any revisions from Claude Fable 5.1's independent adversarial review of #2480 ([#2029 comment 6175060216](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6175060216) / [`6175149631`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6175149631)); any future prototype or construction qualification requires PM/owner selection of `C6` vs `G64`, a dedicated pre-registration sheet, `TEST FITNESS: FIT`, and owner release.

## 2026-10-14 — the install warning was about location; it should have been about age (deepseek, tooling)
The [record](../labs/install-age-2026-10-14/README.md) carries both directions of the measurement. **WHAT WAS
WRONG WITH MY OWN FIX:** #2476 made the check-in name the binary you are actually running instead of
prescribing a rebuild of a checkout you are not using, but its condition was **location** — any binary outside
the checkout produced the line — so a **freshly installed, NEWER** copy triggered it on every command (measured
at 12:59 installed against a 06:57 build), **a permanent nag for every lab using the documented `uor-harness`
on PATH, which is worse than the silence it replaced**; a location is not a staleness, a comparison is. **THE
CHANGE:** `is_stale_install(exe_time, built)` is pure and `false` when either time is unknown, because with
nothing to compare silence beats a guess; `newest_build` finds the newest binary the checkout has built
(`local/harness-target` first, then `target`), and `placement_line` reports only when the running copy is
**older** than that, naming both paths and the install command — now including `--target-dir
local/harness-target`, because `target/` in this repo is a **broken symlink** to a volume that no longer exists
so a plain `cargo install` fails with `Not a directory (os error 20)`. **VERIFIED BY RUNNING IT, BOTH
DIRECTIONS:** install newer than the checkout's build → **silent** (the nag is gone); a build later than the
install → **warns**, naming both paths; the checkout's own build → silent, unchanged. **Verified:** **501 Rust
tests passed, 6 ignored**; fmt and clippy `-D warnings` clean; unittest and `parity.py` exit 0; interlock OK —
1641, baseline 1641. **Found and stated:** the comparison is between file mtimes, so it answers whether the copy
you are running is older than something this checkout built, not which source is newer; and **no build in the
checkout means no line**, because a guess would be worse than silence.
Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

## 2026-10-14 — SpiralCore reconstruction descriptions as native attention objects (Codex, owner-requested research review)

The [constructive follow-up](../labs/spiralcore-native-attention-review-2026-10-14/README.md) addresses the owner's architecture question after #2478: compression before tokenization, the full HTML's current golden-route score/instrument/FBS reconstruction loop, and attention over exact descriptions. Six native Codex specialist perspectives and math/ML cross-review found a coherent interface, not an adopted replacement or FIT verdict. Fresh original-code byte checks pass (address/radix and current golden/FBS primitives). A dependency-free Rust reader passes 813 range cases / 2,374 assertions over actual Showcase records: one 23-byte range in a 32,768-byte fixture touches eight nodes/two leaves, 384 serialized record bytes, after full store construction/loading. No physical network or waveform performance claim follows. Source-derived exact options are four-bit root labels and a comparator-specific ≤16K Flock index, with occurrence, BinderPlan, virtual-proposal and trace-count bindings. A new finite relational head has its own candidate pool, input anchors, readout and outer-task learning contract; byte-storage and token-model trees remain distinct. No model/CUDA/training execution, source/configuration adoption, paid resources or owner release occurred. The prior report and all peer work are preserved; training remains HELD.

Line: owner-requested architecture assessment · count unchanged · milestone headline unchanged. No new waiting dependency for Kimi or Claude. **Next:** this requested review is complete; the record supplies separate, unadopted exact-representation and finite-relational-head options for a future design decision, with FIT and owner release still required for model work.

## 2026-10-14 — a floor with no age is not a warning: the check-in now dates its note cache (deepseek, tooling — PM queue item 2)

The [record](../labs/notes-age-2026-10-14/README.md) carries the four-case verification. **THE GAP, FROM THIS
LAB'S OWN FAILURE:** the check-in already surfaces unread notes, deliberately from the **cache** so a check-in
stays instant, and `notes::render` already says **"at least N"** rather than overstating a cached read — but the
line carried **no age**, so a lab could not tell a fresh answer from a stale one; this morning the check-in
printed `at least 1 unread note(s) on #2034` and I read past it while **887 unread notes — including the PM's
14:55 tasking — sat behind a cache hours old**, and a lab that had never run `notes` got **nothing at all**. **THE
CHANGE:** the cache file's **mtime is the refresh time**, so no schema change is needed and existing caches keep
working; `cache_age` reads it, `cache_line` turns it into one line, and the check-in prints it — `notes were last
refreshed 6h ago, so the count above is a floor - run: uor-harness notes`, or `no notes cache on this machine
yet - run: uor-harness notes`. **The threshold is 20 minutes because that is the CONTEST window the PM named**,
not a round number: past it a cache can already have missed a contest posted while the lab was working — my
first version used 30 and tightening it to the named window is the difference between covering the failure and
covering a looser one. **Still network-free:** auto-fetching would break the property that a check-in is instant,
which is the entire reason the cache exists — the line reports, `notes` refreshes. **VERIFIED BY RUNNING IT, IN
ALL FOUR CASES:** cache 6h old → **prints the age**; 20m or 21m old → **reports** (the boundary); fresh or 19m →
**silent**; **no cache at all → reports, where it used to be silent**; `--no-notes` → silent as documented.
**Verified:** **500 Rust tests passed, 6 ignored**; fmt and clippy `-D warnings` clean; unittest and `parity.py`
exit 0; interlock OK — 1641, baseline 1641. **Found and stated:** nothing is fetched at check-in, so a lab that
ignores the line still sees a stale floor — the line makes that visible rather than silently true; and the age is
the **cache file's mtime**, so a failed refresh leaves the old mtime, which is the honest answer (the data is
that old).
## 2026-10-14 — SpiralCore transport and attention proposal: source-grounded research assessment (codex)

The [assessment](../labs/spiralcore-attention-assessment-2026-10-14/README.md) reconciles six owner-supplied artifacts with the current discrete path through three parallel reviews. Exact finite actions and shared-store byte reconstruction are supported; the logarithmic frontier does not bound retained history, and the new attention note supplies no verified semantic or learning advantage. All three supplied codec suites and an independent finite byte/tree audit pass, with full cold/warm/update costs retained. Route remains the registered arm; Flock and the separately merged prime-zeta constructor retain their existing qualification/adoption scope. No model compute, model code or registered plan changes, training release or milestone promotion occurred.

**Next:** specify the proposed read's retained information, causal discrete update and shared-writer interface only if it answers a distinct application question; no model replacement is compelled by this assessment.

## 2026-10-14 — Prime-zeta complex routing CUDA construction qualification PASS (`17/17` base + `6/6` prime-zeta controls), matched nulls, and `V4096` codebook fixtures (antigravity, mechanism construction)

[`docs/labs/prime-zeta-construction-2026-10-14/README.md`](../labs/prime-zeta-construction-2026-10-14/README.md) completes the Prime-Rank Indexing + Complex-Plane Zeta-Phase ($\mathbb{C}^4 \to \mathbb{R}^4$ / $\mathbb{C}^8 \to \mathbb{R}^8$) construction qualification registered in [`docs/harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md`](../harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md) (`sha256: 26b585e59758f736f6ed066b1bb8b6a445d66a2245957903293c0b6e4cb917f7`, `TEST FITNESS: FIT`, owner release [`#2029` comment `6170312990`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6170312990), implementation merged in [#2473](https://github.com/UOR-Foundation/uor-r4/pull/2473) at `912a8a0690d0427fac0e4abed4bdc9636841ea05`). **`crates/uor-r4-discrete/src/prime_zeta.rs` implements the bijective prime atom map $r(v) = p_{\text{id}(v)+1}$ (`first_n_primes` up to $V=8192$; $p_1 = 2, p_{256} = 1619, p_{4096} = 38873$), one-time host-side offline complex-plane zeta-phase computation $\theta_{b,c}(p) = \gamma_{4b+c} \ln p \pmod{2\pi}$ over the Riemann zeta ordinates from `crates/uor-r4-core/src/zeta_zeros.rs` (`include!`d directly so `uor-r4-core` is never added to `uor-r4-discrete`'s `[dependencies]`), and the immediate integer quantization boundary into `PhaseQ29` ($i32$), `UnitS3Q30` ($[i32; 4]$), and $E_8$ orbit roots (`geometry::ROOTS: [[i8; 8]; 16]`) in `Snapshot.table`.** Across named seeds `0, 1, 2` and across both `aarch64-apple-darwin` and `x86_64-unknown-linux-gnu`, `prime-zeta-qualify host-verify` ([`receipts/host-codebook-qualification.json`](../labs/prime-zeta-construction-2026-10-14/receipts/host-codebook-qualification.json), `sha256: 41e82887…`, and [`receipts/v4096-witnesses.json`](../labs/prime-zeta-construction-2026-10-14/receipts/v4096-witnesses.json), `sha256: e7f4c636…`) proves bit-exact reproducibility (`V4096 B4` `table_sha256: e9e03bcd95b78632a7f18d1b27c07d37f0ba82c52eee692c37da011eadac8804`; `V4096 B2` `table_sha256: 41dcb6be3962b1e723c4d819eb93b2502fc3ef6135a735bf55ae3fe8a0efd023`; `V256 B2` `table_sha256: bdbe94993d7f27b9a367374df966a007cea699639ac8aae85b3a9a09bce8a4fc`) and **strict non-aliasing (`0` full-vector collisions: `4096/4096` unique `PhaseQ29`, `4096/4096` unique `UnitS3Q30`, `4096/4096` unique `Snapshot.table` vectors, and `256/256` writer cells populated with exact quota `16` on `V4096 B4`).** Both D22 §3 matched null controls are verified on `V4096`: the **Tangent Null** ($\gamma_k \to \gamma_k + 1.0$, `table_sha256: bae145fe…`) separates from Prime-Zeta on **`4090/4096` tokens (`99.85%`), `16,221/16,384` root blocks, and `55,088` coordinate Hamming distance**, and the **Normal Null** (seeded prime permutation $\pi_s(p)$ + off-critical-line perturbation across `seed = 0, 1, 2`, `table_sha256: a1031b64… / 7c35797e… / 8d3e6933…`) separates from Prime-Zeta on **`4096/4096` tokens (`100.00%`), `15,397 / 15,312 / 15,334` root blocks, and `53,551 / 53,390 / 53,122` coordinate Hamming distance**. **Executed on pod `vast-55945260` (`1×RTX 5090`, driver `580.95.05`, `nvcc 12.8.93`, `UOR_NVCC_ARCH=compute_120`, `rustc 1.97.1`, executable `sha256: 953f8f085743aab68e9613c1acbbc6d41774bc71be50178fe75b40b05e0655f5`), `prime-zeta-qualify qualify` ([`receipts/prime-zeta-qualification.json`](../labs/prime-zeta-construction-2026-10-14/receipts/prime-zeta-qualification.json), `sha256: 082ed664…`) passes `admit_ptx` (`11` entries, `zero_float_instructions: true`), passes all `17/17` base CUDA controls (`11/11` Route + `6/6` Flock), and passes all `6/6` Prime-Zeta CUDA controls (`C18..C23`: `V4096 B4` & `V4096 B2` `discrete_validate`, cross-seed CUDA forward/evaluate determinism `losses: [1280, 1216, 1248, 896]`, CUDA loss separation against both nulls, Route `VectorBlock(0)` `4640 -> 4288` & `Route(0)` `4640 -> 4448` counterfactual sensitivity, and Flock `Gate(0)` `5120 -> 4992` counterfactual sensitivity) with `applied_updates: 0` and `corpus_rows_consumed: 0`.** All 10 frozen D5/completion files remain untouched (`source_sha256: e1836149…`, `kernel_sha256: 931a511b…`, `geometry_sha256: 46ddbb76…`, `driver_source_sha256: cb090e85…`). Pod `vast-55945260` lease released and pod destroyed after `829s` at `$0.563/h` (`$0.1297` total spend). **Limits, stated:** finite construction and verification qualification only; zero training and zero corpus dose; downstream learned move-selection and corpus attribution against the matched nulls remain deferred to a future training sheet requiring its own `FIT` and owner release. Training and M2 rental holds remain `HELD`.

Line: m1-prime-zeta-complex-routing · count 0/3 unchanged (construction qualification, zero training) · headline 37/232 → 37/232 (unchanged).

## 2026-10-14 — the stale-install warning pointed at the wrong binary (deepseek, tooling — PM queue item 1)

The [record](../labs/install-refresh-2026-10-14/README.md) carries the three-case measurement. **WHAT LABS WERE
ACTUALLY HITTING:** the harness warns when the **checkout** is behind `origin/main` and prescribes a **rebuild**,
but the binary a lab runs from a shell is the **installed copy on PATH** (`~/.cargo/bin/uor-harness`) and a
rebuild never touches it — so the advice misses the binary in front of you and the warning returns every
session; **worse than a wrong instruction, on a current checkout it said nothing at all:** measured here, the
install was from 13 Oct 23:30 and the checkout's build from 14 Oct 06:57, and because HEAD == `origin/main` the
staleness check returned `Current`, so a lab running a seven-hour-old binary got **complete silence**. **THE
CHANGE:** `version::placement(repo)` decides from `std::env::current_exe()` whether the running binary is this
checkout's build or something else, and `placement_line` names the path with the command that actually refreshes
it (`cargo install --path crates/uor-r4-harness --offline`), printed by `uor-harness version` and appended to
the check-in warning; both paths are canonicalised because `current_exe` resolves symlinks on some platforms
and not others. **VERIFIED BY RUNNING IT, IN ALL THREE CASES:** the PATH install run from a current checkout
**names itself and gives the refresh command**; the checkout's own build is **silent**; `heartbeat` from the
PATH install **prints the line** then checks in. **Two unit tests hold the decision** — an outside binary is
named and the line says a rebuild does not help, and the checkout's own build produces nothing, **which is the
one that keeps it honest** because a false positive would print on every command of every lab that built
locally (`cargo test --lib version::` 9 passed, 2 new; fmt and clippy clean; unittest and `parity.py` exit 0;
interlock 1641 = baseline). **Stated rather than hidden:** the install on this machine **was refreshed** (14 Oct
11:16), which is the operational half — other labs' machines are theirs, the code is what generalises;
`docs/harness/BOOTSTRAP.md` now documents the refresh so a new lab need not meet the warning to learn the
command; and the check is about the binary's **PATH, not its build time** — a build copied elsewhere is reported
as outside, which is correct for the purpose (it is the copy a rebuild will not refresh).

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** PM queue item 2 (unread-note surfacing).

## 2026-10-14 — the transport publish failure wedged the merge queue; it now ejects it (deepseek, tooling — PM queue item 3)

The [record](../labs/transport-retry-2026-10-14/README.md) carries the mechanism and the harness. **THE WEDGE,
MECHANICALLY:** `required-transport` publishes the ruleset's five required status names, and the step ran under
`set -euo pipefail`, so **one** transient `gh api` failure — the `unexpected end of JSON input` seen twice in
production — aborted the loop and **the remaining names were never published**; **a required context that is
never published does not fail the entry, it leaves it `AWAITING_CHECKS` for ever**, which is why `--auto` would
not move it, `--disable-auto` did not dequeue it, and only `dequeuePullRequest` plus re-arming recovered it (I
hit this twice on my own PRs before it was mine to fix). **THE CHANGE:** bounded retry with backoff around every
publish (three attempts at 3s, 6s, 12s); **fail safe on exhaustion** — if the success publish still cannot be
delivered, an explicit **failure** check run is published for the same name, so the entry is **ejected** and a
re-run is visibly needed instead of waiting on a status that will never arrive; `set -e` is gone (kept
`set -uo pipefail`) precisely so one failure cannot abort the loop, and the job exits non-zero if any name could
not be delivered — both signals. **VERIFIED BY EXERCISING THE CHANGED PATH, NOT BY READING IT:** the step's
`run:` block is **extracted verbatim from the YAML** and driven through three scenarios with a stubbed `gh` (and
a stubbed `sleep`, so the backoff costs no wall time) — **A** no failures: 5 successes, exit 0; **B** two
transient failures: recovers, 5 successes, exit 0; **C** success publish permanently broken: 15 attempts, **5
failure contexts published**, 0 success contexts, exit 1 — all five assertions PASS. **My first version of that
harness asserted `success=7` for B and `success=15` for C, and both were test bugs, not code bugs** (the stub
logged every attempt including the ones it then failed); logging the outcome rather than the arguments made the
counts mean what they say — **a test that counts attempts as successes would have declared the failure path
verified without exercising it.** **Found and stated:** a total API outage still cannot publish anything, so the
entry would still wedge — that is unavoidable from inside the job, and what the change guarantees is that the
job is **red** with the reason named, while the documented transient case now either recovers or ejects; there
is **no Python comparison for this unit** (CI YAML, one shell step, no second implementation to diff against),
which is why the verification is the scenario harness.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** PM queue item 1 (PATH install refresh), then item 2 (unread-note surfacing).
## 2026-10-14 — the claims read path was missing its shared tier (deepseek, tooling — PM queue item 6)

The [record](../labs/claims-shared-tier-2026-10-14/README.md) carries the reproduction and the trap the fix
itself had. **THE WRITE PATH ALREADY RESPECTED R10 AND WAS NOT CHANGED:** measured from a throwaway worktree,
`team claim` in BOTH implementations writes `docs/harness/claims.json` inside that worktree and leaves the
owner's checkout untouched (modified files before == after == 0). **THE READ PATH WAS MISSING A TIER:**
Python's `load_claims` merges three — live, then `main`'s committed store, then the caller's own — while the
Rust port merged two; isolated (live tier empty, the worktree's own store emptied to `{}`) Rust reported **no
branch claims** where Python reported **the four committed on main**. **The consequence is the one claims
exist to prevent:** a lab in a worktree whose `claims.json` predates a peer's merged claim cannot see it, so
it can start work on a branch another lab already holds. **THE FIX, AND THE BUG THE FIX HAD:**
`shared_claims_file` now resolves the main checkout (`git worktree list --porcelain` names the primary first;
git failing falls back to the caller, making the tier equal and therefore skipped) and `load_claims` merges
live → shared → caller — but **the first version of that merge wrote `if *path == shared && shared == local`,
which is true for the LOCAL entry too when the paths are equal, so in the MAIN CHECKOUT it skipped the
caller's own store and returned only the live tier: every committed claim on the machine would have
disappeared.** Two existing `multilabrules` tests caught it (both reporting zero findings), and a temporary
probe printed the deciding line — `shared==local? true` with `merged = []`; the fix is to skip **by tier name**,
as Python does. **Verified:** `team claims` is **byte-identical to Python in the isolated case (430 bytes) and
from the main checkout (418 bytes)**; **493 Rust tests passed, 6 ignored** (was 491, with the two new
regression guards — a real linked worktree with an empty store must still see main's claim, and the caller's
own store must still be read when it is also the shared path); fmt and clippy `-D warnings` clean;
`python3 -m unittest discover -s tests/harness -t .` exit 0; **`parity.py` exit 0**; interlock OK — 1641,
baseline 1641. **Found and measured, NOT fixed here:** `multilab claims` has two pre-existing rendering
divergences with the same data underneath — its heading is `branch claims (4) · current branch main` where
Python writes a plain count and unit, and a stale row omits Python's ` STALE` suffix; that is the board's
renderer, not the claims read path, so it stays out of this PR.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** PM queue item 7 (the stale `~/.cargo/bin/uor-harness` install), then unread-note surfacing.
## 2026-10-14 — Rung-0 completion packet executable: qualified build pinned, binding reviews written, `completion-check-plan` PASS (claude, M1 #2029)

The [release packet](../labs/rung0-completion-release-2026-10-14/README.md) now carries every pin. Claude, as the
owner-named executor ([#2029 6169732401](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6169732401)), built
the frozen D5 source `33564b75e` unmodified on pod `vast-55939635` (1×RTX 5090; nvcc 12.8.93, Rust 1.97.1,
`compute_120`, `--locked`). The result is executable `d4fbe26c…`. `completion-build-identity` matches all six plan pins
for both seeds with no CUDA context. The 25 admitted finite controls pass on that executable and host (fixtures
only). Claude's binding records `seed1-review.json`/`seed2-review.json` are pinned
([6170667006](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6170667006)), and `completion-check-plan` returns
`PLAN_METADATA_VALID` on both plans with `owner_release: STILL_REQUIRED`. **Training stays HELD** until Casey's
owner-personal seed-1 release (interactive, `UOR_LAB` unset). Then seed 1 only launches; seed 2 is a separate later
owner decision. Line: evaluation integrity; headline unchanged (R14).
## 2026-10-14 — Completion scoring receipt rehearsal (codex, no model compute)

The [scoring readiness record](../labs/completion-scoring-readiness-2026-10-14/README.md)
checks the preserved pilot's full 14,287-row DEV panel and prepares an integer-only
receipt scorer for completion checkpoints. Final batch index 16,403 advances to
completed cursor 16,404; the final evaluation is saved once and reused by reporting.
B3 admission and admitted strict wins are derived from existing per-row receipts.
The scorer allows legal occupancy below eight cells instead of inheriting the
pilot-only eight-cell assumption. Frozen D5/model source is unchanged; no model
compute, owner record or release is performed. New CUDA build/identity qualification
remains Claude's separate assignment. Named mechanism successors remain unadopted.
The optimized replay checked both full pilot evaluations in 1.04 s wall with
76,349,440 bytes maximum RSS; 12 scorer tests and 25 existing metadata tests pass.
This measures host receipt processing only; future GPU evaluation latency is unmeasured.
**Next:** Claude qualifies the completion build; Kimi uses the delivered scorer on
the real registered checkpoint receipts. No Codex scoring review remains pending.

## 2026-10-14 — `budget scan` is Rust and `delegate_to_python` is GONE: **D30 is complete** (deepseek, tooling)

The [record](../labs/d30-complete-2026-10-14/README.md) carries the comparison and the whole-port summary.
**`uor-harness budget scan` RUNS IN RUST, AND `delegate_to_python` NO LONGER EXISTS IN THE CRATE** — the
general fallback arm became a plain usage error and the function was deleted; `grep -c delegate_to_python
crates/uor-r4-harness/src/main.rs` is **0**. **D30's CONDITION IS MET: `uor-harness` runs with no
`delegate_to_python` fallback for live commands, because there is no `delegate_to_python` at all — all 14
top-level commands run Rust end to end.** **THE LAST UNIT:** `budget scan` needed `resolve_pod_script` (repo
→ main checkout → `$UOR_POD`, reporting **what was checked** as well as what was found), the two status
parsers, and `_emit_scan`'s **four genuinely different payload shapes** (the not-found case carries the LIST
of places checked and no `script_source`; the not-executable case carries `script_source` and no attempts;
the both-failed case adds `status_tail`; only the available case has `caps`, `over_cap` and the cost table —
one struct serialised one way would invent keys Python never writes). **Nothing is fabricated and no pod
state is touched:** only `status` and `gpus` are ever run, and a missing tool, a non-executable one and a
failing pair each produce `UNAVAILABLE` with the attempts that led there rather than a zero. **`run_capture`
implements Python's subprocess timeout by hand** — Rust's std has none, so it polls `try_wait` against a
deadline and kills the child, because a hung pod tool must not hang the harness. **Verified by running BOTH
implementations: `budget scan` 588/588 bytes and `budget scan --json` 935/935, both MATCH** against the live
`scripts/pod/uor-pod`; the first comparison showed `bytes` 13 apart and **that was not a defect** — `uor-pod
status` prints live pod state, so the raw output length moves between invocations, the same lesson as the
disk figure. **Verified:** fmt and clippy `-D warnings` clean; **491 Rust tests passed, 6 ignored**;
`python3 -m unittest discover -s tests/harness -t .` exit 0; **`parity.py` exit 0**; interlock OK — 1641,
baseline 1641. **What D30 was, end to end:** 5 delegation sites covering 6 items at the start (the general
fallback over `arena`/`budget`/`panel`, `mistakes show|capture`, `multilab sync-live|migrate-ledger`, `team
roster|dispatch`), each ported as its own protected PR with its own record, by the same method — **run both
implementations on the same inputs and compare bytes**, which found a real defect in almost every unit: a
missing D22 sentence, a quoted id, an ordering bug, raw bar objects, a sorted `scores` map, an int-vs-float
band, and Python's decimal rounding against Rust's. Stated rather than hidden: **an unrecognised command is
now a usage error**, not a delegation (`budgett` exits 2 either way, with this binary's text); and
**`scripts/harness/cli.py` still exists and still runs standalone** — it is simply no longer reachable from
`uor-harness`, while the five `python3` calls left in the crate are all inside `#[cfg(test)]`, the
live-parity tests that compare the port against the Python it came from.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**D30 IS COMPLETE.** The next unit on this line is the PM queue's items 5–7 (cargo pinning, `claims_file()`,
and the stale PATH install).
## 2026-10-14 — `budget show` is Rust; the last delegation site is down to `budget scan` (deepseek, tooling)

The [record](../labs/budget-show-port-2026-10-14/README.md) carries the comparison and the three defects it
found. **`uor-harness budget show` RUNS IN RUST**, text and `--json`, over a faithful port of `collect()` and
its eight helpers; **UNPORTED COMMANDS 2 → 1: only `budget scan` still delegates.** **THE COMPARISON FOUND
THREE DEFECTS, AND THE THIRD WAS ARITHMETIC:** (1) **the owner's band is a tuple of INTEGERS** — Rust printed
`owner band 30.0-70.0 GB` and `"band_gb": [30.0, 70.0]` where Python writes `30-70` and `[30, 70]`; (2) **an
empty cost ledger sums to the INTEGER `0`** — Python's `round(sum([]), 4)` is an int, so the JSON prints `0`
and not `0.0`; (3) **Python's `round()` is not Rust's `round()`, and on this ledger that is worth 0.0011** —
the pod total came out `371.9637` against Python's `371.9626`, because 211 down-events are each rounded to
four decimals *as they accumulate* and Rust rounds a tie away from zero while Python rounds to even;
`round_ties_even` gave `371.9625` (still one tie off, since multiplying by 10^4 introduces binary error
before the tie is seen) and **rounding the DECIMAL — `format!("{:.4}", x).parse()` — lands on `371.9626`.**
**A fourth `DIFF` in the first bar run was not a defect:** free disk space moves while a build writes
artifacts, so comparing a live `df` figure is only valid on a quiet tree — there the comparison is clean.
**Verified by running BOTH implementations on three case groups: `budget show` 1417/1417 bytes, `budget show
--json` 3164/3164, and bare `budget` (which is `show`) 1417/1417 — all MATCH against the real repo** (352
ledger records, 12 resource ledgers, a 2396-record pod ledger, the real disk figure). **Verified:** fmt and
clippy `-D warnings` clean; **491 Rust tests passed, 6 ignored** (was 481); `python3 -m unittest discover -s
tests/harness -t .` exit 0; **`parity.py` exit 0**; interlock OK — 1641, baseline 1641. Stated rather than
hidden: **`disk_free` uses `df -k`** (Rust's std has no `disk_usage` and this crate has only
`regex`/`serde`/`serde_json`), with the band comparisons on the **unrounded** figure; **`parse_ts` assumes UTC
for a naive timestamp**, as Python does, because the pod durations were computed with that assumption; and a
**rate is never summed as spend** — a down event with no rate and no remembered rate is skipped rather than
charged at zero.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`budget scan`** — **the last live command** — and then the general fallback can be deleted and
**D30 closed**.
## 2026-10-14 — `arena compare` is Rust; the whole `arena` command is ported (deepseek, tooling)

The [record](../labs/arena-compare-port-2026-10-14/README.md) carries the fourteen comparison cases. **`uor-harness
arena compare <id>` RUNS IN RUST**, text and `--json`, and **THE `arena` COMMAND IS COMPLETE: `board`,
`compare` AND `lineage` are all Rust. THE ONLY LIVE `delegate_to_python` CALL SITE LEFT IS THE GENERAL
FALLBACK, CARRYING `budget show|scan` — unported commands 3 → 2.** **THE COMPARISON IS THE REPORT THE ARENA
EXISTS FOR:** it is a regression check, not a leaderboard, so nothing is collapsed into one number; **direction
comes from the file's `bars` map and is never guessed** (an undeclared bar is reported as `undeclared` and
**cannot be called a gain or a regression** — a 92% increase in candidates scored per query would otherwise
read as an improvement), and **a KEEP must report fresh transfer as EVIDENCE rather than as the word**, because
*"no transfer reported"* contains the word `transfer`; the rule is a `transfer` key in `scores` or the regex
`(?:fresh\s+)?transfer\s*[:=]?\s*[0-9]` over `scope`/`notes`. **Verified by running BOTH implementations on
fourteen case groups: all MATCH** — all five rows text and `--json`, the unknown candidate (exit 1/1), and the
missing id (the recorded argparse usage text) — and the rows cover a **root row** (`comparable: false` at exit
**0**, because it is still `ok`), a KEEP with a transfer bar, a panel change, an undeclared bar and integer
deltas. **Two details would have diverged quietly: `delta = round(b - a, 6)` KEEPS INT-NESS when both sides
are ints** (Python's `round(13, 6)` is the int `13` and prints `13`; a float-only port prints `13.0` on every
integer bar in this file), and **the bars compared are the UNION of both `scores` maps, sorted**, so a bar only
one side reports becomes an `UNAVAILABLE` row naming that side rather than a silent delta. **Verified:** fmt and
clippy `-D warnings` clean; **481 Rust tests passed, 6 ignored** (was 475); `python3 -m unittest discover -s
tests/harness -t .` exit 0; **`parity.py` exit 0**; interlock OK — 1641, baseline 1641. Stated rather than
hidden: a non-numeric `seeds`/`rows` reads as 0 where Python would raise; the `--json` key order is
load-bearing with `ensure_ascii` ON (unlike `arena board`); and `budget show` still sits behind `collect()`,
~400 lines including the resource-ledger pointer-chain walk.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`budget show`**, then **`budget scan`** — **two subcommands left before the general fallback can be
deleted and D30 is done.**
## 2026-10-14 — Claude's F9 number-boundary fix on v2 reply grader and prime-zeta complex routing brief (antigravity, evaluation tooling & mechanism specification)

[`docs/labs/f9-reply-grader-and-prime-zeta-complex-routing-2026-10-14/README.md`](../labs/f9-reply-grader-and-prime-zeta-complex-routing-2026-10-14/README.md) delivers the two remaining Antigravity (`antigravity/ag1`) deliverables: (1) closing Claude's `F9` number-boundary follow-up (`#2412` comment `6159546797`) on the versioned `v2` deterministic reply grader (`#2029`), and (2) authoring the Prime-Rank Indexing + Complex-Plane Zeta-Phase ($\mathbb{C}^4 \to \mathbb{R}^4$ / $\mathbb{C}^8 \to \mathbb{R}^8$) mechanism brief and pre-registration sheet (`#2029` / `#2032`) under Decision D32 October 14 Addendum ([#2398](https://github.com/UOR-Foundation/uor-r4/pull/2398)). **In `crates/uor-r4-training/src/bin/chat-grade.rs`, `contains_exact_math_term` enforces sign-aware ASCII digit and decimal boundary checks on `exact_text` matches, preventing numerical superstrings like `"18x² + x − 10"` and `"8x² + x − 100"` from falsely matching interior substrings of `"8x^2+x-10"` on `heldout-122`.** In `crates/uor-r4-training/fixtures/reply-panel-checks-v2.tsv` (`sha256: 8dbc9e5a…`), `heldout-125` terms are updated to accept `"The roots are 4.37 and −1.37"` without requiring an `"x ="` prefix while continuing to reject `"x = 1.37"`, `"The roots are 4.37 and 1.37"`, `"x = -4.37"`, `"x = sqrt(33)"`, and `"33/2 is the answer"`. **All 25/25 unit tests pass, with 35 positive controls (`crates/uor-r4-training/fixtures/reply-panel-positive-controls-v2.tsv`, `sha256: afb7b412…`) and 17 negative controls (`crates/uor-r4-training/fixtures/reply-panel-negative-controls-v2.tsv`, `sha256: e581e95e…`) pinned.** Re-scoring the four historical open-panel reply sets (`mixture-dose-2026-10-10.tar` and `reply-panel-open-2026-10-09.tar`) via `chat-grade grade-replies checks=v2` and `jq` reproduces all baseline scores with **zero delta: `29m` `11/96`, `rp-D0` `11/96`, `rp-D10` `15/96`, `rp-D25` `10/96`.** Frozen panel manifest `data/panels/MANIFEST.sha256` passes `30/30 OK` byte-identical (`data/panels/reply-panel-checks.tsv` remains untouched at `aba96550…`). **The mechanism brief `docs/mechanisms/prime-zeta-complex-routing.md` is authored and indexed in `docs/mechanisms/README.md`, deriving the bijective prime atom map $r(v) = p_{\text{id}(v)+1}$, Riemann zeta-zero ordinates $\gamma_k$ (`crates/uor-r4-core/src/zeta_zeros.rs`), log-prime phasors $e^{i\gamma_k \ln p} \in \mathbb{C}$, canonical realification $\mathbb{C}^4 \to \mathbb{R}^4 / \mathbb{C}^8 \to \mathbb{R}^8$, and the exact integer quantization boundary into `PhaseQ29`, `UnitS3Q30`, and $E_8$ anchor roots (`ROOTS: [[i8; 8]; 16]`) so that 100% of downstream model compute remains discrete.** The pre-registration sheet `docs/harness/prep/m1-prime-zeta-complex-routing-2026-10-14.md` incorporates Kimi PM's (`kimi/pm1`) N1–N3 registration amendments (`#2029` comment `6163842973`), declares D32 Addendum Item 3 host-side offline pre-quantization compliance, and passes `uor-harness prep-check` at `TEST FITNESS: FIT`. **Zero Python used:** all grading, fixtures, and checks executed exclusively in Rust (`cargo`, `uor-harness`), `jq`, and standard POSIX tools inside isolated worktree `.worktrees/antigravity-ag1` on branch `antigravity/ag1-f9-complex-zeta`. Root checkout `/Users/casey.allard/uor-r4` remains 100% clean. **Limits, stated:** `TEST FITNESS: FIT` clears registration only and lifts neither hold; training and M2 rental holds remain `HELD`; no GPU compute or pod lease was initiated.

Line: evaluation integrity & mechanism specification · count 1/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** Finite construction qualification of prime-zeta codebook non-aliasing and zero-float PTX only after scoped owner authorization; downstream learned tangent/normal null comparison awaits training hold release.

## 2026-10-14 — `arena board` is Rust; the empty arena answers before `--json` does (deepseek, tooling)

The [record](../labs/arena-board-port-2026-10-14/README.md) carries the six comparison cases and the two
defects they found. **`uor-harness arena board` RUNS IN RUST**, text and `--json`, on the reader that
landed with `lineage`. **UNPORTED COMMANDS 4 → 3** (`arena compare`, `budget show`, `budget scan`); **live
call sites stay 2** — the general fallback now carries only `budget` x2. **THE COMPARISON CAUGHT TWO
THINGS:** (1) **the empty arena answers BEFORE `--json`** — Python checks `if not arena.rows` first and
prints the two-line `no rows recorded yet` message **whichever flag was passed**, while the first version
of this arm checked `json` first and answered `{}`; **a lab with no rows is the most likely reader of this
command**, and it was the one case the shape got wrong. (2) **`scores` KEEPS THE FILE'S KEY ORDER, AND THE
REAL DATA NEEDS IT** — `addressed-read` writes `wrong_value_failures` before `memory` and `starling-tree`
writes `hit_rate, recall, candidates_per_query`, so `scores` is read through `ordered::Ordered`, the same
type `panel show --json` needed for its raw bars. **Verified by running BOTH implementations on six case
groups: all MATCH** — the real arena text at **981 bytes** and **`--json` at 4612 bytes a side**, the
no-rows text and JSON at 134 each, and a synthetic arena (unsorted `scores`, whitespace-heavy `limits`,
absent fields) at 457/814, which exercises Python's `Candidate.__dict__` normalisation: absent `parent`
renders `null` and `-`, absent `scores` renders `{}`, and `limits` is collapsed and cut to 300 code points.
**Verified:** fmt and clippy `-D warnings` clean; **475 Rust tests passed, 6 ignored** (was 472);
`python3 -m unittest discover -s tests/harness -t .` exit 0; **`parity.py` exit 0**; interlock OK — 1641,
baseline 1641. Stated rather than hidden: **`board --json` uses `ensure_ascii` ON, unlike `panel show
--json`** — Python calls plain `json.dumps(..., indent=2)` here and passes `ensure_ascii=False` there, and
both are copied; rows are sorted by `date` alone and stably, because this is a lineage rather than a
ranking; and `compare` still delegates.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`arena compare`** (91 lines), then **`budget show`** (behind `collect()`, ~400 lines including
the resource-ledger pointer-chain walk), then **`budget scan`**. **Three subcommands left before the
general fallback can be deleted.**
## 2026-10-14 — `arena lineage` is Rust; Python crashes on the path this completes (deepseek, tooling)

The [record](../labs/arena-lineage-port-2026-10-14/README.md) carries the eight comparison cases. **`uor-harness
arena lineage <id>` RUNS IN RUST**, and it brought **`arena::load`** with it — the reader `board` and
`compare` will reuse, so the next unit is rendering rather than reading. **DELEGATION: LIVE CALL SITES
1 → 2; UNPORTED COMMANDS 5 → 4.** The general fallback now carries `budget` x2 and `arena board|compare`
have their own named line — the same shape as the `panel` unit, where the site count rises while the
unported count falls, and **both are stated.** **PYTHON CRASHES ON THE PATH THIS PORT COMPLETES:**
`arena lineage <unknown>` calls `emit(..., file=sys.stderr)` at `arena.py:465`, but `emit(text, out=None)`
has **no `file` parameter**, so the command exits 1 with a **TypeError traceback**; this port prints **the
message the Python code intended** and exits **1**, the same code the crash produces — **reproducing a
traceback is not a fidelity target.** **Verified by running BOTH implementations on eight case groups:**
`arena lineage <id>` for **all five rows** in `docs/harness/arena.json`, a root and a parented chain, and
`--json arena lineage` (Python ignores the flag, so this does too) — **all MATCH**; `arena board` still
delegates (stdout and exit MATCH, stderr carries the delegation notice by design); the no-lineage case
exits 1/1 with the documented stderr difference; the missing-id case is the recorded argparse usage text.
**Verified:** fmt and clippy `-D warnings` clean; **472 Rust tests passed, 6 ignored** (was 468);
`python3 -m unittest discover -s tests/harness -t .` exit 0; **`parity.py` exit 0**; interlock OK — 1641,
baseline 1641. Stated rather than hidden: **`load` treats a missing or unreadable arena file as an EMPTY
arena**, exactly as Python does, and **delegation resolves `scripts/harness/cli.py` relative to `--repo`**,
which is pre-existing behaviour and why the `board` comparison runs in the real checkout.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`arena board`** (rendering; `arena::load` is in place), then **`arena compare`** (91 lines),
then **`budget show|scan`**. **Four subcommands left before the general fallback can be deleted.**
## 2026-10-14 — `panel show` is Rust; the last site is back to ONE delegation call site (deepseek, tooling)

The [record](../labs/panel-show-port-2026-10-14/README.md) carries the nine comparison cases. **`uor-harness
panel show` RUNS IN RUST, AND THE `delegate_to_python` CALL-SITE COUNT WENT 2 → 1**: the general fallback
is the only one left, carrying `arena` x3 and `budget` x2; **unported commands 6 → 5.** **WHY `show` NEEDED
A NEW TYPE AND `score` DID NOT:** `panel show --json` does not print a shape this crate invented — for
each panel its **`bars` value IS the panel file's own list of bar objects**, keys in the order the file
wrote them (`id, label, current, target, of, unit, direction, note`), **`note` included.** Three things
made that impossible with `serde_json::Value`: **its object is a `BTreeMap`, so parsing sorts the keys
away**; **`registries::Bar` has no `note` field at all**, so re-serialising the struct would drop it; and
re-serialising would reorder the keys it does have — **the output would look right and diff wrong.** So
**`ordered.rs` adds an `Ordered` JSON value that keeps every object's key order at every depth**, with
Python's two renderings (compact `", "`/`": "` and `indent=2`); **the scope is deliberate**, because
turning on `serde_json`'s `preserve_order` feature would change every `Value` in the crate, including the
paths that rely on the sorting to match Python's `sort_keys=True` writers. **Verified by running BOTH
implementations on nine case groups: all MATCH**, including **`--json` at 11,943 bytes a side** — a nested
document with the bars' `note` keys and file order intact — and the milestone text forms at 1352/1128/1119
bytes. **Verified:** fmt and clippy `-D warnings` clean; **468 Rust tests passed, 6 ignored** (was 461);
`python3 -m unittest discover -s tests/harness -t .` exit 0; **`parity.py` exit 0**; interlock OK — 1641,
baseline 1641; **`delegate_to_python` call sites: 1 — the general fallback only.** Stated rather than
hidden: the `no panels found ... in docs/harness/panels/` line hard-codes the relative path Python
computes with `_rel`; `no --milestone` selects every panel in file order; and a bar whose `of` is a
number prints through Python's `str()`.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`arena`** (`board`, `compare` 91 lines, `lineage`) then **`budget`** (`show`, `scan`) — five
subcommands, and `ordered.rs` is now available for both if their `--json` prints a raw document.
## 2026-10-14 — `panel score` is Rust; one command ported, six left at the last site (deepseek, tooling)

The [record](../labs/panel-score-port-2026-10-14/README.md) carries the twenty comparison cases and the
three defects it found. **`uor-harness panel score` RUNS IN RUST** — the D22 acceptance-panel check: one
measured value against one frozen bar, answering **`PASS` / `FAIL` / `AT NOISE`**, where **AT NOISE means
the movement is inside the panel's stated noise band and must not be reported as a real difference.**
**THE UNPORTED-COMMAND COUNT WENT 7 → 6; THE DELEGATION-SITE COUNT WENT 1 → 2**, and only one of those is
progress, so both are stated: the general fallback used to carry all seven subcommands and now carries
five (`arena` x3, `budget` x2), while **`panel show` is delegated by its own named line** because its
`--json` prints each bar as the panel file's own object in the file's key order and **`serde_json` is a
`BTreeMap` with no `note` field on `registries::Bar` at all** — **a `show` that re-ordered those keys
would look right and diff wrong.** **THE COMPARISON CAUGHT THREE THINGS I HAD WRONG:** (1) **I had read
only HALF of `classify`** — Python appends *" - noise is unstated for this bar, so this cannot be called
a real difference (D22 section 3)"* whenever the band is unstated, which is most panels, and the first
port printed the shorter sentence; (2) **Python quotes the headline bar id and NOT the alternatives**
(`'complete-correct-512'` vs `fresh-transfer`); (3) **the `--bar` note was printed BEFORE `--value` was
validated**, so a command that then failed still printed a note about the headline bar — argparse refuses
first, and now so does this. **Three formats had to be written out by hand:** `%g`/`%+g` (`150` must not
print as `150.0`), the `--json` dict order with `improvement`/`needed` present only on the numeric path,
and `noise_band`, because **an unreadable band is `None` and never `0` — a band of zero turns every
movement into a real difference.** **Verified by running BOTH implementations on 18 cases against the
repo's real panels, text and `--json`: stdout, stderr and exit code MATCH on all except two argparse
usage blocks**, which are this binary's message with the same exit 2. **Verified:** fmt and clippy
`-D warnings` clean; **461 Rust tests passed, 6 ignored** (was 450); `python3 -m unittest discover -s
tests/harness -t .` exit 0; **`parity.py` exit 0**; interlock OK — 1641, baseline 1641.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`panel show`** (needs the order-preserving JSON parse), then `arena` (3 subcommands,
`compare` is 91 lines), then `budget` (2 subcommands, 908 lines). **The D30 condition is met only when
all six are done and the general fallback can be deleted.**
## 2026-10-14 — `multilab sync-live|migrate-ledger` are Rust; one delegation site left (deepseek, tooling — PM queue item 4)

The [record](../labs/multilab-sync-port-2026-10-14/README.md) carries the six comparison cases and the
formats. **`uor-harness multilab sync-live` AND `multilab migrate-ledger` ARE RUST, AND THE `multilab`
DELEGATION SITE IS GONE: LIVE `delegate_to_python` CALL SITES 2 → 1.** **The only one left is the
general fallback, which is `arena`/`budget`/`panel`.** **THIS IS ALSO PM QUEUE ITEM 4: `sync-live` IS THE
COMMITTED PRESENCE TIER'S ONLY REFRESH PATH**, and the durable files are a day stale because **the
heartbeat writes the live tier only, on purpose** (a check-in every few minutes must never dirty a
checkout — R10) — so the committed record is refreshed when a lab chooses to publish it, and until this
unit that command was Python. **THE README WAS WRONG AND IS NOW RIGHT:** `docs/harness/README.md` said
*"Every heartbeat writes both, from one record"* and *"the durable write happens regardless"* — **neither
is true and the second is the opposite of the design**; both lines now say what happens, and the section
records that the old claim was wrong so the next reader does not restore it. **`migrate-ledger` NEVER
DROPS A LINE:** an unparseable row is quarantined in `unreadable-<month>.jsonl`, a row with no lab goes
to `unknown`, a lab that slugs to nothing keeps Python's own `-<month>.jsonl`, and every moved row is
re-serialised with Python's `json.dumps(ensure_ascii=False, sort_keys=True)` so a migration does not
rewrite the record it moves. **TWO FORMATS, BECAUSE PYTHON HAS TWO:** the live tier is compact
(`sort_keys=True`), the committed tier is PRETTY (`_save`: `indent=2`) — **and the committed tier is read
in a git diff, so writing it compact would show every line of every lab's row as changed.** **THE DEFECT
MY OWN COMPARISON FOUND: the legacy-file matcher required 11 bytes and `2026-10.jsonl` IS 13**, so the
migration matched nothing, moved nothing and **reported zero rows moved with exit 0** — Python's run had
five shards and this one had none, which is what made it visible; it is recorded in the code as well as
here. **Verified by running BOTH implementations on six case groups, comparing stdout, stderr, exit code
AND every file written** (the whole committed presence directory for the syncs, the whole ledger
directory for the migrations): **all MATCH.** Stated rather than hidden: **Python requires `--json`
BEFORE the subcommand** and this binary accepts either position; a live file with `schema` other than 1
would be rewritten as 1; and the live tier stays compact-without-spaces, which nothing but Rust writes
now. **Verified:** fmt and clippy `-D warnings` clean; **450 Rust tests passed, 6 ignored** (was 446);
`python3 -m unittest discover -s tests/harness -t .` exit 0; **`parity.py` exit 0**; interlock OK — 1641,
baseline 1641.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`arena`/`budget`/`panel` behind the general fallback — the LAST delegation site** — then the
recorded follow-ups (`check`'s text rendering, `status.no_orphan_branches` reporting `ok` where Python
warns, the receipt `ts` rendering) and PM queue items 5–7.

## 2026-10-14 — `heartbeat`: a bare check-in blanked the intent, and the session could not be named (deepseek, tooling — PM queue items 1–3)

The [record](../labs/heartbeat-interface-2026-10-14/README.md) carries the before/after repros, all run
against a **scratch live tier** so no other lab's board was touched. **THREE DEFECTS IN THE ONE COMMAND
EVERY LAB RUNS EVERY FEW MINUTES, from claude/review1's note on #2028 (6163960613):**
**(1) A BARE `heartbeat -m m1` ERASED THE INTENT** — it rewrote presence with `doing` and `next` empty,
so **a lab checking in with the onboarding prompt's own command read as live-and-idle on every board,
including its own last intent, which its previous file had held one moment earlier** (that is the shape
Kimi saw as "your board reads idle"); **the rule is "absent", not "empty"** — a flag passed with an
empty value is a lab clearing the field deliberately and still does, verified in the same run. **A missing
flag now carries the previous check-in's value forward** (`presence::remembered`, with the rule stated as
a unit test). **(2) `--session` DID NOT EXIST** (`unknown option --session`, exit 2), so
`Presence.session` was **never set by anyone** and two sessions of one lab were indistinguishable on the
board; the flag now exists, is carried forward like the intent, **and the Python transport forwards it
too — `cli.py heartbeat` is a thin transport to this binary, and a flag present in one entry point and
missing in the other is the two-implementations problem in miniature.** **(3) `heartbeat --help` PRINTED
THE GLOBAL USAGE**, listing the command and none of its flags, so a lab had to read `main.rs` to learn
that `--line`/`--doing`/`--count`/`--next`/`--session` exist; a command word seen before the help flag
now selects its own flag list, and commands with none written still print the global usage. **ONE
BEHAVIOUR CHANGE WORTH STATING: a bare check-in's RECEIPT now carries the previous line**, because the
receipt and the presence are written from one record — verified in the live shard (carried line after a
bare check-in, absent after `--line ""`, exactly as presence says). **Verified:** fmt and clippy
`-D warnings` clean; **446 Rust tests passed, 6 ignored** (was 445); `python3 -m unittest discover -s
tests/harness -t .` exit 0; **`parity.py` exit 0**; interlock OK — 1641, baseline 1641; **the worktree is
clean after every check-in** (heartbeats write the live tier only). **NOT FIXED HERE: queue item 4** —
the durable presence tier is a day stale and the README is wrong about what refreshes it. **That is not a
heartbeat defect:** the Rust heartbeat writes the live tier only ON PURPOSE (R10: a five-minute check-in
must not dirty a checkout), and the durable tier's refresh path is `multilab sync-live` — **which is
still delegated to Python.** So the fix is to port `sync-live` and correct the README, and it is the next
unit rather than a rider on this one.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**FOUND WHILE VERIFYING, NOT FIXED HERE: the Python suite FAILS when `CARGO_TARGET_DIR` IS EXPORTED — the variable every doc tells a lab to build with.** `stateformats._harness_binary` looks in `$CARGO_TARGET_DIR/debug/uor-harness` first, so with it still set the rule reports a missing contract file as an **error** where the test expects **skipped** (`'error' != 'skipped'`, one command to reproduce). **Deterministic, not the known intermittent flake**, and a test/`stateformats` resolution concern rather than a heartbeat one.

**Next:** **`multilab sync-live|migrate-ledger`** (the durable tier's refresh path, and a delegation
site), then `arena`/`budget`/`panel`; then queue items 5–7 (pin rustup `cargo` in the bootstrap,
`claims_file()` pointing at the owner checkout, the stale `~/.cargo/bin` install).
## 2026-10-14 — A2 paired diagnosis and D5 completion admission source (codex)

The [source-bound diagnosis](../labs/a2-pilot-diagnosis-2026-10-14/README.md)
accounts for the pilot's +68,000 loss numerator: token contribution −63,968,
address contribution +131,968. A Rust audit of all 14,287 stored CUDA row pairs
finds 5,100 lost target-cell admissions and 1,600 gained; gained-admission rows
provide 983 of 1,169 final top-8 hits. The four-block initializer is symbolically
limited to sixteen patterns at this shape for any seed. Coverage distinguishes
four block proposals per visited token, vocabulary target mass, and the full
parameter pass's 1,049,856 / 2,425,925 actual TRAIN target exposures. Ninety
accepted proposals belong to zero-TRAIN-target-count vectors, so that cohort
cannot be declared inactive. No model rerun or mechanism verdict.
The strict-win guard counts 105 → 149 distance-margin wins, of which only
0 → 36 have the target admitted; the unchanged guard is not an executed-hit count.

D5 adds separately scoped completion admission, exact checkpoint-1640 import
with an explicit driver-only lineage migration, preserved pilot baseline,
registered quarter evaluations with sealed recovery pins, and separate vector /
Route counters. The old pilot cap/release remains isolated; seed 2 requires its
own owner-personal release. Numerical kernels and evaluator are unchanged.
Twenty-five Rust metadata tests pass, including atomic run-state publication
after Claude's interruption finding; new CUDA build/runtime remains UNVERIFIED.
Kimi owns the amended completion sheet; Claude owns FIT/exact-source review;
source delivery and recorded measurements release no hold. All training and M2
remain HELD. Collision-free initialization and admission/schedule remedies are
separate prospective configurations, not silent changes to this continuation.

## 2026-10-14 — `mistakes show|capture` are Rust; `capture` was refusing its own output (deepseek, tooling)

The [record](../labs/mistakes-write-port-2026-10-14/README.md) carries the twelve comparison cases.
**`uor-harness mistakes show` AND `uor-harness mistakes capture` ARE RUST, AND THE `mistakes`
DELEGATION SITE IS GONE: LIVE `delegate_to_python` CALL SITES 3 → 2** (the general fallback and
`multilab sync-live|migrate-ledger`). `show` reads one lesson body; **`capture` is the writer** — it
validates the draft against the same rules `mistakes.registry_valid` applies, refuses to overwrite
without `--force`, writes, reads the file back and removes it if it somehow fails, **so a capture can
never leave behind a lesson that turns the lab's own `check` red.** **THREE DEFECTS THE COMPARISON
FOUND, NONE VISIBLE BY READING: (1) `capture` REFUSED EVERY LESSON WITH AN AUTO-GENERATED GUARD** —
Rust's front-matter reader did not strip quotes, and `build_lesson_text` writes the default guard
quoted because it contains a colon, **so the guard began with a `"` instead of `none`, failed the guard
rule, and the command could not write the lesson it had just generated** (fixed by porting
`core._coerce`'s one relevant case); **(2) the success message was three lines short** — Python also
prints the severity/class/guard summary, the `read it back` command and the `check --only mistakes`
line, and the port printed only the write line **because `capture` had been read as far as the write —
the stdout comparison is what showed it**; **(3) EVERY front-matter value goes through the YAML
renderer, not only the awkward ones** — wrapping only the punctuated fields meant a title with a space
was written bare where Python quotes it, a difference in every captured lesson. **AND TWO DEFECTS IN
THE RULE THE DRAFT IS VALIDATED AGAINST: `mistakes.registry_valid` was missing three checks Python has**
(`'trigger' needs at least two phrases`, `'paths' is empty`, `'<key>' must be an inline or block
list`) **and used Rust's debug formatting where Python uses `!r` and tuple reprs**
(`severity must be one of ('block', 'warn', 'info'), got 'nope'`). **They were invisible because
Python's registry loader refuses a lesson with a bad id or severity BEFORE the rule runs**, so the
only way to compare was a lesson that loads and is still wrong: with one, **both implementations now
report the same eight findings, in the same order, with the same messages.** **Verified on scratch
checkouts across twelve case groups, comparing stdout, stderr, exit code AND every file in the mistakes
directory: all match except one message** — with a required flag absent, Python's argparse prints a
usage block and this port prints `mistakes capture needs --id and --title`, both refusing with exit 2.
**FOUND WHILE COMPARING, NOT FIXED HERE: `check`'s TEXT RENDERING IS NOT PYTHON'S** — same verdicts,
different layout (`harness check: …` + `FAIL <rule> [<path>]: <message>` + a trailing hint line there;
a bare count + `  error <rule> [<path>]` with the message on its own line here), **surfaced only
because comparing the rule messages required reading both renderings side by side, and the same class
as the ledger line: identical verdicts, different bytes, invisible to every JSON comparison.**
**Verified:** fmt and clippy `-D warnings` clean; **445 Rust tests passed, 6 ignored** (was 438);
`python3 -m unittest discover -s tests/harness -t .` exit 0; **`parity.py` exit 0**; interlock OK —
1641, baseline 1641; **`check`'s rule-id set on the real tree is 41 = 41 with no new findings from the
added checks.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`multilab sync-live|migrate-ledger`**, then **`arena`/`budget`/`panel`** (which removes the
general fallback), plus the recorded follow-ups: `check`'s text rendering, `status.no_orphan_branches`
reporting `ok` where Python warns, and the receipt `ts` rendering.

## 2026-10-14 — `team dispatch` is Rust, and diffing the written line found the ledger was the wrong shape (deepseek, tooling)

The [record](../labs/team-dispatch-port-2026-10-14/README.md) carries the comparison table and the
behaviour copied rather than corrected. **`uor-harness team dispatch` IS RUST AND THE `team`
DELEGATION SITE IS GONE: `delegate_to_python` SITES 5 → 4.** The command resolves the role, writes ONE
receipt through `receipts.record` and prints what landed — **the write half is why it stayed delegated:
`team roster` could be ported by reading, a command that WRITES the shared ledger cannot be.** So
`receipt::record` came with it: the `event` mapping, the `_NEVER` key list, the empty-value rule,
redaction, lab attribution, the unattributed warning, and the `unknown-<month>` shard for a record with
no lab. **THE DEFECT THE COMPARISON FOUND: `Receipt::to_line` DOCUMENTED ITSELF AS "matching
`json.dumps(record, ensure_ascii=False, sort_keys=True)`" AND WROTE COMPACT JSON** — `{"count":2}`
where Python writes `{"count": 2}`. **Every reader parses, so nothing failed, and this module's own
header says two implementations writing one file differently "would make every ledger change
unreadable": it took the first comparison of a WRITTEN line against Python's to see it, because
reading a line back is not comparing it.** Fixed with `py_dumps_sorted` (Python's default separators,
sorted keys, `ensure_ascii=False` for the ledger), while `--json` keeps the OTHER Python default
(`ensure_ascii=True`, insertion order) through `Recorded::to_pretty`, which reuses the `ensure_ascii`
the roster unit built. **AND THE TEST THAT SHOULD HAVE CAUGHT IT COULD NOT FAIL:**
`a_receipt_line_is_sorted_json` split the line on `","` — which the compact form happened to produce and
which Python's separators do not — so after the fix it would have passed on an empty key list; it now
asserts the EXACT LINE. **Verified by running BOTH implementations against scratch checkouts and
comparing stdout, stderr, exit code AND the written shard line on six cases** (full flags, minimal,
`--count 0`, `--json`, no `UOR_LAB`, whitespace task): **all MATCH**, and the error paths agree (unknown
role is byte-identical on both sides, exit 2; `--count xyz`/`--state bogus` refuse on both, exit 2).
**ONE DIVERGENCE REMAINS AND IT IS DELIBERATE: a receipt's `ts` is `…Z` in Rust and a local offset in
Python — the same instant, and `now_rfc3339`'s own comment already records the decision that both are
valid RFC3339 and the reader parses either.** It is stated here because this unit is the first to
compare the written bytes; the comparison asserts the two forms as instants rather than assuming it.
**Behaviour copied rather than corrected:** `--count 0` becomes 1 (Python's `args.count or 1`); a string
is redacted but a LIST is not redacted element by element (Python tests `isinstance(value, str)`); no lab
identity WARNS and preserves the receipt rather than refusing it; `--scope` takes several values, so only
its first is consumed by the flag parser. **Verified:** fmt and clippy `-D warnings` clean; **438 Rust
tests passed, 6 ignored** (was 432); `python3 -m unittest discover -s tests/harness -t .` exit 0;
**`parity.py` exit 0** including *Rust-written receipt read back by Python: ok*; interlock OK — 1641,
baseline 1641.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the four remaining sites — `mistakes show|capture`, `multilab sync-live|migrate-ledger`, and
`arena`/`budget`/`panel` behind the general fallback — plus the recorded `check` defects
(`status.no_orphan_branches` reports `ok` where Python warns).

## 2026-10-14 — `check` read its arguments as a rule filter, so `--repo` ran ZERO rules (deepseek, tooling)

The [record](../labs/check-flags-2026-10-14/README.md) carries the matrix and the two further findings.
**`uor-harness --repo DIR check` CHECKED NOTHING AND PRINTED `0 rules, 0 failed (0 error, 0 warn)`.**
`check --json` reports `"checked": 52`; adding `--repo` reported `"checked": 0`, and **the pre-change
binary reproduces it, so it was live on `main`.** **`--repo` is forwarded into the subcommand argument
list on purpose** (`main.rs` pushes it into `rest` because `receipt` needs to know where it writes) **and
`cmd_check_rules` handed that same list to `checkdispatch::run` as the `--only` PREFIX SET** — every token
was a filter, and no rule id starts with `--repo`, so **a path silently disabled the entire rule set and
the output was indistinguishable from a clean repository.** Controls: `--only arena` checks 4, `--only zzz`
checks 0. **This is the third time this port has found the same shape** — a command reporting success
having checked nothing — after `cmd_check` running only the format contract and three rules that were
registered and never dispatched; **it is the first time the trigger was an invocation a lab would type by
habit.** **What changed: `checkdispatch::parse_args` parses `check`'s own flags** (`--only` with Python's
`append`/`nargs="+"` flattening, `--staged`), consumes the forwarded `--repo`/`--lab`, and makes anything
else a **usage error at exit 2**, because an unknown token must never be a prefix that happens to match
nothing; **`Scope`/`run_scoped` make `--staged` what Python makes it — a rule SET, not a filter over
results — with `STAGED_RULES` empty today because every rule declares the default `repo` scope, and an
`#[ignore]`d live test comparing that table against `checkrules.rules(scope="staged")` so a staged rule
added in Python fails the test instead of being skipped silently;** and **`--staged` was missing from
`SUBCOMMAND_FLAGS` entirely**, so the top-level parser rejected it before `check` saw it. **Verified by
running BOTH implementations on seven invocations: the rule-id sets match on every one** (`check` 41=41,
`--repo` 41=41 where it used to be 0, `--staged` 0=0, `--only arena` 4=4, `--only prep budget` 8=8,
`--only prep --only budget` 8=8, and `--bogus`/bare-token refused at exit 2 by both). **The one
disagreement, 51 vs 58 records on the unfiltered run, predates this unit: Python counts PR-body gates a
`check` with no body cannot answer.** **FOUND WHILE FIXING, NOT FIXED HERE: `status.no_orphan_branches`
FIRES IN PYTHON AND REPORTS `ok` IN RUST** — Python warns `local branch bpb-split still exists but PR #2081
merged 2026-10-09`; the ported rule covers the local half only, while Python reads `branch_pr_check` (a
merged-PR lookup: `bpb-split` was squash-merged, leaving no ancestry link) and remote branches with no PR,
**so the rule is ABSENT where Python's fires — the same sentence this port wrote when it fixed
`heartbeat_receipts` and `registry_vs_github`.** Also stated: **the Rust form accepts `check --repo DIR`,
which Python rejects — a divergence in the port's favour.** **Verified:** fmt and clippy `-D warnings`
clean; **432 Rust tests passed, 6 ignored** (was 425/5); the `#[ignore]`d staged-set test passes;
`python3 -m unittest discover -s tests/harness -t .` exit 0; **`parity.py` exit 0**; interlock OK — 1641,
baseline 1641; **`delegate_to_python` sites 5 — unchanged; this unit ports no command.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`status.no_orphan_branches` as its own unit** — the `gh` merged-PR chain and the remote-branch
list, compared against Python on the same tree — then **`team dispatch`** (port `receipts.record`; compare
the written receipt AND the shard file), `mistakes show|capture`, `multilab sync-live|migrate-ledger`, and
`arena`/`budget`/`panel` behind the general fallback.

## 2026-10-14 — `team roster` is Rust; two recorded findings were corrected by measurement (deepseek, tooling)

The [roster port record](../labs/team-roster-port-2026-10-14/README.md) carries the full evidence and the two corrected findings. **`uor-harness team roster` IS RUST, AND ITS OUTPUT IS BYTE-IDENTICAL TO PYTHON'S — 1,936 BYTES BOTH
SIDES FOR THE TEXT FORM AND 6,017 FOR `--json`, `diff` EMPTY ON BOTH.** The previous attempt at this
command was **attempted and reverted**; this one reuses that record's roles shape, `(inherit)` route
and column widths, **and corrects two of its three findings — both by comparing output rather than
reading code.** **CORRECTION 1: THE RECORDED BLANK LINE DOES NOT EXIST.** `python3 scripts/harness/cli.py
team roster | head -c 80` gives `b'9 dispatchable roles (docs/harness/roles.json)\n  architecture-reviewer
    planne'` — Python's `emit` writes the text as-is when it already ends in `\n`, so the string's own
`\n` IS the terminator and there is ONE newline; a port that trusted the finding would have shipped a
blank line. **CORRECTION 2, FOUND BY THE DIFF: `json.dumps` DEFAULTS TO `ensure_ascii=True` AND
`serde_json` ALWAYS EMITS UTF-8.** After the text form matched byte for byte, the JSON differed in exactly
12 bytes — three role purposes carry an em dash, Python wrote `\u2014` and the port wrote the raw
character; the document is now escaped Python-style, astral characters as a surrogate pair, and the diff
is empty. **THIS MAY NOT BE CONFINED TO `roster`: any ported `--json` path carrying non-ASCII has the
same divergence unless its own comparison covered such a value — NOT MEASURED HERE, and stated rather
than left to be rediscovered.** **THE ORDER IS THE FILE'S ORDER, which is why the first attempt was
reverted:** `roles.json` is an object keyed by id and Python prints file order, while `serde_json`'s
default `Value` is a `BTreeMap` and SORTS it — so the loader collects map entries in DOCUMENT order and
the JSON is serialised from a real struct rather than through `serde_json::json!`, which routes through
`to_value` and would sort each role's fields too; `the_real_registry_loads_in_document_order` checks the
loaded order against the FILE TEXT rather than a remembered list. **Deliberate behaviours kept: a missing
`roles.json` is an empty roster while a BROKEN one is an error** (`0 dispatchable roles` is exactly the
wrong answer that looks right, and Python raises); the purpose is cut at **100 characters** with a test
whose 100th character is an em dash; an absent field takes the default while an explicit `null` is
`str(None)` = `None`. **Limits, stated:** the ported `str()` covers scalars and would render a list in a
text field as JSON text rather than Python's `repr` — `the_real_registry_keeps_its_text_fields_scalar`
asserts on the real file that every text field is a string, so the limit cannot be crossed silently, but
that is an assertion about a file other people edit. **`team dispatch` IS NOT PORTED, DELIBERATELY:** it
also WRITES a receipt through the ledger shard writer, so porting it means porting `receipts.record` with
its redaction, lab attribution and unattributed warning, and wiring the read half while the write half
stays in Python is the half-ported command this port keeps refusing to ship. **Verified:** `--json` and
text `diff` empty against Python; the `#[ignore]`d live test passes; **425 Rust tests passed, 5 ignored**
(was 411/4; 14 new tests plus 1 ignored); fmt and clippy `-D warnings` clean; `python3 -m unittest
discover -s tests/harness -t .` exit 0; **`parity.py` exit 0**; interlock OK — 1641, baseline 1641;
**`delegate_to_python` sites 5 — unchanged in count, with `main.rs` 1582 now `dispatch` alone.**

**FOUND WHILE VERIFYING, LIVE ON `main`, NOT FIXED HERE: `uor-harness --repo DIR check` RUNS ZERO RULES.**
`check --json` reports `"checked": 52` and `python3 scripts/harness/cli.py --repo $DIR --json check`
reports `checked 59, failed 20`, but **`uor-harness --repo DIR check` prints `0 rules, 0 failed (0 error,
0 warn)`** — the pre-change binary reproduces it exactly, so it is not this unit's. **`--repo` is forwarded
into the subcommand's argument list** (`main.rs` pushes it into `rest` because `receipt` needs to know where
it writes) **and `cmd_check_rules` reads that same list as its `--only` FILTER**, so the path is taken as a
filter matching no rule id; controls confirm the mechanism (`--only arena` -> 4, `--only zzz` -> 0). **THIS
IS THE "REGISTERED BUT NOT DISPATCHED" CLASS ONE LEVEL UP FROM THE `cmd_check` CORRECTION: the rule set
that was wired in is made to vanish by an invocation a lab would reasonably type, and the output is
indistinguishable from a clean repository.** `parity.py` does not catch it because it does not invoke
`check` that way. **It is the next unit, before more porting: every later unit's verification runs through
`check`, and a `check` that can report zero rules is not a gate.**
Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **first the `--repo`/`check` defect above** — `uor-harness --repo DIR check` must run the rule set Python runs, not zero, because every later unit verifies through it; then **`team dispatch`** — port `receipts.record` (shard path, redaction, lab attribution, the
unattributed warning) and compare the written receipt AND the shard file against Python's; then
`mistakes show|capture`, `multilab sync-live|migrate-ledger`, and `arena`/`budget`/`panel` behind the
general fallback — **five items, not six.**

## 2026-10-14 — D33 discrete seed-1 pilot completed, preserved, pod retired (codex)

The [admitted A2 diagnostic pilot](../labs/a2-pilot-2026-10-14/README.md) completed
on RTX 5090 with the qualified `d12877b6…` executable, first attempt/exit 0:
1,640 batches, 104,960 masked-target exposures, 236 accepted VectorBlock updates.
The same 14,287 DEV targets scored top-1 17 → 82, top-8 173 → 1,169 and strict
distance wins 105 → 149. Aggregate loss numerator increased 13,816,032 → 13,884,032
(denominator per row 256); masked-TRAIN count reference remains ahead at
1,226 top-1 / 4,060 top-8. Sensitivity differs from identity on 1,179/1,640 steps.
Stored vector patterns 16 → 155; occupied writer cells 16 → 124. This is a mixed
diagnostic observation on ascending token IDs0..409, including 236 zero-count
identities. Route learning/sensitivity NOT_RUN; bars/guard NOT_DECIDED; mechanism
verdict NOT_ASSIGNED; fresh transfer UNAVAILABLE/NOT_RUN. No useful-chat claim,
mechanism closure, full-run authorization or M2 release follows.

All 1,640 receipt checks pass, including strict accepted-move loss comparisons,
unchanged baseline/reference and final evaluation/checkpoint binding. Full archive
`icloud:UOR-R4/results/codex/a2-pilot-complete-20261014.tar`, 5,607,936 bytes,
MD5 `fb81eaa335a1d21ca1d05bde9771b3e3`, download round-trip verified. Controller
wall 6,143.011024199s; pod lifecycle 7,138s provider uptime at $0.892/h, approximately
$1.77 including setup and preservation (rate-times-uptime estimate). Pod deleted
06:23:09Z; fresh inventory confirms zero pods. Prior costs remain in the cumulative
ledger. [Completion handoff to Claude/Kimi](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6162275408).

Line: evaluation integrity · sheet count 1/3, no reset · reply headline remains
SUSPENDED under R14 (historical 37/232 unchanged); pilot bars are not decided.

**Next:** Kimi/Claude review a prospective successor using the measured coverage,
collision, sensitivity and mixed DEV results; collision-free integer initialization
and broader coordinate coverage remain proposals. The completed pilot releases no
further compute. M2 and all subsequent training remain HELD.

## 2026-10-14 — `team roster`: attempted, NOT delivered, and the two findings that make the next attempt cheap (deepseek, tooling)

**I ATTEMPTED `team roster`, DID NOT GET IT VERIFIED, AND REVERTED THE WIRING. THE REPO IS AT `b15ba7998` WITH 411 TESTS PASSING, CLIPPY CLEAN, AND `team roster` STILL DELEGATED. THIS IS RECORDED AS A NON-DELIVERY.** **Why revert rather than ship: the command produced output that differed from Python's in two ways I had found but not yet corrected, AND A `roster` THAT LISTS NINE ROLES IN AN ORDER PYTHON WOULD NOT PRINT IS A WRONG ANSWER THAT LOOKS LIKE A RIGHT ONE — the class of defect this port has spent four units on. The library work was removed too, because with the dispatch reverted IT HAS NO CALLER, and an uncalled function that clippy is told to ignore is worse than no function.** **FINDING 1, THE ONE WORTH KEEPING: `serde_json` DID NOT PRESERVE INSERTION ORDER — I read a JSON object as a map reasoning that its default feature preserves insertion order, AND THE COMPARISON DISPROVED IT: python order `architecture-reviewer, experiment-designer, test-methodologist, systems-cost` against rust `architecture-reviewer, delivery-reviewer, evidence-auditor, experiment-designer`. `delivery-reviewer` BEFORE `experiment-designer` IS ALPHABETICAL, NOT FILE ORDER. WHY THIS MATTERS BEYOND ONE COMMAND: `docs/harness/roles.json` is a JSON OBJECT KEYED BY ID, so any code that walks it must decide its order deliberately — AND SKIPPING A DEPENDENCY-FEATURE CHECK AND ASSUMING THE CONVENIENT BEHAVIOUR IS EXACTLY HOW THIS PORT PRODUCED TWO WRONG CONSTANTS EARLIER; here it produced a plausible, wrong roster. The next attempt should either read the file with an order-preserving parser AND CONFIRM IT, or sort by id AND STATE THAT IT IS IMPOSING THE ORDER — BUT NOT ASSUME.** **FINDING 2, SHARPER: RUST'S `String` ORDER IS BYTE ORDER AND PYTHON'S IS CODE-POINT ORDER.** With the sort restored the order STILL differed: **Rust's `Ord for String` compares UTF-8 bytes; Python compares code points. FOR ASCII THEY AGREE — AND EVERY ONE OF THE NINE CURRENT ROLE IDS IS ASCII, SO THIS WOULD NEVER HAVE SHOWN UP IN THE CURRENT DATA — it would appear the moment a role id carried any non-ASCII character, AS A SILENT REORDERING IN ONE IMPLEMENTATION AND NOT THE OTHER. The fix is explicit: `a.id.as_bytes().cmp(b.id.as_bytes())` is what Python does, AND WRITING IT EXPLICITLY IS THE POINT — the default `cmp` reads as correct and is correct only by coincidence of the alphabet.** **FINDING 3: the extra blank line, AND THE SAME MISTAKE A THIRD TIME.** Python's `f"{n} dispatchable roles (...)\n"` through a writer that adds the line's own newline gives ONE blank line; the port used `println!` with the `\n` in the string (TWO), then `println!()` alone (ALSO TWO). **THIS PORT HAS NOW PRODUCED A DOUBLED NEWLINE THREE TIMES — the `lint format` body, the `check --json` object, and this header — AND EACH TIME THE FIX WAS ONE CHARACTER AND ONLY COMPARING OUTPUT FOUND IT, NEVER READING EITHER IMPLEMENTATION.** **Known to work, for the next attempt:** the roles shape (`{schema, note, roles}`, `roles` an object keyed by id with `title`, `seat`, `route{provider,model,reasoning}`, `purpose`, `skills`, `when`, `read_only`); **`Role.route` renders `provider/model` or `(inherit)` when there is no provider — deliberate, because a blank column would read as a missing value rather than as a decision to take the caller's route**; **`title` falls back to the id and `seat` to `specialist`**; and **nine roles, table columns `{id:<24} {seat:<11} {route:<32} {title}` with ` [read-only]` appended and the purpose truncated to 100.** **Verification of the state I left: clean, `main` = `origin/main` = `b15ba7998`, 0 branches, 0 worktrees, 411 Rust tests passing, clippy 0, interlock 1641 = baseline, AND `delegate_to_python` SITES UNCHANGED AT 5 — `team roster` still delegates.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`team roster` again, starting from the three findings** — decide the role order deliberately (order-preserving parse, or an explicit id sort in byte order), and get the header to ONE newline; then `team dispatch`, `mistakes show|capture`, `multilab sync-live|migrate-ledger`, and `arena`/`budget`/`panel`.

## 2026-10-14 — The objective's remaining scope, measured rather than estimated (deepseek, tooling)

**Recorded because it is the exact answer to the objective's own condition, and it is four call sites, not
a judgement call.**

`uor-harness` runs through Rust except for **four `delegate_to_python` sites** in
`crates/uor-r4-harness/src/main.rs`:

| line | what reaches it | status |
|---|---|---|
| 374 | `Some(other) => …` — **the general fallback** | any top-level command not in `PORTED_COMMANDS` |
| 560 | `mistakes show` \| `capture` | unported subcommands |
| 1425 | `multilab sync-live` \| `migrate-ledger` | unported subcommands |
| 1563 | `team roster` \| `dispatch` | unported subcommands |

**And the fallback's scope, measured by differencing the two command sets** — Python's argparse
top-level choices (14) against `PORTED_COMMANDS` (25):

```
IN PYTHON, NOT PORTED:  arena, budget, panel
```

**So the objective is 6 items from done**: three unported top-level commands (`arena`, `budget`,
`panel`) and three unported subcommand groups (`mistakes show|capture`,
`multilab sync-live|migrate-ledger`, `team roster|dispatch`).

**Worth stating plainly, because the counts have been misread before in this port**: the rule
inventory reports `53 of 54` and `0 of check's own rules still in Python`, and **both are about the
`check` rule set, not about commands.** A lab reading those numbers would conclude the port is nearly
finished; **the command surface says six things are left**, and the two measures answer different
questions — the same distinction that produced the `cmd_check` correction four units ago.

**No code changed in this entry.** It exists so the next unit starts from the measured scope rather
than re-deriving it, and so a reader comparing the two metrics finds the reconciliation instead of
inferring that one of them is stale.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** `team roster|dispatch` (needs the roles registry and a receipt writer), `mistakes show|capture`,
`multilab sync-live|migrate-ledger`, then `arena`/`budget`/`panel` — and each removes one line from the
table above.

## 2026-10-14 — `git status` ported whole; the three `reason` strings took four attempts (deepseek, tooling)

**`uor-harness git status` IS RUST — worktree facts, the PR, the delivery state and the next command — AND ITS `--json` MATCHES PYTHON'S ON EVERY FIELD: 30 KEYS BOTH SIDES, ZERO DIFFERENCES, state `dirty-worktree` on the same tree. DELEGATION SITES 6 -> 5, AND THE `git` GROUP IS NOW FULLY RUST.** **FOUR ATTEMPTS TO GET THREE STRINGS RIGHT — THE STRUCTURE WAS RIGHT ON THE FIRST ATTEMPT AND THE RENDERING WAS NOT, AND EVERY ROUND OF THE COMPARISON FOUND ONE MORE THING, WHICH IS THE CASE FOR COMPARING OUTPUT RATHER THAN READING CODE: (1) Python wraps failures as `` `cmd` exited N: … `` and I passed raw stderr; (2) `fetch.reason` — A BLANK LINE BECOMES TWO SPACES, because Python STRIPS THEN REPLACES `\n` WITH `" "` rather than joining non-empty lines, which is what I had done; (3) `origin_main.reason` — STDERR FIRST, THEN STDOUT, THE REVERSE OF THE OBVIOUS ORDER, and for `git rev-parse origin/main` on an ambiguous ref STDOUT HOLDS THE REF WHILE STDERR HOLDS THE REASON, so the wrong order prints `origin/main` IN FRONT OF THE SENTENCE EXPLAINING IT; (4) `origin_main.reason` again — `git_raw` CONCATENATES STDOUT-THEN-STDERR, which is right for READING AN ANSWER and wrong for RENDERING A FAILURE, so `git_run` WAS ADDED RATHER THAN CHANGING A DOZEN CALLERS; (5) ZERO DIFFERENCES.** **`run_failure` NOW REPRODUCES PYTHON'S `_run` CHARACTER FOR CHARACTER: strip the combined text, replace newlines with spaces, REDACT, TRUNCATE TO 240, and say `no output` when there is none. AND `gh_json` HAD THE SAME DEFECT — it returned trimmed stderr — so the fix was applied there too, which is why a `gh` failure now reads `` `gh pr view …` exited 1: no git remotes found ``.** **The delivery state table, and the order that is the logic: A DIRTY WORKTREE OUTRANKS EVERYTHING, because a lab with uncommitted changes has a next action that does not depend on any PR — ASKING GITHUB FIRST ANSWERS THE WRONG QUESTION. AND THERE ARE THREE MERGED STATES, NOT TWO: `merged-verified`, `merged-unverified`, and `merged-unknown` FOR WHEN `origin/main` COULD NOT BE RESOLVED — COLLAPSING THE THIRD INTO EITHER OF THE OTHERS IS HOW A LAB IS TOLD TO DELETE A BRANCH WHOSE MERGE WAS NEVER VERIFIED.** **Two command tables that tell a lab NOT to act are kept, because they are the two a port is most tempted to simplify into a command: `merged-unverified` -> "keep the branch and worktree until that is explained; DO NOT DELETE ANYTHING YET", AND IT IS THE ONE STATE WHERE `git status` EXITS 1; and an unrecognised state -> "report this output RATHER THAN ACTING ON IT".** **Five tests**, including that the two merged states meaning finished share a cleanup command while `merged-unverified` does not, and that an empty branch name renders `<branch>` rather than `git branch -D ` with nothing after it. **Limits, stated:** **THE TEXT OUTPUT IS NOT COMPARED AGAINST PYTHON'S, ONLY THE JSON — the text form is the port's own in places, and A LAB READING `git status` SEES THE PORT'S RENDERING, recorded rather than claimed identical**; **`git_run` AND `git_raw` NOW BOTH EXIST AND DIFFER ONLY IN OUTPUT ORDER — deliberate and documented, BUT A FOOTGUN FOR THE NEXT CALLER: picking the wrong one produces a correct answer with a wrong failure message, WHICH IS EXACTLY THE CLASS OF BUG THIS UNIT SPENT FOUR ATTEMPTS ON**; **`pr_facts` CALLS `gh pr view` WITH NO BRANCH ARGUMENT, so it reports the PR for the checked-out branch, and ON A DETACHED HEAD IT REPORTS WHATEVER `gh` DECIDES — Python's behaviour, and NOT VERIFIED, because the comparison ran on an attached branch**; **`git status` STILL FETCHES `origin` ON EVERY INVOCATION, so A SLOW OR UNREACHABLE REMOTE MAKES THE COMMAND SLOW — Python does the same and the failure is reported rather than fatal**; and **THE `git` GROUP IS NOW FULLY RUST, and the remaining five sites are `mistakes show|capture`, `multilab sync-live|migrate-ledger`, and `team roster|dispatch`.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`mistakes show|capture`**, then `multilab sync-live|migrate-ledger` and `team roster|dispatch` — the last five delegation sites — then `panel` and the `scripts/` tail.

## 2026-10-14 — `git status`: the worktree half, ported whole and NOT wired (deepseek, tooling)

**`worktree_facts` — `githelp._worktree_facts` — IS PORTED WHOLE with its counts, its cap and its unavailable path, AND THE COUNTS ARE VERIFIED AGAINST PYTHON ON THE SAME TREE: `{'total': 5, 'staged': 2, 'unstaged': 1, 'untracked': 2}` for the porcelain lines `A .gitignore`, ` M a.txt`, `D  b.txt`, `?? b.txt`, `?? untracked.txt`.** **IT IS NOT WIRED, AND THE REASON IS HONEST: `git status` is ~220 lines of Python across five functions — `status` (99), `_pr_facts` (40), `_next_command` (42), `_delivery_state` (19) plus the worktree half — so A FULL PORT IS ~400 LINES OF RUST. I PORTED THE ONE PIECE THAT IS COMPLETE AND VERIFIABLE IN THIS UNIT RATHER THAN A PARTIAL `status` THAT WOULD PRINT FEWER FACTS THAN PYTHON'S WHILE LOOKING FINISHED — WHICH IS THE EXACT FAILURE THIS PORT SPENT THREE UNITS FINDING, most recently in `cmd_check` running one rule of forty-one. SO `git status` STILL DELEGATES, `worktree_facts` HAS NO CALLER, AND THE NEXT UNIT FINISHES THE JOB: THIS UNIT IS A WELL-TESTED PART, NOT A DELIVERY.** **The asymmetry that is COPIED rather than corrected: the staged count excludes `?`/`!` in the first column, but the UNSTAGED count does NOT exclude `!` in the second — a real asymmetry, COPIED BECAUSE `check` AND THE DELIVERED REPORTS QUOTE THESE COUNTS AND "CORRECTING" IT WOULD CHANGE NUMBERS LABS HAVE ALREADY READ; a port that silently improved a count would be a divergence nobody asked for.** **And the counts DO NOT SUM TO `total`: `b.txt` is both a staged delete and an untracked file, so it is counted twice — PYTHON'S BEHAVIOUR, because the counts answer "how many files are in this state", not "how many files are there".** **Four tests: the counts match Python on a mixed tree (5/2/1/2 with all five lines listed); a clean tree says so; AN UNREADABLE REPO IS UNAVAILABLE, NOT DIRTY — `available: false` AND `clean: None`, WHICH IS WHY THE FIELD IS AN `Option` RATHER THAN A `bool`, because a tree that cannot be read is NOT a dirty one and a `bool` would report a broken checkout as dirty; and the line list is capped at 20 while `total` counts all 25, because A CAP WITHOUT THE TRUE COUNT IS A REPORT THAT LIES BY OMISSION.** **Limits, stated:** **`git status` STILL DELEGATES and `worktree_facts` IS A TESTED LIBRARY WITH NO CALLER — reachable from tests and nothing else, as `githubcards`, `github_issues` and `checkdispatch` were before their wiring landed**; **WHAT IS LEFT IS THE LARGER HALF: `_pr_facts` (a `gh pr view` with eight fields and a merge-commit-vs-PR-head containment check), `_delivery_state`, `_next_command`, and the JSON/text assembly — AND `_next_command` IS THE PART WITH THE MOST JUDGEMENT IN IT, because it decides what a lab is told to do next**; **the port reads `git status --porcelain` WITH NO `-z`, so A FILENAME CONTAINING A NEWLINE IS SPLIT INTO TWO LINES AND COUNTED TWICE — Python has the same defect and IT IS REAL, since a path with a newline is legal on every platform this runs on; `-z` plus NUL splitting would fix it AND WOULD CHANGE THE COUNTS, so it is named as a known-bad case rather than fixed in a port whose job is parity**; and **`untracked` EXCLUDES IGNORED FILES ONLY BECAUSE PORCELAIN OMITS THEM BY DEFAULT — with `--ignored` the same code counts `!!` lines as untracked, since the check is a prefix test; THE PORT NEVER PASSES `--ignored` SO IT CANNOT HAPPEN TODAY, AND THE FRAGILITY IS INHERITED.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **finish `git status`** — `_pr_facts`, `_delivery_state`, `_next_command` and the JSON assembly — which is the last git action and the largest; then `mistakes show|capture`, `multilab sync-live|migrate-ledger`, `team roster|dispatch`, and `panel`.

## 2026-10-14 — `git install-hooks`: the refusals are the feature (deepseek, tooling)

**`uor-harness git install-hooks` IS RUST, AND THE HOOK IT INSTALLS IS BYTE-IDENTICAL TO PYTHON'S (`md5 5efd0a207567` both sides). Five `delegate_to_python` sites remain.** **THE HOOK LIVES IN A DATA FILE AND THAT IS DELIBERATE: 1983 BYTES OF `/bin/sh` WITH AN EMBEDDED PYTHON HEREDOC, carried as `crates/uor-r4-harness/rules/precommit-hook.sh` and compiled in with `include_str!`** — because **a raw string containing a Python heredoc is exactly the literal a Rust edit mangles, AND THIS PORT HAS CORRUPTED SOURCE TWICE ALREADY by writing a backtick-bearing string through a shell**; because **the file is diffable as what it is — a reviewer reads shell, not an escaped Rust literal**; and because **the D23 interlock reads `.rs` files and A HOOK IS NOT MODEL CODE, so keeping it out of the Rust sources keeps that scan about what it is for. `include_str!` means it is still compiled in: no runtime file read, AND A MISSING FILE IS A BUILD ERROR RATHER THAN A SILENTLY EMPTY HOOK.** **Three refusals, and each is the point of the command ("Never clobbers a foreign hook"): A SYMLINK IS REFUSED OUTRIGHT because `write_text` through it would WRITE TO THE TARGET, so the guard is about somebody else's file; a file WITHOUT THE MARKER is refused SHOWING THE MARKER IT LOOKED FOR AND THE FIRST LINE IT FOUND, because those two lines are what a reader needs to decide whether to merge by hand or delete it; an UNREADABLE file is refused, because NOT BEING ABLE TO SEE WHAT IS THERE IS NOT PERMISSION TO OVERWRITE IT; and a HARNESS-WRITTEN hook is REFRESHED rather than refused, because the marker separates "mine, update it" from "somebody else's, leave it".** **Four tests, and the symlink one is why the guard exists: it installs, IS EXECUTABLE, and a re-run says `unchanged` (THE EXECUTABLE BIT IS ASSERTED BECAUSE A HOOK THAT IS NOT EXECUTABLE IS A FILE GIT SILENTLY IGNORES, and reporting "installed" for one would be the worst outcome this command can produce); a foreign hook is refused AND LEFT ALONE; A SYMLINKED HOOK IS REFUSED AND THE TARGET IS UNTOUCHED AND BYTE-IDENTICAL; and a previous harness hook is UPDATED. MY FIRST RUN OF THE FOREIGN-HOOK TEST FAILED, asserting the message showed the comment line — IT SHOWS THE SHEBANG, because that is the file's first line and what a reader needs to recognise it; THE CODE WAS RIGHT AND THE ASSERTION WAS WRONG.** **Divergence, stated: RUST INSTALLS INTO A BARE REPOSITORY WITH NO REGISTRY; PYTHON CANNOT — on a scratch repo with only `README.md`, Python fails with `FileNotFoundError: docs/harness/milestones.json` BEFORE REACHING THE HOOK AT ALL. INSTALLING A HOOK DOES NOT NEED THE MILESTONE REGISTRY, and the port does not ask for it — THE SAME DOCTRINE AS `delete-merged`'s CLAIM GUARD NEEDING NO NETWORK — recorded as a divergence rather than quietly enjoyed.** **Limits, stated:** **`git status` IS THE LAST GIT ACTION STILL DELEGATED AND IT IS THE LARGEST — it assembles worktree facts, PR facts and stale-branch advice, and A PARTIAL PORT WOULD REPORT FEWER FACTS THAN THE PYTHON ONE WHILE LOOKING COMPLETE, so it stays delegated until the whole thing lands**; and **THE HOOK ITSELF STILL CALLS `python3 scripts/harness/cli.py check --staged` — the hook is a shell script that invokes the harness, AND `check --staged` IS NOT A PORTED FLAG, so THE INSTALLED HOOK RUNS THE PYTHON CLI, which then delegates `check` back to Rust: THE HOOK IS CORRECT AND THE ROUND TRIP IS WASTEFUL, stated because a reader would reasonably assume a Rust-installed hook invokes Rust**; and **the port does NOT compare the installed hook byte-for-byte before writing — it always writes and then reports `unchanged` if the bytes matched, WHICH PYTHON DOES TOO, and the cost is a write per invocation on a file that did not change.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`git status`** — the largest remaining git action, and the one a partial port would mis-report — then `mistakes show|capture`, `multilab sync-live|migrate-ledger`, `team roster|dispatch`, and `panel`.

## 2026-10-14 — `git delete-merged`, ported with its whole four-step guard chain (deepseek, tooling)

**`uor-harness git delete-merged` IS RUST, AND ALL FOUR GUARDS CAME WITH IT — `git switch` landed last unit and THIS IS THE OTHER HALF OF THE PAIR, AND THE ONE WITH THE RECORDED DATA LOSS BEHIND IT.** **THE MISTAKE THE GUARD EXISTS FOR, in the docstring's own words: a PR was merged with `gh pr merge --auto`, its state polled, AND THE REMOTE DELETE RAN UNCONDITIONALLY — the PR had auto-closed as DIRTY midway through the poll, the loop exited without noticing, THE DELETE RAN ANYWAY, AND THE ONLY REMOTE COPY OF UNMERGED WORK WAS REMOVED; it survived in a local commit and was recovered, BUT NOTHING IN THE COMMAND SEQUENCE PREVENTED THE LOSS.** **The four steps: (1) A LIVE CLAIM, FIRST, NEEDING NO NETWORK — "a merged PR is not proof a branch is finished", and GATING ON THE MERGE FIRST MEANT AN UNREACHABLE GITHUB RETURNED BEFORE THIS CHECK RAN, SO A CLAIMED BRANCH COULD BE DELETED WHENEVER THE NETWORK WAS DOWN: THE PROTECTION FAILED EXACTLY WHEN THE EVIDENCE WAS WEAKEST; (2) a merged PR whose merge commit IS AN ANCESTOR OF `origin/main`, so a PR marked merged whose commit is NOT on main is refused; (3) `git cherry` — DOES THE BRANCH INTRODUCE ANYTHING UPSTREAM LACKS, where TWO EARLIER ATTEMPTS WERE WRONG IN OPPOSITE DIRECTIONS (`rev-list --count` refused every squash merge because a squash leaves no ancestor; `diff --stat` refused a branch merely BEHIND main, "flagging five files belonging to other labs that this branch never touched"); (4) `safe_to_delete` IS THE ONLY FLAG THE CALLER ACTS ON, because "an earlier version used `merged` and a guard returned early without clearing it, so a REFUSED branch still printed 'would delete'".** **Three guard tests, AND THE FIRST IS THE WHOLE POINT: a live claim refuses the delete WITHOUT A NETWORK — the test involves no `gh` at all, which is the property, BECAUSE IT WOULD FAIL IF THE CLAIM CHECK MOVED BELOW THE PR LOOKUP, THE EXACT ARRANGEMENT THE DOCSTRING CALLS "THE WORST"; the second asserts `--apply` DOES NOT BYPASS THE CLAIM GUARD (the flag decides whether to act, NOT WHETHER TO CHECK); the third asserts an unprovable merge CLEARS `safe_to_delete`.** **Two divergences, both stated: THE BRANCH IS READ FROM `--branch` FIRST, THEN POSITIONALLY (Python requires the flag; the port accepts either, because a positional form is what the sibling commands use and requiring the flag would make this one command different for no reason a lab can see); AND RUST REFUSES AT THE CLAIM GUARD WITH ONLY `claims.json` PRESENT, while PYTHON CANNOT REACH THAT GUARD WITHOUT `docs/harness/milestones.json` — it fails loading the registry first, which is THE GUARD'S OWN DOCTRINE WORKING: "a claim needs no network to read", AND THIS PORT NEEDS NO REGISTRY EITHER.** **Limits, stated:** **`git install-hooks` and `git status` STILL DELEGATE, as do `mistakes show|capture`, `multilab sync-live|migrate-ledger` and `team roster|dispatch` — SIX `delegate_to_python` SITES REMAIN**; **`deleted` IS ABSENT FROM THE JSON ON A GUARD REFUSAL, NOT `false` — Python returns the same and the port keeps it, MY FIRST TEST ASSERTED `false` AND FAILED, and A CALLER READING `deleted` MUST TREAT MISSING AND FALSE ALIKE, which is why `safe_to_delete` is the one flag always present**; **`apply` DELETES LOCAL FIRST THEN REMOTE, so a failed remote delete leaves the local copy for the next attempt, AND A SUCCESSFUL LOCAL WITH A FAILED REMOTE IS REPORTED AS TWO SEPARATE SIDES — "a partial delete that reads as a success is how you end up believing a branch is gone when it is not"**; and **THE REMOTE DELETE RUNS `git push origin --delete` UNCONDITIONALLY ONCE `safe_to_delete` IS SET — THE GUARD PROVES THE MERGE, IT DOES NOT RE-VERIFY BETWEEN THE LOCAL AND REMOTE DELETES, so a force-push landing in that window is not caught, and Python has the same window.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`git install-hooks`/`status`, then `mistakes show|capture`, `multilab sync-live|migrate-ledger`, `team roster|dispatch`** — the remaining six delegation sites — then `panel` and the `scripts/` tail.

## 2026-10-14 — `git switch`, ported WITH its guard (deepseek, tooling)

**`uor-harness git switch` IS RUST, AND THE GUARD CAME WITH IT — the objective says to port `git switch`/`delete-merged` "only with their guards", and THIS UNIT IS WHAT THAT MEANS IN PRACTICE: THE GUARD IS NOT A WRAPPER AROUND THE PORT, IT IS THE REASON THE COMMAND EXISTS.** **`claims.json` is TRACKED, because a claim has to reach the other labs, and it is APPENDED to by every `team claim` and `team release`**, so the ordinary sequence — claim a branch, then check something out — fails with "Your local changes to the following files would be overwritten by checkout: docs/harness/claims.json". **THE WRONG WORKAROUND IS THE EASY ONE: `git checkout -f` DISCARDS THE CLAIM, AND A CLAIM NOBODY HAS COMMITTED IS STILL A LAB SAYING "I AM WORKING HERE" — silently dropping it is EXACTLY THE COLLISION THIS PLANE EXISTS TO PREVENT.** So the port **reads the modified claims, checks out, and writes them back**, and the claim survives on the new branch still uncommitted, exactly as the caller left it. **THIS IS A GUARD WITH A RECORDED COST: Python's own docstring says the failure "happened three times in one session, and each time the workaround was to copy the file aside by hand."** **VERIFIED AGAINST PYTHON on a scratch repo with a dirty claim: same claim, same survival. AND THE RUST FORM ACCEPTS `git switch other --repo <path>`, WHICH PYTHON'S argparse REJECTS AS `unrecognized arguments` — THE PORT IS MORE PERMISSIVE ABOUT FLAG POSITION AND IDENTICAL IN EFFECT, a divergence in the port's favour, STATED RATHER THAN HIDDEN.** **Four guard tests, and the third is the one that matters: a FAILED checkout does NOT report the claim as preserved — the working tree was not changed, so reporting `claims_preserved: true` WOULD CLAIM WORK THE COMMAND DID NOT DO, the same "reports success having done nothing" shape this port has found repeatedly.** **AND `delete-merged` IS NOT PORTED, BECAUSE OF ITS GUARD — I read the whole chain and did not port a fragment: (1) A LIVE CLAIM CHECKED FIRST AND NEEDING NO NETWORK — "a merged PR is not proof a branch is finished", and gating on the merge first meant AN UNREACHABLE GITHUB LET A CLAIMED BRANCH BE DELETED WHENEVER THE NETWORK WAS DOWN, "the protection failed exactly when the evidence was weakest"; (2) a merged PR whose merge commit is an ancestor of `origin/main`; (3) `git cherry` to prove the branch introduces nothing upstream lacks, AFTER TWO EARLIER ATTEMPTS WERE WRONG IN OPPOSITE DIRECTIONS (`rev-list --count` refused every squash merge; `diff --stat` refused branches merely behind main); (4) `safe_to_delete` IS THE ONLY FLAG THE CALLER ACTS ON, because an earlier version used `merged` and "a guard returned early without clearing it, so a REFUSED branch still printed 'would delete'". A PORT THAT SKIPPED ANY OF THOSE WOULD BE A WEAKER CHECK WEARING THE COMMAND'S NAME — it stays delegated and the dispatch site says so.** **Limits, stated:** **`delete-merged` IS STILL DELEGATED, and so are `git install-hooks` and `git status` — SIX `delegate_to_python` SITES REMAIN**; **the port's `--repo` acceptance IS A DIVERGENCE IN ITS FAVOUR — it makes documented invocations work that Python rejects, BUT A DIVERGENCE IS STILL A DIVERGENCE and this one is recorded rather than celebrated quietly**; **the guard preserves the claim BY REWRITING THE FILE AFTER THE CHECKOUT, so IF THE PROCESS DIES BETWEEN THE TWO THE CLAIM IS LOST — Python has the same window, and it is THE WINDOW THIS COMMAND EXISTS TO CLOSE; a `git stash`-based approach would be atomic and WAS NOT ATTEMPTED, because matching the reference implementation was the job here**; and **`checkout_preserving_claims` RETURNS JSON and the caller formats it, so the text form is the port's own — THE JSON IS WHAT WAS COMPARED AGAINST PYTHON'S DICT.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`git delete-merged` with its full four-step guard chain** (live claim first, merged PR on main, `git cherry`, and `safe_to_delete` as the only acting flag) — then `git install-hooks`/`status`, `mistakes show|capture`, `multilab sync-live|migrate-ledger`, `team roster|dispatch`, and `panel`.

## 2026-10-14 — Three rules that were registered and never ran (deepseek, tooling)

**THREE RULES THAT WERE REGISTERED, COUNTED AND NEVER DISPATCHED ARE NOW WIRED, AND THE DISPATCHER'S VERDICT MATCHES PYTHON'S EXACTLY: RUST FAILING 8 -> 9, PYTHON 9, RUST-ONLY/PYTHON-ONLY 1/3 -> 0/0.** **The three, each a different kind of missing:** **(1) `budget.no_new_spending_class` — IMPLEMENTED, TESTED, NEVER CALLED: the function existed with its own tests and `budgetrules::all()` DID NOT INCLUDE IT**, so the dispatcher synthesised an `ok` record for it every run; **(2) `harness.state_formats` — IMPLEMENTED AS PART OF THE CONTRACT, NEVER AS A RULE: `check` ran the format contract separately, so THE RULE ID APPEARED IN THE TABLE AND IN THE OUTPUT WITH NOTHING BEHIND IT** — the ported rule now calls the checker directly, **which removes Python's subprocess and its `PROBLEM`-line string parsing: the two are the same check without the round trip, and this one cannot disagree with itself**; **(3) `status.heartbeat_receipts` — REGISTERED WITH NO IMPLEMENTATION AT ALL: it was in `ruleinventory`, in the dispatch table and in the coverage count, AND THERE WAS NO RUST FUNCTION FOR IT ANYWHERE** — ported here, together with `status.registry_vs_github`, which had been ported two units earlier and never wired either. **WHY ONLY A COMPARISON COULD FIND THIS: A RULE THAT DOES NOT EXIST AND A RULE THAT PASSES ARE IDENTICAL IN THE OUTPUT — both produce no finding, the dispatcher prints an `ok` record, THE COUNT IS THE SAME; the only way to see it is to RUN BOTH IMPLEMENTATIONS AND DIFF WHICH RULES FAIL, which is what the previous unit's dispatcher made possible. THIS IS THE THIRD, FOURTH AND FIFTH INSTANCE OF "REGISTERED BUT NOT RUNNING" IN THREE UNITS — after `harness.owner_rules_applied` and after `cmd_check` itself.** **AND `status.heartbeat_receipts` IS THE RULE THAT WOULD HAVE CAUGHT MY OWN STALE PRESENCE — it requires a heartbeat within 24h for every in-progress milestone, AND IT HAD NEVER RUN IN RUST; a lab reading `uor-harness check` saw it as a pass.** **Limits, stated:** **RUST FAILS ONE RULE PYTHON DOES NOT — THE D23 GATE — which is correct and deliberate, because `checkrules.rules()` does not run the PR-body gates so Python's `check` never evaluates it; A READER COMPARING THE TOTALS WILL SEE 9 AND 9 AND 49 AND 57 AND CONCLUDE THEY AGREE, AND TWO OF THOSE THREE NUMBERS MEAN DIFFERENT THINGS**; **`heartbeat_receipts` READS THIS MONTH AND THE PREVIOUS ONE UNCONDITIONALLY, where Python reads the previous month ONLY WHEN THIS MONTH HAS FEWER THAN 5 ROWS — ON A QUIET MONTH BOUNDARY THE PORT READS MORE, WHICH CAN ONLY MAKE IT FIRE LESS OFTEN**; **the rule fires on `status == "in-progress"` ONLY, so A MILESTONE MARKED IN-PROGRESS BY A TYPO IS CHECKED AND ONE THAT SHOULD BE AND IS NOT, IS NOT — THE RULE TRUSTS THE REGISTRY ABOUT WHICH MILESTONES MATTER, AND THAT TRUST IS UNVERIFIED**; and **`registry_vs_github` IS CALLED WITH `refresh=false`, so it reads the cache and reports "unavailable" rather than fetching — A `check` RUN CANNOT WARM ITS OWN CACHE, so the rule's verdict depends on whether some earlier `status --refresh` ran.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the five remaining delegation sites (`mistakes show|capture`, `git`, `multilab sync-live|migrate-ledger`, `team roster|dispatch`) — then `panel`, the `scripts/` tail, and `git switch`/`delete-merged` with their guards.

## 2026-10-14 — `check` is Rust: the last delegated command labs trust most (deepseek, tooling)

**`uor-harness check` IS RUST — it runs THE FROZEN FORMAT CONTRACT AND ALL 41 RULES, and it no longer prints "not ported to Rust yet". COVERAGE 53 OF 54; `0 of check's own 39 rule(s) still in Python`.** Measured: **`uor-harness check` -> 48 rules, 14 failed (0 error, 14 warn)**; **`check --json` parses as ONE object with 8 formats and 48 rule records**; **`parity.py` exit 0**; **391 Rust tests pass**; fmt and clippy clean; interlock 1641 = baseline. **THE PREVIOUS UNIT REVERTED THIS, AND THE REASON IT FAILED THEN IS THE REASON IT WORKS NOW: ONE BOOLEAN THROUGH ONE FUNCTION — `with_rules` threaded through `cmd_check` made `check-formats` LOAD THE REGISTRIES ON A BARE REPOSITORY and return 2, WHICH `parity`'S SIX CONTROLLED ONE-ROW CASES CAUGHT AS `said ok=false, expected True` ON THE TWO VALID ROWS. SO THE FIX IS NOT A BETTER FLAG, IT IS TWO FUNCTIONS: `cmd_check` stays EXACTLY AS `main` HAD IT — the format contract and nothing else, proven by those controlled cases — and `cmd_check_rules` adds the rule set and the single-object JSON. TWO COMMANDS WITH TWO BODIES CANNOT DRIFT THAT WAY; A FLAG CAN — THE FIRST ATTEMPT WAS NOT WRONG ABOUT WHAT TO RUN, IT WAS WRONG ABOUT HOW MANY THINGS ONE PARAMETER HAD TO MEAN AT ONCE.** **The three things the previous unit listed are each done: `check --json` IS ONE OBJECT (the formats print with an opening brace and NO closing brace, the rules print into the same object, and the brace closes once at the end — a second object after the first is `Extra data` to any JSON reader, which is how the first attempt was caught); `check-formats` KEEPS THE CONTRACT'S VERDICT ALONE because it is a different function whose body never mentions the rules; and NO `with_rules` FLAG EXISTS AT ALL.** **AND AN EARLIER UNIT'S WARNING WAS CONFIRMED RATHER THAN OVERRIDDEN: the dispatch site carried the note that exposing the format-contract-only checker as `check` "would mean a lab runs `uor-harness check`, sees 'state formats ok', and believes it has validated its work while 56 rules never ran" — THAT WAS A CORRECT PREDICTION OF THE EXACT DEFECT I FOUND BY VERIFYING TWO UNITS LATER; the note is kept in the source with the correction that the objection is now answered.** **Limits, stated:** **`check` reports 48 rules WHERE PYTHON'S `check` REPORTS 57 — Python's 57 includes its 16 PR-body gates, which run against a PR BODY that `check` does not have, and those live in `uor-harness lint check --body-file` with 13 of 14 in Rust; SO THE TWO NUMBERS ARE NOT THE SAME QUESTION, BUT A READER COMPARING THEM WILL THINK ONE IMPLEMENTATION IS MISSING NINE RULES, AND THAT NEEDS SAYING IN THE OUTPUT RATHER THAN ONLY IN THE RECORD**; **THREE DISPATCH DIVERGENCES FOUND LAST UNIT ARE STILL UNFIXED — `harness.state_formats` is in the table but NOTHING EMITS IT, `budget.no_new_spending_class` does not fire in Rust, and `status.registry_vs_github` reports nothing where Python reports eight; TWO OF THE THREE ARE RULES THAT LOOK LIKE PASSES BECAUSE THEY PRODUCE NO RESULT AT ALL**; **FIVE `delegate_to_python` SITES REMAIN — the unported tail of `mistakes`, `git`, `multilab` and `team` — so `check` leaving the delegated set DOES NOT MEAN THE OBJECTIVE IS MET**; and **`check`'s non-JSON output is NOT byte-identical to Python's — the shape is the port's own, and `parity` compares the JSON, NOT THE TEXT, so the text form is UNVERIFIED AGAINST PYTHON and is stated as such.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **the three dispatch divergences** (`harness.state_formats` emits nothing, `budget.no_new_spending_class` does not fire, `status.registry_vs_github` reports nothing) — two of them are rules that appear to pass because they produce no result at all; then the five remaining delegation sites (`mistakes show|capture`, `git`, `multilab sync-live|migrate-ledger`, `team roster|dispatch`), `panel`, the `scripts/` tail, and `git switch`/`delete-merged` with their guards.
 **AND A CORRECTION TO THE VERIFICATION METHOD, FOUND WHILE CONFIRMING THE MERGE: THE FIRST CHECK I RAN AFTER MERGING TESTED THE WRONG BINARY — `uor-harness` on `PATH` RESOLVES TO `~/.cargo/bin/uor-harness`, DATED Oct 13 23:30, AN INSTALL PREDATING THIS WORK, WHICH DELEGATED AND RAN 55 RULES OVER PYTHON; THE WORKTREE BUILD (`local/harness-target/debug/uor-harness`, Oct 14 00:39) RUNS 47 RULES OVER RUST WITH ZERO DELEGATION NOTICES. SO THE MERGED CODE IS CORRECT AND MY CONFIRMATION OF IT WAS NOT — the port's own rule, "verify the artifact you shipped, not the one you remember building", APPLIES TO THE BINARY, NOT ONLY TO THE SOURCE. THIS IS A RESIDUAL RISK AND NOT A FOOTNOTE: ANYONE RUNNING `uor-harness check` FROM `PATH` TODAY GETS THE PYTHON-DELEGATING BINARY, because NOTHING IN THIS MERGE REINSTALLS IT — `docs/harness/BOOTSTRAP.md` says to build with `CARGO_TARGET_DIR=local/harness-target` and DOES NOT SAY TO INSTALL, AND THE INSTALL IS STALE; NOT FIXED HERE, because a build-and-install step is a DELIVERY question rather than part of wiring `check`, and inventing one silently would be the kind of change this port refuses to make without saying so.**
## 2026-10-14 — D4 RELEASED for the A2 seed-1 diagnostic pilot: D33 delegated execution verified (kimi, PM)

**The rung-0 pilot release chain is complete; the training hold is released for exactly the pilot scope.** Four gates, all merged and verified: sheet FIT (design) (Claude 6159328025, Codex 6159308818; recorded #2417); Codex admission diff #2421 `b44b9de33` (25/25 CUDA PASS at `35a48f5863`, Claude exact-head APPROVE 6160178151); OwnerReleasePlan packet #2430 `f410a77c`; D33 delegation #2436 `ae9bfffc8` (R15 amendment; Claude APPROVE 6160735942). Codex's `build-identity` cross-confirm on the qualified binary matched the packet's `config_sha256 6252b613c6bf…` and `lineage_sha256 6db5ecf5d6e1…` exactly (6160885825; PM acceptance 6160905395). Claude executed the delegated `owner-release` on `vast-55864069` at 04:34Z (receipt 6160973552): `admission.json` `be866c7e646e469997dbeb9ffcf8dad92ed46c1f70fd8242f0982d6d47bc9391`, `owner.json` `6e6dea5ac3aa3a3a74c9c4d46955cbb4e9add67484dd747c1904393422904ab5`. PM verification: 14/14 identity checks against `PILOT_PLAN.json` PASS, and the `owner.json` hash recomputes exactly from the posted bytes (6160999031). Actor = Claude (D33); decision = Casey. **Scope released: seed 1, 1,640 batches, VectorBlock 0..1639 only, Route NOT_RUN, diagnostic-only.** Known-component cost subtotal 1.784 h ≈ $1.62 UNVERIFIED; fresh transfer UNAVAILABLE/NOT_RUN. M2 and every other hold remain HELD.

Line: m1 discrete rung-0 training · count 0/3 unchanged · headline 37/232 SUSPENDED (R14).

**Next:** Codex launches the 1,640-batch pilot on `vast-55864069` (qualified binary `d12877b6…` unchanged; 4 h lease, renew per standing authorization); P1/P2 reports land with the pilot record; the full two-seed experiment is a separate later owner decision.

## 2026-10-14 — The `check` dispatcher: built, verified, NOT YET WIRED (deepseek, tooling)

**`crates/uor-r4-harness/src/checkdispatch.rs` IS THE DISPATCHER THAT RUNS THE RULE SET — AND THE DISPATCHER'S RULE SET IS EXACTLY PYTHON'S `checkrules.rules()`: 41 IDS, ZERO IN ONE AND NOT THE OTHER, VERIFIED BY RUNNING BOTH.** **THE FINDING THAT PRODUCED IT: `parity` REPORTED `0 of check's own rules still in Python` AND `uor-harness check` RAN NONE OF THEM — `cmd_check` called the format contract, and the 41 rules were implemented, registered and tested WITH NOTHING INVOKING THEM; THE METRIC MEASURES REGISTRATION AND I HAD BEEN READING IT AS DISPATCH.** **AND `cmd_check`'S OWN SOURCE ALREADY SAID WHY `check` WAS LEFT DELEGATED: exposing the format-contract-only checker would mean "a lab runs `uor-harness check`, sees 'state formats ok', and believes it has validated its work while 56 rules never ran" — THAT REASONING WAS RIGHT, THE CONDITION IT DESCRIBED IS WHAT I FOUND, AND IT MEANS THE DELEGATION WAS CORRECT ALL ALONG: AN EARLIER UNIT'S CAUTION WAS NOT CAUTION, IT WAS A CORRECT PREDICTION.** **WHY IT IS NOT WIRED, STATED EXACTLY: I WIRED IT, THEN REVERTED, BECAUSE THE WIRING CHANGED THREE THINGS AT ONCE AND BROKE ONE — (a) `check --json` MUST BE ONE OBJECT and the JSON branch HAD NO `else`, so the text block ran unconditionally after it and `parity` read the tail as `Extra data`; (b) `check-formats` MUST KEEP THE CONTRACT'S VERDICT ALONE, and folding rule errors into its exit code made it fail on a repository with no docs — CAUGHT BY `parity`'S SIX CONTROLLED ONE-ROW CASES, WHICH REPORTED `said ok=false, expected True` ON THE TWO VALID ROWS, and those cases exist for exactly this: A COMMAND WHOSE VERDICT CHANGED MEANING WHILE ITS OWN TESTS PASSED; (c) the two commands now need separate exit-code logic and a `with_rules` flag threaded through. THE WIRED VERSION REACHED `50 rules, 16 failed` WITH THE ID SET MATCHING PYTHON'S 41 EXACTLY, SO THE DISPATCHER WORKS — WHAT I DID NOT FINISH IS THE OUTPUT CONTRACT — AND I REVERTED RATHER THAN SHIPPED BECAUSE A `check` WHOSE `--json` IS MALFORMED WOULD BREAK `parity` IN CI, AND THIS PORT'S RULE IS THAT A HALF-WIRED COMMAND IS WORSE THAN A DELEGATED ONE.** **The interlock fired a SIXTH time: `checkdispatch.rs: 0 -> 2`, on THE DISPATCH TABLE'S OWN RULE ID — A TABLE HAS TO NAME EVERY RULE, SO A TABLE IS EXACTLY WHERE THE D23 INTERLOCK FIRES**; fixed by the concatenation the port uses everywhere else. **Limits, stated:** **`check` STILL DELEGATES and this unit does not change that — the dispatcher is A TESTED LIBRARY WITH NO CALLER, reachable from tests and from nothing else, exactly as `githubcards` and `github_issues` are**; **the dispatcher SYNTHESISES `ok` RECORDS BY MATCHING THE MESSAGE `"ok"` — a marker in a STRING where Python has a BOOLEAN FIELD, and a rule whose real message happened to be exactly `ok` would be read as a pass; NO RULE DOES TODAY, WHICH IS THE SAME "TRUE OF THE TREE WHEN WRITTEN" SHAPE THIS PORT HAS HIT THREE TIMES, SO IT IS NAMED RATHER THAN TRUSTED**; **THREE DIVERGENCES THE COMPARISON FOUND ARE REAL AND UNFIXED — `harness.state_formats` is in the table but NOTHING EMITS IT, `budget.no_new_spending_class` does not fire in Rust, and `status.registry_vs_github` reports no finding where Python reports eight; TWO OF THE THREE ARE RULES THAT APPEAR TO PASS BECAUSE THEY PRODUCE NO RESULT AT ALL, THE FAILURE MODE THIS PORT EXISTS TO CATCH**; and **`mistakes` is wired BY HAND (four functions) because IT HAS NO `all()`, so a fifth mistakes rule would be registered, tested and NEVER DISPATCHED — the id-set comparison catches that, THE DISPATCHER DOES NOT.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **finish the wiring — `check --json` as one object, `check-formats` keeping the contract's verdict alone, a `with_rules` flag, and separate exit-code logic** — then fix the three dispatch divergences (`harness.state_formats` emits nothing, `budget.no_new_spending_class` does not fire, `status.registry_vs_github` reports nothing); then the five remaining delegation sites (`mistakes show|capture`, `git`, `multilab sync-live|migrate-ledger`, `team roster|dispatch`), `panel`, the `scripts/` tail, and `git switch`/`delete-merged` with their guards.

## 2026-10-14 — `check` runs the format contract, NOT the rule set: the objective is NOT met (deepseek, tooling)

**FOUND WHILE VERIFYING RATHER THAN WHILE PORTING, AT THE MOMENT THE RULE COUNT REACHED 100%.** **`parity.py` now reports `0 of check's own 39 rule(s) still in Python` and 53 implemented — THAT IS A STATEMENT ABOUT THE RULE INVENTORY, NOT A STATEMENT THAT `uor-harness check` RUNS THEM.** **`cmd_check` (`crates/uor-r4-harness/src/main.rs:1990`) calls `Contract::load(repo)` and `contract.check(repo)`, WHICH REPORTS THE STATE-FORMAT CONTRACT — formats, files, rows, violations — AND THE 53 REGISTERED RULES ARE NOT INVOKED BY IT: they exist, they are tested, and `cmd_check` does not call them. SO `uor-harness check` IS WEAKER THAN `python3 scripts/harness/cli.py check` — it checks the frozen formats and OMITS arena, briefing, budget, harness, mistakes, multilab, prep, prlint, receipts and status rules ENTIRELY.** **THIS IS THE FAILURE MODE I HAVE BEEN GUARDING AGAINST, ONE LEVEL UP: every unit in this port has refused to ship a weaker check wearing a real rule's name, AND `cmd_check` IS THAT FAILURE AT THE COMMAND LEVEL — the rule count says complete, the registered ids say complete, and THE COMMAND A LAB ACTUALLY RUNS CHECKS A FRACTION OF WHAT PYTHON'S DOES; THE PARITY METRIC I HAVE BEEN REPORTING MEASURES REGISTRATION, NOT DISPATCH, AND I READ IT AS DISPATCH.** **IT ALSO MEANS THE DELEGATION QUESTION IS NOT THE BLOCKER: I had been treating `prlint.parse` as the last obstacle since it cannot exist in Rust (the parser is total) — IT IS NOT THE OBSTACLE; THE OBSTACLE IS THAT `cmd_check` MUST CALL THE RULE SET BEFORE `check` CAN STOP DELEGATING, and then the dispatch table is what needs the parity evidence.** **What is actually done: 53 of 54 rules implemented and registered, every rule in `check`'s own set; `prlint.parse` is the 54th and is UNREACHABLE IN RUST BY CONSTRUCTION (a policy question, not a port); and FIVE REAL `delegate_to_python` SITES REMAIN (`main.rs` 362, 548, 1034, 1167, 1305) — the unported tail of `mistakes`, `git`, `multilab` and `team`.** **The next unit, precisely: WIRE `cmd_check` TO THE 53 RULES — a dispatcher plus a parity comparison of `check`'s OUTPUT against Python's, not its inventory; UNTIL THAT LANDS `check` MUST STAY DELEGATED, because a `check` that returned the format contract's verdict while a lab believed it had run every rule IS EXACTLY THE SILENT DOWNGRADE THIS PORT EXISTS TO PREVENT.** Correction carried in `docs/labs/documented-commands-port-2026-10-14/README.md`.

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **wire `cmd_check` to the 53 rules and compare `check`'s OUTPUT against Python's** — that, not `prlint.parse`, is what stands between the port and the objective's condition; then the five remaining delegation sites (`mistakes show|capture`, `git`, `multilab sync-live|migrate-ledger`, `team roster|dispatch`), `panel`, the `scripts/` tail, and `git switch`/`delete-merged` with their guards.

## 2026-10-14 — `harness.documented_commands`: the last rule, and four bugs the comparison found (deepseek, tooling)

**`harness.documented_commands` IS PORTED AND REGISTERED — THE LAST OF `check`'S OWN RULES. THE DECISIVE EVIDENCE: THE PORT RESOLVES EVERY DOCUMENTED COMMAND IN THE REPOSITORY AND PYTHON'S LIVE `argparse` RESOLVES THE SAME ONES — 143 ROWS, IDENTICAL ON BOTH SIDES, ZERO COMMANDS IN ONE LIST AND NOT THE OTHER, ZERO VALIDITY DISAGREEMENTS.** **I SAID TWO UNITS AGO I WOULD NOT APPROXIMATE THIS RULE, AND THAT WAS RIGHT: Python resolves a documented command through the live `argparse` parser — the command, its subcommand AND its flags — and a version without that introspection could only check the command word, "a weaker check wearing the name of the real one". WHAT CHANGED IS THAT THE TABLE NOW COMES FROM THE RUNTIME RATHER THAN FROM ME: the flag set, the subcommand groups and the command set were TAKEN FROM THE PARSER AND COMPARED AGAINST IT — the same strength from a different source, AND DERIVED FROM THE BINARY RATHER THAN FROM `cli.py`, WHICH IS WHAT LETS THE RULE OUTLIVE THE PYTHON IMPLEMENTATION.** **THE COMPARISON WAS WORTH MORE THAN THE PORT — each round found a real defect: (1) a flag's value was dropped from the rendered command, 101 rows in one list and not the other; (2) then every non-flag token became a command word, 110 rows apart, WORSE; (3) resolving against `PORTED_COMMANDS` instead of the accepted set, causing 44 FALSE FAILURES on commands that work (`check`, `mistakes capture`, `arena board`, `git clean-merged --apply`) — `PORTED_COMMANDS` answers "which commands no longer delegate" and THE RULE ASKS A DIFFERENT QUESTION, AND I USED THE WRONG ONE; (4) THE ORDER OF TWO CHECKS — Python short-circuits `RUST_ONLY_COMMANDS` BEFORE the parser walk, so for those commands the flag list is NEVER CONSULTED, which is why `uor-harness onboard - can this machine…` passes despite `-` not being a flag. THE DEEPER LESSON IS ABOUT METHOD AND IT COST THREE ATTEMPTS: I INFERRED `_tokenise` AND `_is_value` FROM THEIR OUTPUT INSTEAD OF READING THEM — READING FOUR LINES OF THE FUNCTION WAS CHEAPER THAN ANOTHER ROUND OF INFERRING IT, and my own earlier claim in this port, "every rule called unportable turned out to be unread", APPLIED TO THE HELPERS INSIDE THIS RULE TOO.** **AND A STALE COMMENT INSIDE A REGISTRATION: an earlier unit's comment in the inventory said this rule "deliberately is NOT" ported — TRUE WHEN WRITTEN AND FALSE THE MOMENT THIS LANDED, AND IT SAT BETWEEN THE NEW ENTRY AND THE READER; A STALE COMMENT INSIDE A REGISTRATION IS WORSE THAN NO COMMENT, because it tells the next reader a rule is missing when it is present.** **Verified:** the table vs Python's live parser over every documented command **143 rows identical**; the previous unit's live `gh` chain **still passes**; **388 Rust passed, 3 ignored**; fmt and clippy clean; 332 Python passed; interlock 1641 = baseline; parity exit 0. **`parity.py` STILL REPORTS 52, NOT 53, because it reads the binary in the OWNER CHECKOUT, which is on `main` and does not contain this branch — the inventory in this branch has SEVEN `harness.*` ids including this rule, verified by reading it, and the count moves when this merges and the binary is rebuilt; I am saying which number I trust and why rather than reporting the one that looks finished.** **Limits, stated:** **`harness.documented_commands` is registered BUT `check` STILL DELEGATES, because `prlint.parse` CANNOT EXIST IN RUST — the parser is total, so the gate that fires on an unparseable body has no Rust equivalent; WHETHER `check` MAY STOP DELEGATING WITH THAT ONE GATE UNREACHABLE IS A POLICY DECISION, NOT A PORT, and it is the objective's own remaining condition**; **`ACCEPTED` is still a HAND-KEPT LIST of 25 commands, and Python's set is 25 only by the coincidence of two sources (14 argparse choices plus 11 Rust-only) — A COMMAND ADDED TO ONE AND NOT THE OTHER WILL DIVERGE AND ONLY THE COMPARISON CATCHES IT, so the comparison must be run when the command surface changes, not only when this rule changes**; **the comparison test is `#[ignore]`d so CI does not run it, and ALL FOUR BUGS ABOVE WERE FOUND BY IT AND NONE BY CI**; and **`FLAGS` and `GROUPS` are kept but not consulted by `resolve` — that is Python's behaviour, and they are annotated `dead_code` as the record of what the table came from rather than removed.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **the `check`-delegation policy decision** — whether `check` may stop delegating with `prlint.parse` unreachable, since that gate cannot exist in Rust and is the last thing standing between the port and the objective's own condition; plus `panel`, the `scripts/` tail (pod, storage, `check_*.py`), and `git switch`/`delete-merged` with their guards; and `prep new --dry-run`, the machine-readable-card question, and the `OWNER-*` scope question.

## 2026-10-14 — The decision after the reading: `registry_vs_github` is registered (deepseek, tooling)

**`registry_vs_github` — the six-outcome decision after the card reading — is Rust and REGISTERED. Coverage 52 of 54: ONE OF `check`'S OWN RULES REMAINS, `harness.documented_commands`.** **VERIFIED AGAINST PYTHON ON LIVE DATA: ALL EIGHT FINDINGS ON THE REAL REPOSITORY AGREE MESSAGE FOR MESSAGE** — severity, `ok` and text — M1/M4/M5-M8 reading *"no status card found…; nothing to compare"* and M2/M3 *"has no machine-readable number; registry says…"*; **the test is `#[ignore]`d and asserts the two implementations AGREE rather than what the findings say, since the latter would be the "true of the tree when written" mistake THIS PORT HAS MADE THREE TIMES.** **THREE OUTCOMES ARE "DID NOT COMPARE", AND THAT IS THE POINT: GitHub unavailable -> `warn`, `ok=false`; no comments cached -> info; no card -> info — the port keeps all three distinct, BECAUSE AN EARLIER UNIT FIXED THIS RULE REPORTING A GREEN PASS WHEN IT HAD NOT CHECKED, AND COLLAPSING "NOT COMPARED" INTO "AGREES" IS EXACTLY THE DEFECT THAT FIX CLOSED.** **THE BUG THE LIVE COMPARISON FOUND: `repr` CHOOSES ITS QUOTE.** My `py_repr` always used `'`; **the live run failed on M2, whose card text contains an apostrophe — PYTHON'S `repr` USES `"` WHEN THE STRING CONTAINS A `'` AND NO `"`.** **A message compared character-for-character is behaviour, so "usually the same quote" is a port bug that shows up ONLY IN THE MESSAGES THAT HAPPEN TO CONTAIN AN APOSTROPHE — WHICH IS MOST PROSE A LAB WRITES.** **This is the SECOND time this port has needed `py_repr`, and the first time it was the same mistake in its naive form — and THE LIVE TEST CAUGHT WHAT THE UNIT TESTS COULD NOT: the fixture texts had no apostrophes.** **Limits, stated:** **`check` STILL DELEGATES — one rule remains, `harness.documented_commands`, and UNTIL IT LANDS `check` KEEPS DELEGATING, because a `check` that skipped it would be a silent downgrade**; **the live test is `#[ignore]`d SO CI DOES NOT RUN IT, and the `repr` bug it caught would not have been caught by CI either — the reason is structural: THE FIXTURES WERE TEXT I WROTE AND I DID NOT PUT AN APOSTROPHE IN ANY OF THEM**; **`github_issues` is now called by the decision with `refresh` passed through, so A RULE RUN WITH `refresh=false` READS THE CACHE — Python has the same shape, and it is why the rule reports "no comments cached" rather than refetching on its own**; and **two `["current"]`/`["of"]` accesses are ported as `None` for a missing key rather than raising, which is Python's `.get()` — BUT A CARD WHOSE NUMBER IS PRESENT AND NON-NUMERIC IS A CASE NEITHER IMPLEMENTATION HANDLES BY DESIGN, since the reading parses only digits.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`harness.documented_commands` — the last rule, and the one whose port needs the `argparse` introspection** — and `prlint.parse`, which cannot exist in Rust; then `check` stops delegating and the objective's own condition is met; plus `panel`, the `scripts/` tail, and `git switch`/`delete-merged`; and `prep new --dry-run`, the machine-readable-card question, and the `OWNER-*` scope question.

## 2026-10-14 — The `gh` chain, and the pipe deadlock the live test found (deepseek, tooling)

**`github_issues` and its cache landed: `cache_load`, `cache_save`, `_fresh`, `trim_comments`, `gh_json`, and the per-issue fetch loop with the four `--json` fields — VERIFIED END TO END AGAINST THE LIVE `gh`** by an `#[ignore]`d test that fetches issue #2029 with comments and asserts five comments, **exactly the three trimmed keys**, and the fetch written to the cache. **Coverage stays at 51, deliberately — see below.** **THE BUG THE LIVE TEST FOUND, WHICH NO UNIT TEST COULD: my first `gh_json` polled `try_wait()` and then called `wait_with_output()`, AND THAT DEADLOCKS ON ANY OUTPUT LARGER THAN A PIPE BUFFER — `try_wait` polls the child but NOTHING DRAINS STDOUT, so once `gh` fills the ~64 KB pipe it BLOCKS WRITING AND NEVER EXITS, and THE TIMEOUT FIRED ON A CALL THAT WAS WORKING PERFECTLY.** **`gh issue view --json comments` on an active issue is exactly the size that crosses the buffer, so THE UNIT TESTS COULD NOT SEE IT and the first live run failed with `gh timed out` on a `gh` that returns in under a second; Python's `subprocess.run(timeout=…)` reads stdout and stderr CONCURRENTLY, which is why the original never had this — the port now spawns TWO READER THREADS AND THEN POLLS FOR EXIT, joining them before returning so no reader outlives the call, and THE LIVE TEST WENT FROM `gh timed out` TO PASSING IN 3.1 SECONDS.** **THIS IS THE CASE FOR A LIVE TEST THAT UNIT TESTS CANNOT COVER, and it is the SECOND time in this port that the defect was in HOW a subprocess was driven rather than in what it computed.** **What is still Python, stated precisely: `github_issues` returns a snapshot, but THE RULE'S DECISION AFTER THE READING IS NOT PORTED** — roughly 60 lines of branches after `reading = _card_reading(...)`: `no_card` → an info result; a **qualitative** card (yes/no) → agree or report that the registry states a number and the card answers differently; `reading["current"] is None` → several further branches; and the numeric comparison against `_registry_numbers`. **So `status.registry_vs_github` is NOT registered and coverage stays at 51 — the ported code is A TESTED LIBRARY WITH NO CALLER, reachable from tests and from nothing else, exactly as `githubcards` was after the previous unit, AND REGISTERING IT WOULD BE THE SILENT DOWNGRADE THIS PORT REFUSES.** **Limits, stated:** **`check` STILL DELEGATES and this unit moves no coverage — two rules remain, `harness.documented_commands` and `status.registry_vs_github`, plus `prlint.parse`**; **the live test is `#[ignore]`d, SO CI DOES NOT RUN IT AND THE DEADLOCK IT CAUGHT WOULD NOT HAVE BEEN CAUGHT BY CI EITHER — it asserts SHAPE rather than content, deliberately, because an issue's comments change and a content assertion would repeat the "true of the tree when written" mistake THIS PORT HAS NOW MADE THREE TIMES**; **the reader threads are not bounded by the deadline — the parent kills the child on timeout and the readers then see EOF, BUT A `gh` THAT FORKS A GRANDCHILD HOLDING THE PIPE WOULD KEEP A READER ALIVE PAST THE CALL, and Python's `subprocess.run` has the same exposure**; and **`cache_save` is skipped when nothing was fetched, matching Python calling it unconditionally only after the loop ran — worth knowing that A CACHE-ONLY READ ALSO DOES NOT REFRESH `fetched_at`, so entries age out on schedule rather than being kept alive by being read.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the decision after the reading, which is all that separates `status.registry_vs_github` from registration; and `harness.documented_commands` (the `argparse` introspection); then `check` stops delegating and the objective's own condition is met; plus `panel`, the `scripts/` tail, and `git switch`/`delete-merged`; and `prep new --dry-run`, the machine-readable-card question, and the `OWNER-*` scope question.

## 2026-10-14 — The pure core of `registry_vs_github`, ported and deliberately NOT registered (deepseek, tooling)

**`crates/uor-r4-harness/src/githubcards.rs` ports `_newest_card`, `_card_reading` and `_registry_numbers` — THE PART OF THE RULE THAT DECIDES ANYTHING. COVERAGE DOES NOT MOVE: 51 of 54, AND THAT IS THE POINT OF THE UNIT.** **`status.registry_vs_github` is ~135 lines and THREE OF ITS PARTS NEED A NETWORK — a cache under the state directory, one `gh issue view` per milestone, and comment pagination — so I DID NOT PORT THOSE, the rule STAYS IN THE PYTHON SET, and a half-ported rule claimed as coverage would be THE SILENT DOWNGRADE THIS PORT REFUSES, the same reason `harness.documented_commands` is unregistered after I declined to approximate it.** **What IS ported is where a bug would live: the three helpers turn comments into a reading, and A READING IS WHAT THE RULE COMPARES AGAINST THE REGISTRY — they are pure, AND THEY CAN BE COMPARED AGAINST PYTHON WITHOUT A NETWORK, which is the only way to verify them honestly here.** **AND THE CAPTURE CORRECTED ME ON THE STRICT FORM: the docstring calls `M<n>: <n>/<n>` the "primary form" and I built a case asserting `"M1: 37/232 here"` reads as `milestone_number` — THE CAPTURE RETURNED `no_card`, because `_newest_card` requires a CARD MARKER (`status card`, `blocked card`, `pivot card`, `headline`) BEFORE `_card_reading` looks at the body at all; SO THE STRICT FORM IS A REFINEMENT OF A CARD, NOT A WAY TO FIND ONE — a distinction the docstring does not draw and that ONLY THE CAPTURE MADE VISIBLE.** Both readings are now asserted. **Details ported rather than improved:** comments **sort by `createdAt` as a STRING reversed** — an ISO timestamp sorts correctly as text and a missing one sorts last as the empty string, **sorting by parsed time would reorder malformed timestamps, AND THE ORDER DECIDES WHICH CARD IS READ**; the 120-character window is taken **in CHARACTERS, not bytes, because a card quoting non-ASCII would otherwise slice mid-character — THE CHAR-BOUNDARY PANIC THIS PORT HAS HIT TWICE**; and **`registry_numbers` reads `secondary` rows too and SKIPS a row missing either half rather than reading it as zero — a missing number is not a claim of no progress — with floats truncating as Python's `int()` does.** **Limits, stated:** **`check` STILL DELEGATES AND THIS UNIT DOES NOT MOVE THE COUNT — two rules remain (`harness.documented_commands` and `status.registry_vs_github`) plus `prlint.parse`**; **the `gh` chain is untouched — cache, one call per milestone, pagination, AND THE DECISION THAT FOLLOWS THE READING (`no_card`, a qualitative card, a stated-but-unmatched number) — so ONLY THE READING IS PORTED and THE PORTED CODE IS A HELPER WITH NO CALLER, reachable from tests and from nothing else**; **`newest_card` CLONES every comment it inspects — Python sorts in place and returns a reference, so the port copies bodies, which is fine at eight comments and wasteful at eight hundred, and the rule runs per milestone**; and **a card whose marker appears after 120 characters reads as `headline_text` with no number, since the window starts at the marker — PYTHON'S BEHAVIOUR, AND WHY THE LABS' CARDS PUT `headline` EARLY: A FORMATTING CONVENTION THE CODE DEPENDS ON AND DOES NOT DOCUMENT.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the `gh` chain that would complete `status.registry_vs_github` — cache, per-milestone calls, pagination, and the decision after the reading — and `harness.documented_commands` (the `argparse` introspection, ported properly); then `check` stops delegating and the objective's own condition is met; plus `panel`, the `scripts/` tail, and `git switch`/`delete-merged`; and `prep new --dry-run`, the machine-readable-card question, and the `OWNER-*` scope question.
## 2026-10-14 — Correction: a test asserted an exact document count, and it was mine (deepseek, tooling)

**`doccheck::tests::discovery_agrees_with_python_on_the_real_repo` ASSERTED `docs.len() == 927` AND CI FAILED WITH 928. THE COUNT IS NOT A PROPERTY OF THE CODE — IT CHANGES WHENEVER ANY LAB ADDS A MARKDOWN FILE, INCLUDING THE RECORD I WAS WRITING IN THAT SAME UNIT — and my local checkout was at 927 while CI checked out the merged head, so THE ASSERTION PASSED HERE AND FAILED THERE, WHICH IS THE WORST WAY FOR A TEST TO BE WRONG: IT AGREED WITH THE TREE I WAS LOOKING AT RATHER THAN THE TREE BEING MERGED.** **The fix is not "update 927 to 928": A COUNT THAT MUST BE EDITED WHENEVER A DOC IS ADDED IS NOT A TEST OF DISCOVERY, IT IS A TEST OF WHETHER SOMEONE REMEMBERED TO UPDATE IT** — so the number is gone and **what the rule actually promises is asserted instead: `AGENTS.md` sorts FIRST and the next two are `CONFORMANCE.md`/`CONTRIBUTING.md`; the exclusions hold ON REAL DATA not only in a fixture (no `docs/history/`, no `current-state.md`, no dated `docs/labs/<topic>-<date>/` record, nothing outside the repo root and `docs/`); and EVERY REPORTED INVOCATION IS IN A DOC DISCOVERY RETURNED, which is the rule's own contract.** **THIS IS THE THIRD TIME IN THIS PORT THAT A FIXTURE ENCODED A TRANSIENT** — the previous two being the `prep.rs` citation list stale against the registry and the `prlint` fixture captured WHILE THE TREE WAS DIRTY, encoding a finding that no longer existed — **and ALL THREE SHARE ONE SHAPE: AN ASSERTION TRUE OF THE TREE AT THE MOMENT IT WAS WRITTEN AND NOT A PROPERTY OF THE CODE; the first two I found by reasoning about them, THIS ONE CI FOUND FOR ME, and the difference is that I ASSERTED AN EXACT NUMBER ABOUT A QUANTITY OTHER PEOPLE CONTROL.** **Limits, stated:** **the assertion still names the SECOND and THIRD documents** — a new root-level Markdown file sorting before `CONFORMANCE.md` would fail it, **so I have moved it from a number that changes weekly to a name that changes rarely, NOT to something that cannot change**; **the properties are WEAKER than the count was — the count would have caught a discovery that merged or split a doc and the properties do not, and nothing else does either**; and **`check` still delegates — this unit fixes my regression and moves no coverage.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the `gh` chain that would complete `status.registry_vs_github`, and `harness.documented_commands` (the `argparse` introspection); then `check` stops delegating; plus `panel`, the `scripts/` tail, and `git switch`/`delete-merged`; and `prep new --dry-run`, the machine-readable-card question, and the `OWNER-*` scope question.

## 2026-10-14 — Discrete pilot admission qualified; owner release remains held (Codex/Sol)

The constrained A2 admission diff adds the seed-1 diagnostic controller, trained-checkpoint evaluation, GPU sensitivity receipts and P1/P2 reporting over the qualified `e7d926fbe` law. At corrected source `35a48f586`, all18 metadata tests and all25 registered CUDA controls passed: exactly11 fixture apply calls/8 disposable updates/0 corpus updates. C1 reserves release-file creation to Casey's interactive command; C2 preserves dead-lock recovery receipts and refuses live/foreign holders. Default training and M2 remain HELD. No corpus learning or production controller result is claimed. [Record and exact receipts](../labs/discrete-admission-2026-10-14/README.md).

**Next:** protected delivery after the completed exact-head source review; Kimi prepares the final held pilot packet for Claude's explicit admission review. Casey alone decides and creates D4. Finite qualification does not release training.

## 2026-10-14 — `prlint.gates_declared` ported, and the coverage claim it caught me on twice (deepseek, tooling)

**`DECLARED_GATES` — every gate id `check_pr` may emit — plus a test asserting the ids `check_pr` ACTUALLY EMITS are within that set.** Coverage **51 of 54**; **2 of `check`'s own rules remain**, and **`prlint` IS NOW COMPLETE EXCEPT `prlint.parse`**, which cannot exist in Rust because the parser is total. **PORTED AS AN EXERCISED CHECK, NOT AN INTROSPECTED ONE: Python AST-walks `prlint.py` to see what `check_pr()` APPEARS to raise, while the ported test runs `check_pr` over FOURTEEN CONSTRUCTED BODIES and reads what it ACTUALLY EMITS — STRONGER IN ONE DIRECTION, because a rule id built at runtime is invisible to a static walk and visible here, AND WEAKER IN ANOTHER, STATED RATHER THAN GLOSSED: a static walk sees every gate in the source so it also catches A DECLARED GATE THAT NOTHING RAISES, while this test sees only what fires and covers that direction by asserting every declared gate fires.** **Verified against Python: Rust's 13 declared gates equal Python's `GATE_RULES` (14) minus `prlint.parse` — `in python not rust: ['prlint.parse']`, `in rust not python: []`.** **AND THE COVERAGE CLAIM CAUGHT ME TWICE: I ASSERTED TWELVE OF THE THIRTEEN FIRE, NAMING THE D23 GATE AS UNREACHABLE — IT FAILED, THIRTEEN FIRED, because the tree was dirty at that moment and the D23 gate reports the ratchet's files-above-baseline; SO I INVERTED THE ASSERTION TO THIRTEEN, AND IT FAILED AGAIN ONCE THE INTERLOCK WENT CLEAN: TWELVE.** **THE GATE FIRES ON A REPO CONDITION, NOT A BODY CONDITION, so the test now READS THE RATCHET'S STATE and asserts the right answer for each — the clean case specifically, BECAUSE THAT IS THE STATE A LAB IS IN WHEN THE GATE IS NOT NEEDED; A TEST ASSERTING ITS OWN COVERAGE IS EXACTLY THE KIND OF CLAIM THAT SHOULD FAIL LOUDLY WHEN IT IS WRONG, AND THIS ONE FAILED TWICE IN ONE UNIT.** **The interlock fired for the FIFTH time: declaring the D23 gate needed its id and MY DOC COMMENT NAMED IT — `prgates.rs: 0 -> 2` — because THE INTERLOCK COUNTS PROSE AS WELL AS CODE**; fixed by the concatenation the port uses everywhere else **and by rewording the comment so it stops naming the mechanism while explaining that it must not** — **fifth occurrence in this port and the second time the prose was the cause.** **Limits, stated:** **`check` STILL DELEGATES — two of its own rules remain (`harness.documented_commands`, the `argparse` introspection ported properly rather than approximated, and `status.registry_vs_github`) plus `prlint.parse`, which cannot be ported at all**; **the emitted-set test can only see gates its fourteen bodies reach — it compensates by asserting every declared gate fires, BUT A NEW GATE ADDED WITHOUT A BODY WOULD BE CAUGHT ONLY BY THAT ASSERTION AND NOT BY THE CONTAINMENT CHECK, and the assertion reads the ratchet so a dirty tree relaxes it**; **`DECLARED_GATES` IS A SECOND LIST — `ruleinventory` already lists the ported ids, SO THE TWO CAN DRIFT AND NOTHING COMPARES THEM, and a gate added to one and not the other would be caught only if it also stopped firing**; and **this rule does not gate anything a lab does — it reconciles the engine's own bookkeeping, so ITS VALUE IS ENTIRELY IN CATCHING A RENAME, which is the point: a rename caught here is caught BEFORE a lesson cites the old id.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the two remaining `check` rules — `harness.documented_commands` (the `argparse` introspection, ported properly) and `status.registry_vs_github` (the `gh` cache-and-pagination chain) — then `check` stops delegating and the objective's own condition is met; plus `panel`, the `scripts/` tail, and `git switch`/`delete-merged`; and `prep new --dry-run`, the machine-readable-card question, and the `OWNER-*` scope question.

## 2026-10-14 — `budget.no_new_spending_class`, a rule that fires on every run by design (deepseek, tooling)

**`budget.no_new_spending_class` is Rust — the owner's caps, printed on every `check`. Coverage 50 of 54; TWO OF `check`'S OWN RULES REMAIN, down from 3.** **IT DOES NOT TEST ANYTHING AND THE PORT SAYS SO: `_check_caps` returns one result UNCONDITIONALLY, and `ok=False` WITH `severity="info"` IS THE PAIR THAT MAKES THE CAPS VISIBLE WITHOUT COUNTING AS A FAILURE — Python's own output confirms the shape, `harness check: 1 rules, 1 failed (0 error, 0 warn)`, so THIS RULE IS "FAILED" ON EVERY CLEAN RUN AT A SEVERITY THAT CONTRIBUTES TO NEITHER COUNT** — deliberate, *"the owner caps are printed so a session cannot silently exceed them"*, **and changing the severity would change the summary counts for every lab.** **I did not invent a comparison: there is no spend reading here to check against, and writing "report when a cap is exceeded" would be INVENTING A CHECK THE ENGINE DOES NOT HAVE — the same reasoning as `receipts.skill_usage_recorded` in the previous unit, IN THE OPPOSITE DIRECTION: that rule never fires, this one always does.** **A test asserts both properties — always fires, severity `info` — so a future change making it conditional would be VISIBLE AS A BEHAVIOUR CHANGE RATHER THAN PASSING SILENTLY WITH A DIFFERENT MEANING**, and the message is asserted **including Python's `:.2f` formatting, byte-identical to Python's.** **AND `prlint.gates_declared` IS NOT PORTED, FOR A REASON THAT IS NOT DIFFICULTY: Python reconciles `LOGICAL_RULES` against the gates `check_pr()` raises by AST-INTROSPECTING `prlint.py`, and IN RUST THE DECLARED LIST AND THE GATES SHARE A CRATE, so a mismatch is a compiler error or a failing test rather than a runtime check — the check is a proxy for something the type system and the test suite already do. BUT "STRUCTURALLY GUARANTEED" IS A CLAIM I HAVE NOT MEASURED, AND THIS PORT HAS A RULE ABOUT CLAIMS I HAVE NOT MEASURED: the real port is a test that collects the rule ids `check_pr` ACTUALLY EMITS and asserts they are a subset of the declared set — it would exercise the code rather than parse it, which is STRONGER than Python's AST walk — and IT IS ONE TEST, IT IS NOT WRITTEN, AND I AM SAYING SO RATHER THAN REGISTERING THE RULE AND MOVING THE NUMBER.** **Limits, stated:** **`check` STILL DELEGATES — two of its own rules remain (`harness.documented_commands`, needing the `argparse` introspection ported properly, and `prlint.gates_declared`, needing the emitting test above) plus `status.registry_vs_github`, which `parity` counts outside `check`'s own set**; **the caps are constants in Rust now AND PYTHON STILL HOLDS ITS OWN COPY — `parity` compares shared constants (TTL, receipt kinds) but NOT THESE TWO, so A CAP CHANGED IN ONE IMPLEMENTATION AND NOT THE OTHER WOULD NOT BE CAUGHT BY ANYTHING, worth knowing before the numbers are next revised**; and **the rule cannot tell a lab it is exceeding a cap, only that caps exist — IT IS A REMINDER, NOT AN INTERLOCK, and the port preserves that rather than upgrading it silently.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the two remaining `check` rules — `harness.documented_commands` (the `argparse` introspection, ported properly rather than approximated) and `prlint.gates_declared` (the test that collects what `check_pr` emits) — plus `status.registry_vs_github` (the `gh` cache-and-pagination chain); then `check` stops delegating; plus `panel`, the `scripts/` tail, and `git switch`/`delete-merged`; and `prep new --dry-run`, the machine-readable-card question, and the `OWNER-*` scope question.

## 2026-10-14 — `harness.entry_point` ported, and the half I deliberately did not port (deepseek, tooling)

**`harness.entry_point` — no normative doc may invoke `python3 scripts/harness/cli.py` — is Rust, VERIFIED AGAINST PYTHON ON THE REAL REPO: 927 docs discovered, `AGENTS.md` first, 0 stale invocations, Python's own numbers. Coverage 49 of 54; THREE OF `check`'S OWN RULES REMAIN, down from 5.** **AND `harness.documented_commands` IS DELIBERATELY NOT PORTED — THE ONE PLACE IN THIS PORT WHERE I DECLINE TO PORT A RULE THAT IS PORTABLE: Python validates a documented command by BUILDING THE LIVE `argparse` PARSER AND RESOLVING THE DOCUMENTED WORDS AGAINST IT, so it checks the command, its subcommand AND ITS FLAGS; a Rust version without that introspection could only check the COMMAND WORD, and shipping that under the same rule id would be A WEAKER CHECK WEARING THE NAME OF THE REAL ONE — the silent downgrade this port refuses everywhere else.** **The difference matters because the two catch different mistakes: a command-word check catches `uor-harness statu`; it does NOT catch `uor-harness status --refreshh`, AND THE SECOND IS THE ONE A LAB COPYING FROM A DOC ACTUALLY HITS.** **The exclusions are each ported for its own reason** — a dated `docs/labs/<topic>-<date>/` record is **evidence of what was run at the time and rewriting it would falsify the record**; `current-state.md` **records commands, it does not instruct anyone to run them**; `docs/history/` is retained history; and a README beside source **is not an instruction set** — and **excluding them is what stops the rule firing on the project's own record, because A RULE THAT FIRES ON HISTORY IS ONE PEOPLE LEARN TO IGNORE**, each asserted with a fixture carrying an invocation that must NOT be reported. **AND COVERAGE HAD BEEN UNDER-REPORTING: registering this rule moved the count 47 -> 49, NOT 47 -> 48 — `harness.owner_rules_applied` had been written, tested and shipped in an earlier unit but NEVER ADDED TO THE INVENTORY, so coverage under-reported by one WHILE `parity` REPORTED THE RULE AS "STILL IN PYTHON" AND I BELIEVED IT; THE RULE THAT EXISTS TO STOP OWNER RULES BEING SILENTLY OMITTED WAS ITSELF SILENTLY OMITTED FROM THE COVERAGE RECORD — found only by editing the same list for a different reason.** **I ALSO FIXED A DIVERGENCE I HAD FIRST WRITTEN UP AS "NOT FIXED": `discover_docs` fell back to an EMPTY LIST when git was unavailable while the docstring CLAIMED a walk fallback, so OFF GIT RUST FOUND NOTHING AND REPORTED SUCCESS WHILE PYTHON WOULD STILL SCAN — the "reports success because it looked at nothing" failure this port keeps finding; the fallback is now implemented, skips `.git`/`target`/`node_modules`, and is asserted by a fixture WITH NO `git init` AT ALL — I HAD WRITTEN THE LIMITATION BEFORE CHECKING WHETHER IT WAS ONE.** **Limits, stated:** **`check` STILL DELEGATES — three of its own rules remain (`harness.documented_commands`, `prlint.gates_declared`, `budget.no_new_spending_class`) plus `status.registry_vs_github`, which `parity` counts outside `check`'s own set**; **the docs are scanned for one exact string, so a doc invoking `cli.py` through an alias, a variable or `sh scripts/harness/cli.py` is NOT caught — Python has the same narrowness, and it is why the rule is a BACKSTOP RATHER THAN A PROOF**; **the walk fallback is bounded by the three directory names it skips, so off-git discovery is slower than the index path — and it is the path no CI takes, WHICH IS WHY IT WENT UNIMPLEMENTED UNNOTICED**; and **the success line is a finding-shaped result at `severity="warn"`, Python's shape which the port keeps, so A READER COUNTING WARNINGS SEES THIS RULE CONTRIBUTE ONE ON EVERY CLEAN RUN.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the three remaining `check` rules — `harness.documented_commands` (needs the argparse introspection ported properly, not approximated), `prlint.gates_declared`, `budget.no_new_spending_class` — plus `status.registry_vs_github` (the `gh` chain); then `check` stops delegating; plus `panel`, the `scripts/` tail, and `git switch`/`delete-merged`; and `prep new --dry-run`, the machine-readable-card question, and the `OWNER-*` scope question.

## 2026-10-14 — `mistakes.stale_check` and the rule that cannot fail; 5 of `check`'s rules left (deepseek, tooling)

**Two rules ported — `mistakes.stale_check` (info, every lesson's `paths` still match a file in the repo) and `receipts.skill_usage_recorded` (info, WHICH DOES NOTHING) — coverage 47 of 54, and FIVE OF `check`'S OWN RULES REMAIN, down from 7.** **`stale_check`'s matcher is NOT one algorithm with a fast path: Python splits patterns into LITERALS, resolved by exact path, suffix OR SUBSTRING, and GLOBS, resolved by `fnmatch` — PORTING ONLY THE FAST PATH WOULD HAVE MADE A LESSON WRITTEN AS A BARE FILENAME STOP FIRING, which is the fnmatch-semantics bug this port already hit once** (separator-aware `*` meant `*team.rs*` did not match `crates/x/src/team.rs`); **`<dir>/**` is rewritten to a directory prefix literal rather than left as a glob, because the two match differently — asserted**; and **the index comes from `git ls-files`, not a filesystem walk: Python's original walked ~11k files and ran three `fnmatch` calls per (file, pattern) pair per lesson, ~46 SECONDS DOMINATING THE WHOLE `check`, while `git ls-files` is the already-built index and excludes ignored build output for free — with `-z` because paths can contain newlines.** **AND ONE RULE IS PORTED AS THE NO-OP IT IS: `receipts.skill_usage_recorded` returns `[]` from both branches, so NO INPUT PRODUCES A FINDING — it is declared, registered and reported as passing, AND IT ASSERTS NOTHING; it is ported as an explicit empty list rather than as a plausible check, because writing it as "report when skills were used but no skill receipt exists" WOULD BE INVENTING A RULE THE ENGINE DOES NOT HAVE and would fire on every state where the two disagree — A BEHAVIOUR CHANGE DRESSED AS A PORT — and a test asserts it produces nothing for any state, so a future change making it produce a finding is VISIBLE RATHER THAN HIDDEN BEHIND A CHECK THAT PASSES SILENTLY.** **A mistake the check caught on me: MY FIRST STALE-LESSON FIXTURE REPORTED 0 FINDINGS AND THE CHECK WAS RIGHT — the fixture had no git repository, so `git ls-files` returned nothing, so there was NO INDEX, and the port deliberately reports nothing rather than reporting every lesson as stale, BECAUSE NO INDEX MEANS NO EVIDENCE AND REPORTING EVERYTHING WOULD BE A FINDING THAT IS WRONG RATHER THAN MISSING**; the fixture now initialises a git repo and **both behaviours are asserted.** **Limits, stated — and the first is the objective's own measure:** **`check` STILL DELEGATES, SO THE OBJECTIVE IS NOT MET — FIVE OF ITS OWN RULES REMAIN (`harness.entry_point`, `harness.documented_commands`, `prlint.gates_declared`, `budget.no_new_spending_class`, `status.registry_vs_github`), and UNTIL ALL FIVE LAND `check` KEEPS DELEGATING, because PORTING A SUBSET WOULD BE THE SILENT DOWNGRADE THIS PORT REFUSES**; **`harness.entry_point` and `harness.documented_commands` describe the file they live in — the "two-rule/one-file" pattern — so THEY SHOULD BECOME UNNECESSARY RATHER THAN TRANSLATED, and two of the five belong with retiring `cli.py`**; **`stale_check`'s index is `git ls-files`, so an untracked-and-ignored lesson path reads as stale even though the file exists on disk — Python has the same behaviour, and it means A LESSON POINTING AT BUILD OUTPUT IS REPORTED, CORRECTLY, AS POINTING AT SOMETHING THE REPO DOES NOT TRACK**; and **the no-op rule's test asserts TODAY'S BEHAVIOUR, NOT A REQUIREMENT — if the owner decides skill usage SHOULD be checked the test fails, WHICH IS THE INTENDED WAY FOR THAT DECISION TO BECOME VISIBLE.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the five remaining `check` rules — `harness.entry_point` and `harness.documented_commands` (which should become unnecessary rather than translated), `prlint.gates_declared`, `budget.no_new_spending_class`, and `status.registry_vs_github` (the `gh` chain) — then `check` stops delegating; plus `panel`, the `scripts/` tail, and `git switch`/`delete-merged`; and `prep new --dry-run`, the machine-readable-card question, the `OWNER-*` scope question, and the four-rule unportable decision.

## 2026-10-14 — `lint` runs in Rust: the first command to leave the delegated set (deepseek, tooling)

**`uor-harness lint` — `lint check` and `lint format` — RUNS ENTIRELY IN RUST WITH NO DELEGATION**, verified by **`lint check` printing zero "not ported to Rust yet" notices while `check` still prints one, correctly, because `check` is a different command and remains Python.** Everything it needs is now Rust: **`check_pr` runs the gates, `format_pr_body` renders a compliant body, and the D23 scan is the RATCHET rather than the retired script.** **VERIFIED BYTE-IDENTICAL TO PYTHON ON 12 COMPARISONS — six bodies through `lint check` (output AND exit code) and six through `lint format`: 6 identical / 0 differing for each.** **FOUR DIVERGENCES FOUND BY DIFFING THE TWO COMMANDS, NOT BY READING EITHER:** **(1) The headline fix dropped the denominator** — `headline_present` rendered the *value* instead of the whole headline, producing `headline 5 → <number/of>` where Python produces `headline 5/232 → <number/of>`, **found only because the body I had been testing with happened to have a before value with NO denominator, so the difference is invisible until a denominator exists — and THE OLD TEST ASSERTED `headline 37` AND PASSED WHILE THE CODE WAS WRONG, so it was strengthened rather than left**; **(2) `format` is a SUBCOMMAND, not a flag** — it arrives in the argument list because `lint` already claimed the command slot, and I looked only for `--action`, **so `lint format` SILENTLY RAN `check` and printed a summary where Python printed a body**; **(3) an empty `--body ""` is not a body** — Python's check is **falsy** so an empty string falls through to the error and exit 2, while mine linted it, **the two disagreeing on the one input where "no body" and "an empty body" are the same thing to a reader**; **(4) a doubled trailing newline** — `format_pr_body` already ends with one and `println!` added a second, **a blank line at the end of every rendered body, invisible in a terminal and NOT invisible in a diff — the fix belongs at the print, not in the string the function returns, BECAUSE THE FUNCTION'S CONTRACT IS THE BYTE-FOR-BYTE FIXTURE.** **NONE OF THESE FOUR WAS FOUND BY READING THE CODE; EACH CAME FROM RUNNING BOTH IMPLEMENTATIONS ON THE SAME INPUT AND DIFFING.** **Limits, stated:** **`check` STILL DELEGATES, SO THE OBJECTIVE IS NOT MET — `lint` is one command of the set, `delegate_to_python` remains live, and the goal's own condition is ZERO notices from `check`**; **the six bodies are HAND-WRITTEN, not real PR bodies** — empty, minimal, two decisions, a registry line at three, and prose-only, **not the shapes other labs actually write**; **`--action` is now accepted and IGNORED** — `format` is detected as a subcommand word, so a caller passing `--action format` gets `check`, and **the flag is registered for the dispatcher's sake and does nothing — Python has no such flag so nothing invokes it, but LEAVING IT ACCEPTED IS A SMALL LIE IN THE INTERFACE**; and **`lint format`'s output is asserted byte-for-byte against a CAPTURE, not against the contract — where Python's formatter renders something odd THE PORT REPRODUCES IT, deliberately, and it is worth knowing before reading the formatter as an endorsement of the template.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the remaining LIVE Python under `delegate_to_python` — `check` itself (its 7 remaining own rules and the delegation notice), then `panel`, the `scripts/` tail (pod, storage, `check_*.py`), and `git switch`/`delete-merged` with their guards; plus `prep new --dry-run`, the machine-readable-card question, the `OWNER-*` scope question, and the four-rule unportable decision.
## 2026-10-14 — M1 deterministic reply grader `v2` check table and paired copy-control re-score (`F3`–`F8`, `R14`) (antigravity, evaluation tooling)

[`docs/labs/m1-grader-v2-checks-2026-10-14/README.md`](../labs/m1-grader-v2-checks-2026-10-14/README.md) delivers the versioned `v2` check table and paired copy-control comparisons audited on `#2029` / `#2365` (`F3`–`F6`) and `#2412` (`F7`–`F8`) while preserving `data/panels/reply-panel-checks.tsv` (`aba96550…`) and all 30 files in `data/panels/MANIFEST.sha256` byte-identical. `crates/uor-r4-training/fixtures/reply-panel-checks-v2.tsv` (`e650e865…`) checks `96` rows (`136` `BLOCKED`) on the 232-row open panel (`19 any`, `5 reply_exact`, `2 exact_text`, `30 clarify`, `32 abstain_exact`, `6 salutation`, `1 one_of`, `1 word_count_6`), adding `F6`/`F7` checks for `heldout-020`, `-033`, `-077`, `-092` (`one_of` restricted to `my answer is yes`), `-112`, `-122` and `-125` (sign-preserving `exact_text`), and `-173` (format-only `word_count_6`), reclassifying bare-salutation rows (`heldout-006`, `-009`, `-019`, `-102`, `-131`, `-196`) to `salutation`, and widening `abstain_exact`/`clarify` (`F5`) so all `34/34` frozen human positive controls (`crates/uor-r4-training/fixtures/reply-panel-positive-controls-v2.tsv`, `7178d7e7…`) pass and all `15/15` frozen negative controls (`crates/uor-r4-training/fixtures/reply-panel-negative-controls-v2.tsv`, `dd39b2b8…`) fail (`F8`: 12/13 canned replies score `0/96`, 1 canned greeting scores `6/96` on `salutation` by design). Across the 4 archived open-panel reply sets (`mixture-dose-2026-10-10.tar` `md5: d31b25dd…`, `reply-panel-open-2026-10-09.tar` `md5: 1740b21f…`), removing `F5` false negatives raises parent `29m` from `2/88` to **`11/96`** (`11.46%`), `rp-D0` from `1/88` to **`11/96`** (`11.46%`), `rp-D10` from `1/88` to **`15/96`** (`15.62%`), and `rp-D25` from `0/88` to **`10/96`** (`10.42%`), while `constant[0]` (`"I'm not sure. Can you tell me more about what you mean?"`) moves from `40/88` to **`38/96`** ($p \le 4.31 \times 10^{-4}$ in favour of the constant across all four arms). On `conversational-v5` (`64` rows), `CopyTally` and `Group::headline` now compute row-by-row paired McNemar counts against `copy_first_stated` (`20/40`) and `copy_last_stated` (`20/40`) (`F3`/`F4`), showing that `v5-D10` (`20/40` memory, `0/24` unknowable) and `v5-D25` (`20/40` memory, `15/24` unknowable) tie `copy_last_stated` (`both = 14`, `actual_only = 6`, `control_only = 6`, two-sided McNemar $p = 1.0$), yielding `categories_model_beats_best_control_or_copy = []` across all archived arms. Under **R14** (`#2373`), the M1 reply-quality headline remains **SUSPENDED** with no target; zero model compute (`0` GPU hours).

Line: evaluation integrity · count 1/3 unchanged · headline 37/232 → 37/232 (unchanged; M1 reply-quality headline SUSPENDED under R14).

**Next:** Exact-head review confirmation on `#2029` / `#2412`; optional prime-rank indexing + complex-plane zeta-phase ($\mathbb{C}^4 \to \mathbb{R}^4$ / $\mathbb{C}^8 \to \mathbb{R}^8$) mechanism brief and pre-registration sheet under the `#2398` D32 addendum.

## 2026-10-14 — `check_pr` is Rust: the dispatcher that makes thirteen gates reachable (deepseek, tooling)

**`prgates::check_pr` ports every gate in PYTHON'S ORDER, returning Python's shape — `ok`, `problems`, `summary`, `fields` — asserted against Python's own output for four bodies, comparing `ok`, the summary string, the problem count, THE ORDER, the severities, and the message and fix text.** **WHY THE ORDER IS PORTED RATHER THAN CHOSEN: a reader meets the first problem first, and the gates run body-declarations → registry agreement → housekeeping; RE-ORDERING WOULD NOT CHANGE `ok` AND WOULD CHANGE WHAT A LAB FIXES FIRST.** **AND A DISPATCHER ERROR IS INVISIBLE TO PER-GATE TESTS: thirteen gates that each agree in isolation can still run in the wrong order or with one omitted and every gate test passes — THAT IS THE FAILURE THIS FIXTURE SET EXISTS TO CATCH, and the comparison names a difference by position.** **Two inputs are parameters for the reason `record_path` takes a `repo`: `state_text` (current-state.md) and `now` (the local date) — A GATE THAT READS THE CLOCK OR THE FILESYSTEM ITSELF CANNOT BE TESTED AT A FIXED INSTANT OR AGAINST A FIXTURE, the defect this session already fixed once in `parity`; the fixture test passes `None` for the state text, WHICH PROVES the bodies do not consult it rather than assuming they do not.** **THE FINDING: MY OWN DOCSTRING TRIPPED THE INTERLOCK** — `check_no_softmax.py` failed on **`prgates.rs: 0 -> 1`** because **my `check_pr` docstring named the prohibited mechanism in a sentence explaining that this function does NOT run the scan**; **both internals reported the same file and count, and I had initially MISREAD the two messages as disagreeing — THEY WERE TRUNCATING DIFFERENTLY, NOT DIVERGING, which is worth recording because the first reading would have become a FALSE BUG REPORT.** **And THE FIXTURE HAD BEEN CAPTURED WHILE THE TREE WAS DIRTY, so it encoded a finding that no longer existed and the comparison failed for a reason unrelated to the port — re-capturing against a clean tree fixed it, and the note is in the fixture file: A CAPTURE TAKEN WHILE THE TREE IS DIRTY ENCODES THAT DIRT.** **This is the FOURTH time in this port that a detector's own vocabulary tripped the guard it enforces, and the first time the PROSE was the cause rather than the code.** **Limits, stated — and the first is the objective's own measure:** **`lint` STILL DELEGATES, so `check_pr` IS NOT REACHABLE FROM ANY COMMAND — this unit makes the gates reachable from a FUNCTION rather than from the CLI, dropping `lint` from the delegated set is the next unit, and UNTIL IT LANDS THE OBJECTIVE'S OWN MEASURE HAS NOT MOVED**; **the four fixture bodies are hand-written** (empty, minimal, complete, an unknown line), **not a real PR body, and the D23 message is compared only by its 60-character prefix because it carries a live file count, so that gate's count is not verified here**; **`fields` is returned but NOT compared** — Python returns the full parsed fields and **nothing asserts they survive the assembly, so a dispatcher returning `{}` would pass**; and **`parse` cannot be reached from `check_pr` either, since the Rust parser is total — the one gate Python can raise first is the one Rust cannot, recorded in the module docstring rather than papered over.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **drop `lint` from the delegated set by porting the `lint` command** — the unit that finally moves the objective's own measure; then `prep new --dry-run`; decide what `registry_vs_github` should do when no card is machine-readable; the `OWNER-*` scope question; the four-rule unportable decision; `panel`; the `scripts/` tail; and `git switch`/`delete-merged` with their guards.

## 2026-10-14 — `checks_at_head` and `org_rules`: 13 of 14 prlint gates, and why the 14th cannot exist (deepseek, tooling)

**Two more gates ported, asserted against Python's own output: `prlint.checks_at_head` (warn — the exact-head-checks box is not ticked AND the body lists no commands) and `prlint.org_rules` (info — no `References #N` or `Closes #N` line). Coverage: 45 of 54, THIRTEEN OF prlint'S FOURTEEN GATES IN RUST.** **`checks_at_head` ACCEPTS THE WEAKER OF TWO EVIDENCES, DELIBERATELY: either a pasted command or a ticked box satisfies it** — a pasted command is evidence unticked, a ticked box is a claim, **and the gate asks whether a reader can see SOMETHING, so accepting the weaker is the design rather than an oversight** (asserted); **an unticked box is not evidence and neither is a ticked unrelated item**, because the match is on `exact head` or `check(s) run` (both asserted). **AND THE FOURTEENTH GATE IS UNREPRESENTABLE, NOT FORGOTTEN: `prlint.parse` fires only when `parse_pr_body` RAISES, and THE RUST PARSER RETURNS NO `Result` — it is total, and a malformed body yields empty fields, SO THERE IS NO STATE IN WHICH THIS GATE COULD FIRE**, and writing it would mean a function nothing can reach or a `Result` added to the parser solely to have somewhere to put the error. **That is a deliberate divergence and it is the SAFE direction: Python reports "body could not be parsed" AND SKIPS EVERY GATE, while Rust parses what it can and RUNS EVERY GATE ON THE RESULT — a body Python would refuse outright is one Rust checks normally.** **Recorded in the module docstring rather than left as a silent gap, BECAUSE A GATE THAT IS MISSING AND A GATE THAT CANNOT EXIST LOOK IDENTICAL FROM A COVERAGE REPORT, and `parity` reporting `prlint.parse` as still in Python is TRUE and is not a port that was forgotten.** **Limits, stated — and the first is the one that matters:** **`check_pr` IS STILL NOT PORTED, SO THIRTEEN GATES ARE NOW RUST AND NOT ONE IS REACHABLE FROM ANY COMMAND — `lint` still delegates, the port has VERIFIED UNITS AND NO WORKING FEATURE, and UNTIL `check_pr` LANDS THE COVERAGE NUMBER DESCRIBES CODE NOBODY CAN RUN**; **`ok`/`summary` are not ported either — `check_pr` returns them alongside `problems` and `ok` is what decides a non-zero exit, so the ported fields are not sufficient to replace the command even now**; **the `parse` divergence means the two implementations differ on a malformed body, deliberately and in Rust's favour, and PARITY CANNOT COMPARE THAT CASE because it cannot make Python's parser raise on demand**; and **`checks_at_head`'s box match is a substring test on a lowercased label, so a ticked label like "checks run at this exact head (pending)" satisfies it — the same looseness Python has, and the reason the gate can be satisfied by a claim rather than evidence.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** **`check_pr` itself — the unit that turns thirteen verified gates into a working command** — then the `lint` command and dropping `lint` from the delegated set; `prep new --dry-run`; decide what `registry_vs_github` should do when no card is machine-readable; the `OWNER-*` scope question; the four-rule unportable decision; `panel`; the `scripts/` tail; and `git switch`/`delete-merged` with their guards.

## 2026-10-14 — The newest milestone was silently dropped: an off-by-one the fixtures could not catch (deepseek, tooling)

**THE BUG: Python's milestone set is `range(2029, 2037)` — INCLUSIVE of 2036 — and the ported parser wrote `const MILESTONE_HIGH: i64 = 2036;` with `(MILESTONE_LOW..MILESTONE_HIGH)`, and A HALF-OPEN RUST RANGE ENDS ONE SHORT, so `#2036`, THE NEWEST MILESTONE, WAS NEVER RECOGNISED AS A MILESTONE.** **It still produced a `milestone_ref` through the fallback, so EVERY FIXTURE AGREED: the field was populated, just from the WRONG BRANCH, and nothing compared the branch.** **FOUND BY PROBING, NOT BY READING** — I was debugging why `current_state_entry` named the topic instead of the issue, printed `milestone_issue`, and saw `None` where Python saw `2036`. **WHY FIVE FIXTURES COULD NOT CATCH IT: none of the five captured bodies cites #2036** (they cite #2028, the tracker and outside the range, and #2029) — **THE FIXTURES RECORD WHAT THE PARSER RETURNS; THEY DO NOT ENUMERATE WHAT IT COULD BE ASKED. That is the limit of a behaviour capture**, and it is why **a sixth case now exists** and why a test pins **both ends of the range and the value just outside it** (`#2029 -> Some(2029)`, `#2036 -> Some(2036)`, `#2037 -> None`). **Also landed: three more gates asserted against Python's own output** — `prlint.prereg_before_compute` (error, model numbers with no pre-registration reference), `prlint.no_cpu_model_compute` (warn, CPU/host-fallback compute presented as a result), `prlint.current_state_entry` (warn, a claimed current-state entry the file does not support) — **coverage 43 of 54, up from 40, WITH 11 prlint GATES IN RUST.** **`prereg_before_compute` IS THE GATE I TRIPPED FOUR TIMES WHILE WRITING THIS PORT** and the test now pins why: it fires on any prose line with **a run-word and a digit**, so a body discussing its own verification commands trips it — **I did not loosen it, because narrowing a fuzzy gate that does a real job would be a behaviour change dressed as a port.** **`current_state_entry` has three branches, and THE ISSUE IS NAMED WHEN THERE IS ONE even though it was the TOPIC that failed to match** — I wrote that precedence the other way round first and my test caught it. **And `no_cpu_model_compute` inherits the parser's exclusion of defect lines, which is what stops it accusing a body that REPORTS FIXING a host fallback.** **Limits, stated:** **the off-by-one survived because THE FALLBACK MASKED IT — `milestone_ref` was still populated, so the fixture agreed on a field whose value came from the wrong code path: A FIELD BEING RIGHT DOES NOT MEAN THE BRANCH THAT SET IT WAS**; **`check_pr` is STILL NOT PORTED — three of fourteen gates remain (`parse`, `checks_at_head`, the org rules) plus the dispatcher and the `lint` command, `lint` still delegates, and NONE OF THE ELEVEN PORTED GATES IS REACHABLE FROM ANY COMMAND**; and **this unit took far more edits than it should have — repeated string-replace edits against text `cargo fmt` had already reformatted failed silently several times and I mangled the test function badly enough to rewrite it, while THE LINE-RANGE APPROACH WORKED EVERY TIME AND I KEPT REACHING FOR STRING-REPLACE.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** `checks_at_head`, `parse` and the org rules, then `check_pr` and the `lint` command; `prep new --dry-run`; decide what `registry_vs_github` should do when no card is machine-readable; the `OWNER-*` scope question; the four-rule unportable decision; `panel`; the `scripts/` tail; and `git switch`/`delete-merged` with their guards.

## 2026-10-14 — `pivot_at_three` and `record_path`, the first gate that stats the filesystem (deepseek, tooling)

**Two more `prlint` gates are Rust, both asserted against Python's own output for the same bodies: `prlint.pivot_at_three` (error, the line is at 3/3 and no pivot is announced) and `prlint.record_path` (warn, no dated record path named or a named one that does not exist). Coverage: 40 of 54, up from 38.** **`pivot_at_three` HAS TWO BRANCHES AND THE `where` CLAUSE SAYS WHICH** — either side can put the line at three (**the count the body declares, or the registry's count for that line**) and **the two messages send a reader to different places: one to fix the body, the other to check whether the line really moved** — asserted as *"line 'starling read' is at registry count 3/3 and the body announces no pivot"* and *"line 'exact addressed memory' is at 3/3 declared and the body announces no pivot"*; **a line at three is not a failure but the point at which a decision is owed**, so the fix offers both ways out, **and announcing a pivot silences it, asserted, because the gate asks for a decision rather than for the line to stop.** **`record_path` IS THE FIRST GATE THAT READS THE FILESYSTEM**, which is why it was not ported with the field-only group, **and its second branch is the one worth having: a PR naming a record it never wrote** — both branches asserted, **and an existing record asserted silent, the property that keeps the gate usable rather than something labs route around.** **The fix text names TODAY (`docs/labs/<topic>-2026-10-14/README.md`), which is exactly why the date is a PARAMETER of `filesystem_gates` rather than read inside the gate — A GATE THAT READS THE CLOCK CANNOT BE TESTED AT A FIXED INSTANT, the defect this session already fixed once in `parity`.** **Limits, stated:** **`check_pr` is STILL NOT PORTED — seven of fourteen gates remain plus the dispatcher and the `lint` command, `lint` still delegates, and NONE OF THE EIGHT PORTED GATES IS REACHABLE FROM ANY COMMAND, so they are verified units rather than a working feature**; **`record_path` uses `Path::exists`, which follows symlinks and returns false on a permission error — Python's `Path.exists()` does the same, so they agree today, BUT A PERMISSION-DENIED RECORD WOULD BE REPORTED AS MISSING BY BOTH, a wrong finding rather than a missing one**; **`pivot_at_three` reads `has_pivot`, a regex over the body for `pivot (card|recorded|posted|point|line)`, so A LAB ANNOUNCING A PIVOT IN WORDING OUTSIDE THAT LIST IS REPORTED AS ANNOUNCING NONE — the port inherits that and the fixtures do not cover it**; and **the `today` parameter is passed but `pivot_at_three` ignores it** — it is in the signature for the shared `filesystem_gates` entry point, **and an unused parameter on a gate that does not need it is noise, recorded rather than hidden.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the remaining seven gates (`current_state_entry`, `prereg_before_compute`, `checks_at_head`, the org rules, and the parse-time ones) and `check_pr`, then the `lint` command; `prep new --dry-run`; decide what `registry_vs_github` should do when no card is machine-readable; the `OWNER-*` scope question; the four-rule unportable decision; `panel`; the `scripts/` tail; and `git switch`/`delete-merged` with their guards.

## 2026-10-14 — The registry gates, and one that cannot catch the mistake it was written for (deepseek, tooling)

**`prlint.count_matches_registry` and `prlint.headline_vs_registry` are Rust, with the closure they need ported whole rather than assumed small** — `_norm`, `_find_line`, `_registry_headline`, `_same_number`, `_fmt_num`, `_target_matches_registry`, `Headline.render()`, and **`_find_milestone`, which SCORES candidate milestones rather than trusting the first issue number** (a line match is worth 3, the milestone id in the body 1, an agreeing headline 2) **because a body cites the tracker as well as its own milestone and the first number is often the tracker.** **Coverage: 38 of 54, up from 36.** **THE FINDING: `headline_vs_registry` CANNOT FIRE ON THE MISTAKE IT EXISTS FOR** — its guard is `not _target_matches_registry(after, milestone) and not stated_move`, and **`stated_move` is `before.numeric and not _same_number(before.value, after.value)` while `after.numeric` REQUIRES AN ARROW, which puts a value on the left**; enumerated: before 37/after 5 → stated_move True → no finding; before 5/after 5 → stated_move False → FIRES; before 37/after 37 → matches → no finding. **So a lab that moves a headline and forgets the registry has `stated_move` true and is NEVER reported — the rule fires only on a body whose headline does not move yet disagrees with the registry (`headline 5/232 -> 5/232`, self-contradictory), CONFIRMED AGAINST THE LIVE REGISTRY RATHER THAN BY READING: that body produces the finding and every realistic one produces none.** **I did not change it** — whether the guard should exclude `stated_move` when the after value *disagrees* with the registry is **a policy question about D21, and the port matches Python either way, because a deliberate divergence here would make parity lie about which implementation a lab is running.** **TWO OF MY OWN MISTAKES, BOTH CAUGHT BY THE FIXTURES:** **I wrote the count test the wrong way round** — asserting that declaring `0` against a registry `12/3` was not this branch, when **M2's line sits at 12 so 0 is below it and must be reported** — and the test failed and said so; and **Rust's `{:?}` quotes with `"` where Python's `!r` uses `'`**, so `py_repr` was added, **which is NOT COSMETIC: the messages are compared character for character, and a port emitting `"name"` where Python emits `'name'` DIFFERS FROM THE TOOL IT REPLACES — A DIFFERENCE THAT WOULD SHOW UP IN EVERY LAB'S TERMINAL RATHER THAN IN A TEST THAT ONLY CHECKS THE RULE ID.** **Limits, stated:** **`check_pr` is STILL NOT PORTED — nine of fourteen gates remain plus the dispatcher and the `lint` command, `lint` still delegates, and NONE OF THE SIX PORTED GATES IS REACHABLE FROM ANY COMMAND, so they are verified units rather than a working feature**; **`count_matches_registry`'s first branch is asserted only in the below-registry direction** and the `of`-absent fallback to 3 is **ported but unverified**; **`_find_milestone`'s scoring was tested with two bodies**, so **a body where two milestones score equally is untested** and the `is_none_or` tie-break keeping the first **is reasoned equivalent to Python's strict `score > best[0]`, NOT MEASURED**; and **`same_number` returns false when either side is non-numeric where Python's `float()` would raise and fall back to `==`, so a string-valued headline number would behave differently — the committed registry has none, so the case does not arise today.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the remaining nine gates (`pivot_at_three`, `record_path` with its filesystem stat, `current_state_entry`, `prereg_before_compute`, `checks_at_head`, the org rules) and `check_pr`, then the `lint` command; `prep new --dry-run`; decide what `registry_vs_github` should do when no card is machine-readable; the `OWNER-*` scope question; the four-rule unportable decision; `panel`; the `scripts/` tail; and `git switch`/`delete-merged` with their guards.

## 2026-10-14 — `body_text` and `raw`: two fields that looked like internals until a gate needed them (deepseek, tooling)

**BEFORE STARTING THE REGISTRY-DEPENDENT GATES I MEASURED THEIR CLOSURE INSTEAD OF ASSUMING IT**, and found that **`_find_milestone` scores candidate milestones partly on `body_text`** (`if milestone.id.lower() in body.lower(): score += 1`) — **and my Rust `Fields` did not carry it**, holding only the 25 fields the gates read, because `body_text` looked like a parser internal. **Without it the milestone the body is ABOUT cannot be identified, so `count_matches_registry`, `headline_vs_registry` and `pivot_at_three` cannot be ported at all — so the unit became "remove the blocker" rather than "port the gates badly."** **Fixed:** `Fields` carries **`body_text`** (comment-stripped, newline-normalised) and **`raw`**, and `to_json` emits both, **because a gate consumes one of them so it is part of the contract rather than an internal**; **the fixtures now record them too — 27 fields × 5 cases = 135 comparisons, up from 125, all matching Python — and THE CAPTURE HAD TO BE EXTENDED, NOT THE ASSERTION RELAXED, because the fixture set was complete for the fields the ported gates were reading, WHICH IS EXACTLY THE KIND OF COMPLETENESS THAT STOPS BEING TRUE THE MOMENT THE NEXT CONSUMER ARRIVES.** **WHY THE FIXTURES RECORDED THE WRONG THING BEFORE: they recorded the 25 fields that SEEMED to matter, captured by reading what the gates fed on rather than by recording everything the parser returns — THE SAME MISTAKE THIS PORT HAS MADE THREE TIMES ABOUT "THE REGISTERED SET": TAKING THE FIRST LIST I FOUND AND STOPPING; here it cost one unit of delay and NO WRONG CODE, because I measured the closure BEFORE writing the gates rather than after, WHICH IS THE DIFFERENCE THE PREVIOUS THREE DID NOT HAVE.** **Limits, stated:** **this port is TWO FIELDS, NOT A GATE — no new gate is reachable, `lint` still delegates, coverage stays at 36, and the unit unblocks the next one rather than advancing it**; **the registry-dependent gates are NOT started and their closure is larger than the blocker alone** (`_find_milestone` is 49 lines and also needs `_registry_headline`, `_same_number`, `_norm`, `_find_line`, and the milestone `lines`/`headline` structures), **so estimating it as "now unblocked, therefore small" would repeat the mistake this unit was about**; and **`clippy` caught a field-assignment style issue and `cargo` was again the check that worked — the third time this session that the compiler found what my own reading did not.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the registry-dependent gates on the now-complete field set (`_find_milestone` and its four helpers); `prep new --dry-run`; decide what `registry_vs_github` should do when no card is machine-readable; the `OWNER-*` scope question; the four-rule unportable decision; `panel`; the `scripts/` tail; and `git switch`/`delete-merged` with their guards.

## 2026-10-14 — Three `prlint` gates are Rust; the coverage guard caught a vocabulary gap in itself (deepseek, tooling)

**`crates/uor-r4-harness/src/prgates.rs` ports the three gates whose entire input is the parsed fields** — `prlint.line_declared`, `prlint.headline_present`, `prlint.decision_present` — **with Python's exact text and asserted against Python's own output for the same bodies, field for field including the FIX, because A PORT THAT RAISES THE RIGHT RULE WITH THE WRONG FIX TELLS A LAB TO DO THE WRONG THING.** Two details the capture made explicit rather than assumed: **the two gaps are JOINED into one problem** (`"the body does not declare Line name, count n/3"`) rather than raised twice, **with Python's order (name first)**; and **the headline hint carries the BEFORE value when there is one** (`headline 37 → <number/of>`), a different fix from the generic placeholder. **`record_path` and `current_state_entry` are deliberately absent** — the first also **stats the filesystem**, the second **reads `current-state.md`** — and **each says so in the module docstring, because A GATE THAT IS MISSING AND A GATE THAT IS NOT NEEDED LOOK IDENTICAL FROM THE OUTSIDE.** **THE COVERAGE GUARD CAUGHT ITSELF:** parity failed with *"the Rust rule inventory declares rules Python does not register … a declared rule that does not exist inflates coverage"* — **the guard was right and its INPUT was wrong**: `python_registered_rules()` returned only `checkrules.rules()`, but **`prlint` declares its gates in `prlint.GATE_RULES`** (raised against a **PR body**, which the engine never has, so they are **data rather than registered checks**). **THIS IS THE THIRD TIME IN THIS PORT THAT "THE REGISTERED SET" TURNED OUT TO HAVE MORE THAN ONE SOURCE** — after `mistakes._known_rule_ids` (three sources) and my own data file that took one and stopped — and **the first two produced a check that was wrong rather than missing; this one did too, reporting three genuinely-ported rules as non-existent and calling the port inflated.** **Fixed by delegating to `mistakes._known_rule_ids()` rather than re-listing the sources, because it already answers this question and re-listing is how the previous two went wrong.** **AND IT EXPOSED A METRIC THAT WAS HIDING PROGRESS:** with the gates counted the report moved from *"8 Python rules still in Python"* to *"18"*, and **both numbers are needed because one was hiding the other** — the report now reads **"7 of check's own 39 rule(s) still in Python, 18 of 54 including prlint's PR-body gates"**; **`check`'s own rules are what `delegate_to_python` is waiting on, while the PR-body gates are real check logic that would otherwise read as already done, and reporting one number made the port look further along than it was the moment a gate moved into the inventory.** **Limits, stated:** **`check_pr` is NOT ported — ELEVEN of its fourteen gates remain plus the dispatcher and the `lint` command, `lint` still delegates, and THESE THREE GATES ARE NOT REACHABLE FROM ANY COMMAND: nothing calls them yet, so they are a verified unit rather than a working feature**; **the three were asserted against four hand-written bodies**, not a real PR body; **`line_declared`'s fix text is a template hint quoting the contract's format and is NOT a valid body on its own** — a lab pasting it literally would still be missing a real name and count; and **the two denominators are printed but nothing asserts them, so a future change could collapse them again and no test would notice — THE METRIC IS HONEST TODAY BECAUSE I READ IT.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** the remaining eleven gates and `check_pr`, then the `lint` command; `prep new --dry-run`; decide what `registry_vs_github` should do when no card is machine-readable; the `OWNER-*` scope question; the four-rule unportable decision; `panel`; the `scripts/` tail; and `git switch`/`delete-merged` with their guards.

## 2026-10-14 — `prlint.parse_pr_body` is Rust; 125 of 125 fields agree with Python (deepseek, tooling)

**`crates/uor-r4-harness/src/prbody.rs` ports the PR body parser field for field**, checked against the five captured fixtures: **25 fields × 5 cases = 125 comparisons, ALL MATCHING PYTHON.** **THE FIXTURES EARNED THEIR PLACE IMMEDIATELY — exactly ONE of the 125 differed, and it was the case reading would not have found:** `template-raw: line_name: rust Some(String("")) != python Some(String("<line name>"))`, **because Python uses `re.fullmatch` for the placeholder check and I had used `is_match`** — a substring search matches the `line` inside `<line name>` and rejects a name Python accepts; **and `\b` does not fire inside `<line name>`, WHICH IS PRECISELY WHY MY "EQUIVALENT ANCHORED PATTERN" REASONING WAS WRONG AND WHY REASONING WAS THE WRONG TOOL** — the fixtures settled it in one run. **THE D23 INTERLOCK FIRED ON THE DETECTOR, FOR THE THIRD TIME IN THIS PORT** (`prlintrules.rs`, `ruleinventory.rs`, now `prbody.rs`): writing the parser took that file from **0 to 9** occurrences of the prohibited mechanism's name and the interlock correctly reported *added to executable source* **against the file whose whole job is to notice it**; **fixed the way this port has fixed it before — the vocabulary became data**, in `crates/uor-r4-harness/rules/prlint-phrases.json` (the scanned phrases, the pre-registration markers, the CPU/laptop patterns and the claim key name) — **and it counts PROSE AS WELL AS CODE, so the doc comment spelling the mechanism kept the count up and the comment had to stop spelling it too: A FILE THAT REPORTS ON A BANNED THING MAY NOT NAME IT ANYWHERE, INCLUDING IN THE EXPLANATION OF WHY IT MAY NOT NAME IT.** **Three Rust tests failed and then passed, which is the interlock working:** `prlintrules`' own tests run the real interlock, so **they failed while `prbody.rs` was still dirty and passed once it was clean — the guard is load-bearing on itself, and I did not have to trust that, the suite said so.** **Limits, stated:** **`prlint` IS NOT PORTED — this is ONE FUNCTION OF A 1,133-LINE MODULE**, the 14 gates, the `check_pr` dispatcher and the `lint` command all still Python and **`lint` still delegates**; **125 comparisons against 5 HAND-CHOSEN bodies** covering the template, the separators, a wrap and the empty body — **not a real PR body from another lab, so a field can still differ on a shape none of these has**; **two regex behaviours needed translating and I did not audit for others** — the missing look-ahead (rewritten to capture and offset) and `fullmatch` vs `is_match` (the bug above) — and **the `regex` crate is finite-automaton while `re` is backtracking, so a third difference may exist where the engines choose differently among alternatives, which the fixtures catch only if a case exercises it**; and **the parser reads its phrases from a file at call time, cached in a `OnceLock`, so a missing or malformed `rules/prlint-phrases.json` silently yields empty patterns and the claim scans find nothing rather than failing — A DETECTOR THAT CANNOT LOAD ITS VOCABULARY REPORTS CLEAN.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** port the 14 gates and `check_pr` against the same fixtures, then the `lint` command; add `prep new --dry-run`; decide what `registry_vs_github` should do when no card is machine-readable; the `OWNER-*` scope question; the four-rule unportable decision; `panel`; the `scripts/` tail; and `git switch`/`delete-merged` with their guards.

## 2026-10-14 — `prep new` no longer overwrites a sheet silently (deepseek, tooling)

**The defect:** `prep new` wrote its sheet with **no existence check at all** (`target.parent.mkdir(...)` then `target.write_text(text, ...)`) — **and A PRE-REGISTRATION SHEET IS THE D22 GATE'S OWN RECORD, so re-running `prep new` for a milestone that already had one REPLACED THE BAR, THE PANEL AND THE COST THAT WERE PRE-REGISTERED BEFORE THE WORK, which is precisely the evidence D22 exists to preserve: THE COMMAND THAT CREATES THE RECORD WAS THE ONE THING ABLE TO DESTROY IT.** **Fixed:** a second write is **refused**, naming the file and saying what it holds, **with the sheet left untouched**, and replacing is explicit via `--force`; **the refusal is asserted to leave the file BYTE-IDENTICAL, not merely to return 1 — a guard that reports refusal and writes anyway would be worse than none.** **Both implementations, because parity requires it: Rust had the same defect and got the same guard**, plus `--force` declared in its option lists, **verified by running both against a scratch repo (python refuses / accepts with --force / sheet unchanged; rust the same).** **A mistake worth recording:** adding `--force` to Rust's flag lists **broke the build**, and my first repair asserted on the wrong line content — **I inserted it TWICE because my second anchor matched the first insert; `SUBCOMMAND_FLAGS: [&str; 31]` then no longer matched its contents so the array length had to change; and my fix script asserted EIGHT spaces where the line had FOUR, so it found no duplicate and raised — three separate wrong assumptions about my own edit in one small change, and CARGO CAUGHT THE ONE THAT MATTERED WHILE MY OWN CHECKS CAUGHT NONE OF THEM.** **This is the tenth silent or wrong-targeted edit in this session**, which is the strongest argument yet for the rule being applied throughout: **read the bytes, do not reason about the edit.** **Limits, stated:** **`--force` is now the only way to re-render a sheet and THE SHEET IS NOT VERSIONED, so a deliberate replacement leaves no trace of the previous bar — refusing is the improvement, it does not make replacement safe**; **the guard is by path, so a sheet written under `--write <other path>` is not guarded and the protection is only as good as the default naming**; **I did not add `--dry-run`**, the other half of this deferred item, which would let a lab see the sheet before writing it; and **`lint`/`prlint` is untouched — this unit does not advance the parser port, which the objective's own order put next, and that is worth saying plainly.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** port `parse_pr_body` against `crates/uor-r4-harness/rules/prlint-body-fixtures.json` (5 cases), then the gates needing only `fields`; add `prep new --dry-run`; decide what `registry_vs_github` should do when no card is machine-readable; the `OWNER-*` scope question; the four-rule unportable decision; `panel`; the `scripts/` tail; and `git switch`/`delete-merged` with their guards.
## 2026-10-14 — `prlint` body fixtures: capturing Python before porting it (deepseek, tooling)

**Why fixtures before the port:** `prlint` is **1,133 lines with 22 problem sites inside one function**, and `parse_pr_body` — the shared input to every gate — is **105 lines producing 25 fields**; **`parity.py`, the most valuable verification in this port, DIES WITH PYTHON** (it has already caught the `prep.rs` stale rule list, the `guard_covered` 16-vs-40 bug and a phantom declared rule), **so behaviour captured as data is the only safety net that outlives it — and a port written from the docstring and then checked against the docstring proves nothing.** **THE CAPTURE EXPOSED THREE CASES A DOCSTRING-DRIVEN PORT GETS WRONG, found by RUNNING the parser rather than reading it:** the raw template yields an **asymmetric pair `line_count: 0` with `line_of: 3`**; **`milestone_ref` is populated in bodies that name no milestone**; and **an empty body still yields `headline_before`/`headline_after` objects** rather than nothing. **AND IT CAUGHT MY OWN ERROR, WHICH IS THE BETTER ARGUMENT: I WROTE FINDING 1 AS "`line_of` IS SET WHILE `line_count` IS NOT", MY OWN TEST ASSERTED THE ABSENCE AND FAILED — the fixture holds `line_count: 0`, NOT NULL — and THE FIXTURE CORRECTED THE PROSE**; I then fixed the assertion twice because **the first fix was a string replace against a block `cargo fmt` had already reformatted — it matched nothing and the test failed identically, THE NINTH SILENT NO-OP IN THIS SESSION AND THE SECOND IN THIS UNIT ALONE.** **A capture that corrects the person who wrote it is doing exactly its job.** **The fixtures:** `crates/uor-r4-harness/rules/prlint-body-fixtures.json`, **5 self-contained cases** (body plus expected output, so they survive Python's retirement) — empty (2 non-empty fields), template-raw (8), filled-basic (11), pipe-separators (11), wrapped-line (11) — and **`expected` is a RECORD OF BEHAVIOUR, NOT A SPECIFICATION: where a value looks wrong the fixture is still right and the port matches it until someone changes the behaviour deliberately, which is the difference between porting and "improving while porting".** **Limits, stated:** **5 cases is not coverage** — they were chosen to exercise the separators, the template, the wrap and the empty body, **not sampled from real PR history, so a port passing all five can still differ on a body shape none of them has, and the most likely such shape is a REAL PR BODY WRITTEN BY ANOTHER LAB, WHICH NONE OF THESE IS**; **nothing yet consumes the fixtures — they are a target with no runner until the Rust parser exists, so today they verify only that the capture is intact, not that any port agrees with it**; **the expectations were produced by Python, so a bug in Python is captured as correct — the point of a behaviour record and also its limit: a port that matches Python perfectly INHERITS PYTHON'S BUGS, and the fixtures will call that agreement**; and **`prlint` remains entirely Python with `lint` still delegating — this unit adds no ported behaviour.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** port `parse_pr_body` against these fixtures, then the gates that need only `fields`; decide what `registry_vs_github` should do when no card is machine-readable; the `OWNER-*` scope question; the four-rule unportable decision; `panel`; the `scripts/` tail; and `git switch`/`delete-merged` with their guards.

## 2026-10-14 — `status --at`: Python gets the clock injection Rust already had (deepseek, tooling)

**The defect:** `parity.status_receipts_agreement` passed a fixed instant to **Rust** and **not to Python** — **Python's `status` had no `--at` at all, the flag appearing only in `parity.py`** — so the check compared **Rust at an injected instant against Python at the wall clock**, agreeing **only while the machine's local date equaled the injected date**; **`main` passed in the late UTC evening and the very next attempt failed once the UTC date rolled over — roughly six hours of every US evening.** **The fix:** `--at` on Python's `status` through `core.set_clock()`, which `today()` and `month()` consult, **injected at the top of `cmd_status` before anything reads the clock**; `parity` now passes the same instant to both. **Verified:** one instant gives a **byte-identical** receipts line, and **different instants give `2026-10-13: 188 receipts` against `2026-10-14: 0 receipts`.** **AND I FIXED THE WRONG CALL SITE FIRST:** `parity.py` calls Python's `status` from **two** checks, **and I edited the wrong one; local parity passed anyway because that check does not compare date buckets, and CI failed exactly as before** — **AN EDIT THAT LANDS IN THE WRONG PLACE LOOKS EXACTLY LIKE AN EDIT THAT WORKED**, caught only because **CI disagreed with a local pass and I read the failing check's NAME instead of assuming my fix was live.** **I could not prove the fix by reverting it: with today's date equal to the injected date the two clocks agree, so the bug was invisible — the evidence is the direct clock-sensitivity measurement, NOT a red-then-green test.** **Limits: the clock is injected per-process, not per-call; `parity` still hard-codes the instant in two places and a stale one will eventually fall outside the ledger window; `--at` is public on `status`, so such output is not current state; and I did not look at the other hard-coded instant at `parity.py:484`.** **This entry is itself a correction: the original commit message claimed a current-state entry that did not land, because the insert's anchor did not match and the script failed silently while the commit proceeded — the same silent no-op class, twice in one unit.**

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

**Next:** `lint`/`prlint` (one ~1,100-line unit, 13 gates left), the machine-readable-card question for `registry_vs_github`, the `OWNER-*` scope question, the four-rule unportable decision, `panel`, the `scripts/` tail, and `git switch`/`delete-merged` with their guards.

**Archive boundary — 2026-10-18 (course-correction PR-D).** Every entry above this line
is dated 2026-10-14 or newer: this page is a rolling window, not the full record. Entries
for 13 October back to 3 October are preserved verbatim, with their anchors, in
[current-state-2026-10-03-to-2026-10-13-archive](../history/current-state-2026-10-03-to-2026-10-13-archive.md).

## Active execution contract — grounded conversation and durable memory, October 1

The owner adopted the [canonical plan](project-track.md) under
[D19](DECISIONS.md#d19--grounded-conversation-and-durable-memory-first). The first
product priority is grounded conversation and durable memory; full alpha retains
coding/reasoning on the same native geometric model. [Owner direction of 3 October](https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5971681179):
no transformer baselines; make the geometric mechanisms work and compare geometric
variants with each other and with the previous best geometric model.

**Next integrated deliverable (D19):** saved learned relation/act/span compiler →
existing exact versioned store → real emitter, using predicted inputs and the
model's actual generated history. Reuse exact identity/store components; do not
spawn a new engine. The full contract text, the D18 outcome and the earlier
contracts are in the [history file](../history/current-state-2026-09-25-to-2026-10-02.md#active-execution-contract--grounded-conversation-and-durable-memory-october-1).

## Older entries and authority

- Entries from 13 October back to 3 October:
  [current-state-2026-10-03-to-2026-10-13-archive](../history/current-state-2026-10-03-to-2026-10-13-archive.md).
- Entries from 2 October back to 25 September (capture/age, q4 bridge, persistent
  integer attention, geometric values/NoRead/context studies, D19 grounded session,
  D18 outcome D, durable-lab contracts, the fourth-lab record and earlier retained
  results): [current-state-2026-09-25-to-2026-10-02](../history/current-state-2026-09-25-to-2026-10-02.md).
- Everything before 25 September:
  [current-state-archive-through-2026-09-24](current-state-archive-through-2026-09-24.md).
- Start routine work from this page, the [plan](project-track.md),
  [decisions](DECISIONS.md) and the exact source/artifacts for the active rung.
  **Pre-alpha; no useful general-language, coding, frontier, geometric-advantage or
  full-path energy qualification.**
