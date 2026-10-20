# Exact addressed memory at the read: the product-key address replaces the search, the wrong-value rate falls 47 → 34 of 120, and the ledger says what it costs

References #2029 (M1 acceptance, **D22 order 2** — *"Exact addressed memory … train a memory-equipped stack in"*).
Lab: DeepSeek, session `deepseek/addrread`. Runs 2026-10-11 09:46–11:16 UTC: **four fits at 39.5 minutes each**, one pod,
≈ $2.40 (three seeds at the last read layer, one placement arm at the first). Panel scoring on the laptop CPU.

> **One line.** The read is the answer path ([attribution](../labs/answer-path-2026-10-11/README.md),
> [ablation](../labs/head-ablation-2026-10-11/README.md)) and its *scoring* is exhausted (two negatives: exactness at 13.4×
> cost, sharper separation at zero cost). So this cycle replaces the read's **computation** at the answer layer: `memory_layers=8`
> puts the **product-key memory** — a query map, sub-key scoring and a value gather, i.e. an address lookup — in place of the
> softmax search at the last read layer, trained **in from step 0**. Against matched no-memory controls at the same seeds:
> **wrong-value failures 47 → 34 over 120 rows (−13, −28 %)**, on **two of three seeds** (−7 and −6, the third 0 — the
> pre-registered bar), with the headline **exactly flat (57/120 against 57/120)**. The ledger says what it costs: per-token
> compute at the replaced layer **−90 %** (and −97 % at 4,096 context), but **state +36.9 M parameters** and, as implemented,
> **20× the wall time per training step** (host bridge). **KEEP + iterate** (D22 §2): the mechanism met its target with no
> headline loss; the iteration is a leaner codebook and a fused kernel, both named below.

## Design (pre-registered before compute)

| | |
|---|---|
| change | `memory_layers=8` — the memory **replaces the read** at the last read layer (the layer whose removal erases the answers): `memory_heads=4 memory_key_dim=128 memory_sub_keys=256 memory_top_k=32 memory_score=dot` |
| arms | **three seeds** (`data_seed` 20261009/10/11), memory present from step 0 |
| matched controls | **the keeper cycle's W=0 arms Ctl1/Ctl2/Ctl3** — same base, store, seeds, steps, recipe; no memory and no supervision. The memory is the only difference |
| placement arm | one extra seed-1 arm at `memory_layers=2` (the **first** read layer — the ablation's second-most critical) |
| base / data / recipe | `chat-29m-B-lr5e-4`, the adopted `mix-10` store (`888241261c378138…`), `steps=2000 batch=16 lr=2e-4 warmup=100 protocol=2 context=384 policy=full_prefix pointer=32`, supervision **off** |
| bar | ≥ 5 fewer wrong-value failures than the matched control on ≥ 2 of 3 seeds, the third not worse; **or** ≥ +4 memory rows over 120. Secondary: memory not below the noise, BPB within 0.02 |

## Results

### The failure mode — the bar, met
| pair | data order | addressed read | control | Δ memory | addressed WV | control WV | Δ WV |
|---|---|---:|---:|---:|---:|---:|---:|
| **A1 vs Ctl1** | 20261009 | 20/40 | 20/40 | 0 | **10** | 17 | **−7** |
| A2 vs Ctl2 | 20261010 | 18/40 | 20/40 | −2 | 16 | 16 | 0 |
| **A3 vs Ctl3** | 20261011 | 19/40 | 17/40 | **+2** | **8** | 14 | **−6** |
| **aggregate** | — | **57/120** | **57/120** | **0** | **34** | **47** | **−13** |
| placement: B1 (layer 2) vs Ctl1 | 20261009 | 16/40 | 20/40 | −4 | 15 | 17 | −2 |

**Bar MET**: two of three seeds clear the −5 wrong-value bar (−7, −6) with the third not worse, aggregate wrong-value
**−13 of 120**, and the headline is **flat** (57 = 57, per-seed 0 / −2 / +2 — inside the ±3-row noise).
**Placement matters**: the same memory at the **first** read layer is worse than its control (−4 memory, −2 wrong-value),
so the address belongs where the answers are made, not where the context is first read.

### The ledger — a real compute reduction, paid for in state and (for now) in wall time
| | read (control) | **addressed memory** | change |
|---|---:|---:|---:|
| **parameters** | 28,958,761 | **65,868,585** | **+36.9 M (+127 %)** |
| MACs/token at the replaced layer, context 384 | 1,769,472 | **173,056** | **−90 %** |
| MACs/token at that layer, context 4,096 | 6,029,312 | **173,056** | **−97 %** |
| MACs/token, whole model, context 384 | 30,203,136 | **28,606,720** | **−5.3 %** |
| **wall per training step** | **0.058 s** | **1.19 s** | **20×** (host bridge; GPU utilisation 3–27 %) |
| memory `check_pass` over 120 | 57 | 57 | 0 |
| wrong-value failures over 120 | 47 | **34** | **−13** |

The memory's per-token compute is a **table lookup** — a query map, sub-key scoring and a gather — so its cost does not grow
with context: **at 4,096 tokens the read it replaces costs 35× more**. Its +36.9 M parameters are a **codebook** (state read by
address, not per-token dense arithmetic), and the 20× wall time is an **implementation** fact: this configuration runs the exact
CPU forward on host copies, the stack's rule for configurations without a fused CUDA kernel. Both are the iteration's targets.

## What it establishes

1. **The project's own addressed-memory mechanism moves the failure mode on the answer path.** Replacing the read with an
   address at the layer that carries the answers cuts wrong-value failures by 28 % over 120 rows at no headline cost — the
   first result in this line where the *mechanism* (not supervision of the existing one) did the work.
2. **Placement follows the ablation.** The same memory at the first read layer is worse than no memory; at the last it is
   better. Where the answer is formed is where the address belongs.
3. **The compute argument is real per token and currently swamped by state and implementation.** −90 % at the replaced layer
   (−97 % at 4 K), +127 % parameters, 20× training wall. Only the middle column is mathematics; the other two are choices
   (a leaner codebook) and engineering (a kernel), which is exactly what a ledger is for.

## Limitations

- **Three seeds, 40 memory rows** (±3 rows) — the standing declared deviation from ≥ 120 rows; the per-seed signs differ
  (0 / −2 / +2 on the headline), so the aggregate is the honest summary.
- **BPB is pending, and here is exactly why**: the training pod (`4p200tih1jcy2h`) was reaped when its three-hour lease
  expired before its evaluations ran, and the two pods started afterwards landed on volumes that do **not** carry
  `chat-cuda-20261008/data/heldout.u16` — the stream every BPB number in this line has been measured on. The interim
  quality reading is the arms' own development NLL against their matched controls (0.375 vs 0.377; 0.420 vs 0.413;
  **0.373 vs 0.307** — the third seed worse), which is why this is a real open item rather than a formality, and why the
  first action of the next cycle is to re-measure it on a pod that mounts that volume. **Nothing in this record claims a
  BPB result.**
- **One memory configuration** (256 sub-keys, 128 key-dim, 4 heads, top-k 32): the +36.9 M parameters live in the codebook,
  and a leaner one (64 sub-keys, 64 key-dim, top-k 16 — the cycle-8 layer-4 arms) was *not* run here.
- **The supervision was off** in these arms, so this is memory-versus-search alone; the keeper's supervision cannot run at
  layer 8 when the memory replaces it there — that combination is the follow-up with the supervision moved to another read layer.

## Cost

| | |
|---|---|
| pod | `4p200tih1jcy2h`, 2 × RTX 5090: four fits at **2,373–2,375 s** each (39.5 min, host-bridge-bound) ≈ $2.40 |
| laptop CPU | four v5 reply sweeps (64 rows, cap 64) |
| external | none beyond the pod; inside the ≤ 4 pods / ≤ $8/h caps |

## Evidence

- Arm reports and models: `/workspace/uor-r4/deepseek/addrread-20261011/runs/{A1,A2,A3,B1}/`, pulled and bundled to
  cloud-store as `[BUNDLE]`; controls from the previous cycle's run.
- The paired comparison is the same judge-free row classifier used in the last three cycles
  (`docs/labs/read-binding-powered-2026-10-11/README.md` for the controls' identity).
- Pre-registration and fitness review: [#2029 comment 6107473230](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6107473230).

## CORRECTION (added 2026-10-11, same day): the intervention was an MLP replacement, not a read replacement

**This record mislabels what was changed.** `memory_layers=L` replaces **layer `L`'s MLP** with the product-key memory —
`StackModel::mlp` returns the memory for a listed layer (*"The product-key memory in place of `layer`'s MLP"*), the config
documents `layers` as *"Layers whose MLP is replaced by a memory"*, and the unit test is
`memory_layers_replace_the_mlp_train_and_round_trip`. The read at layer 8 was never touched.

So everywhere above where this record says *"replaces the read"*, *"the read's computation"*, *"the read it replaces"* or
*"the addressed read"*, read **"replaces that layer's MLP"** / **"the MLP it replaces"** / **"the MLP-memory arm"**.

**What this changes in the claims, and what it does not.** Every measurement stands exactly as measured — the frozen panel
(wrong-value 47 → 34 of 120), the parameters (+36.9 M), the training wall (20×), and the MAC comparison. But the *compute
comparison* is against the **MLP** at that layer, not the read: **173,056 MACs against the MLP's 1,264,896 at context 384
(7.3× fewer, −86 %)**, which is the number the [geometry audit](../integration/geometry-audit-2026-10-11/README.md) already
computed for this operator. The layer-2-versus-layer-8 arm contrasts **MLP placement**, not read placement.

**Consequence for the line, stated plainly:** the read-replacement question the [attribution](../labs/answer-path-2026-10-11/README.md)
and [head ablation](../labs/head-ablation-2026-10-11/README.md) point at **has not been run**. What this cycle tested is
*a product-key lookup in place of an MLP at one layer*, which is a compute trade (fewer multiplies, more state), not an
addressing mechanism for the read. The result is corrected here rather than withdrawn; the data is unaffected.
