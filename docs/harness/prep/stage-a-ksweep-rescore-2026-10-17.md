# Pre-registration — Stage-A K-sweep re-score

## 5.1 Header

- **Date / owner:** 2026-10-17 / Kimi PM (kimi/pm1), drafted by DeepSeek design seat
- **Base:** `origin/main = d325c3037b5a6cebdf9401636f9ac0cf91a259fe`
- **Authority:** [Stage-A sweep record](../../labs/rung0-stage-a-sweep-2026-10-15/README.md); [Stage-A instrument](../../labs/stage-a-admission-instrument-2026-10-15/README.md); [admission sheet](m1-discrete-rung0-admission-2026-10-15.md); [M4 sheet](m4-finite-relational-head-2026-10-15.md).
- **Line:** M1 qualification; count **0/3**, suspended headline **37/232**, unchanged. No mechanism negative, KEEP or count reset.
- **Delivery:** docs only; all new observations `NOT_RUN`. This sheet is a re-analysis design, not a training release.
- **Scope:** diagnostic. Training/M2/Stage B **HELD**; Q4 **NOT_EXECUTABLE**; no K>8-trained lineage is run. FIT releases no hold.

## 5.2 Question and smallest falsifiable hypothesis

**Question.** At the tested scope — the four preserved seed-1 quarter checkpoints, `C192 / B4 / V4096`, `ReadSpec::Route`, K=8..16, frozen 14,287-row DEV panel — does the sealed Stage-A 36-point table reproduce exactly (integrity), and what does its Q4 sub-panel composition say about K (new information)? The aggregate ceiling and unconditional top-1/top-8 curve are already published (2,400 at K=16/Q4; 3,242/3,206/2,509 at the earlier quarters; Q4 top-1 149→145 from K=8→16), so **H1 is an integrity reproduction, not new information**. The new information is the fixed-admitted-set prediction test (H2) and the copy-successor/complement composition (H3).

**H1 (integrity reproduction).** The sealed 36-point record reproduces exactly, including the four K=8 full-panel stored-value controls and the Q4 K=8 sub-panel controls (`P_succ`, `P_comp` and the four buckets), and a host re-analysis of its per-row records is admissible without a new GPU run. H1 is a record-integrity check only; it adds no ceiling evidence.

**H2 (fixed-admitted-set widening), with the multiplicity rule.** The **primary comparison** is Q4 K=8 vs K=16 on the fixed admitted set `A_8` (the 1,920 rows whose target is admitted at K=8): compare top-1 hits at K=8 and K=16 on the identical row set; floor `max(24, 2·sqrt(b+c))`. This is the decision-relevant endpoint and holds the admitted population fixed. H2 predicts that the primary comparison does not rise by the floor; widening K adds admitted rows (1,920 → 2,400 at Q4) but not conditional top-1.

All other checkpoint/K steps (Q1–Q3, and K=8..15 within each quarter) are **secondary/descriptive**: they are reported with the same fixed-set discordance, but a secondary firing is hypothesis-generating only and **cannot disconfirm H2 alone**; a secondary-triggered conclusion requires confirmation on the primary comparison. The 4 checkpoints × 8 K-steps = 32 comparisons are correlated (the `A_K` sets are nested), so they are not independent tests; the primary-comparison rule replaces a Šidák bar.

**H3 (sub-panel composition).** H3's metric is **top-8**, because the M4 `B1` primary bar is top-8, the 39-row floor is derived from the Q1–Q3 `P_succ` top-8 rates, and top-8 has substantially more events than top-1 on this sub-panel (170 vs 29 at Q4 K=8). Let `Δ_succ = top8_succ(K=16) − top8_succ(K=8)` and `Δ_comp = top8_comp(K=16) − top8_comp(K=8)` at Q4. H3 predicts that the two panels respond differently to K: `|Δ_succ − Δ_comp| ≥ max(39, 2·sqrt((b_succ+c_succ) + (b_comp+c_comp)))`, where `b/c` are the paired discordances against the K=8 baseline on each panel. The 39-row floor is **top-8-derived and conservative**; top-1 is reported descriptively.

**Disconfirming observations.**
- H1 is disconfirmed by any K=8 full-panel or Q4 K=8 sub-panel control mismatch, or any of the 36 points not reproducing; that is an instrument fault (D31), not a finding.
- H2 is disconfirmed **only** if the primary Q4 K=8 vs K=16 comparison on `A_8` rises by ≥ `max(24, 2·sqrt(b+c))`. A secondary checkpoint/K-step firing is hypothesis-generating only and does not disconfirm H2 by itself.
- H3 is disconfirmed if the sub-panel difference is below its bar. Conflict/complement splits are descriptive and cannot fire a decision.

## 5.3 Observable signature, controls and design

**Observable signature.** Per (checkpoint, K) cell: exact integer counts of `top1_correct`, `top8_correct`, `strict_wins`, `admitted_strict_wins`, `target_admitted_rows`, plus per-row joins to `P_succ`, `P_comp`, `P_any_succ`, the four distance buckets, and the admitted/conditional split. New: the **primary** Q4 K=8 vs K=16 fixed-`A_8` top-1 `b/c`; the **secondary** per-checkpoint, per-K-step fixed-`A_K` `b/c` (descriptive); and the Q4 K=8 vs K=16 per-panel top-8 deltas `Δ_succ`, `Δ_comp` with their `b/c`. Unconditional counts and conditional rates (`top1/B3`, `top8/B3`) are reported descriptively; the primary fixed-admitted-set paired discordance is the decision quantity.

**Strongest simple controls.** The four K=8 full-panel stored-value cells must reproduce the sealed quarter counts exactly: 96/1090/177/41/2201; 94/967/178/42/2131; 96/953/214/44/1954; 149/747/105/65/1920 (`rung0-stage-a-sweep-2026-10-15/README.md:50-64`). In addition, the Q4 K=8 sub-panel controls must reproduce **29/170/435/11/20** on `P_succ` and **120/577/1485/54/85** on `P_comp` (`relation-subpanel-audit-2026-10-15/README.md:62-63`), with bucket row counts `[50, 2872, 900, 25]` and bucket values `d_1_8` 0/3/5/0/0, `d_9_32` 25/144/364/11/19, `d_33_96` 4/23/62/0/1, `d_97_192` 0/0/4/0/0. A wrong sub-panel join must fail these controls.

**Primary design (host-only, $0 GPU).**
1. Fetch the sealed archive `icloud:UOR-R4/results/kimi/stage-a-sweep-1-20261015.tar` (MD5 `b274beb321ce9232bd3690659fee7890`, 33,802,240 bytes); verify the manifest and all 38 blake3 hashes.
2. Parse `sweep-1/report.json` and the 36 `evaluation-*.json` records (33,627,092 derived bytes). Each record is row-lossless (`row_columns`/`rows`); join by row ordinal.
3. Re-derive the 36-point table; verify it against `rung0-stage-a-sweep-2026-10-15/README.md:72-109`.
4. Join to the DEV partition by ordinal; verify `ordered_row_binding_sha256 = e4ef44fb…` and the `P_succ`/`P_comp` counts (3,847 / 10,440 / 1,335).
5. Verify the Q4 K=8 sub-panel controls and bucket counts above; any mismatch is a sub-panel/instrument fault.
6. Compute the fixed-admitted-set analysis: the **primary** Q4 K=8 vs K=16 comparison on `A_8`; then the secondary per-checkpoint, per-K-step `A_K` comparisons descriptively.
7. Compute H3's `Δ_succ`, `Δ_comp`, their paired `b/c` and the quadrature bar.
8. Verify per-row monotone target admission across K (the sealed record already did this; re-check).

**Fallback design (GPU, only if the sealed record is missing or fails verification).** Re-run the 36 evaluations with the reviewed Stage-A instrument (executable `712cd0cc…`, source `719f38d9…`, compiled model `e1836149…`, kernel `931a511b…`, geometry `46ddbb76…`) on the same four checkpoints, same 14,287 DEV rows, K=8..16, 224 batches each, zero-apply. <15 GPU-minutes, <$0.25; one 1-hour lease at $0.908/h = $0.908, one RTX 5090 through `scripts/pod/uor-pod` (Vast.ai). This is a **record-recovery** run, not a new ceiling measurement.

**Dose.** 36 host re-analyses, or 36 GPU evaluations in the fallback (4 checkpoints × 9 K values). No training; no K>8-trained lineage; no checkpoint mutation.

## 5.4 Bars and decision rule

**Bars.** Primary fixed-admitted-set comparison (Q4 K=8 vs K=16 on `A_8`): `max(24, 2·sqrt(b+c))`. **24 rows** is a rounded 2-SE floor at 149/14,287 (2 SE = 24.3; ≈23.4 at 149/1,920); the paired `2·sqrt(b+c)` bar is operative. Secondary checkpoint/K-step comparisons use the same floor but are descriptive. H3 bar: `max(39, 2·sqrt((b_succ+c_succ) + (b_comp+c_comp)))`; **39 rows** is the top-8-derived, conservative `P_succ` floor (`m4-finite-relational-head-2026-10-15.md:30-31`). Full-panel descriptive floors: 24 (top-1), 63 (top-8), 87 (B3); the 3,847-row `P_succ` end-to-end floor is 39 (top-8-derived). Paired discordance `c − b` with null SE `sqrt(b+c)`. Wilson 95% intervals for every cell. The G-A gate 4,894 is reported as the reference, not re-litigated.

| Precedence | Observation | Interpretation | Next decision |
| --- | --- | --- | --- |
| 0 | Any K=8 full-panel or Q4 K=8 sub-panel control mismatch, manifest/hash/row-binding mismatch | INSTRUMENT FAULT | Diagnose/repair (D31); not a finding |
| 1 | Sealed record missing and the GPU fallback unavailable | UNAVAILABLE | Do not infer a ceiling; no Stage-B release |
| 2 | All 36 points reproduce; all K=8 controls exact | H1 integrity PASS; the sealed record is valid | Proceed to the primary H2 comparison and H3; no GPU re-run |
| 3 | **Primary** Q4 K=8 vs K=16 on `A_8` rises by ≥ its floor | H2 disconfirmed: widening buys prediction too | Stage-B K choice uses `J(K)` and the smallest K that maximizes it; report the secondary per-step table |
| 4 | A **secondary** checkpoint/K-step rises by ≥ its floor while the primary does not | Hypothesis-generating only; cannot disconfirm H2 alone | Record it in a descriptive note; the primary result stands, and confirmation requires the primary comparison |
| 5 | Primary does not rise; secondary steps contain a ≥-floor sign change (at least one falls by ≥ its floor and at least one does not fall; excludes a secondary rise — row 4) | H2 supported (admission-only), non-uniform (fall+flat) | Report every step; do not summarize as a single slope; Stage-B must change admission/selection, not just K |
| 6 | Primary does not rise; secondary steps are monotone flat/falling (no ≥-floor sign change) | H2 supported (admission-only), flat/falling | Stage-B design must change the admission/selection policy, not just K |
| 7 | H3 supported: sub-panel difference ≥ its bar (modifier) | The panel composition matters | Panel-weight the Stage-B K choice with `J_succ = 2·top8_succ + 1·top8_comp` |
| 8 | H3 not supported (modifier) | No registered sub-panel divergence | Use `J_full = top8_full` (1:1 weights); do not weight by sub-panel |

Rows 0–6 select the mechanism read and the admissibility of a K-widening; rows 7–8 select the panel weighting and apply only after row 2. A row-3 result (H2 disconfirmed) and a row-7 result can coexist: the primary and secondary tables say whether widening buys prediction, and H3 says how to weight the K choice.

**Multiplicity rule (binding).** The primary comparison is a single pre-registered test at the decision-relevant endpoints (Q4, K=8 vs K=16, `A_8`). The 32 checkpoint/K-step comparisons are correlated (nested `A_K` sets) and are not independent; no Šidák or Bonferroni correction is applied because the family is not used as a family of decision tests. Secondary firings are recorded as descriptive evidence and cannot disconfirm H2 without primary confirmation.

**Panel-weighting rule (pre-registered, applied only after H1 passes).** For K ∈ {8..16}:
- if H3 is supported, `J(K) = 2·top8_succ(K) + 1·top8_comp(K)` (equivalently `top8_full(K) + top8_succ(K)`); the integer weights 2:1 put the M4 primary sub-panel first while retaining the complement as a non-regression term;
- if H3 is not supported, `J(K) = top8_full(K)` (1:1 weights).
- `K* = argmax J(K)`; ties break by **smaller K**, then **larger `top1_full(K)`**, then **larger `B3(K)`**. No post-hoc reweighting. This is a decision aid, not a capability claim; it cannot fire if H1 fails.

**What changes the decision.** The published ceiling (2,400 vs 4,894) is fixed. The new numbers are the **primary fixed-admitted-set comparison** (Q4 K=8 vs K=16 on `A_8`) and the **Q4 sub-panel top-8 deltas**. If the primary comparison is flat or falling, the Stage-B design must change the admission/selection policy, not just K. If it rises, widening buys prediction and `J(K)` selects the K. Secondary steps are descriptive and cannot substitute for the primary.

## 5.5 Instrument validity, lineage, resource projection, stop conditions

**Instrument validity.** The K=8 controls are the qualification; any mismatch is an instrument fault (`stage-a-admission-instrument-2026-10-15/README.md:20-38`). The Q4 K=8 sub-panel controls (29/170/435/11/20 and 120/577/1485/54/85) and bucket counts `[50, 2872, 900, 25]` must reproduce exactly; a wrong sub-panel join is a fault, not a composition finding. The archive hashes and the 38-file manifest must verify before analysis. The per-row monotone-admission check must pass. The DEV row binding must match `e4ef44fb…`. The Q4 evaluation receipt must match `4fff1d3c…`.

**Loaded artifacts.** Base `origin/main = d325c3037b5a6cebdf9401636f9ac0cf91a259fe`; the four checkpoints `a5a22dce8a174bbfe368cd8d166170a6c34bf59137b1418943a9d0422cae4cdd` (Q1, 315 accepted updates), `dde889aa6137c0e28e18dd5ae99a823b6850342286a0d065bdaa82de49b05822` (Q2, 366), `102beed4608f1ddd70d68039bdae8d9372b3c05de2dbc4bc98b96d7595e40982` (Q3, 384), `4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf` (Q4, 407); the 36 evaluation records (33,627,092 derived bytes); the DEV panel (`0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`); the Q4 evaluation `4fff1d3c04ff6107666b2b8281ef2bbedf3019eeff6bff682bbbb8687bf888c8`. No generation; teacher-forced scoring only.

**Stage-A instrument identity (full hashes from the sealed identity record).** Executable `712cd0cc7e0d7934c136575b0f4bf6cc9bccd9b5709e8eb9b722e5d92b5efc02`; instrument source `719f38d990ffad9053ee5d1b2c6bbfafabdb7cd3274943099395d3ffdf58bb84`; compiled model `e18361495276e491525674fb3fdce308b0ad787c7fd6860a811adb60dacd9e1a`; kernel `931a511bece20641ae00c6b0e943f4f2a380c0f35a824d85fcc0447f1e1ccf5b`; geometry `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`. Driver source `70b45863860c25cf3579e76367b785ea4830edbc50e3902ef4a5ca2335bf7676`; stored Q4 driver `cb090e85e78805acd2b2347373340168a9e2f87aaf72a1dcb42f091e9ff1f7b4`. The TRAIN pins `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` / `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` are carried by `docs/labs/k16-qualification-restart4-2026-10-16/data-file-sha256.txt`.

**Lineage.** The sealed archive (MD5 `b274beb321ce9232bd3690659fee7890`, 33,802,240 bytes), the 36-point table, the Stage-A instrument identity, the Q4 receipt, the DEV binding, and the Q4 K=8 sub-panel control table. The archive's 38 blake3 hashes and the cloud MD5 round-trip are the preservation record.

**Resource projection.** Host-only re-analysis: **$0 GPU**, seconds to minutes of host work, <50 MiB derived records. GPU fallback: <15 GPU-minutes, <$0.25, one 1-hour lease at $0.908/h = **$0.908**, one RTX 5090 through `scripts/pod/uor-pod` (Vast.ai). No training; no new spending class.

**Stop conditions.** First hash/manifest/row-binding/K=8-control/sub-panel-control mismatch stops the analysis and preserves the partial record for PM disposition. The GPU fallback stops on identity, parity, epoch/apply, resource or disk-floor faults. No on-lease patch or extra sample.

**Record schema.** `uor-r4.stage-a-rescore/1`, with `applied_updates: 0`, `model_compute_executed: false` (host) / `true` (fallback), per-row records and exact aggregates, source/checkpoint/evaluation hashes, the `P_succ`/`P_comp` partition binding, the Q4 K=8 sub-panel controls, the primary `A_8` K=8 vs K=16 `b/c` table, the secondary per-checkpoint/K-step `b/c` table, and the H3 `Δ_succ`/`Δ_comp` table.

## 5.6 D22, claim discipline, FIT and assumptions

**D22.** The G-A failure already closed **eval-time widening of the K=8-trained codebook** — a configuration, not a mechanism. A K>8-trained lineage remains an untested configuration. This re-score cannot reopen the closure and cannot close the mechanism.

**Claim discipline.** No capability claim. H1 is a record-integrity check; H2/H3 are composition analyses of a sealed diagnostic, not fresh transfer and not a Stage-B result.

TEST FITNESS: FIT (design) — Kimi PM seat with independent DeepSeek architecture review, [PR #2572](https://github.com/UOR-Foundation/uor-r4/pull/2572) at `f53c5198a`, copyedits `b67567d01`. FIT authorizes no compute; execution releases are separate PM steps.

**FIT / execution release.** Design FIT only; no compute. Q4 remains **NOT_EXECUTABLE**. The host re-analysis is non-model work; the GPU fallback needs a separate PM release and exact pins. Training remains HELD.

**Assumptions the lead must resolve.**
1. Whether the iCloud archive is accessible and current; if not, whether a GPU fallback is authorized.
2. Whether the sealed per-row records contain the fields needed for the primary `A_8` and secondary fixed-set joins and the sub-panel joins (the README says row-lossless; I did not open the archive).
3. Whether the PM wants the host re-analysis only, or a GPU replication as a record-integrity control.
4. Whether the H3-supported 2:1 `J(K)` weighting (or the 1:1 full-panel fallback) is the intended Stage-B K rule.
5. Confirm the 24-row rounded floor, the paired `2·sqrt(b+c)` bar, the 39-row top-8-derived sub-panel floor, and the primary-comparison multiplicity rule.
