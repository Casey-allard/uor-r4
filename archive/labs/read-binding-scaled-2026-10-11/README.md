# Read binding at 8,000 steps: the mechanism's advantage doubles and the paired test turns significant — wrong-value 49 → 23 of 120, memory 48 → 64

References #2029 (M1 acceptance, D22 order 4 iteration). Lab: DeepSeek, session `deepseek/scaled`.
Runs 2026-10-11 17:31–18:31 UTC: **one 2 × RTX 4090 pod, six fits of 8,000 steps at 573–582 s each, ≈ $1.90**, released
and deleted at the last fit. Panel scoring on the owner's laptop.

> **One line.** The line's best result was the powered read binding KEEP at `steps=2000` (wrong-value 47 → 30 of 120,
> memory 57 → 65, three seeds against matched controls). The necessary next run was not another mechanism: it was **the
> same mechanism, trained four times longer**, so the comparison is mechanism-versus-no-mechanism at equal budget.
> At 8,000 steps the mechanism's advantage **doubles**: wrong-value failures **49 → 23 over 120 rows (−53 %)**, memory
> **48 → 64 of 120 (+16)**, with **every seed positive** and **two of three individually significant** (p = 0.031, 0.035;
> Fisher χ² = 18.4 on 6 df), 36 rows repaired against 10 broken. The mechanism's own objective sharpens with the budget
> too: bound mass **0.969–0.971** against the 2,000-step run's 0.896–0.905, competing mass **0.026–0.027** against
> 0.081–0.090. **KEEP + iterate.**

## The run, and what it was for

| | |
|---|---|
| base / data | `chat-29m-B-lr5e-4`, the adopted `mix-10` store and its own `binding_labels.json[l]` sidecar (`source=recall`) |
| recipe | the keeper's exactly, `steps 2000 → 8000`, `batch=16 lr=2e-4 warmup=100 protocol=2 context=384 policy=full_prefix`, `pointer=32`, `device=cuda` |
| arms | **R1/R2/R3**: `read_binding_supervision=0.1`, seeds 20261009/10/11 |
| matched controls | **K1/K2/K3**: the same 8,000 steps with `read_binding_supervision=0` — equal budget, mechanism the only difference |
| the pod | one 2 × RTX 4090 pod, both GPUs at **100 %** for the whole batch, three fits per GPU; **released and deleted the moment the last fit ended** ($0/h since) |

## Results

### The failure mode, paired row by row
| pair | data order | read binding WV | control WV | repaired | broken | **p** (exact McNemar) | read binding memory | control memory |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| R1 vs K1 | 20261009 | **8** | 15 | 10 | 3 | 0.0923 | **20/40** | 16/40 |
| **R2 vs K2** | 20261010 | **10** | 20 | 14 | 4 | **0.0309** | **20/40** | 13/40 |
| **R3 vs K3** | 20261011 | **5** | 14 | 12 | 3 | **0.0352** | **24/40** | 19/40 |
| **aggregate** | — | **23** | **49** | **36** | **10** | **Fisher χ² = 18.4 (6 df)** | **64/120** | **48/120** |

**Bar: MET on the failure mode** (23 ≤ the 30 the pre-registration allowed, against a control at 49) and **the headline
moves with it** (64 against 48 — the mechanism's contribution, which is the comparison that matters here). Stated plainly:
the arms' absolute memory is **64/120 against the 2,000-step keeper's 65/120 — one row under, inside the ±3-row noise**,
while the *controls* fell to 48/120, so the mechanism's **relative** advantage went from **+8 to +16 rows** of 120.

### The mechanism's own objective and the budget
| run | bound mass | competing mass | dev NLL (three seeds) | memory over 120 | wrong-value over 120 |
|---|---|---|---:|---:|---:|
| keeper, 2,000 steps | 0.896 / 0.899 / 0.905 | 0.090 / 0.086 / 0.081 | 0.363 / 0.394 / 0.309 | **65** | **30** |
| **this run, 8,000 steps** | **0.9689 / 0.9702 / 0.9706** | **0.0268 / 0.0256 / 0.0257** | **0.240 / 0.235 / 0.325** | **64** | **23** |
| controls, 8,000 steps | — | — | 0.266 / 0.302 / 0.288 | 48 | 49 |

**Bound mass rises with the budget and competing mass falls threefold** (0.026 against 0.090), which is the mechanism
doing exactly what it claims while the answers follow it — the opposite of the [competing-mass margin cycle](read-binding-margin-2026-10-11/README.md),
where the masses improved while the panel did not: here the improvement comes from *more training of the same signal*,
and the panel moves.

## What it establishes

1. **The mechanism scales with budget, and its advantage grows.** At 2,000 steps it bought +8 memory rows over its
   controls and −17 wrong-value failures; at 8,000 steps, **+16 rows and −26 failures**. That is a mechanism whose value
   increases with training rather than saturating — the cheapest possible evidence that it is worth iterating.
2. **The paired test turns significant.** Two of three seeds are individually significant and the combined test is
   χ² = 18.4 on 6 df; 36 rows repaired against 10 broken. At 2,000 steps only one seed was significant.
3. **The controls got worse as they trained longer without the signal** (57/120 → 48/120 memory, 30 → 49 wrong-value),
   which is the clearest statement of what the supervision is doing: without it, more steps on this data drift the read
   away from the asked value.

## Limitations

- **Three seeds, 40 memory rows** (±3 rows) — the standing declared deviation from the brief's ≥ 120 rows.
- **BPB is not reported**: under the owner's metric direction it is a diagnostic, not a bar, and the pod's volume did not
  carry the stream this cycle. The development NLLs are reported instead (0.240 / 0.235 / 0.325 against the controls'
  0.266 / 0.302 / 0.288 — better on two seeds, worse on the third).
- **The reply panel is graded separately** (three arms on the frozen judge) and is reported with the cycle's card rather
  than here.
- **Nothing about serving changed**, so no D11 or token-rate claim is made this cycle.

## Cost

| | |
|---|---|
| pod | one 2 × RTX 4090 pod, ~1.0 h, six fits at 573–582 s, both GPUs busy throughout: **≈ $1.90** |
| laptop CPU | six v5 reply sweeps and the paired classification (cheap work kept off the pod) |
| pod discipline | inputs checked on the volume **before** the run; the earlier `sm_120`-binary failure on 4090s cost 4 minutes and was fixed by re-bootstrapping for the pod's own arch; the stale report roots from that failure were cleared; **released and deleted at the last fit, $0/h since** |

## Evidence

- Arm reports and models: `/workspace/uor-r4/deepseek/scaled-20261011/runs/{R1,R2,R3,K1,K2,K3}/`, pulled and bundled to
  cloud-store as `[BUNDLE]`; the paired test is the same judge-free classifier used in the last four cycles.
- Pre-registration: [#2029 comment 6111708079](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6111708079).
