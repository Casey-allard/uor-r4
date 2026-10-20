# redteam2_sci: science and novelty review of ROADMAP_DRAFT (2026-09-26, reviewed after the 17:3x update)

Labels: **M** Measured here (`$S/lab/exp/redteam2_sci/`, one thread, each run <1 min), **D** Derived, **L** Literature (retrieved this session), **H** Hypothesis.

## Verdict

- **The chat plan is sound; the geometric claim is not yet.** As drafted, GCM-L is 4-bit SmolLM2 with dot attention rewritten in curvature form. So far curvature stays near zero unless forced, and forcing it costs loss.
- **The three "exact conversions" are exact only in a limit, where they are dot attention.** Near it, curvature adds one scalar times a fixed quartic feature, not a hierarchy mechanism.
- **The two pillars conflict.** Exact conversion keeps MIPS geometry (queries far from their best keys), which key-built indexes handle badly; indexable geometry belongs to distance-scored heads, which do not convert exactly.
- **Most components are prior art** (table below).

## Defects

### CRITICAL

**C1. "Hyperbolic in name only": the κ≈0 basin absorbs the plan** (§2 row "Exact conversions", §3.2, M1b, owner decision 3).

- **D.** For the lifted points, d_κ² = |q−k|² − κ[(|q|²−|k|²)²/4 + |q−k|⁴/12] + O(κ²). So the key-norm score is q·k − |q|²/2 + κ[(|q|²−|k|²)²/8 + |q−k|⁴/24], and the intrinsic score subtracts a further κ|k|⁴/6.
  - **M** (`taylor_check.py`): at κ = 1e-5 the error falls from 7.5e-2 (flat) to 3.5e-5 (first order).
- **Norm completion** puts every query on one hyperbolic sphere and every key on another (radius 4.1–6.4). It is exact only as QC→∞. The radius channel, which is the claimed hierarchy carrier, is constant by construction (math2 §4).
- **Curvature never left the basin unless forced.** All M:
  - the lead's Q/K knowledge distillation (KD): log ε went from −6 to −5.98;
  - Rust e2e self-KD: κ fell from 2.5e-3 to 4e-4 in 60 steps (`e2e/kn_kd`);
  - math2 E11: QC went from 1.0e4 to 1.02e4, with non-root accuracy stuck at 0.4%;
  - the running `ft_hpol_s2`: log ε reached −5.6 to −5.8, i.e. t ≈ 0.002–0.006, where t = κ·median|k|² is the dimensionless curvature.
- **Forcing curvature costs loss.**
  - **M** (`curvature_drive.py`, stand-in, one layer set to genuinely curved t = 1, no retraining): intrinsic +0.012 to +0.017 bits/byte; key-norm +0.015 / +0.067 / +0.161.
  - E11b's direct curved lift crashed raw accuracy from 91% to 1% before recovering.
- **L, same pattern elsewhere.**
  - RADLADS converted Qwen3-8B to Softpick attention with scores "rivaling or exceeding" the teacher. The distilled model "still contained these sinks", so the new attention's purpose was not achieved (2505.03005 App. D).
  - FPS-T reports that κ-GCN with curvatures initialised at zero "does not perform well… potentially due to issues in optimization" (2309.04082).
- **Consequence.** The likeliest deliverable is a 4-bit integer-served SmolLM2: a transformer backbone, against AGENTS.md and the owner's "redesign attention" goal.
- **Fix.** Relabel the §2 row "flat-limit reparametrisations, exact only as κ→0". Get the owner's answer to decision 3 **before** M1b/M2. Pre-register: if fewer than about 10% of heads end at t ≥ 0.3 *and* beat a matched-capacity control (M4), report "SmolLM2-135M, 4-bit, integer-served; geometry not in the backbone".

### MAJOR

**M1. Exact conversion and "geometry pays as index" conflict** (§1.2, §3.3, M4). **M** (`real_qk_admission.py`): D8 cycle-3 read heads (width 64), 8,192 keys pooled across windows, 1,024 queries, 128 k-means cells. The age bias and NoRead slot are omitted; one clustering seed.

| Head | cos(q, top-1 key) | Top-1 admitted at 2.5% / 5.5% / 25% scored | Attention mass admitted at 2.5% |
|---|---:|---:|---:|
| Dot, seeds 3 and 2 | 0.33 / 0.31 | 79% / 92–93% / 99.8% | 58% |
| Lorentz, trained from scratch | 0.95 | 93% / 99.2% / 100% | 60% |
| exp2 synthetic | 0.995 | 99.8% at 2.8% | — |

- **Dot heads show RetrievalAttention's query–key gap** (L, 2409.10516: IVF scans about 30–50%).
- **exp2's routing rule reverses on real heads.** The L2 rule ⟨q,c⟩ − ‖c‖²/2 admits 37% at 1.5% scored against 68% for plain MIPS. Its 98.2% vs 19.5% came from near-copy queries.
- **Distance scoring makes heads indexable** (a point for the geometry), but exactly converting scores keep MIPS's argmax at κ→0 (**D**), and the pure-distance read does not convert (zero-shot 4.01 bits/byte on the stand-in).
- **M4's gate is far off.** It asks for ≥99% of attention mass at ≤2% scored; real heads give 58–61% at 2.5%.
- **Fix.** Gate M4 on real SmolLM2 query/key dumps; test query-aware indexes; give index-friendly geometry only to heads or memory *trained* with distance scores.

**M2. The stand-in cannot discriminate between geometries** (§2 table and its "Reading").

- **M** (`standin_locality.py`, rows t ≥ 32):
  - attention mass within 3 bytes is 68% / 93% / 98% for layers 0 / 1 / 2;
  - mass beyond 32 bytes is 10% / 0.9% / 0.8%;
  - a content-free Toeplitz pattern (fixed by distance) gives 2.92 bits/byte; no attention gives 4.22.
- **Its heads are local byte mixers, not induction, retrieval or sink heads**, so all six scores recover to within 0.004 bits/byte, even one starting at 4.01.
- **The intrinsic-over-key-norm preference may reverse on sink heads.** On arch2's synthetic sink head, intrinsic reaches KL 3.75 nats at κ = 1 while key-norm stays at 0 (M, arch2).
- **Fix.** Drop "swapping score geometry is cheap"; in M1a report KL against t separately for sink heads.

**M3. math2's decomposition: confounded headline, mislabelled ceiling** (§1.2).

- **Enclosing-scope copies are twice as far back.** Mean copy distance is 34 tokens for enclosing scope against 18 for same scope (**M**, `scope_vs_distance.py`).
- **Regressing the Lorentz−Dot gap on category + log₂(distance):**
  - the enclosing coefficient is −0.0005 [−0.037, 0.029], −0.002 [−0.035, 0.028] and −0.060 [−0.093, −0.026] in seeds 2 / 3 / 4;
  - the Lorentz gain grows with copy distance in all three seeds: −0.018 / −0.027 / −0.013 nats per doubling, CIs exclude 0.
  - A residual hierarchy effect is not excluded: in the 17–48-token bin, enclosing copies beat same-scope copies by 0.03–0.05 nats in all three seeds.
  - "Enclosing" was the one consistent category out of 15 category × seed tests.
- **The "≤0.02-nat perfect-oracle ceiling" is not a ceiling.** It measures what six scope parameters add to a 14-parameter copy+bigram model, which is a lower-bound-type number.
- **The "0.40 nats from context" is for a predictor with no parametric knowledge.** SmolLM2 already attends over its context, so an index can only *save cost*: about 7% of step time at 2K (sys2).
- **Fix.** Report distance-matched effects, relabel the oracle, mark §1.2's last sentence as Hypothesis.

**M4. M1b cannot decide.**

- **Annealing imposes κ**, so κ histograms are not evidence that heads want curvature.
- **The control is missing.** Near the limit, curvature is a scalar times a fixed feature, so the comparison must be against dot + per-key potential + score² (the matched-capacity control math2 used in E11). Plain dot is not enough.
- **The grid is not dimensionless.** anneal_to ∈ {−3, −2, −1} means t = 0.5–1.3 / 3.5–9.4 / 26–70 on stand-in norms (**M**).
- **Two seeds are too few** (cycle 3 needed four for 0.018 nats), and **chat "hierarchy events" are undefined.**
- **Fix.**
  - Set the grid in t per head; add the control; pre-register a threshold against seed noise.
  - Define chat events first (e.g. entity recall at distance); log whether κ decays after the anneal.
  - Pair seeds: ft.py currently compares dot seed 1 with hpol seed 2.

**M5. Novelty is overstated.** arch2's "new mechanism: a curvature homotopy" and similar claims need citations (table below).

### MINOR

- **m1. The energy thesis (§1.3) does not depend on the geometry.** The savings come from ≤4-bit look-up-table (LUT) weights and running on efficiency cores (E-cores). The KV saving comes from a radius+direction code, which is norm-explicit/shape-gain quantisation and works on dot keys too.
  - Add a 4-bit dot-KV baseline and a 36-bit norm-explicit quantisation (NEQ) baseline for dot keys (L, NEQ 1911.04654).
- **m2. sys2 already showed the "no multiplier" rule does not pay** (a multiplier's energy share is ~0%; tables cost 1.8× hardware; M/D sys2). Put sys2's question 4 to the owner; it is missing from §6.
- **m3. Literature re-check, all confirmed:** HELM near chance (MMLU 23–26; CSQA ≈20) and 1.5–1.8× training time (2505.24722); the RADLADS Softpick quote; T-MAC's "70%" (abstract) and "20.6/61.2/51.3%" (§5.4) (2407.00088).

## Novelty: what is known, and what may be new

| Component | Status (L = retrieved this session) |
|---|---|
| Score −βd_H(q,k) − c (the cycle-3 read) | Gulcehre et al., 1805.09786 |
| Squared-Lorentz score; RoPE as a spatial Lorentz rotation | HELM (HoPE), 2505.24722 |
| Lift that recomputes x₀ from space coordinates | HypLoRA LLR (2410.04010); HELM. HypLoRA App. E derives ΔQ ≈ BAx + (‖x‖²/6K)·BAx at small norm |
| Per-head learnable κ, initialised at 0, "equivalent to the original Transformer as κ→0" | FPS-T, 2309.04082; κ-GCN, 1911.05076 |
| Radius+direction key code | NEQ / shape-gain, 1911.04654; gain–shape quantisation for sub-1-bit KV caches, 2607.01065 (search snippet only) |
| Key-built index fails on query→key search | RetrievalAttention, 2409.10516 |
| **Possibly new (H; not found)** | Polarisation scores whose κ→0 limit reproduces a *pretrained* head exactly (the intrinsic one is the classical-MDS Gram entry with hyperbolic distances); the hyperbolic norm-completion lift; Busemann-landmark admission for attention; the flat-basin negative result (E11); Gromov-product attention |

## Steelman, both ways

- **"Geometry is decorative."**
  - The cycle-3 gain is +0.018 nats, below the lab's own 0.03 bar, and two of three seeds' "hierarchy" gain is distance (M3).
  - Converted curvature stays near 0 (C1).
  - Dot and hyperbolic keys tie on flat recall (exp2).
  - The projected energy savings do not depend on geometry (m1).
  - HELM-class models are near chance (L).
  - So 4-bit SmolLM2 on T-MAC-style kernels and E-cores, with a 4-bit KV cache, captures nearly all of GCM-L's projected energy win at lower risk. No equal-budget transformer or RWKV/Mamba baseline has been run at chat scale.
- **"The geometric path is right."**
  - Hyperbolic keys learn sparse hierarchies that dot keys never learn (85–90% vs 0.4%, M, math2).
  - Distance-scored heads co-locate queries with keys, making attention indexable (M1: 99% top-1 at 5.5% scored vs 92%).
  - HypLoRA gains with curvature K = 0.5–1, well away from the flat limit (L).
  - All favour heads and memory *trained* hyperbolic, not flat-limit conversion.

## Constraint check

| Proposal | No float or multiplier at serving | ≤4-bit additive weight maps | No transformer backbone | Weight transfer |
|---|---|---|---|---|
| GCM-L with κ≈0 heads snapped to int8 dot tables | OK via tables | OK (weights). q·k, gating and value mixing are activation products, which D0-b does not cover; tables satisfy the letter only | **Violated unless owner decision 3 says otherwise** | OK |
| Engram / product-key memory | OK | OK | Needs the D5 ruling | — |

## What is right and should not change

- Conversion as the only in-budget route to chat; SmolLM2-135M first; 4-bit before ternary.
- M1a/M1c first; math2's E11/E11b honesty and the updated falsifier; checkpoints, sealed reports, paired data; hyperbolic geometry for trained memory/index layers.

## The one test to run first

**Flat-limit curvature drive on real SmolLM2-135M-Instruct heads** (owner's M1, under 1 h, no training; add to M1a). On about 200 held-out chat sequences, per head:

1. g_h = ∂L/∂t_h at t = 0, where t = κ·median|k|² (exact, from the first-order form in C1), under 360M-KD and under chat NLL;
2. the exact zero-shot ΔL at t ∈ {0.1, 1};
3. cos(q, top-1 key), and top-1 admission at 2–5% scored with a key-built index.

**Pre-registered rule:** if no head has −g_h·1 larger than its t = 1 cost, skip the κ-score arms of M1b. Keep dot heads, move hyperbolic geometry to trained memory and index layers, and tell the owner the backbone is a quantised transformer.

**Stand-in demo** (M, `curvature_drive.py`, ~1 min):
- g = −0.0073 / −0.0011 / −0.0004 bits/byte per unit t for intrinsic (batch s.d. 0.004–0.008);
- the self-KD gradient is 1e-9, as expected;
- the t = 1 costs above exceed the first-order gains, so the stand-in fails the rule.
