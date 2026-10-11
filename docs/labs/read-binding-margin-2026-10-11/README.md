# The competing-mass margin: the read separates twice as sharply and the answers do not move

References #2029 (M1 acceptance, D22 order 4 iteration; mechanism: read binding,
[brief](../mechanisms/read-binding.md)). Lab: DeepSeek, session `deepseek/margin`. Runs 2026-10-11 08:25–08:32 UTC
(**three fits, 117–122 s each**) plus BPB evaluations; panel scoring on the laptop CPU.

> **One line.** The keeper's read-binding supervision maximizes the mass on the *asked* value; the brief's open
> question is whether a **competing-mass margin** — pushing the read *off* the other stated values — does better.
> Implemented as a hinge and trained at three seeds against the keeper's own matched arms: the hinge **halves the
> competing mass** (0.086–0.090 → 0.042–0.046, its own causal check, at **zero parameters and zero inference
> MACs**) and the **answers do not move**: wrong-value failures **30 → 30 over 120 rows**, memory **65 → 64**, with
> the per-seed signs flipping (seed 1: 12 → 10; seed 2: 5 → **14**; seed 3: 13 → **6**). **REJECT this configuration**
> — the bar needed ≥ 5 fewer wrong-value failures on ≥ 2 of 3 seeds and got one — mechanism stays open (D22 §1).

## Design (pre-registered before compute)

| | |
|---|---|
| change | the objective gains `0.5 · mean(relu(competing mass − 0.03))` — inert below 3 % competing mass, active from the first step (the keeper's competing mass is 0.081–0.090 on every seed) |
| arms | **three seeds** (`data_seed` 20261009/10/11) at `read_binding_supervision=0.1` + the margin above |
| matched baseline | **the keeper's own arms S1/S2/S3** — same base, store, steps, recipe and seeds; the margin flags are the only difference |
| everything else | `chat-29m-B-lr5e-4`, the adopted `mix-10` store with its `binding_labels.json[l]` sidecar (`source=recall`), supervision from step 0, `steps=2000 batch=16 lr=2e-4 warmup=100 context=384 device=cuda` |
| bar | ≥ 5 fewer wrong-value failures than the matched arm on ≥ 2 of 3 seeds **or** ≥ +4 memory rows in aggregate; memory not below, BPB within 0.01 |

## Results

### The margin's own objective — the mechanism does exactly what it claims
| arm | bound mass | **competing mass** | hinge term | keeper's competing mass (matched seed) |
|---|---:|---:|---:|---:|
| **M1** | 0.9012 | **0.0462** | 0.0397 | 0.0897 |
| **M2** | 0.9078 | **0.0422** | 0.0361 | 0.0858 |
| **M3** | 0.9060 | **0.0433** | 0.0373 | 0.0812 |

**Competing mass halves on every seed** and bound mass rises slightly. This is the mechanism's own target met.

### The answers — paired against the keeper, row by row
| pair | data order | margin arm memory | keeper memory | Δ | margin arm wrong-value | keeper wrong-value | Δ |
|---|---|---:|---:|---:|---:|---:|---:|
| M1 vs S1 | 20261009 | 18/40 | 19/40 | −1 | 10 | 12 | **−2** |
| M2 vs S2 | 20261010 | 20/40 | **24/40** | **−4** | **14** | **5** | **+9** |
| M3 vs S3 | 20261011 | **26/40** | 22/40 | **+4** | **6** | 13 | **−7** |
| **aggregate** | — | **64/120** | **65/120** | **−1** | **30** | **30** | **0** |

**Bar NOT MET**: one seed of three reaches the −5 wrong-value bar; the aggregate is a wash (0 wrong-value change,
−1 memory row). Float BPB: M1 **1.17485** against the keeper's S1 1.17142 and the anchor's 1.17425 — a +0.0034
regression against its matched arm, inside the 0.01 the bar allows; M2 and M3 are still evaluating and are named
pending.

### The compute ledger
| | keeper | **+ margin** | change |
|---|---:|---:|---:|
| parameters | 28,958,761 | 28,958,761 | **0** |
| MACs/token, served (context 384) | 30,203,136 | 30,203,136 | **0** (training-only term) |
| wall per training step | 0.058 s | 0.060 s | **+~4 %** |
| competing mass | 0.086–0.090 | **0.042–0.046** | **−50 %** |
| wrong-value failures over 120 | 30 | **30** | **0** |
| memory over 120 | 65 | **64** | −1 |

## What it establishes

1. **Sharper separation of the read is not where the remaining failures live.** The margin halves the quantity the
   brief names as the fix and changes nothing in aggregate. So the ~30 wrong-value failures of 120 rows are **not**
   explained by "the read spread onto the competitors" — that is now measured, not assumed. Whatever produces them
   survives a read that is twice as decisive between the asked value and its distractors.
2. **The keeper's gain came from steering, not from sharpening.** Read binding moved the panel (47 → 30 wrong-value
   against controls) by putting mass on the asked value at all; pushing the same quantity further buys nothing. The
   mechanism's second lever is spent in this configuration.
3. **Zero-cost does not mean free of trade-offs.** The term costs no parameters and no inference arithmetic and still
   costs ~4 % of training wall — and on one seed it is markedly worse (5 → 14 wrong-value). Per-seed variance here
   (±5 rows on wrong-value) exceeds the panel's ±3-row noise, which is why the three-seed design was required and
   why a single-seed reading of this cycle would have been wrong in either direction.
4. **It sharpens the case for changing what the read *does*.** Two cycles in a row have now found the read's
   *scoring* exhausted — one negative for geometric exactness at 13.4× the cost, one neutral for sharper separation
   at zero cost. An **address** instead of a *search* is the remaining change on this path that alters the
   computation rather than the sharpness of the same softmax, and it is also the only one with a compute argument
   (the read is 17.6 % of the stack's MACs at context 384 and 40.5 % at 4,096).

## Limitations

- **Three seeds, 40 memory rows** (±3 rows of noise) — the standing declared deviation from the brief's ≥ 120 rows;
  the per-seed spread here is larger than that noise, so the aggregate is the honest summary.
- **Two of three BPB evaluations pending** (M2, M3); M1's is inside the bar's 0.01 but above its matched arm.
- **The margin weight and level are one configuration** (0.5 at 0.03): a stronger weight might over-constrain and a
  weaker one under-constrain; the negative closes this configuration, not the margin idea.
- **The masses remain train-side**, as in the previous cycles.

## Cost

| | |
|---|---|
| pod | `rtbm6u9bvvi16c`, 2 × RTX 5090: three fits at **117–122 s** each ≈ 6 minutes of GPU, plus three BPB evaluations ≈ $0.90 |
| laptop CPU | three v5 reply sweeps (64 rows, cap 64) |
| external | none beyond the pod; inside the caps |

## Evidence

- Arm reports and models: `/workspace/uor-r4/deepseek/margin-20261011/runs/{M1,M2,M3}/`, pulled and bundled to
  cloud-store as `[BUNDLE]`; BPB reports in the same volume's `bpb-*/`.
- Paired comparison against the keeper: `readbind2-20261011/score/v5-{S1,S2,S3}` against
  `margin-20261011/score/v5-{M1,M2,M3}`, classified with the same judge-free row checker.
- Pre-registration and fitness review: [#2029 comment 6107040390](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6107040390);
  the term itself: [#2196](https://github.com/UOR-Foundation/uor-r4/pull/2196).
