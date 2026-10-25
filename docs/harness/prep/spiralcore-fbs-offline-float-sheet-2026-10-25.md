# Offline-Float SpiralCore/FBS Projection Sheet: Full-Rung Descent, Capacity, Tokens/s, and D11 Export Gap — 2026-10-25

- **Roadmap item:** [`spiralcore-fbs-offline-float-sheet`](../../roadmap/items/spiralcore-fbs-offline-float-sheet.md) (`#2891`).
- **Lab:** `antigravity`. **Milestone:** `research` (offline float training under `CHARTER.md` §3.7 / D37; mechanism [`spiralcore-fbs`](../../mechanisms/spiralcore-fbs.md)).
- **Dependencies (`done` on `main`):** [`float-training-module`](../../roadmap/items/float-training-module.md) (`#2823`, `crates/uor-r4-float-train`) and [`r-secant-surrogate-descent`](../../roadmap/items/r-secant-surrogate-descent.md) (`#2886`, `G-Fbs-Descent = PASS` at `1b5a6abe3`, [record](../../labs/r-secant-surrogate-descent/README.md)).
- **Applicable owner decisions:** D11, D21, D22, D23, D24, D25, D28, D29, D31, D32, D37, D67, D68.

## 1. Question

When trained end-to-end on the native geometric float stack (`crates/uor-r4-float-train`, `QuaternionWeights::SpiralcoreFbs`, `RecGates::Geometric`, `Gate::Hamilton`, `NativeVectorWriter` with `Objective::MarginHinge { rivals: 8 }`, `kept_cells = 8`, zero softmax/squashing/matmul) over `1,024` steps on `mw-balp/train` across seeds `1` and `2`:
1. Does learned SpiralCore/FBS root-edge secant routing (`Fbs_1Atom_Active`) beat both frozen random tree routing (`Fbs_FrozenQueries_Null`) and bare atom selection without $\mathrm{Cl}(0,6)$ signed-axis transport (`Fbs_IdentityTransport_Control`) on the `14,287`-row `ft/dev` panel beyond the two-seed noise band?
2. Does selecting a single 8D input atom per output block (`Fbs_1Atom_Active`) suffice, or does a 2-atom routed superposition (`Fbs_2Atom_Mix`, testing the capacity hypothesis of `docs/mechanisms/spiralcore-fbs.md:33,76`) materially improve `ft/dev` loss and top-1 accuracy?
3. What are the synchronized CUDA decode/context/training `tokens/s` (lead metric, `CHARTER.md` §1) and the D11 integer table export gap (`CHARTER.md` §3.7) on `ft/dev`?

## 2. Base artifact, mechanisms, and arms (defined exactly)

- **Base artifact & prospective 8D block-count justification (`docs/mechanisms/spiralcore-fbs.md:76`):**
  Runs in `crates/uor-r4-float-train` at the rung-0 native geometric stack shape (`pattern = "rrrrrr"` = 6 layers, `width = 256` = $32 \times 8\text{D}$ atoms, `heads = 4`, `mlp_hidden = 384` = $48 \times 8\text{D}$ atoms, `batch = 4`, `context = 192`, `vocab = 4096`, `kept_cells = 8`, `Objective::MarginHinge { rivals: 8 }`, `RecGates::Geometric`, `Gate::Hamilton`).
  - *Why $32$ input atoms ($256\text{D}$) and $48$ MLP atoms ($384\text{D}$):* Both $32 \le 3^4 = 81$ and $48 \le 3^4 = 81$ close cleanly inside a $4$-stage ordered base-3 (`ternary`) FBS binder tree (`ProjectionPlan::new`, well within the 6,560-atom / 8-stage compiler bound), align with the 64-lane ($256/4$) quaternion recurrence scan and 256-cell `NativeVectorWriter`, and require $8,448$ query floats per layer ($50,688$ `.spiralcore_fbs` query scalars across 6 layers).
- **Four matched geometric arms (trained for `1,024` optimizer steps per seed $s \in \{1, 2\}$ from shared seed-$s$ initialization $L_0^{(a,s)} = L_0^{(s)}$ with `StackAdamW`):**

| Arm | Definition | Seeds | Steps |
|---|---|---:|---:|
| `Fbs_1Atom_Active` | Standard `QuaternionWeights::SpiralcoreFbs`: each output block $o$ routes query $q_{o,m} \in \mathbb{R}^8$ across $4$ base-3 stages to select one input atom $\sigma(o)$ and one 64-state $\mathrm{Cl}(0,6)$ signed-axis action $A_o$ ($y_o = A_o x_{\sigma(o)}$), updated by `spiralcore_fbs_bwd` root-edge secant + exact inverse input adjoint $A_o^{-1} d y_o$. | `1, 2` | `1,024` |
| `Fbs_2Atom_Mix` | Two-atom routed capacity control (`spiralcore-fbs.md:33,76`): each output block $o$ routes two independent query paths $(q_{o,1}, q_{o,2})$ and combines the two transported 8D atoms via norm-preserving signed sum $y_o = \frac{1}{\sqrt{2}}\big(A_{o,1} x_{\sigma_1(o)} + A_{o,2} x_{\sigma_2(o)}\big)$ (exported to D11 as integer sum with shift), testing whether 1-atom selection bottlenecks projection capacity. | `1, 2` | `1,024` |
| `Fbs_IdentityTransport_Control` | Pure atom-gather control without SpiralCore $\mathrm{Cl}(0,6)$ transport: identical base-3 tree selection $\sigma(o)$ and root-edge secant updates on $q_{o,m}$, but fixes the transport operator to $A_o \equiv I_8$ ($y_o = x_{\sigma(o)}$), isolating the effect of $\mathrm{Cl}(0,6)$ signed-axis rotations. | `1, 2` | `1,024` |
| `Fbs_FrozenQueries_Null` | Static tree-routing control: identical forward $y_o = A_o x_{\sigma(o)}$ and exact inverse input adjoint $A_o^{-1} d y_o$, but zeros query credit ($d q_{o,m} \equiv 0$) so `.spiralcore_fbs` queries remain frozen at seed-$s$ initialization while all other stack parameters train normally. | `1, 2` | `1,024` |

## 3. Instrument checks (`CHARTER.md` §4.2 — run before calling any mechanism result)

1. **`I-Finite-CUDA`:** Zero CUDA kernel errors (`error == 0` on every `spiralcore_fbs_fwd`/`bwd` call), zero `NaN`/`Inf` in activations, gradients, or `MarginHinge` loss across all `1,024` steps, and non-zero step-0 query credit $\|d q\|_2 > 0$ in active arms.
2. **`I-DEV-Denominator`:** The `ft/dev` evaluator scores all `14,287` eligible response targets across `42,856` payload tokens (`400` documents) in both continuous `f32` and D11-exported integer mode (`evaluated_targets == 14287`).
3. **`I-Route-Mobility`:** In `Fbs_1Atom_Active` and `Fbs_2Atom_Mix`, at least $1.0\%$ of active `(output, stage)` queries cross an $E_8$ root boundary ($\Delta c_m \neq 0$) over `1,024` steps while step-1024 per-step route churn remains $< 0.50$.
4. **`I-Export-Exactness`:** On synthetic exact root-aligned fixtures, the compiled D11 integer table (`(selected_atom: u16, signed_action_state: u8 in 0..64)` per output block + Q12 `i32` `NativeVectorWriter`) reproduces the `f32` forward pass with zero top-1 discrepancy.

## 4. Bar and decision table

Let $L_{\text{dev}}^{(a,s)}$ and $\text{top1}_{\text{dev}}^{(a,s)}$ be the `14,287`-row `ft/dev` `MarginHinge(rivals=8)` loss and top-1 hit count at step `1,024`, let $\Delta_{\text{noise},L} = \max_a |L_{\text{dev}}^{(a,1)} - L_{\text{dev}}^{(a,2)}|$ be the two-seed DEV loss noise band (`CHARTER.md` §4.3), and let $F_I = \max(24, \lceil 2\sqrt{b + c}\rceil)$ be the paired McNemar top-1 noise threshold on `14,287` DEV rows:

| Gate | Pre-registered Bar | What PASS unlocks | What FAIL decides (`I-1..4` green) |
|---|---|---|---|
| `G-Fbs-Sheet-Fit` | On both seeds $s \in \{1, 2\}$: `Fbs_1Atom_Active` (or `Fbs_2Atom_Mix`) reduces `ft/dev` loss below step-0 initialization AND beats both `Fbs_FrozenQueries_Null` and `Fbs_IdentityTransport_Control` by $\ge \Delta_{\text{noise},L}$ on `ft/dev` loss (or by $b - c \ge F_I$ on paired `ft/dev` top-1 hits), with synchronized CUDA `tokens/s` reported. | Confirms offline-float SpiralCore/FBS learning and speed on the `14,287`-row DEV panel; unlocks fresh V2 transfer evaluation in the same run. | Closes this 6-layer `1,024`-step `StackAdamW` float configuration (`CHARTER.md` §4.1); SpiralCore/FBS remains `ACTIVE` on the discrete path (`crates/uor-r4-discrete`) and `READY` in float. |
| `G-Fbs-Capacity` | Diagnostic comparison between `Fbs_2Atom_Mix` and `Fbs_1Atom_Active` on both seeds: if $L_{\text{dev}}^{(\text{2Atom},s)} \le L_{\text{dev}}^{(\text{1Atom},s)} - \Delta_{\text{noise},L}$ on both seeds, 2-atom routed mixing is selected as the superior float candidate; otherwise 1-atom transport is retained as sufficient at $2\times$ lower projection cost. | Selects the winning capacity arm (`1Atom` vs `2Atom`) for D11 export-gap and fresh-V2 transfer scoring. | N/A (both arms are lawful geometric SpiralCore/FBS configurations; the comparison decides the block-routing arity). |
| `G-Fbs-ExportGap` | For the winning active arm, the D11 integer export (`(atom, action)` lookup table + Q12 `i32` shared vector writer, zero float/multiply instructions) evaluated on all `14,287` `ft/dev` targets has loss gap $|L_{\text{dev,int}}^{(a,s)} - L_{\text{dev,f32}}^{(a,s)}| \le \Delta_{\text{noise},L}$ and paired top-1 gap $|b - c| \le F_I$ on both seeds (`CHARTER.md` §3.7). | Qualifies the float-trained SpiralCore/FBS projection for lossless D11 integer table export. | Disqualifies direct post-training Q12 rounding for this float checkpoint (`CHARTER.md` §3.7) and requires QAT / root-snapped forward training before export. |
| `G-Fbs-Transfer` | If `G-Fbs-Sheet-Fit` and `G-Fbs-ExportGap` pass, the winning arm beats `Fbs_FrozenQueries_Null` on the sealed `14,479`-target fresh V2 draw (`0b7e754c…`) on both seeds (`CHARTER.md` §4.3). | Satisfies all reopen conditions of `docs/mechanisms/spiralcore-fbs.md:72-78` and closes `#2891`. | Closes `#2891` with a recorded negative transfer result (`CHARTER.md` §4.1); mechanism status stays `READY` in float. |

## 5. Data split, checkpoints, and pins

All inputs are fetched from `caseyallard/uor-r4-store:w2-inputs/2026-10-25/` and verified via `sha256sum -c`:
- **TRAIN (`mw-balp/train`):** `tokens.u16` SHA-256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` (`7,512,797` tokens / `2,425,925` eligible targets); `response_mask.u8` SHA-256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`.
- **DEV (`ft/dev`):** `tokens.u16` SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` (`42,856` payload tokens / `14,287` eligible targets across `400` documents); `response_mask.u8` SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- **Sealed fresh V2 transfer draw (`fresh_v2`, evaluated only on DEV `KEEP`):** `tokens.u16` SHA-256 `0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8` (`44,678` payload tokens / `14,479` eligible targets); `response_mask.u8` SHA-256 `a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e` (`seal` `df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f`).
- **Tokenizer (`vocab_size = 4096`):** `tokenizer.json` SHA-256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- **Kernel & compiler pins:** `crates/uor-r4-float-train/src/spiralcore_fbs.cu` (`517bffcc3c2029ddd171e75fd8fc1eb532755b30e683c66bd230382416480b0d`), `crates/uor-r4-float-train/src/spiralcore_fbs.rs` (`6eb2b09d5602322b7628a83f4cd3c2ee35c59d78786cb349fd92c4519ebbe3cd`), `crates/uor-r4-float-train/src/cuda_stack.cu` (`c577fce92ab965798857bd3f9d19ab86f65326dd501d8f8daf1537c0c9e9cb17`).

## 6. Steps and minimum test power (`CHARTER.md` §4.3 / D22 §3)

1. **Steps (probe ladder + full run on 1 $\times$ leased RTX 5090 via `scripts/pod/uor-pod`):**
   - *Step 1 (Probe a — synthetic & export exactness check, ~1 min):* Run `--mode synthetic-probe` (`I-Finite-CUDA` and `I-Export-Exactness` on exact fixtures plus 8-step descent check across all 4 arms).
   - *Step 2 (Probe b — throughput & boundary-crossing check, ~2 min):* Run `--mode throughput-probe` (32 steps on `mw-balp/train` + full `14,287`-target `ft/dev` timing) to verify `I-DEV-Denominator`, `I-Route-Mobility`, and project wall time $< 30$ minutes total.
   - *Step 3 (Full 2-seed run, replicated across two clean `nvcc` builds A & B):* Run `--mode full` (`4` arms $\times$ `2` seeds $\times$ `1,024` steps on `mw-balp/train`, scoring `ft/dev` in both `f32` and D11-exported integer mode at steps `0, 256, 512, 1024`, plus `fresh_v2` if `G-Fbs-Sheet-Fit` and `G-Fbs-ExportGap` pass).
   - *Step 4 (Archive & teardown):* Upload sealed receipts and checkpoints to `caseyallard/uor-r4-store:results/antigravity/spiralcore-fbs-offline-float-sheet-2026-10-25/` with `SHA256SUMS`, release and tear down the pod.
2. **Minimum test power:**
   - **Seeds:** `2` independent seeds (`seed in {1, 2}`).
   - **Evaluation rows:** `14,287` frozen `ft/dev` targets (`42,856` tokens) + `14,479` sealed `fresh_v2` transfer targets (`44,678` tokens; $\gg 200$ rows required by `CHARTER.md` §4.3).
   - **Trained in from step 0:** YES (`1,024` steps $= 786,432$ training tokens per arm/seed).

## 7. Cost, stop rule, and dispatch gate

- **Cost (charged cumulatively, 1 $\times$ RTX 5090 at $\approx \$0.90\text{–}\$1.89/\text{h}$ inside global $\le 4$ pods, $\le \$8.00/\text{h}$ cap):**
  - `preparation`: $\$0.00$ (laptop Rust authoring and stub-`nvcc` check).
  - `run` + `controls`: $\approx 90\text{ s}$ per `1,024`-step arm $\times 8$ arm-seeds $= 12\text{ min}$ per build ($\approx 24\text{ min}$ across Builds A & B, $\approx \$0.60$).
  - `evaluation`: $\approx 4\text{ min}$ total for `14,287`-row `ft/dev` (`f32` + D11 integer export) and `fresh_v2` scoring ($\approx \$0.10$).
  - `retries`: $\approx \$0.40$ allowance for one learning-rate calibration retry if `I-Route-Mobility` $< 1.0\%$.
  - `storage`: $< 64\text{ MiB}$ retained receipts/tables, respecting the `30 GiB` local free-space floor.
  - **Total estimated cost:** $\approx \$0.70$ (hard planning ceiling $\$2.00$, `--hours 2`).
- **Stop rule:** Stop immediately if `I-Finite-CUDA`, `I-DEV-Denominator`, or `I-Export-Exactness` fails, or if `I-Route-Mobility` remains $< 1.0\%$ after one learning-rate calibration pass. Never cancel a healthy run for a self-set time estimate (`CHARTER.md` §5.4).
- **Dispatch gate:** Runs on the lab's own fresh-context `TEST FITNESS: FIT` review (`CHARTER.md` §4.5), D68 decision review, and D67 pre-run review (`CHARTER.md` §5.1).

## 8. Non-author `TEST FITNESS` review (`CHARTER.md` §4.5)

TEST FITNESS: FIT

- Reviewed by fresh-context non-author reviewer (`antigravity` subagent `c217e9a3-92f6-4836-9077-d75c682ebfad`, 2026-10-25): all 8 `REQUIRED_BLOCKS` present and complete; all 4 geometric arms (`Fbs_1Atom_Active`, `Fbs_2Atom_Mix`, `Fbs_IdentityTransport_Control`, `Fbs_FrozenQueries_Null`) defined lawfully in `crates/uor-r4-float-train` under D37 with zero softmax/squashing/matmul; 8D block counts (`32` input atoms, `48` MLP atoms in 4-stage base-3 binder trees) prospectively justified; bars (`G-Fbs-Sheet-Fit`, `G-Fbs-Capacity`, `G-Fbs-ExportGap`, `G-Fbs-Transfer`), noise thresholds ($\Delta_{\text{noise},L}$, $F_I$), instrument checks (`I-Finite-CUDA`, `I-DEV-Denominator`, `I-Route-Mobility`, `I-Export-Exactness`), lead metric (`tokens/s`), D11 export gap, SHA-256 digests, power (`2` seeds, `14,287` DEV + `14,479` fresh V2 targets), cost ($\approx \$0.70$), and stop rules verified (`DECISION REVIEW: APPROVE` under D68).
- A FIT verdict clears compute for the configuration in that sheet only (D22).
