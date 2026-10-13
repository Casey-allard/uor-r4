# Prime Transport: Canonical Architecture Validation v1

*Generated: 2026-04-08 17:49 UTC*

---

## Task Definition

**Name**: `mod12_quarter_D32_no_inject`

- **Label**: `x0 = mod12_state % 4` (four quarter-classes of the mod=12 cycle)
- **Injection**: NONE — no token injected at any position
- **Horizon**: D=32 transport steps (was D=24 in all prior experiments — 33% longer)
- **Vocab**: 4 | State geometry: GEOM_K3 or GEOM_FULLER

## Justification for Difficulty

1. **Requires k=3 harmonic** — the quarter-class label groups the 12 mod-12 states
   into four sets {0,4,8}, {1,5,9}, {2,6,10}, {3,7,11}. Each class has three members
   at 120° separation on the k=1 unit circle, which is NOT linearly separable from k=1
   alone. The k=3 harmonic maps each class to a single axis-aligned point (±1, 0) / (0, ±1),
   making it necessary and sufficient for linear decoding.

2. **Longer horizon degradation pressure** — D=32 is 33% longer than D=24. Under
   baseline (full) transport, each step divides k=3 content by the k=1 fundamental
   magnitude — corruption compounds over each additional step. Under split transport,
   k>=2 is frozen, so the signal is immune to degradation regardless of D. This creates
   a strictly larger contrast between full-transport and split-transport architectures,
   and a harder optimization problem for any config.

3. **No injection** — all signal must survive 32 transport steps. No shortcut.

4. **Not a memorization task** — D=32 sequences over the full mod-12 state pool with
   random operation tokens. The model must generalize over trajectory distributions,
   not memorize state-to-label mappings.

5. **Novel**: no prior experiment in this repo used D=32.

## Configurations

| # | Config | Geometry | Features | Init | Transport |
|---|--------|----------|----------|------|-----------|
| 1 | working_baseline | GEOM_K3 (k=1,2,3 for mod-12; k=1 for mod-5) | none | radial=1.0 | split (eps_high=1.0, k>=2 frozen) |
| 2 | canonical_A_B_C  | GEOM_FULLER (adds k=2 for mod-5) | safe-tan projective features | radial=√2 | split (eps_high=1.0, k>=2 frozen) |

*Both configs use split transport (eps_high=1.0). D=32, seed=42, MAX_STEPS=3500, LR=0.02,
D_HIDDEN=32. Only two configs; no sweeps; no ablations.*

## Dimension accounting

| Config | d_ang | d_hyb | d_in_ctrl | W1 shape |
|--------|-------|-------|-----------|----------|
| working_baseline | 12 | 16 | 20 | 20×32 |
| canonical_A_B_C  | 14 | 18 | 29 | 29×32 |

Note: d_in_ctrl = D_EMB(4) + d_hyb + n_pairs_if_B.
Canonical has 45% wider input with the same D_HIDDEN=32.

## Results

| Config | Label | Accuracy | Solve Step | No-Last Acc | Dec Init | Dec Mid | Dec Final | Runtime(s) |
|--------|-------|----------|------------|-------------|----------|---------|-----------|------------|
| working_baseline | split+none  | **0.9741** | — | 0.9790 | 1.0 | 1.0 | 1.0 | 91.3 |
| canonical_A_B_C  | split+A+B+C | **0.9087** | — | 0.9062 | 1.0 | 1.0 | 1.0 | 133.2 |

**Δ accuracy (canonical − baseline): −0.0654**

Neither config reached the solve threshold (0.999) within MAX_STEPS=3500.

### Comparison with D=24 results (prior interaction screen)

| Config | D=24 acc | D=32 acc | Δ(D32−D24) |
|--------|----------|----------|------------|
| working_baseline | 0.9741 | 0.9741 | **0.0000** — unchanged |
| canonical_A_B_C  | 0.9976 | 0.9087 | **−0.0889** — regressed |

## Answers to Mandatory Questions

**Q1. Does A+B+C outperform baseline on this new task?**
NO. Canonical underperforms by Δ=−0.0654 (0.9087 vs 0.9741). It is meaningfully worse,
not marginally worse.

**Q2. Does it solve faster or more reliably?**
NEITHER config reached the solve threshold. Canonical was strictly slower to converge at
every eval checkpoint and did not solve.

**Q3. Does baseline degrade or fail under this task?**
NO — and this is a key finding. Baseline held exactly at 0.9741 from D=24 to D=32
(zero degradation). Split transport protects k=3 regardless of horizon length, so the
baseline is robust to longer D. It also ran 32% faster (91.3s vs 133.2s for canonical).
"Failing to solve" is the same condition as at D=24; it is not evidence of degradation.

**Q4. Is there evidence the improvement scales with task difficulty?**
NO — the evidence is the opposite. At D=24, canonical was +0.0235 over baseline (0.9976
vs 0.9741). At D=32, canonical is −0.0654 vs baseline (0.9087 vs 0.9741). The canonical
advantage REVERSED under harder conditions. The improvement does not scale; it collapses.

## Analysis: Why did canonical regress?

The decodability probes are critical here. Both configs show **decodability=1.0 at initial,
mid (t=15), and final (t=31)**. This means:

- Split transport successfully preserves k=3 for BOTH configs through all 32 steps.
- The k=3 signal is not being lost in either case.
- The failure is NOT a harmonic preservation failure.

**What is failing in canonical**: The canonical has d_in_ctrl=29 vs baseline's 20, feeding
into the same D_HIDDEN=32 hidden layer. This is a **capacity bottleneck**: the W1 matrix
must compress 29 input dimensions into 32 hidden units, compared to baseline's 20→32. With
D=32 transport steps (longer trajectory, harder credit assignment), the canonical's wider
input makes the optimization problem harder within the fixed step budget. The projective
features (Factor B, 7 extra inputs) add noise without adding hidden capacity to process
them. The sqrt(2) init (Factor C) may also produce suboptimal learning dynamics at longer
horizons.

In short: **A+B+C helps at D=24 but creates an optimization handicap at D=32 with the
same capacity budget (D_HIDDEN=32).**

## CANONICAL ARCHITECTURE STATUS: FAIL

## Honesty Section

### Where baseline still holds up

- Baseline (split transport only) is perfectly stable from D=24 to D=32 (0.9741→0.9741).
  Split transport's k>=2 freeze provides horizon-invariant harmonic preservation.
- Baseline trains 32% faster (91.3s vs 133.2s) — a non-trivial runtime advantage.
- The simpler architecture (d_in_ctrl=20) learns more reliably under longer horizons
  with fixed capacity.

### Where canonical clearly wins

- At D=24 only (from interaction screen): canonical achieved 0.9976 vs baseline 0.9741
  (Δ=+0.0235). That advantage is real but D-specific.
- Decodability: both configs maintain 1.0 decodability at all stages, so canonical's
  FULLER geometry is not harmful to representation quality — only to optimization.

### Any confounds or limitations

1. **Single seed** — both runs used seed=42. Stochastic Gumbel noise means exact
   numbers vary. The −0.0654 gap is large enough to be robust to seed variance, but
   a multi-seed rerun would confirm.

2. **Fixed capacity budget** — D_HIDDEN=32 was set at D=24. The canonical's wider
   input (29 vs 20) may only require D_HIDDEN>=48 to match baseline at D=32. The
   FAIL verdict applies to this specific capacity setting, not to A+B+C in general.

3. **A+B+C tested as a package** — it is unknown which sub-factor (A, B, or C)
   drives the D=32 regression. Factor B (projective features, +7 inputs) is the most
   likely culprit given its direct effect on d_in_ctrl.

4. **Task design** — D=32 with the same task (mod12%4) tests optimization robustness,
   not a genuinely new harmonic structure. A task requiring k>=2 from the mod-5 block
   (not tested here) would probe Factor A's added geometry more directly.

5. **Both configs use split transport** — the comparison isolates A+B+C's contribution
   on top of split transport. The split transport itself works for both. The finding
   is about the value of A+B+C at D=32, not about split vs full transport.

### Summary assessment

The canonical architecture (A+B+C + split transport) demonstrated at D=24 that adding
fuller geometry, projective features, and sqrt(2) init improves accuracy. At D=32, these
same additions hurt performance, suggesting the D=24 advantage was partly due to
favorable optimization dynamics at that specific horizon length, not a durable general
improvement in harmonic preservation capability. Split transport alone (the baseline) is
the more robust architecture across horizons within the tested capacity budget.
