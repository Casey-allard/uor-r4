# Offline-Float Six-Recurrence Fit Sheet: Dyadic Recurrence Gates, Quaternion Transport, Tokens/s, and D11 Export Gap — 2026-10-25

- **Roadmap item:** [`recurrence-gates-six-recurrence-fit`](../../roadmap/items/recurrence-gates-six-recurrence-fit.md) (`#2890`).
- **Lab:** `antigravity`. **Milestone:** `research` (offline float training under `CHARTER.md` §3.7 / D37; mechanism [`recurrence-gates`](../../mechanisms/recurrence-gates.md)).
- **Dependencies (`done` on `main`):** [`float-training-module`](../../roadmap/items/float-training-module.md) (`#2823`, `crates/uor-r4-float-train`) and [`r-secant-surrogate-descent`](../../roadmap/items/r-secant-surrogate-descent.md) (`#2886`, `G-Rec-Descent = PASS` at `1b5a6abe3`, [record](../../labs/r-secant-surrogate-descent/README.md)).
- **Applicable owner decisions:** D11, D21, D22, D23, D24, D25, D28, D29, D31, D32, D37, D67, D68.

## 1. Question

When trained end-to-end on the six-recurrence native geometric float stack (`crates/uor-r4-float-train`, `pattern = "rrrrrr"`, `RecGates::Geometric`, `Gate::Hamilton`, `QuaternionWeights::SpiralcoreFbs`, `NativeVectorWriter` with `Objective::MarginHinge { rivals: 8 }`, `kept_cells = 8`, zero softmax/squashing/matmul) over `1,024` steps on `mw-balp/train` across seeds `1` and `2`:
1. Does the 16-entry dyadic fade/keep table (`RATE_TABLE` $\lambda = 2^{-(i+1)}$, `KEEP_TABLE` $\kappa = \sqrt{1 - \lambda^2}$) with clamped-linear opening $s_t = \mathrm{clamp}(g_t + d, 0, 1)$ and `rate_select_bwd` secant credit (`Rec_Geometric_S3_Active`) beat frozen interior rate-bin initialization (`Rec_FrozenRates_Null`) and the GPU train-split response count-table baseline (`1,226 / 14,287` = `8.58%` top-1, `28.42%` recall@8) on the `14,287`-row `ft/dev` panel?
2. Does non-commuting $S^3$ unit-quaternion state transport (`Rec_Geometric_S3_Active`) beat commutative $U(1)$ transport (`Rec_Geometric_U1_AbelianControl`) and identity transport (`Rec_Geometric_IdentityTransport_Control`, `rotation = false`), testing the position/transport hypothesis on a squash-free geometric stack (`docs/mechanisms/recurrence-gates.md:63,84`)?
3. What are the synchronized CUDA decode/context/training `tokens/s` (lead metric, `CHARTER.md` §1) and the D11 integer table export gap (`CHARTER.md` §3.7) when binding the 16-entry dyadic rate index $i \in \{0..15\}$ and discrete rotation carry on `ft/dev`?

## 2. Base artifact, mechanisms, and arms (defined exactly)

- **Base artifact:** Runs in `crates/uor-r4-float-train` at the six-recurrence rung-0 shape (`pattern = "rrrrrr"` = 6 recurrence layers, `width = 256` = $64$ quaternion lanes of $4$ floats, `heads = 4`, `mlp_hidden = 384`, `batch = 4`, `context = 192`, `vocab = 4096`, `kept_cells = 8`, `Objective::MarginHinge { rivals: 8 }`, `RecGates::Geometric`, `Gate::Hamilton`, `QuaternionWeights::SpiralcoreFbs`). Per-lane `rec.decay` initializes at interior rate-bin centers `((lane % 16) + 0.5) / 16` (`0.03125..0.96875`) so all 16 dyadic clocks start active (`rung0-complete-path-2026-10-13/README.md:45-47`).
- **Four matched geometric arms + GPU train-split count-table baseline (trained for `1,024` steps per seed $s \in \{1, 2\}$ from shared seed-$s$ initialization $L_0^{(a,s)} = L_0^{(s)}$ with `StackAdamW`):**

| Arm | Definition | Seeds | Steps |
|---|---|---:|---:|
| `Rec_Geometric_S3_Active` | Full `RecGates::Geometric` + `RotationGroup::Quaternion` (`rotation = true`): clamped-linear opening $s_t = \mathrm{clamp}(g_t + d, 0, 1)$, 16-entry dyadic rate/keep table (`rate_select_fwd`/`bwd` secant credit), non-abelian $S^3$ unit-quaternion scan $h_t = \lambda_t u_t \otimes h_{t-1} + \kappa_t b_t$ (`quaternion_scan_fwd`/`bwd`), and `Gate::Hamilton` output gate. | `1, 2` | `1,024` |
| `Rec_Geometric_U1_AbelianControl` | Matched commutative transport control (`RotationGroup::U1`, `geometric_stack_native.rs:65-70`): identical 6-recurrence architecture, clamped-linear opening, 16-entry dyadic table, and secant credit, but zeroes the $j, k$ quaternion components before unit normalization so every transition lies in the abelian subgroup $U(1) = \{a + b\mathbf{i}\}$. | `1, 2` | `1,024` |
| `Rec_Geometric_IdentityTransport_Control` | Matched no-rotation state-carry control (`rotation = false`): identical clamped-linear opening, 16-entry dyadic fade/keep table, secant credit, and Hamilton output gate, with identity transport $u_t \equiv (1, 0, 0, 0)$ ($h_t = \lambda_t h_{t-1} + \kappa_t b_t$). | `1, 2` | `1,024` |
| `Rec_FrozenRates_Null` | Static rate-selection control: identical `RecGates::Geometric` + $S^3$ quaternion scan forward pass, but zeroes `rate_select_bwd` secant credit ($d s_t \equiv 0$) so `rec.decay` and gate-opening rate scores remain frozen at seed-$s$ interior bin-center initialization while all other stack parameters train normally. | `1, 2` | `1,024` |
| `TrainSplit_CountTable_Ref` | Non-neural count-table baseline (`CHARTER.md` §4.4): GPU token-frequency table over the `2,425,925` response-masked targets of `mw-balp/train` (never reading `ft/dev` targets; scores `1,226 / 14,287` = `8.58%` top-1 and `28.42%` recall@8 on `ft/dev`). | `—` | `0` |

## 3. Instrument checks (`CHARTER.md` §4.2 — run before calling any mechanism result)

1. **`I-Standalone-CUDA-Parity`:** Standalone CUDA device qualification of `rate_select_fwd`/`bwd` and `quaternion_scan_fwd`/`bwd` (`docs/mechanisms/recurrence-gates.md:70,80`), verifying exact f32 bit-pattern table reads ($\le 5\times 10^{-13}$ rate error, $\le 4.6\times 10^{-10}$ keep error, $\le 5.5\times 10^{-8}$ secant error), zero `NaN`/`Inf`, and nonzero step-0 opening/decay credit $\|d s\|_2 > 0$ across all 6 recurrence layers.
2. **`I-DEV-Denominator`:** The `ft/dev` evaluator scores all `14,287` eligible within-document response targets across `42,856` payload tokens (`400` documents, resetting recurrence state at `<|bos|>` = `0`) in both continuous `f32` and D11-exported integer mode (`evaluated_targets == 14287`).
3. **`I-Rate-Mobility-And-Plateau`:** In active arms (`Rec_Geometric_S3_Active`, `U1`, `IdentityTransport`), $\ge 5.0\%$ of recurrence lanes change dyadic rate bin $i \in \{0..15\}$ over `1,024` steps while $< 80\%$ of active lane-tokens trap in the upper f32 keep plateau $i \in \{12..15\}$ (or `analytic_keep_secant_f64` activates per `#2886`).
4. **`I-Export-Exactness`:** The D11 integer recurrence exporter binds the 16-entry dyadic rate index $i \in \{0..15\}$ (exact bit-shift `>> (i + 1)` for $\lambda = 2^{-(i+1)}$ and Q12 table lookup for $\kappa$), snapped Q8/icosian discrete rotation carry, and Q12 `i32` `NativeVectorWriter` with zero float/multiply instructions, reproducing exact synthetic fixtures with zero top-1 discrepancy.

## 4. Bar and decision table

Let $L_{\text{dev}}^{(a,s)}$ and $\text{top1}_{\text{dev}}^{(a,s)}$ be the `14,287`-row `ft/dev` `token_margin` hinge loss (`Objective::MarginHinge { rivals: 8 }`, read on the token term alone) and top-1 hit count at step `1,024`, let $\Delta_{\text{noise},L} = \max_a |L_{\text{dev}}^{(a,1)} - L_{\text{dev}}^{(a,2)}|$ be the two-seed DEV loss noise band (`CHARTER.md` §4.3), and let $F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$ be the paired McNemar top-1 noise threshold on `14,287` DEV rows:

| Gate | Pre-registered Bar | What PASS unlocks | What FAIL decides (`I-1..4` green) |
|---|---|---|---|
| `G-Rec-Sheet-Fit` | On both seeds $s \in \{1, 2\}$: `Rec_Geometric_S3_Active` reduces `ft/dev` `token_margin` loss below step-0 initialization, beats `Rec_FrozenRates_Null` by $\ge \Delta_{\text{noise},L}$ on `ft/dev` loss (or $b - c \ge F_I$ on paired `ft/dev` top-1 hits), AND its D11 integer export beats `TrainSplit_CountTable_Ref` (`1,226 / 14,287` top-1, `28.42%` recall@8) on `ft/dev`, with synchronized CUDA `tokens/s` reported. | Confirms six-recurrence learning, D11 integer DEV quality above the count table, and speed; unlocks fresh V2 transfer scoring. | Closes this 6-recurrence `1,024`-step `StackAdamW` configuration (`CHARTER.md` §4.1); `recurrence-gates` stays `READY` while discrete routed reads remain `ACTIVE`. |
| `G-Rec-Transport` | Diagnostic attribution on both seeds $s \in \{1, 2\}$: `Rec_Geometric_S3_Active` ($S^3$) beats `Rec_Geometric_IdentityTransport_Control` (`rotation = false`) and `Rec_Geometric_U1_AbelianControl` ($U(1)$) by $\ge \Delta_{\text{noise},L}$ on `ft/dev` loss (or $b - c \ge F_I$ on paired top-1). | Establishes that non-abelian $S^3$ quaternion rotation carries sequence order better than abelian/identity carry on a squash-free geometric stack (`recurrence-gates.md:63,84`). | Selects the winning carry mode (`S3`, `U1`, or `Identity`) for D11 export-gap and transfer evaluation; if `Identity` ties `S3`, rotation carry is not needed in this recurrence configuration. |
| `G-Rec-ExportGap` | For the winning active arm, the D11 integer export (dyadic bit-shift `>> (i + 1)` + Q12 keep table + discrete rotation carry + Q12 `i32` writer, zero float/multiply instructions) on all `14,287` `ft/dev` targets has loss gap $|L_{\text{dev,int}}^{(a,s)} - L_{\text{dev,f32}}^{(a,s)}| \le \Delta_{\text{noise},L}$ and paired top-1 gap $|b - c| \le F_I$ on both seeds (`CHARTER.md` §3.7). | Qualifies the geometric recurrence mode for lossless D11 integer table export (`recurrence-gates.md:39,82`). | Disqualifies direct post-training integer rounding for this float checkpoint (`CHARTER.md` §3.7) and requires discrete-snapped forward training (QAT) before export. |
| `G-Rec-Transfer` | If `G-Rec-Sheet-Fit` and `G-Rec-ExportGap` pass, the winning arm's D11 integer export beats `Rec_FrozenRates_Null` and `TrainSplit_CountTable_Ref` on the sealed `14,479`-target `fresh_v2` draw (`0b7e754c…`) on both seeds (`CHARTER.md` §4.3). | Satisfies all reopen conditions of `docs/mechanisms/recurrence-gates.md:77-83` and closes `#2890`. | Closes `#2890` with a recorded negative transfer result (`CHARTER.md` §4.1); mechanism status stays `READY` in float. |

## 5. Data split, checkpoints, and pins

All inputs are fetched from `caseyallard/uor-r4-store:w2-inputs/2026-10-25/` and verified via `sha256sum -c`:
- **TRAIN (`mw-balp/train`):** `tokens.u16` SHA-256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` (`7,512,797` tokens / `2,425,925` eligible response targets); `response_mask.u8` SHA-256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- **DEV (`ft/dev`):** `tokens.u16` SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` (`42,856` payload tokens / `14,287` eligible targets across `400` documents); `response_mask.u8` SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- **Sealed fresh V2 transfer draw (`fresh_v2`, evaluated only on DEV `KEEP`):** `tokens.u16` SHA-256 `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8` (`44,678` payload tokens / `14,479` eligible targets); `response_mask.u8` SHA-256 `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e` (`seal` `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`).
- **Tokenizer (`vocab_size = 4096`):** `tokenizer.json` SHA-256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- **Kernel & source pins:** `crates/uor-r4-float-train/src/cuda_stack.cu` (`c577fce92ab965798857bd3f9d19ab86f65326dd501d8f8daf1537c0c9e9cb17`), `crates/uor-r4-float-train/src/geometric_stack_native.rs` (`03aec162e3e40144a360e1f4381b9aeb142c505115e05dd59e1459157d2b24a4`), `crates/uor-r4-float-train/src/geometric_stack_native_cuda.rs` (`03cc8a1166bf51bbeece6259886158acfbca7409a624aeb782b8f9ff7d0960e1`), `crates/uor-r4-core/src/native_geometric/learner/hamilton_transport.rs` (`f376dced3e326a80fc67edabc59d0c607b9ffece6a27548e0bc74957afe96147`).

## 6. Steps and minimum test power (`CHARTER.md` §4.3 / D22 §3)

1. **Steps (probe ladder + full run on 1 $\times$ leased RTX 5090 via `scripts/pod/uor-pod`):**
   - *Step 1 (Probe a — standalone CUDA kernel & export exactness check, ~1 min):* Run `--mode synthetic-probe` (`I-Standalone-CUDA-Parity` for `rate_select_fwd`/`bwd` and `quaternion_scan_fwd`/`bwd`, `I-Export-Exactness`, plus 8-step descent check across all 4 geometric arms).
   - *Step 2 (Probe b — throughput & rate-mobility check, ~2 min):* Run `--mode throughput-probe` (32 steps on `mw-balp/train` + full `14,287`-target `ft/dev` timing) to verify `I-DEV-Denominator`, `I-Rate-Mobility-And-Plateau`, and project wall time $< 30$ minutes total.
   - *Step 3 (Full 2-seed run, replicated across two clean `nvcc` builds A & B):* Run `--mode full` (`4` geometric arms $\times$ `2` seeds $\times$ `1,024` steps on `mw-balp/train` + `TrainSplit_CountTable_Ref`, scoring `ft/dev` in both `f32` and D11-exported integer mode at steps `0, 256, 512, 1024`, plus `fresh_v2` if `G-Rec-Sheet-Fit` and `G-Rec-ExportGap` pass).
   - *Step 4 (Archive & teardown):* Upload sealed receipts and exported D11 tables to `caseyallard/uor-r4-store:results/antigravity/recurrence-gates-six-recurrence-fit-2026-10-25/` with `SHA256SUMS`, release and tear down the pod.
2. **Minimum test power:**
   - **Seeds:** `2` independent seeds (`seed in {1, 2}`).
   - **Evaluation rows:** `14,287` frozen `ft/dev` targets (`42,856` tokens) + `14,479` sealed `fresh_v2` transfer targets (`44,678` tokens; $\gg 200$ rows required by `CHARTER.md` §4.3).
   - **Trained in from step 0:** YES (`1,024` steps $= 786,432$ training tokens per arm/seed).

## 7. Cost, stop rule, and dispatch gate

- **Cost (charged cumulatively, 1 $\times$ RTX 5090 at $\approx \$0.90\text{–}\$1.89/\text{h}$ inside global $\le 4$ pods, $\le \$8.00/\text{h}$ cap):**
  - `preparation`: $\$0.00$ (laptop Rust authoring and stub-`nvcc` check).
  - `run` + `controls`: $\approx 90\text{ s}$ per `1,024`-step arm $\times 8$ arm-seeds $= 12\text{ min}$ per build ($\approx 24\text{ min}$ across Builds A & B, $\approx \$0.60$).
  - `evaluation`: $\approx 4\text{ min}$ total for `14,287`-row `ft/dev` (`f32` + D11 integer export + `TrainSplit_CountTable_Ref`) and `fresh_v2` scoring ($\approx \$0.10$).
  - `retries`: $\approx \$0.40$ allowance for one learning-rate calibration retry if `I-Rate-Mobility-And-Plateau` fails.
  - `storage`: $< 64\text{ MiB}$ retained receipts/tables, respecting the `30 GiB` local free-space floor.
  - **Total estimated cost:** $\approx \$0.70$ (hard planning ceiling $\$2.00$, `--hours 2`).
- **Stop rule:** Stop immediately if `I-Standalone-CUDA-Parity`, `I-DEV-Denominator`, or `I-Export-Exactness` fails, or if `I-Rate-Mobility-And-Plateau` remains out of bounds after one learning-rate calibration pass. Never cancel a healthy run for a self-set time estimate (`CHARTER.md` §5.4).
- **Dispatch gate:** Runs on the lab's own fresh-context `TEST FITNESS: FIT` review (`CHARTER.md` §4.5), D68 decision review, and D67 pre-run review (`CHARTER.md` §5.1).

## 8. Non-author `TEST FITNESS` review (`CHARTER.md` §4.5)

TEST FITNESS: FIT

- Reviewed by fresh-context non-author reviewer (`antigravity` subagent `a0104701-a870-4eab-ac9b-9126317a332d`, 2026-10-25): all 8 `REQUIRED_BLOCKS` present and complete; all 4 geometric arms (`Rec_Geometric_S3_Active`, `Rec_Geometric_U1_AbelianControl`, `Rec_Geometric_IdentityTransport_Control`, `Rec_FrozenRates_Null`) and the GPU train-split response count-table baseline (`TrainSplit_CountTable_Ref`, `1,226 / 14,287` top-1) defined lawfully in `crates/uor-r4-float-train` under D37 with zero softmax/squashing/matmul; bars (`G-Rec-Sheet-Fit`, `G-Rec-Transport`, `G-Rec-ExportGap`, `G-Rec-Transfer`), noise thresholds ($\Delta_{\text{noise},L}$, $F_I$), instrument checks (`I-Standalone-CUDA-Parity`, `I-DEV-Denominator`, `I-Rate-Mobility-And-Plateau`, `I-Export-Exactness`), lead metric (`tokens/s`), D11 integer table export gap, SHA-256 digests, power (`2` seeds, `14,287` DEV + `14,479` fresh V2 targets), cost ($\approx \$0.70$), and stop rules verified (`DECISION REVIEW: APPROVE` under D68).
- A FIT verdict clears compute for the configuration in that sheet only (D22).
