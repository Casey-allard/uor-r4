# Pre-registration — M1 learned-attention link

## 7.1 Header

- **Date / owner:** 2026-10-17 / Kimi PM (kimi/pm1), drafted by DeepSeek design seat.
- **Authority:** owner priority on issue #2029, posted 2026-10-17 04:42Z, "close the missing learned-attention link"; PM disposition [#2029 comment 6229949186](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6229949186) accepting it as the active M1 line; [D22](../../integration/DECISIONS.md) (a negative closes a configuration, never a mechanism); [D24](../../integration/DECISIONS.md) (no CPU model compute; no GPU on non-decisive work); [D32](../../integration/DECISIONS.md) (integer-only substrate, no softmax/matmul/float); [D34](../../integration/DECISIONS.md:1642-1660); [finite-relational-head](../../mechanisms/finite-relational-head.md); [M4 sheet](m4-finite-relational-head-2026-10-15.md); [D34 successor-table control](d34-successor-table-control-2026-10-17.md).
- **Line:** M1 qualification; count **0/3**, suspended headline **37/232**, unchanged. No mechanism negative, KEEP, acceptance or count reset.
- **Delivery:** **docs only**. This sheet authorizes **NO training**. All new observations are `NOT_RUN`. The interleaver, the +1 leaf combine-gate initialization and the Gate-admitting learning entrypoint are **Unit 1**, separately assigned to Codex. This sheet binds their reviewed contract; it does not implement them.
- **Scope:** trained-in discriminator design, not a capability claim. **Q4 NOT_EXECUTABLE**; Training/M2/Stage B **HELD**. FIT releases none of these holds. Execution needs FIT + PM release; any D-Release-class decision needs owner release. This configuration is not D-Release-class.
- **Hard dependency:** the frozen Route completion executor rejects Gate receipts (`completion.rs:744-747`). A Relation artifact cannot be produced by the existing completion path. Unit 1's separately reviewed, opted-in Gate-admitting entrypoint is a precondition, not a design choice.

## 7.2 Question and smallest falsifiable hypothesis

**Question.** Does a **trained-in** `ReadSpec::Relation { kept: 8, sweeps: 1 }` artifact — the first learned Relation checkpoint — outperform a **matched relation-disabled control** under the identical declared initialization, and the **T2 context-free successor table**, on the 3,847-row copy-successor panel (`P_succ`) **and** its 10,440-row complement (`P_comp`), with actual Relation target admission reported separately from conditional token rank, and with a declared fresh-transfer confirmation?

**Fixed definitions.**
- `P_succ` = 3,847 rows (26.9266% of 14,287); `P_comp` = 10,440 rows; conflict rows = 1,335 (`P_any_succ \ P_succ`); buckets `[50, 2872, 900, 25]`; zero at `d=1`; `ordered_row_binding_sha256 = e4ef44fb…`.
- `B3_R` = the Relation arm's own target-admitted rows on the evaluated slice. `B3_C` = the control's own target-admitted rows. `B3_I` = the intersection of `B3_R` and `B3_C` on `P_succ`. **`B3_I` is the primary paired denominator; `B3_R` and `B3_C` are reported separately and are not substituted for it.**
- `T2` = the context-free TRAIN successor table from [D34 §6.4 item 3](d34-successor-table-control-2026-10-17.md): pairs `(tokens[p-1], tokens[p])` at the exact masked TRAIN targets the Relation artifact consumed; most frequent successor; tie by lowest token id; model-matched unigram fallback. It is **rebuilt on the actual training budget** of the new artifact (16,404 batches → 1,049,856 targets; 32,808 → 2,099,712), not fixed to the seed-1 1,049,856.
- `U_succ` = the model-matched unigram top-1/top-8 computed on `P_succ`, not the full-panel 1,226.
- **T2 restrictions (stated, not inferred).** `T2_R` = T2 restricted to the Relation arm's admitted cells on `B3_R` (H_geom's Relation-vs-T2 test). `T2_C` = T2 restricted to the control's admitted cells on `B3_C`. `T2_I` = T2 restricted to the common `B3_I` admitted cells; **H_null uses `T2_I` for both arms.** No arm is compared to a T2 restricted to a different arm's cells.
- `F_I = max(24, 2·sqrt(b_I + c_I))` on `B3_I`; `F_{T,R} = max(24, 2·sqrt(b_{T,R} + c_{T,R}))` on `B3_R` (Relation vs `T2_R`); `F_{T,I} = max(F_{T,I,R}, F_{T,I,C})`, the larger of the two paired floors for Relation vs `T2_I` and control vs `T2_I` on `B3_I`; `F_A = max(24, 2·sqrt(b_A + c_A))` on the trained-vs-ablation paired rows; `F_C` = the complement non-regression floor: **`max(22, 2·sqrt(b+c))` top-1** (the complement-specific 22 is the rounded 2 SE at 120/10,440 ≈ 21.8 rows), 63 top-8, 87 B3.

**H_geom (learned relation link).** On `B3_I`, the trained Relation arm's conditional top-1 **exceeds the matched relation-disabled control by `F_I`** (`Relation ≥ control + F_I`); on `B3_R`, it exceeds `T2_R` by `F_{T,R}` (`Relation ≥ T2_R + F_{T,R}`); on `P_comp`, it does not fall below the control by more than `F_C` (`Relation ≥ control − F_C`); and the within-checkpoint zero-relation ablation loses at least `F_A` on `P_succ` relative to the trained Relation. The fresh transfer reproduces the `P_succ` direction and floor.

**H_control (no learned relation effect).** On `B3_I`, **`control ≥ Relation + F_I`**, or the within-checkpoint zero-relation ablation is within `F_A` of the trained Relation. The relation trits add nothing at this configuration.

**H_null (frequency only).** On the common `B3_I` cells, **`T2_I ≥ Relation + F_{T,I}` and `T2_I ≥ control + F_{T,I}`** for both arms. The sub-panel is explained by token-frequency/successor identity, not contextual learning. `U_succ` is the marginal reference.

**H_mixed / INCONCLUSIVE.** **All within-floor results** — `|Relation − control| < F_I`, or `|Relation − T2_R| < F_{T,R}`, or the both-arm H_null within `F_{T,I}` but not a strict H_null — are H_mixed/INCONCLUSIVE, not H_geom or H_control. Also: a `P_succ` win with a complement regression beyond `F_C`; or `B3_I < 200`/observed power <80%. No decision fires.

**Disconfirming observations.**
- H_geom is **disconfirmed** by any of: `Relation < control + F_I`; `Relation < T2_R + F_{T,R}`; complement regression beyond `F_C`; ablation drop `< F_A`; fresh-transfer failure.
- H_control is **disconfirmed** by `Relation ≥ control + F_I` **and** ablation drop `≥ F_A`.
- H_null is **disconfirmed** by **`Relation ≥ T2_I + F_{T,I}` OR `control ≥ T2_I + F_{T,I}`** (one arm beating the table suffices to reject frequency-only).
- All within-floor results are **H_mixed/INCONCLUSIVE**, never H_geom/H_control/H_null.
- All hypotheses are subject to the §7.6 evaluability rule and the §7.8 initialization/activation checks. If the initialization check fails, no hypothesis fires.

**Initial-unobservability statement (binding).** At the current zero init, `discrete_init` sets `gates[i]=0` for the whole inventory (`discrete.cu:358`); the combine path skips a zero gate (`discrete.cu:227`, `:258`), and the relation scores skip a zero trit (`discrete.cu:182`). With all gates zero, the Flock/Relation read outputs `snap(query,1)` — the query root per block — so relation-trit changes are **causally inert**. A null result under the zero-init configuration would be uninterpretable and is **not** a mechanism negative. The +1 leaf combine-gate init below is the declared amendment that makes the relation path observable and the test falsifiable.

## 7.3 The missing learned-attention link and the declared initialization amendment

**Source facts on current main (`fa62a670a`).**
- `SchedulePolicy` has only `CanonicalVectorThenReadV1` (`training.rs:31-35`). `InterleavedReadEvery8V1` exists only as a plan in the M4 sheet; it is not implemented.
- At `V4096/B4/C192`, `read_parameters = gate_count = 20 + 25 = 45`; `coordinates = 16,384 + 45 = 16,429` (`lib.rs:140-153`; `training.rs:184-193`).
- A 16,404-batch canonical pass executes coordinate indices `0..16,403`. The read coordinates are the last 20 (`16,384..16,403`), i.e. **Gate(0..19) = the 20 level-combine gates**. It reaches **zero relation-trit coordinates** (`Gate(20..44)`). The dispatch's shorthand "ZERO Gate coordinates" is imprecise: it is zero **relation** Gate coordinates; 20 combine-gate coordinates are reached, once each.
- `completion.rs:744-747` admits only `VectorBlock` and `Route` receipts and returns `Err("completion never admits flock gates")` on any Gate receipt. `PreparedDriver::transition` is coordinate-kind agnostic (`training.rs:859-897`), so the missing link is the **entrypoint's admission of Gate receipts**, not the transition math.
- The read is integer and writer-shared (`discrete.cu:125-267`, `:307-345`); no dense/float path is introduced.

**Declared amendment: +1 leaf combine gate per payload block.**
- Set `gates[b*stages + 0] = +1` for `b = 0..3` (at `stages = 5`: Gate(0), Gate(5), Gate(10), Gate(15)), identically on the Relation arm and the matched relation-disabled control.
- Effect: the level-0 consensus is added to the query residual (`terms = 2`), so the read is context-sensitive at initialization and the **5 level-0 relation trits** (`Gate(20..24)`, edge types `e=0..4` at level 0) become causally active. The 20 higher-level relation trits remain inert until their combine gates learn a nonzero value; they are learnable under the interleaved schedule.
- It is **architecture-selected before data/results**, recorded in the run configuration with the exact four indices and values, and never derived from the target token, the DEV evaluation, or any answer index.
- As implemented in Unit 1 (#2575, source-reviewed through `d833da01d`), `ReadInitialization::ActiveLeafV1` selects a deterministic literal gate-storage upload **after the byte-unchanged production `discrete_init`**, followed by GPU storage readback of the complete literal pattern at epoch 0. This is declared initialization storage, with no learning apply, target-dependent computation or epoch increment. The production initializer ABI/body, zero/default initialization and its serialization are preserved. The four indices and values are recorded in the run configuration. Any deviation is an instrument fault. This corrects #2577's description of the superseded `f6c7fa5e1` internal selector; the declared initialization effect, hypotheses, bars, controls and dose are unchanged.
- **Pin/bridge disclosure (binding).** Serialization is preserved, not pin acceptance: `Device::from_snapshot` rejects snapshots whose stored source/kernel hashes differ from the running build, so the frozen Q4 artifacts (`4477eb5c…`, evaluation `4fff1d3c…`) are refused under Unit-1 source unless the separately reviewed evaluation-only bridge accepts/re-pins them. `Snapshot` is `deny_unknown_fields` with `read_initialization` defaulted: old snapshots read fine under new code, but new ActiveLeafV1 snapshots cannot be read by a frozen old reader. Both boundaries are named here so the release packet prices and reviews the bridge explicitly.

**Matched relation-disabled control (primary design).**
- Use `ReadSpec::Relation { kept: 8, sweeps: 1 }` with the 25 relation trits **pinned to 0** and their proposals refused/no-op, so the control keeps the **same 45-coordinate cycle**, the same interleaved schedule, the same +1 leaf combine init, and the same combine-gate proposal frequency as the treatment.
- Do **not** use `ReadSpec::Flock` as the primary control. A Flock model has a 20-coordinate read cycle; at 16,404 batches its combine gates receive ~102–103 proposals each, versus ~46 in the 45-coordinate Relation arm. That is a proposal-frequency confound, not a matched control. The Flock arm may be retained as a secondary relation-off ≡ Flock identity check, labeled with the confound.
- The only permitted difference between the treatment and the primary control is whether the 25 relation trits are learnable/active.

## 7.4 Controls and matching

| Control | Role | Matching |
| --- | --- | --- |
| **Relation arm** | Treatment: `ReadSpec::Relation { kept: 8, sweeps: 1 }`, interleaved, +1 leaf combine init | — |
| **Relation-off control (primary)** | Same `ReadSpec`, same 45-coordinate cycle, same +1 leaf init; 25 relation trits pinned 0, relation proposals refused | Exact spec/schedule/init/seed/budget/objective/corpus match; only relation enablement differs |
| **Within-checkpoint zero-relation ablation** | Trained Relation checkpoint, in-memory `gates[20..45]=0`, combine gates and codebook unchanged; zero-apply | Same checkpoint, same rows; attribution, not retraining |
| **T2 context-free successor table** | Ordinary comparator: does contextual learning beat frequency/token identity? | Same actual training targets; admission-restricted to the Relation arm's admitted cells per D34 §6.4 item 7 |
| **U_succ** | Marginal reference on `P_succ` | Model-matched 1,049,856 or 2,099,712 targets, depending on budget |
| **Flock arm (secondary only)** | relation-off ≡ Flock identity check | Same +1 leaf init; **not** proposal-frequency matched (20- vs 45-coordinate cycle) |
| **Route arm (optional only)** | Additional comparison if the PM requires it | Not a causal control; a Route-only win cannot attribute a relation effect |

**Matched fields (must be identical on the treatment and primary control):** `Spec { vocabulary: 4096, blocks: 4, context: 192, kept_cells: 8 }`; `batch_rows = 64`; seed(s); ascending response-masked corpus order; objective policy (fixed at release); batch budget; interleaved schedule `s % 8 == 7`, `r = (s/8) % 45`; +1 leaf combine gates (0,5,10,15); shared writer (`discrete.cu:307-345`); geometry sha256 `46ddbb76…`; tokenizer `d36d3e87…`; the same opted-in Gate-admitting entrypoint; the same DEV and fresh evaluation path. The only permitted config difference is relation-trit enablement. Record both config hashes and the diff.

**Explicitly excluded:** no oracle source index; no answer-derived gate; no teacher or external logits; no hidden dense/float path; no DEV-selected setting. The +1 leaf gates are fixed before data/results; K, sweeps, schedule, seeds, budget and objective are fixed before training; the fresh draw is sealed before training.

## 7.5 Design and dose

**Unit 1 dependency (Codex, separately reviewed).** The interleaver, the +1 leaf combine-gate initialization and the Gate-admitting learning entrypoint are Unit 1. This sheet binds their contract: the entrypoint must admit Gate receipts, record per-coordinate `attempted / any_candidate_cost_differs_from_identity / accepted_updates` for `VectorBlock`, `GateCombine (0..19)` and `GateRelation (20..44)`, and leave the frozen Route completion executor unchanged. No on-lease patch; no modification of `completion.rs:744-747`.

**Training arms and budget.**
- Primary dose: `B = 16,404` batches per arm (one full interleaved cycle at `schedule_cycle = 16,404`), two arms (Relation + Relation-off control).
- Seed plan: **two seeds (1 and 2)** per arm as the reproducibility plan (§7.6 seed-floor bullet: reproducibility-only, not a seed-noise-decisive claim). One seed is the reduced-power fallback; the M4 39-row floor is a top-8 proxy, not a measured top-1 spread. The PM/owner chooses the seed plan and the cost.
- Power-up option: `B = 32,808` (two cycles) if the primary is underpowered; doubles the relation-trit attempts and the training cost. Not automatic.
- No training is authorized by this sheet. The dose is registered; execution needs FIT + PM release + a resource admission.

**Dose arithmetic — the relation coordinates must actually be reached.** Under `s % 8 == 7`, the read proposal index is `r = (s/8) % 45`; `Gate(0..19)` are combine gates, `Gate(20..44)` are relation trits. The table below is exact integer arithmetic for the candidate budgets.

| Batch budget | Read proposals | Combine proposals (Gate 0..19) | Relation proposals (Gate 20..44) | Per combine gate | Per level-0 relation trit (Gate 20..24) | Per higher relation trit (Gate 25..44) | +1 leaf gates (0,5,10,15) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 4,101 | 512 | 237 | 275 | 11–12 | 11 | 11 | 12/12/12/12 |
| 8,202 | 1,025 | 460 | 565 | 23 | 23 | 22–23 | 23/23/23/23 |
| 12,303 | 1,537 | 687 | 850 | 34–35 | 34 | 34 | 35/35/34/34 |
| 16,404 | 2,050 | 920 | 1,130 | 46 | 46 | 45 | 46/46/46/46 |
| 32,808 (power-up) | 4,101 | 1,826 | 2,275 | 91–92 | 91 | 91 | 92/92/91/91 |
| 16,404 canonical (current) | 20 | 20 | **0** | 1 | **0** | **0** | 1/1/1/1 |
| 18,432 (s%9==8, optional) | 2,048 | 920 | 1,128 | 46 | 46 (3 trits) / 45 (2 trits) | 45 | 46/46/46/46 |

The canonical pass reaches the combine gates once and the relation trits **zero** times. The interleaver reaches the relation trits 275 / 565 / 850 / 1,130 times at the four quarter budgets; the level-0 relation trits (the ones activated by the +1 leaf init) get 11 / 23 / 34 / 46 proposals. The +1 leaf-gate cells are stated in the exact order Gate(0)/Gate(5)/Gate(10)/Gate(15): 12/12/12/12, 23/23/23/23, 35/35/34/34, 46/46/46/46, 92/92/91/91. This is the whole point of the line.

**Training and evaluation sequence.**
1. Unit 1 source reviewed; exact source/PTX/config pins; the +1 leaf gate indices/values recorded; the relation-off control's pinned trits recorded.
2. Initialize both arms from the same deterministic `Device::new_with_read` seed/ordinal with `ReadInitialization::ActiveLeafV1` selected identically at epoch 0. Run the §7.8 causal-activation fixture before training.
2a. Pre-flight battery (§7.12.1): `init-corpus` + `eval-corpus` at the exact attention config; report `B3_init(P_succ/P_comp/full)` and T1/T1b/T2/U_succ; hard gate `B3_init(P_succ) ≥ 500`. `propose` Gate(22) and Gate(0) sensitivity probes on real `P_succ` rows; inert → stop. `qualify-relation` C2 ACTIVE/INERT plus the §7.8 item 3(iii) INIT LOCK; any failure stops before training for diagnosis.
3. Train each arm for `B` batches in ascending corpus order, one coordinate per batch, same objective. Record per-coordinate attempted/sensitive/accepted counts, the relation-trit ledger, and the combine-gate ledger.
4. At `B`, write both checkpoints; verify the treatment's relation-trit accepted updates and the control's zero relation-trit updates.
5. DEV scoring (zero-apply): one `Device::forward` over the 224 DEV batches per arm, capturing `predictions`, `admitted_cells`, `top1_correct`, `top8_correct`, losses and the hinge components. Report `B3_R`, `B3_C`, `B3_I`, and conditional top-1/top-8 on `P_succ`, `P_comp`, the conflict rows and the full panel. Report the unconditional/ITT top-1 row on all 3,847 `P_succ` rows (§7.12.5).
6. Within-checkpoint zero-relation ablation: copy the trained Relation snapshot in memory, set `gates[20..45] = 0`, leave `gates[0..20]` and the codebook unchanged, evaluate the same 224 DEV batches zero-apply. Report the paired deltas on `P_succ`, `P_comp`, the full panel and `B3`.
7. Fresh transfer: the fresh draw stays sealed until after the paired DEV decision; score the treatment once after that decision; the relation-off control is DEV-only; no post-training fresh scoring until the runner fix lands (§7.12.3).
8. Host: build T2 and `U_succ` on the actual training targets; score them on the same rows with the D34 admission restriction; emit the paired 2×2 tables.

**Zero-apply statement.** No `apply`, proposal, optimizer or epoch advance during DEV, ablation or fresh scoring; `snapshot_unchanged: true`, `updates_applied: 0`. The training arms are the only place `.apply` occurs, and they are the registered dose, not an evaluation.

**No softmax / no matmul / no float / no teacher.** The design introduces no softmax, softplus, sigmoid, tanh, GELU or normalized exponential, and no matrix multiplication/GEMM. The read, writer, gates and tables are integer. No teacher, external logits, oracle index or answer-derived gate. Any prohibited site is a blocker under D23/D25/D32, not a design choice.

## 7.6 Bars and uncertainty

- **Evaluability (precedence over every hypothesis).**
  - `B3_I ≥ 200` with the MDE/precision rule (§7.12.6): the observed paired difference must be ≥ `F_I = max(24, 2·sqrt(b+c))`; `MDE_rel(B3_I) = 24/B3_I` (12.0% at 200; 5.5% at 435; 3.0% at 800) is reported for precision. The observed-power guard is retired. Report `B3_R`, `B3_C`, `B3_I`, `MDE_rel`, and the exact 2×2.
  - **Unconditional/ITT `P_succ` row.** `top1_ITT`/`top8_ITT` on all 3,847 rows (non-admitted = wrong), `F_ITT = max(24, 2·sqrt(b+c))`; co-reported, cannot rescue a conditional miss; a conditional win with an ITT regression is H_mixed (§7.12.5).
  - The initialization check (§7.8 items 1–3) passes.
  - T2 conservation passes; the fresh draw is sealed.
  - If any check fails, no hypothesis fires.
- **`P_succ` primary matched set, `B3_I`.** Paired discordance `c − b`; null SE `sqrt(b+c)`; `F_I = max(24, 2·sqrt(b+c))`. The 24-row minimum is the rounded 2-SE floor at 149/14,287; the paired term is operative.
- **Relation vs T2, `B3_R`.** Same paired rule, `F_{T,R} = max(24, 2·sqrt(b+c))`, with `T2_R` restricted to the Relation arm's admitted cells per D34 §6.4 item 7.
- **Full sub-panel, 3,847 rows.** The 39-row floor is the M4 `P_succ` floor; it is top-8-derived and conservative. Report end-to-end counts (Relation, control, T2, `U_succ`) and the admission-restricted upper bounds.
- **Complement, 10,440 rows (co-primary non-regression).** `F_C`: **`max(22, 2·sqrt(b+c))` top-1** (the complement-specific 22 is the rounded 2 SE at 120/10,440 ≈ 21.8 rows), 63 top-8, 87 B3. Report `B3_C` and conditional top-1/top-8 separately; a `P_succ` win with a complement regression beyond `F_C` is H_mixed, not H_geom.
- **T2 matching (stated).** H_geom uses `T2_R` (Relation-admitted restriction). H_null uses `T2_I` (common `B3_I` restriction) for both arms, with `F_{T,I} = max(F_{T,I,R}, F_{T,I,C})`. `T2_C` is reported for the control side. No arm is compared to a T2 restricted to a different arm's cells.
- **Zero-relation ablation.** Paired on `P_succ` and the complement; `F_A = max(24, 2·sqrt(b+c))`. Report the trained-vs-ablation 2×2 and the ablation's own `B3`.
- **Seed floor and second-seed role.** The M4 39-row floor is top-8-derived; the H_geom primary is top-1. This design **does not claim seed-noise-decisive power**. The second seed is a **reproducibility-only** check: it must reproduce the direction and the pre-registered paired floors, but no seed-to-seed top-1 spread is estimated from two seeds, and the 39-row proxy is not relabeled as a measured top-1 seed floor. The DEV decision uses the paired `F_I`/`F_{T,R}`/`F_A` floors and the fresh transfer; the second seed cannot rescue a DEV result that misses those floors.
- **Fresh transfer.** Pre-registered at seal time: ≥2,000 response-masked targets, ≥500 copy-successor targets at the ~26.9% base rate, same tokenizer and C192 windowing, disjoint from TRAIN/DEV. Floors: `max(20, 2·sqrt(b+c))` on the fresh matched set (top-1), with the exact counts bound before training. A DEV win without a fresh-transfer confirmation is DEV-only and is not a KEEP. Scored after the paired DEV decision; relation-off transfer is DEV-only (§7.12.3).
- **Wilson 95% intervals** for every rate. **Top-8** reported alongside top-1; the 63-row top-8 floor applies to full-panel and complement top-8 statements.
- **D22 power.** Two seeds is the reproducibility plan, not a seed-noise-decisive claim (see the seed-floor bullet). One seed is the reduced-power fallback. The fresh draw is the separate acceptance draw; it is not used for model selection.

## 7.7 Decision rule

Apply in order; the first matching row fires.

| Precedence | Observation | Interpretation at this configuration | Next decision |
| --- | --- | --- | --- |
| 0 | Init check fails; relation-off control has any nonzero relation trit or accepted relation update; frozen Route executor changed; same-writer/config/corpus/objective/budget mismatch; hash/epoch/apply fault; D23/D25/D32 site | INSTRUMENT FAULT | Diagnose/repair (D31); never a mechanism negative |
| 1 | `B3_I < 200`, or the observed paired difference is below the 24-row absolute floor / `MDE_rel(B3_I)` rule (§7.12.6); fresh draw not sealed; T2 conservation fails | UNDERPOWERED / INVALID | Report exact counts and `MDE_rel`; no hypothesis fires; lead resolves |
| 2 | `T2_I ≥ Relation + F_{T,I}` **and** `T2_I ≥ control + F_{T,I}` on the common `B3_I` cells | H_null: frequency/token identity explains the sub-panel | Diagnose the panel/table; no relation claim; do not close the mechanism |
| 3 | `control ≥ Relation + F_I`, or ablation within `F_A` of the trained Relation | H_control: no learned relation effect at this configuration | Close the configuration (D22); the mechanism family stays open |
| 4 | `Relation ≥ control + F_I` but `Relation < T2_R + F_{T,R}`; or a `P_succ` win with a complement regression beyond `F_C`; or any within-floor band (`|Relation − control| < F_I` or `|Relation − T2_R| < F_{T,R}`); or the both-arm H_null comparison is within `F_{T,I}` but not a strict H_null; or a conditional win with a `top1_ITT` regression beyond `F_ITT` (§7.12.5) | H_mixed / INCONCLUSIVE | Report the exact 2×2s and the failing condition; do not claim a learned relation link; lead decides whether to power up |
| 5 | `Relation ≥ control + F_I`, `Relation ≥ T2_R + F_{T,R}`, complement non-regression within `F_C`, ablation drop ≥ `F_A`, and the fresh transfer reproduces the direction/floor | H_geom: the learned **copy-successor** link is established at this configuration (ITT row co-reported, §7.12.5) | Proceed to the M4/M2 B1–B3 and the owner's next decision; no general capability claim |
| 6 | DEV meets H_geom but the fresh transfer fails | DEV-only diagnostic | No KEEP; report the failure; do not close the mechanism (D22) |

**What changes the decision.** The first decisive number is the paired difference on `B3_I` against the matched relation-disabled control. The second is the paired difference against T2. A win over the control but not over T2 is **frequency/context not yet separated**; a win over T2 but not over the control is **parameter/budget, not relation**. Only the joint win, with the ablation drop and the complement non-regression, supports H_geom.

## 7.8 Instrument validity

1. **Declared init check.** The loaded treatment and control snapshots have exactly `gates[0]=gates[5]=gates[10]=gates[15]=+1`; all other combine gates are 0; the control's 25 relation trits are exactly 0. Any mismatch is a fault.
2. **Control lock check (45-coordinate proof).** The control's recorded per-coordinate ledger must prove the **45-coordinate cycle**, not merely a zero relation count. At 16,404 batches it must show: `proposed(GateRelation)=1,130` and `refused(GateRelation)=1,130` (or an equivalent typed refusal/no-op count), `accepted_updates(GateRelation)=0`, and every relation-trit value still 0; `proposed(GateCombine)=920` with **46 proposals on each of Gate(0..19)** and `accepted_updates(GateCombine)` reported. `attempted(GateRelation)=0` alone is **not** proof: it cannot distinguish the 45-coordinate Relation-off control from a 20-coordinate Flock-like arm. The treatment's ledger reports the same inventory with relation proposals admitted. Any control relation-trit change, or a ledger that does not show the 1,130 relation-coordinate reaches, is a fault.
3. **Causal-activation fixture (three parts, before training).** The ACTIVE arm runs at the sealed C2 shape (`B=2, kept_cells=2, width=16`), not at `B=4`, because the B=4/kept_cells=8/width=32 ACTIVE literals cannot be derived with confidence from source alone: a long repeated prefix and the y/z chain change the eligible relation vector, and the address/rival scales differ. The C2 shape gives source-derived exact literals and exercises the same `flock_vector` relation path, the same `+1` leaf combine gate, and the same level-0 `e=2` relation trit. The training-shape four-gate initialization is checked separately by INIT LOCK (iii). All literals below are exact and committed in the test source; no runtime self-comparison, no tolerance.
   - **(i) ACTIVE (C2 shape: `Spec { vocabulary: 5, blocks: 2, context: 6, kept_cells: 2 }`, `ReadSpec::Relation { kept: 1, sweeps: 1 }`, seed 7).** Tokens: `tok_x=[B,A]`, `tok_A=[A,D_ROOT]`, `tok_B=[NEG_A,NEG_B]`, `tok_y=[C_ROOT,A]`, `tok_z=[D_ROOT,A]`. Prefix: `[tok_x, tok_A, tok_B, tok_y, tok_z, tok_A]` with `identity(0,0), identity(1,1), identity(2,2), identity(3,3), identity(4,4), identity(1,5)`; target `tok_B` (token 2). Gates: `gates[2]=+1` (block 1 leaf combine), all other combine gates 0; `gates[6]` (level 0, `e=2`) tested at 0 and +1. This is the sealed C2 gate layout; the training-shape four-gate init is INIT LOCK (iii), not this arm.
     - Baseline `gates[6]=0`: `head == [0,15,2,15]`; level-0 `summary == (vec![2,1,15,5], vec![1], vec![0], vec![15])` (kept node **1**, consensus root 15, eligible 5); output `[A,D_ROOT]`; loss **640**.
     - Active `gates[6]=+1`: eligible level-0 nodes `[0,1,2,4,5]` (node 3 is a level-1 parent) have relation scores **`[0,0,8,0,4]`**; node 5 (`tok_z`) is eligible with rel 4; the query atom is ineligible because `last+1 >= length`. Top[0] is node **2** (`tok_B`), not node 5: node 2 has rel 8; node 5 has rel 4. Baseline tie-break (all rel 0): node 1 wins on distance 0, hamming 0; kept node 1. Active: node 2 rel 8 wins on `rel desc` before any distance key; `head == [0,14,2,15]`; `summary == (vec![2,1,14,5], vec![2], vec![16], vec![14])` (kept node 2, action 16, carried root 14); output `[A,B]`; loss **448**. (Shape map: C2 `gates[2]` ↔ training `gates[5]` block-1 leaf combine; C2 `gates[6]` ↔ training `Gate(22)` level-0 `e=2`; relation trits are per level/edge, shared across blocks.)
     - **Writer-loss derivation (exact).** The writer is `loss_ta = 8·tm + width·am` with `width = 16` (`discrete.cu:326-340`). Baseline: target cell `19=(1,3)` address distance `8+8=16`; admitted cells are `15=(0,15)` at distance 0 and `32=(2,0)` at distance 8 (cell 240 ties at 8 and loses by cell id); `rival_address=0`; rival token `tok_A` distance 0; `tm = 16+16−0 = 32`, `am = 8+16−0 = 24`, loss `8·32 + 16·24 = 640`. Active: target cell 19 still distance `8+8=16`; admitted cells are 15 at distance 8 and 32 at distance 12 (240 ties at 12 and loses by cell id); `rival_address=8`; rival token `tok_A` distance 8; `tm = 16+16−8 = 24`, `am = 8+16−8 = 16`, loss `8·24 + 16·16 = 448`. The exact delta is `(32−24)·8 + (24−16)·16 = 64 + 128 = **192**` (`640 − 448`). The counterfactual `propose(Parameter::Gate(6))` on the baseline snapshot must return candidate losses `[640, 640, 448, MAX×13]` and select ordinal 2 (+1) with `640 → 448`, snapshot unchanged.
   - **(ii) INERT (initial-unobservability control, C2 shape).** Same tokens/prefix/target as ACTIVE, `gates[2]=0` (all combine gates zero), `gates[6]=0` vs `+1`. Assert `vectors()`, `losses`, `predictions`, `token_rivals`, `address_rivals` and `admitted_cells` are **bit-identical** between the two forwards; the internal trace/relation-score buffers may differ and are reported separately. If the output changes, the zero-gate path is not inert — fault, not a mechanism negative.
   - **(iii) INIT LOCK (training shape, unchanged).** The loaded treatment and control snapshots have exactly `gates[0]=gates[5]=gates[10]=gates[15]=+1`; every other combine gate is 0; the control's 25 relation trits are exactly 0. The four indices and values are recorded in the run config. Any mismatch is a fault.
4. **Frozen executor check.** `completion.rs:744-747` is byte-unchanged and still refuses Gate receipts; the new entrypoint is the only path that admits them. Report both source/PTX identities.
5. **Same-writer/config check.** The writer bytes are unchanged; the treatment and control config hashes differ only in relation-trit enablement; corpus, objective, budget, seed, spec, schedule, geometry and the +1 leaf gates are identical.
6. **Zero-apply evaluation.** Snapshot/epoch unchanged before/after DEV, ablation and fresh scoring; `updates_applied: 0`.
7. **T2 conservation.** T2's counts sum to the actual training targets (1,049,856 at 16,404; 2,099,712 at 32,808); tie/fallback deterministic; no token ≥4,096.
8. **Fresh seal.** The fresh tokens/mask and ordered row binding are SHA256-bound before training; the file is not read during training or DEV selection.
9. **`B3_R`/`B3_C`/`B3_I` measurement.** Scored admission uses the eval path's `EvaluationRow`: `target_cell_admitted = admitted_cells − address_rivals` ∈ {0,1}; `B3 = Σ target_cell_admitted` (`device.rs:51-74`; same contract as `completion_baseline`/M4 E2). `Forward::admitted_cells` slots are used only in the pre-flight/propose probes, not the scored B3. `B3_I` is the intersection of the per-row booleans from the two arms' eval rows; no host recomputation of admission (§7.12.7).
10. **D23/D25/D32 ratchets** on the new source; any prohibited site is a blocker.

**Which negatives only trigger diagnosis.** An init-check failure, a control-lock failure, a causal-activation failure, a frozen-executor change, a config mismatch, a hash/epoch fault, a T2 conservation failure or a fresh-seal failure is an **instrument fault** and triggers repair, not a mechanism negative. A `B3_I < 200` underpowered result triggers a power-up decision, not a closure. A null result under the **zero-init** configuration would be uninterpretable and is not admissible as a negative.

## 7.9 Loaded artifacts, lineage and resource projection

| Item | Value |
| --- | --- |
| Base | merged Unit 1 `ef9d30999` (full merge hash in the release packet); `ATTENTION_BASE_SOURCE` updated from `fa62a670a…`; amended sheet re-pinned (§7.12.8) |
| TRAIN tokens / mask | `3dee858f…` / `b5e7352d…` (2,425,925 masked targets) |
| DEV tokens / mask | `0919e90f…` / `a838bb1a…` (14,287 eligible; 3,847 `P_succ`) |
| Tokenizer | `d36d3e87…` |
| Training targets consumed | 1,049,856 at 16,404 batches; 2,099,712 at 32,808 |
| +1 leaf gates | Gate(0), Gate(5), Gate(10), Gate(15) = +1; recorded in config |
| Relation inventory | 20 combine + 25 relation = 45; `coordinates = 16,429` |
| Relation implementation | `discrete.cu:78-267`, `:166-218`, `:182`, `:227`, `:258`, `:270-273`; `device.rs:341-356`, `1231-1246`, `1512-1514`; `lib.rs:76-79`, `116-127`, `140-153`; `relation_qualification.rs:1-120` |
| Frozen executor | `completion.rs:744-747` — Gate receipts refused |
| Learned Relation checkpoint | **does not exist**; hash TBD at release |
| Q4 Route reference | `4477eb5c…`; evaluation `4fff1d3c…`; audit receipt **149/747/1920** (top-1/top-8/B3); `29/435` Route `P_succ` interim. M4 sheet line 22 `94/937/1908` is stale and superseded (§7.12.8) |
| Sealed attribution | 7.32–7.37s per `Device::propose` call; 99.995770–99.998097% counterfactual-dominated |
| T2 | [D34 §6.4 item 3](d34-successor-table-control-2026-10-17.md), rebuilt on the actual training budget |

**Resource projection (no stale 3.74s/batch figure; one propose call per batch).**
- The sealed attribution measures **7.32–7.37s per `Device::propose` call** at the K16/checkpoint-0 shape, with 99.995770–99.998097% of the enclosing propose wall in the counterfactual event. `PreparedDriver::transition` issues exactly **one** `model.propose` call per batch (`training.rs:877`), one transition per batch (`completion.rs:738`), and `Device::apply` launches the apply kernel without proposing (`device.rs:1746-1775`). The propose component is therefore **7.32–7.37s/batch**, not two calls.
- Planning envelope at 7.35s/batch: one arm × 16,404 = **~33.4 h** (7.32s → 33.36 h; 7.37s → 33.58 h); two arms × one seed = **~66.9 h ≈ $61** at $0.908/h; two arms × two seeds = **~134 h ≈ $122**. The 32,808 power-up doubles the corresponding envelope. The sealed K8 seed-1 completion's 15.46 h for 16,404 batches (~3.39s/batch, Route/canonical) is a lower anchor from a different configuration, not the planning basis.
- The reviewed configuration is `Relation { kept: 8, sweeps: 1 }` at K=8, not K16; the actual per-batch cost **must be measured at the reviewed configuration** in the release packet, **including the frozen control's gate-only readback overhead** (the control pays real proposal/sensitivity observations with refused commits). The envelope is a planning range, not a cap or a hard stop; a healthy admitted run is renewed, not cancelled because the estimate elapsed.
- DEV + ablation + fresh scoring: ~1–4 h total at the observed range, <$4.
- Host T2/`U_succ` work: **$0 GPU**.
- One RTX 5090 through `uor-pod` only; `df ≥ 30 GiB`; caps ≤4 pods and ≤$8/h across labs. This docs delivery is **$0 GPU**.
- No training is authorized. The cost is a projection, not a release.

**Stop conditions.** Unit 1 source/FIT not complete → no training. Init/control-lock/causal-activation/frozen-executor/config/hash/epoch/T2/fresh-seal fault → stop and seal. Any D23/D25/D32 site → blocker. `B3_I < 200` and power <80% → underpowered, no decision. Disk floor/resource cap → stop. A healthy admitted run is renewed, not cancelled because a planning estimate elapsed.

**Record schema.** `uor-r4.learned-attention-link/1`: run config (with the +1 gate indices/values and relation enablement), per-coordinate ledgers, per-row DEV/fresh records (`row_ordinal`, `doc_id`, `target_position`, `query_token`, `target_token`, `subpanel`, `distance_bucket`, `admitted_cells[16]`, `target_admitted`, `top1_correct`, `top8_correct`, `prediction`, hinges, `arm`, `seed`), `B3_R`/`B3_C`/`B3_I`, T2/`U_succ` counts, ablation deltas, paired 2×2 tables, source/checkpoint/evaluation hashes, `updates_applied: 0` for evaluations, `zero_apply: true`, fresh seal. Seal with `report_output::claim`/`seal`. Add `claim_scope = "learned_copy_successor_link@Relation+InterleavedReadEvery8V1+ActiveLeafV1+V4096/B4/C192"`, `top1_ITT`/`top8_ITT`, and `MDE_rel(B3_I)` to `uor-r4.learned-attention-link/1`.

## 7.10 D22, claim discipline and assumptions

**D22.** A negative closes the **`(Relation, InterleavedReadEvery8V1, +1-leaf-combine init, this V4096/B4/C192 configuration)`** — never the geometric attention/Relation mechanism family. Stopping a mechanism needs the owner. A null result under zero init is uninterpretable and closes nothing.

**Claim discipline.** No capability claim. This is the first trained-in Relation artifact and a diagnostic/trained-in discriminator at one configuration. A `P_succ` win is not general language or reasoning evidence. The fresh transfer is a confirmation draw, not a capability benchmark. The complement is a co-primary non-regression panel, not a second capability claim. The licensed claim is the **learned copy-successor link at this configuration** (§7.12.9), not a learned-attention or general Relation claim. The ITT row is co-reported; it does not widen the claim.

**Authorization.** This sheet authorizes **NO training**. Execution needs FIT + PM release; any D-Release-class decision needs owner release. Q4 remains **NOT_EXECUTABLE**; Training/M2/Stage B remain **HELD**; counts unchanged.

**Assumptions the lead must resolve.**
- **(a) RESOLVED (owner, 2026-10-17):** the **+1 leaf combine gate per payload block** is the declared amendment; the widened all-level init is not selected. The leaf-only init activates the 5 level-0 relation trits; the 20 higher-level trits remain inert until their combine gates learn. If a null result fires with the ledgers showing the higher-level trits never activated, widening the init is the declared follow-up rung, not a mechanism negative.
- **(b)** Confirm the **10,440-row complement as co-primary** for non-regression (20/63/87 floors), not merely descriptive.
- **(c)** Confirm the **within-checkpoint zero-relation ablation** is in-memory, zero-apply, no retraining, and separate from the matched relation-disabled arm.
- **(d)** Confirm **T2** (context-free, actual-training-targets) as the ordinary comparator, per D34 §6.4 item 3, not T1.
- **(e)** Confirm whether **Route** stays as an additional arm. A Route-only win cannot attribute a relation effect.
- **(f)** Confirm the **primary relation-disabled control is the Relation-off 45-coordinate arm**, not a Flock 20-coordinate arm; the latter has a ~2.25× combine-gate proposal-frequency advantage and is a confound.
- **(g) RESOLVED (owner, 2026-10-17):** **one seed first** — two arms × 16,404 batches at the 7.35s/batch planning envelope ≈ 66.9 h ≈ $61. The second seed is pre-registered as the **immediate reproducibility follow-up if H_geom fires** (it cannot rescue a DEV miss under §7.7); the sealed ≥2,000-row / ≥500-successor fresh transfer satisfies D22's "≥2 seeds OR ≥200 fresh rows" decisiveness prong. The 32,808 power-up remains a lead decision on an underpowered result. The actual cost binds the measured throughput at release.
- **(h) RESOLVED (Unit 2b):** **A = `NearestAddressMarginV1`**, forced by `attention_learning.rs:131-135`. The Q1 admission gate (7.12.2) is the A-specific collapse check; if it fires, the next configuration is **R = `OccupiedCellRankHingeV1`**, not a larger A dose.
- **(i)** Confirm the **fresh-transfer source and size** (≥2,000 rows / ≥500 successor rows) and seal it before training.
- **(j) RESOLVED (Unit 2b):** the MDE/precision rule (§7.12.6) supersedes the bare `B3_I ≥ 200`/observed-power guard. Floor table: 24-row floor = **12.0% at `B3_I=200`, 5.5% at 435, 3.0% at 800**. `B3_R`/`B3_C` are reported separately, not substituted.

## Base artifact

- base: `origin/main = fb4519226`; the opted-in Gate-admitting entrypoint and the declared ActiveLeafV1 initialization from Unit 1 (separately reviewed) are a hard precondition; no learned Relation checkpoint exists — hash TBD at release. Q4 Route reference `4477eb5c…` (§7.9).

## Data split

- split: TRAIN 2,425,925 masked targets (1,049,856 consumed at 16,404 batches; 2,099,712 at the 32,808 power-up); DEV 14,287 eligible rows (`P_succ` 3,847 co-primary, `P_comp` 10,440 co-primary non-regression); fresh transfer draw ≥2,000 eligible rows with ≥500 successor rows, sealed before training (§7.2, §7.6). Training data never includes frozen panel rows; the fresh draw is disjoint from TRAIN/DEV and is not used for model selection.

## Steps

- steps: §7.5 sequence 1–8 — Unit 1 source review and pins; identical declared init on both arms; §7.8 causal-activation fixture before training; B = 16,404 interleaved batches per arm; checkpoints and ledger verification; zero-apply DEV scoring; within-checkpoint zero-relation ablation; sealed fresh transfer scored once; host T2/`U_succ`.

## Minimum test power (D22 section 3)

- seeds: 2 per arm (reproducibility-only; the design does not claim seed-noise-decisive power, §7.6); 1 seed is the reduced-power fallback.
- fresh rows: ≥2,000 eligible / ≥500 successor, sealed before training (≥ the 200-row D22 floor).
- trained in rather than bolted on: YES — the relation path is trained through the interleaved schedule (1,130 relation proposals per arm at 16,404), not probed post hoc.
- decisive guard: `B3_I ≥ 200` or the exact paired observed-power guard; all bars `max(24, 2·sqrt(b+c))` or wider, never narrower than panel noise (§7.6).
- fresh-transfer plan for a KEEP: the sealed fresh draw is scored once after DEV; a DEV win without fresh confirmation is DEV-only, not a KEEP (§7.7 row 6).

## Cost (charged cumulatively, before execution)

- preparation: Unit 1 source + review (no model compute); fixture runs host-side where not cuda-gated.
- run: 7.32–7.37s/batch sealed propose-call envelope → ~33.4 h per arm at 16,404; two arms × one seed ≈ 66.9 h ≈ $61; two seeds ≈ 134 h ≈ $122 at $0.908/h (§7.9; measured throughput at the reviewed K=8 configuration binds at release).
- controls: the matched relation-off control arm is inside the two-arm envelope; T2/`U_succ` host work is $0 GPU.
- evaluation: DEV + ablation + fresh scoring ~1–4 h, <$4.
- retries / storage: archive MD5 round-trip before teardown; sealed per-row records per §7.9 schema.
- no training is authorized by this sheet; these figures are projections, not a release.

## Stop rule

- stop: §7.9 stop conditions — Unit 1/FIT incomplete → no training; any §7.8 instrument fault → stop and seal, diagnosis only (never a mechanism negative); `B3_I < 200` with power <80% → underpowered, no decision; owner caps and the disk floor are hard stops. A healthy admitted run is renewed, not cancelled because a planning estimate elapsed.

## 7.11 Verdict

TEST FITNESS: FIT (design) — DeepSeek architecture-review seat, [PR #2574](https://github.com/UOR-Foundation/uor-r4/pull/2574), exact head `630dd61aeb8f21c8cc8e312196575faf9e59ea1f` (rounds at `a53b90007`, `839a16c6f`, `9c0891c71`); conditional on Unit 1's separately reviewed, opted-in Gate-admitting entrypoint; no training authorized; Q4 NOT_EXECUTABLE; Training/M2/Stage B HELD.

## 7.12 Unit 2b amendments

**Authority and status.** Adversarial soundness review on [#2029 comment 6238298755](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6238298755), returned **SOUND-WITH-EXECUTION-ADDITIONS**; Unit 1 (#2575) merged at `ef9d30999`. These amendments are absorbed into the FIT'd design. Items 7.12.1–7.12.3 and 7.12.7–7.12.8 are execution-packet-level; 7.12.4 resolves an open assumption; 7.12.5 adds a co-reported unconditional row; 7.12.6 replaces the evaluability guard with a precision rule; 7.12.9 narrows claim text. **No amendment changes the direction of H_geom/H_control/H_null, the D22 scope, or the no-training authorization.** The §7.11 FIT verdict stands; the amended sheet hash must be re-pinned (7.12.8).

### 7.12.1 Pre-flight battery — hard gate before training

This is an execution-packet hard gate. No design change; the tooling exists. It makes the §7.8 activation checks a release condition, not a post-training observation.

1. **Init zero-apply DEV evaluation.**
   - `init-corpus CONFIG.json INIT_REPORT_DIR` with the exact attention config: `Spec { vocabulary: 4096, blocks: 4, context: 192, kept_cells: 8 }`, `ReadSpec::Relation { kept: 8, sweeps: 1 }`, `ReadInitialization::ActiveLeafV1`, `SchedulePolicy::InterleavedReadEvery8V1`, `RelationLearningPolicy::LearnV1`, `ObjectivePolicy::NearestAddressMarginV1`, seed 1, `batch_rows = 64`.
   - `eval-corpus CONFIG.json INIT_REPORT_DIR/report.json INIT_EVAL_DIR` — `init-corpus` writes the `DriverCheckpoint` to `report.json`; the runner's `checkpoint-00000.json` is an `AttentionCheckpoint`, not what `eval-corpus` reads.
   - Report `B3_init` on `P_succ`, `P_comp` and the full 14,287 panel, plus T1/T1b/T2/U_succ on the same cells. T1 must be 3,847/3,847 on `P_succ` (panel-definition check); T1b is the second-most-recent rule; T2 is the context-free table rebuilt on the actual training targets; `U_succ` is the model-matched unigram on `P_succ`.
   - **Hard gate:** `B3_init(P_succ) < 500` → **do not train**. Record and return to PM for diagnosis. This is a design-sanity floor, not a mechanism negative. Note the floor sits above the Q4 Route `P_succ` B3 of 435 and the init-shape B3 is unmeasured; a stop here is a pre-flight design-sanity diagnosis for PM review (D31), never a mechanism negative.
2. **Real-shape Gate sensitivity probes.**
   - First bind the config: `attention-build-identity CONFIG.json` runs the attention validator (`init-corpus` alone calls only `DriverConfig::validate`).
   - Build `REQUEST.json` (`{windows, targets}`) from real `P_succ` rows (the actual 192-token causal windows and targets; ≤64 windows per call).
   - Extract the model `Snapshot` (`report.json`'s `model` field) from the `init-corpus` output — `propose` reads a Snapshot, not the `DriverCheckpoint`.
   - `propose INIT_MODEL_SNAPSHOT.json REQUEST.json gate 22` and `propose INIT_MODEL_SNAPSHOT.json REQUEST.json gate 0`.
   - Parse the JSON: inert iff `before == after` and `candidate == 0` for **both** probes. Any `after < before` or nonzero candidate is sensitivity. **Inert → stop before training**; activation/instrument fault (D31), not a mechanism negative.
   - Gate(22) is the level-0 `e=2` relation trit; Gate(0) is the block-0 leaf combine gate. Both must be sensitive at init under the +1 leaf initialization.
3. **`qualify-relation` at the merged head.**
   - `qualify-relation QUALIFY_REPORT_DIR`; C2 ACTIVE and the C2 zero-combine INERT must pass.
   - Run the §7.8 item 3(iii) INIT LOCK training-shape snapshot check separately: exactly `gates[0]=gates[5]=gates[10]=gates[15]=+1`, all other combine gates 0, control relation trits 0.
   - Any failure stops before training for diagnosis.

**Cost.** Init DEV eval ~15–20 min; Gate(22)/Gate(0) probes minutes; `qualify-relation` minutes; host T1/T1b/T2/U_succ minutes. **~27 min + minutes; one 1-hour lease at $0.908/h ≈ $0.91; <$2 total** including storage/transfer. The measured throughput at the merged head binds.

**FIT impact:** none — execution-packet-level hard gate.

### 7.12.2 Q1 admission/ledger evidence checkpoint at 4,096

At the 64-aligned `4,096` checkpoint (`checkpoint_every_batches = 64`), before the final DEV decision:

- Report `B3_Q1(P_succ)` from the eval path (7.12.7), not from `Forward` slots.
- Report the per-family ledger at Q1: expected `VectorBlock = 3,584`, `GateCombine (0..19) = 237`, `GateRelation (20..44) = 275`, read total `512`; with observed `attempted / sensitive / accepted` for each family.
- Report treatment-control admission agreement on the 3,847 `P_succ` rows: `agreement = (# target_admitted_R == target_admitted_C) / 3,847`, with the exact 2×2 (both, R-only, C-only, neither).

**Declared stop-for-diagnosis criteria (pre-registered):**
- **Admission collapse:** `B3_Q1(P_succ) < 500` (the 7.12.1 floor) **or** `B3_Q1(P_succ) < 0.5 × B3_init(P_succ)`.
- **Zero gate sensitivity:** `sensitive(GateRelation) = 0` **and** `sensitive(GateCombine) = 0` over the Q1 interval.

The treatment-control admission agreement is reported for the evidence review; it is **not** an automatic stop criterion. A low agreement is a diagnostic flag, not a mechanism negative.

**No-cancel phrasing (binding).** This is a planned evidence checkpoint, **not a budget stop**. A healthy, progressing run is not cancelled because a self-set budget elapsed; the run continues unless a declared criterion fires. A stop under this section is **for diagnosis** — a D31 blocker, not a mechanism negative and not a KEEP/closure.

**FIT impact:** none — execution-packet-level evidence review.

### 7.12.3 Fresh-transfer scoring order

The fresh draw remains **sealed until after the paired DEV decision** (H_geom/H_control/H_null on DEV). The treatment's fresh transfer is scored once after that decision. The relation-off control is **DEV-only**: `relation_off_transfer_evaluations = 0`; no control fresh-transfer score.

The current runner scores `final-transfer-evaluation.json` inside the post-training sequence (`attention_learning.rs:899`) and `relation-off-transfer-evaluation.json` in the ablation (`:758`). That order is superseded by this sheet. The runner fix is a **follow-up PR before release**; until it lands, the fresh file stays sealed and unscored. This is an execution-order fix, not a hypothesis/bar change.

**FIT impact:** none to hypotheses/bars; execution-order correction.

### 7.12.4 Objective policy (h) — RESOLVED: A (`NearestAddressMarginV1`)

`attention_learning.rs:131-135` refuses any attention config whose `objective_policy != NearestAddressMarginV1`. State **A = `NearestAddressMarginV1`** explicitly on both arms.

**Justification.** The Q1 admission gate (7.12.2) is the A-specific collapse check. The Route precedent is admission 5,680 → 1,920 under A. If the Q1 gate shows admission collapse, the next configuration is **R = `OccupiedCellRankHingeV1`**, not a larger A dose. That is a new pre-registration/owner decision, never an in-run patch.

**FIT impact:** none — resolves an open assumption already forced by code.

### 7.12.5 Unconditional/ITT top-1 row on `P_succ`

On all 3,847 `P_succ` rows, report `top1_ITT` and `top8_ITT` with **non-admitted rows counted wrong** (the full 3,847 denominator), alongside the conditional `B3_I` comparison. Its own paired floor is `F_ITT = max(24, 2·sqrt(b+c))` on the 3,847 rows; the 39-row floor is reported for any top-8 ITT statement.

H_geom still requires the conditional `B3_I` comparison; the ITT row is co-reported and **cannot rescue a conditional miss**. A conditional win with an ITT regression beyond `F_ITT` is H_mixed — the win is admission-driven, not a token-rank gain.

**FIT impact:** reporting addition; no direction change.

### 7.12.6 MDE/precision rule — replaces the bare `B3_I ≥ 200` and the observed-power guard

The evaluability rule of §7.6 is amended to the precision rule:

- `MDE_rel(B3_I) = 24 / B3_I` — the 24-row absolute floor as a fraction of the paired denominator:

| `B3_I` | `MDE_rel` |
| ---: | ---: |
| 200 | 12.0% |
| 435 | 5.5% |
| 800 | 3.0% |

- The pre-registered effect size is the paired floor **`F_I = max(24, 2·sqrt(b+c))`**. The observed `B3_I` must be ≥ 200, and the observed paired difference must be ≥ `F_I`; the relative MDE (`24/B3_I`) is reported so the reader can see the precision.
- If the lead's pre-registered target is a smaller relative effect `δ`, the required `B3_I ≥ 24/δ` (e.g., 435 for 5.5%; 800 for 3.0%).
- The **observed-power guard is retired**; no post-hoc power is reported. The record reports `MDE_rel(B3_I)` and the exact 2×2.

**FIT impact:** precision-rule replacement within the existing `F_I` floor; no hypothesis direction change, absorbed into the FIT'd design.

### 7.12.7 B3 measurement contract

Scored `B3` uses the eval path's `EvaluationRow` fields, not `Forward` slots:

- `target_cell_admitted = EvaluationRow.admitted_cells − EvaluationRow.address_rivals` ∈ {0,1}; `B3 = Σ target_cell_admitted` over rows. `admitted_cells` is the count of admitted cells (≤K); `address_rivals` is the count of non-target admitted cells (`device.rs:51-74`); same contract as `completion_baseline`/M4 E2 (`admitted_cell_count_sum − address_rival_count_sum`).
- `B3_I` is the intersection of the per-row `target_admitted` booleans from the two arms' eval rows; no host recomputation of admission from slots.
- `Forward::admitted_cells` slots (`device.rs:307-310`) are used **only** in the pre-flight/propose probes (7.12.1) and the diagnostic traces; they are not the scored B3.

**FIT impact:** none — measurement contract correction.

### 7.12.8 Pin reconciliation

- **Single base pin:** the merged Unit 1 commit `ef9d30999` (full merge hash recorded in the release packet). `attention_learning.rs:11` currently pins `ATTENTION_BASE_SOURCE = fa62a670a…`; the release packet updates it to `ef9d30999`. The amended sheet changes `ATTENTION_SHEET_SHA256` (`attention_learning.rs:14-15`); re-pin the amended sheet in the runner-fix follow-up PR.
- **Q4 Route reference:** use the audit receipt **`149/747/1920`** (top-1/top-8/B3) from the Stage-A/relation-subpanel audit. The M4 sheet line 22 `94/937/1908` is stale and superseded. The `29/435` P_succ Route interim remains the **Route** B3, not the Relation denominator.

**FIT impact:** none — pin reconciliation.

### 7.12.9 Claim narrowing

H_geom licenses the **"learned copy-successor link at this configuration"** — not "learned attention", "learned relational attention", or a general Relation claim. Record schema field:

`claim_scope = "learned_copy_successor_link@Relation+InterleavedReadEvery8V1+ActiveLeafV1+V4096/B4/C192"`

Any claim text in §7.7 row 5 or §7.10 is read with this scope. The D22 language is unchanged.

**FIT impact:** claim text/record schema only.

### 7.12.10 D22 and status

**D22 unchanged.** A negative closes the **`(Relation, InterleavedReadEvery8V1, +1-leaf-combine init, this V4096/B4/C192 configuration)`** — never the geometric attention/Relation mechanism family. Stopping a mechanism needs the owner.

**Status.** This sheet still authorizes **NO training**. Execution needs FIT + PM release; any D-Release-class decision needs owner release. Q4 remains **NOT_EXECUTABLE**; Training/M2/Stage B remain **HELD**; counts unchanged. The §7.11 FIT verdict stands; the MDE rule and claim narrowing are absorbed as precision/claim-scope amendments, not new hypothesis directions.
