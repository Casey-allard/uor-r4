# Hamilton Gate Offline-Float Sheet: Language Fit, Icosian Export Gap, and Signal Suppression — 2026-10-25

- **Roadmap item:** [`hamilton-gate-offline-float-sheet`](../../roadmap/items/hamilton-gate-offline-float-sheet.md) (#2824).
- **Lab:** `antigravity`. **Milestone:** `research` (offline float training sheet under `CHARTER.md` §3.7 / D37).
- **Dependencies:** [`float-training-module`](../../roadmap/items/float-training-module.md) (#2823, `crates/uor-r4-float-train`, `done`) and [`r-secant-surrogate-descent`](../../roadmap/items/r-secant-surrogate-descent.md) (#2886, `done`).

## 1. Question

On the D37 offline-float geometric stack (`crates/uor-r4-float-train`, `pattern = "rrrrrr"`, `width = 256`, `heads = 4`, `mlp_hidden = 384`, `context = 192`, `vocab = 4096`, `kept_cells = 8`, `Objective::MarginHinge { rivals: 8 }`, `RecGates::Geometric`, `QuaternionWeights::SpiralcoreFbs`), across two paired seeds (`seed in {1, 2}`):
1. **Language fit on integer DEV metrics:** What DEV `top-1`, `top-8`, `admitted_target_count`, `token_margin`, and `tok/s` do continuous `Gate::Hamilton` ($q = g/\sqrt{\|g\|_2^2 + 10^{-12}}$, $\text{out} = q \otimes u$) and hard 120-icosian-snapped `Hamilton_IcosianHard` ($q_{\text{snap}} \in \mathcal{I}_{120}$) achieve on the `14,287` eligible response targets of `ft/dev` after 256 response-masked training steps on `mw-balp/train`, compared against their step-0 initialization, against `IdentityGate_Control` ($q \equiv (1,0,0,0)$) and `RandomGate_Null` (fixed random unit icosian), and against the `mw-balp/train` response-target unigram count table (`1,226 / 14,287` top-1, `4,060 / 14,287` top-8)?
2. **Icosian-snap + integer Q12 export gap:** What is the DEV export gap ($\Delta_{\text{export,top1}} = |\text{top1}_{\text{float}} - \text{top1}_{\text{int}}|$, $\Delta_{\text{export,top8}}$, $\Delta_{\text{export,adm}}$, $\Delta_{\text{export,margin}}$) when continuous float evaluation is exported through the 120-element icosian snap (`H4_ROOT_COEFFICIENTS` / `signed_h4_code_i32`) and integer Q12 fixed-point (`i32`) L1 `NativeVectorWriter` evaluator, relative to the two-seed noise band (`CHARTER.md` §3.7)?
3. **Signal suppression in training:** Does `Gate::Hamilton` suppress signal during training with weight decay (`weight_decay = 0.1`), separating (a) **intrinsic per-4-block amplitude suppression** ($\rho(g,u) = \|\text{out}\|_2 / \|u\|_2 \le 0.5$ inside the $\|g\|_2 \le 5.7735 \times 10^{-7}$ guard core and its $1/r$ backward scale, `#2879`) from (b) **downstream projection cancellation** ($\gamma_{\text{down}} = \|P_{\text{down}}(q \otimes u)\|_2 / \|P_{\text{down}}(u)\|_2$ after `mlp.down` and `rec.out` `SpiralcoreFbs` projections)?

## 2. Mechanisms, arms, steps, and minimum test power (defined exactly)

All model training and DEV evaluation run on a leased `uor-pod` RTX 5090 inside `crates/uor-r4-float-train` (`StackArch::Geometric`, `pattern = "rrrrrr"`, `width = 256`, `heads = 4`, `mlp_hidden = 384`, `context = 192`, `vocab = 4096`, `kept_cells = 8`, `Objective::MarginHinge { rivals: 8 }`, `StackAdamW` with `lr = 0.002`, `weight_decay = 0.1`, `clip = 1.0`, and the `#2886`-calibrated `20.0x` learning-rate multiplier on `layers.*.rec.decay`, zero softmax/squashing/matmul). Each arm trains for `256` optimizer steps (`batch = 4` document-bounded causal windows $\times$ `context = 192` per step, cycling `4` epochs over the first `256` document windows / `64` batches of `mw-balp/train` with $\ge 4$ response targets, scoring eligible response positions `response_mask[p] == 1 && tokens[p] != bos_id`) across paired seeds `1` and `2` (`CHARTER.md` §4.3):

| Arm | Forward gate law (`composed_recurrence` + `mlp`) | Backward gate law | Seeds | Steps |
|---|---|---|---:|---:|
| `Hamilton_Active` | Continuous $r = \sqrt{\|g\|_2^2 + 10^{-12}}$, $q = g/r$, $\text{out} = q \otimes u$ (`hamilton_gate_fwd`). | Exact CUDA `hamilton_gate_bwd`: $\partial u = \bar{q} \otimes d$, $\partial q = d \otimes \bar{u}$, $\partial g = (\partial q - q(q \cdot \partial q))/r$. | `1, 2` | `256` |
| `Hamilton_IcosianHard` | Hard 120-element icosian snap $q_{\text{snap}} = c_{k^*(g)} \in \mathcal{I}_{120}$ ($k^*(g) = \arg\max_{k \in 0..120} \langle g, c_k \rangle$ on `H4_ROOT_COEFFICIENTS`), $\text{out} = q_{\text{snap}} \otimes u$. | Straight-through $S^3$ bridge: $\partial u = \bar{q}_{\text{snap}} \otimes d$, $\partial q = d \otimes \bar{u}$, $\partial g = (\partial q - q_{\text{cont}}(q_{\text{cont}} \cdot \partial q))/r$. | `1, 2` | `256` |
| `IdentityGate_Control` | Identity quaternion $q \equiv (1, 0, 0, 0)$, $\text{out} = u$ (un-gated pass-through at matched non-gate parameters). | $\partial u = d$, $\partial g = 0$. | `1, 2` | `256` |
| `RandomGate_Null` | Deterministic norm-preserving random unit icosian $q_{\text{rand}} \in \mathcal{I}_{120}$ per 4-block ($\|\text{out}\|_2 = \|u\|_2$, no learned steering). | $\partial u = \bar{q}_{\text{rand}} \otimes d$, $\partial g = 0$. | `1, 2` | `256` |
| `TrainUnigram_CountTable` | Lawful non-neural reference (`CHARTER.md` §4.4): response-masked target frequency table over `mw-balp/train` (`2,425,925` targets), scored on the `14,287` DEV rows (`top-1 = 1,226`, `top-8 = 4,060`). | None (count table). | — | `0` |

## 3. Instrument checks (`CHARTER.md` §4.2 — run before calling any mechanism result)

1. **`I-Finite-CUDA`:** Device error flag `error == 0` on every step, zero `NaN`/`Inf` in all forward/backward tensors and margin losses, and step-0 $\|\partial g\|_2 > 0$ on `Hamilton_Active` and `Hamilton_IcosianHard`.
2. **`I-DEV-Denominator`:** Document-bounded causal windowing of `ft/dev/tokens.u16` (`42,856` payload tokens, `400` documents starting at `<|bos|> = 0`) and `ft/dev/response_mask.u8` yields **exactly `14,287`** scored response positions (`p > doc_start && tokens[p] != 0 && mask[p] == 1`), each scored once.
3. **`I-Icosian-Closure`:** All `120` unit roots from `uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS` satisfy $|\|c_k\|_2 - 1| \le 10^{-6}$, their $120 \times 120$ Cayley table is closed on $0..120$ with identity at index `1`, and the nearest-root snap on common-scale `i32` lanes matches `uor_r4_integer::h4_classifier::signed_h4_code_i32` $100\%$.
4. **`I-Q12-Int-Parity`:** On the Q12-quantized integer export (`round(clamp(x * 4096, -32768, 32767))`), the GPU `NativeVectorWriter` L1 cell-and-slot writer matches the pure `i32` integer L1 oracle (`pure_i32_q12_writer_oracle`) with `0` mismatches across the checked DEV rows.

## 4. Bars and decision table

Let $\text{top1}_{\text{int}}^{(a,s)}$, $\text{top8}_{\text{int}}^{(a,s)}$, and $M_{\text{int}}^{(a,s)}$ be the step-256 integer-exported DEV `top-1` count, `top-8` count, and `token_margin` of arm $a$ on seed $s \in \{1, 2\}$ across the `14,287` DEV rows, let $\Delta_{\text{export,top1}}^{(a,s)} = |\text{top1}_{\text{float}}^{(a,s)} - \text{top1}_{\text{int}}^{(a,s)}|$, and let the two-seed noise band be $\Delta_{\text{noise,top1}} = \max(\max_a |\text{top1}_{\text{int}}^{(a,1)} - \text{top1}_{\text{int}}^{(a,2)}|, 398)$ and $\Delta_{\text{noise,top8}} = \max(\max_a |\text{top8}_{\text{int}}^{(a,1)} - \text{top8}_{\text{int}}^{(a,2)}|, 748)$ (`CHARTER.md` §3.7, §4.3).

| Gate | Bar | Unlocks / Meaning on PASS | Meaning on FAIL (with `I-1..4` green) |
|---|---|---|---|
| `G-Hamilton-Fit` | On both seeds $s \in \{1, 2\}$: step-256 integer DEV `top-1` beats `TrainUnigram_CountTable` (`1,226 / 14,287`) AND beats both `IdentityGate_Control` and `RandomGate_Null` beyond the in-run two-seed noise band. | Promotes `Gate::Hamilton` / `Hamilton_IcosianHard` as a positive standalone feed-forward gate result on DEV. | If step-256 improves over step-0 and `RandomGate_Null` (`KEEP_DIAGNOSTIC`) or ties `IdentityGate_Control` below `1,226`, closes only this 256-step micro-rung configuration (`CHARTER.md` §4.1) and records the exact fit for `#2890` / `#2891`. |
| `G-Hamilton-ExportGap` | On both seeds $s \in \{1, 2\}$: $\Delta_{\text{export,top1}} \le \Delta_{\text{noise,top1}}$ (`<= 398` rows) and $\Delta_{\text{export,top8}} \le \Delta_{\text{noise,top8}}$ (`<= 748` rows) under 120-icosian snap + Q12 `i32` L1 `NativeVectorWriter`. | Qualifies the 120-icosian snap + Q12 `i32` writer export path under `CHARTER.md` §3.7 (export gap within the two-seed noise band). | Disqualifies post-hoc icosian snap without hard-forward training (`Hamilton_IcosianHard`) under `CHARTER.md` §3.7. |
| `G-Hamilton-Suppression` | Across all training steps and DEV blocks: reports `min \|g\|_2`, `min \rho(g,u)`, `frac(\rho \le 0.5)`, `max(1/r)`, and downstream projection gain ratio $\gamma_{\text{down}} = \|P_{\text{down}}(q \otimes u)\|_2 / \|P_{\text{down}}(u)\|_2$. | Settles empirically whether weight decay triggers the $1/r \sim 10^6$ guard-core hazard (`frac(\rho \le 0.5) > 0`) or whether `Gate::Hamilton` acts purely by norm-preserving rotation ($\rho = 1$) and downstream projection modulation ($\gamma_{\text{down}}$). | If `frac(\|g\|_2 \le 10^{-3}) > 0` with gradient blow-up, mandates clamping $\|g\|_2$ or hard icosian snapping (`Hamilton_IcosianHard`). |

## 5. Base artifact, data split, checkpoints, and pins

- **Inputs in `caseyallard/uor-r4-store:w2-inputs/2026-10-25/driver-data/`:**
  - `mw-balp/train/tokens.u16`: SHA-256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` (`7,512,797` payload tokens).
  - `mw-balp/train/response_mask.u8`: SHA-256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` (`2,425,925` response positions).
  - `ft/dev/tokens.u16`: SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` (`85,776` B, `42,856` payload tokens, `400` documents).
  - `ft/dev/response_mask.u8`: SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` (`42,856` B, `14,287` eligible response targets).
  - `tokenizer.json`: SHA-256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` (`vocab_size = 4096`, `<|bos|> = 0`).

## 6. Cost, stop, and run bounds

- **Cost:** 1 leased `uor-pod` RTX 5090 instance (`--count 1 --gpu 5090`) inside the global `<= 4` pods and `<= $8.00/h` cap (`CHARTER.md` §5.4), `8` runs (`4` arms $\times$ `2` seeds $\times$ `256` steps) + `14,287`-row float and integer DEV evaluations replicated across two independent `nvcc` builds, `<= 25` minutes total wall time, target `~$0.60` (`<= $1.20` hard stop).
- **Stop rule:** Stop immediately if `I-Finite-CUDA`, `I-DEV-Denominator`, `I-Icosian-Closure`, or `I-Q12-Int-Parity` fails, or if projected wall time exceeds `45` minutes.
- **Dispatch gate:** Both `#2823` and `#2886` are `done` on `main`; runs on the lab's own D67 pre-run review after fresh-context `TEST FITNESS: FIT` and D68 decision review.

## 7. Non-author `TEST FITNESS` review (`CHARTER.md` §4.5, D68, D67)

- **Reviewer:** Fresh-context `research` subagent (`f9fe6674-84f8-4c79-895b-d07cfbe8c4b3`, non-author context reading only `CHARTER.md`, `PLAN.md`, `#2824`, `docs/mechanisms/hamilton-gate.md`, `#2879`, `#2886`, this sheet, and `src/hamilton_sheet.rs` at `a4b77ad5e`).
- **Verdict:** **`TEST FITNESS: FIT`** | **`D68 DECISION REVIEW: APPROVE`** | **`D67 PRE-RUN REVIEW: APPROVE`**.
- **Summary of review:** (1) Directly answers `#2824`'s done-when across all `14,287` eligible DEV response positions (`400` documents) in both continuous `f32` and 120-icosian + Q12 `i32` integer export mode; (2) defines `Hamilton_Active`, `Hamilton_IcosianHard`, `IdentityGate_Control`, `RandomGate_Null`, and `TrainUnigram_CountTable` (`1,226 / 14,287` top-1, `4,060 / 14,287` top-8) across 2 seeds (`1, 2`) with zero softmax/squashing/matmul (`CHARTER.md` §3.1–§3.3); (3) checks `I-Finite-CUDA`, `I-DEV-Denominator`, `I-Icosian-Closure`, and `I-Q12-Int-Parity` before any mechanism verdict (`CHARTER.md` §4.2); (4) enforces `CHARTER.md` §3.7 (export gap within the two-seed noise band) and §4.1 (any negative closes only the 256-step configuration, never the mechanism); and (5) passes all five D67 pre-run criteria inside the global GPU caps (`1x RTX 5090`, `<= 4` pods, `<= $8.00/h`).

TEST FITNESS: FIT - research #2824 hamilton-gate-offline-float-sheet
