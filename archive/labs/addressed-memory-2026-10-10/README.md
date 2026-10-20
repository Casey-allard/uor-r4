# Product-key memory trained into the M1 stack at the anchor's own steps: 19/40 on both seeds against the brief's 27/40 — the device fix works, the configuration does not

References #2029 (M1 acceptance, D22 order 2; mechanism: exact addressed memory). Lab: DeepSeek, session
`deepseek/mem-in`. Runs 2026-10-10 20:07–20:23 UTC (GPU) and 17:58–20:06 UTC (CPU).

> **One line.** The tooling blocker is fixed: the product-key memory's **CUDA forward is its exact CPU forward
> on host copies** ([#2175](https://github.com/UOR-Foundation/uor-r4/pull/2175)), and the same two-seed fit that
> needed **7.4 s/step on CPU** now runs at **0.19 s/step** — 2,000 steps in **387 s** instead of 2.5 hours per
> arm. With the mechanism present from **step 0** and the anchor's own recipe, the two seeds read **19/40** and
> **19/40** on the frozen v5 memory half against the brief's bar of **27/40**: **REJECT for this configuration**,
> which closes the configuration and not the mechanism (D22 §1). One development reading points where the next
> configuration should look: the same memory at **half the steps** reads **22/40** with the line's best
> **wrong-value count (11 of 40)**, so the next cycle's first piece is the **step-matched control** that says
> whether that is the memory or the shorter fit.

## The tooling fix this cycle landed (preparation PR)
| | |
|---|---|
| what | `ProductKeyMemory::cuda_fwd` = `cuda_ops::via_host3`, i.e. the op's exact CPU forward on `to_cpu_storage()` copies of the three inputs, uploaded back — the bridge the stack's other kernel-less configurations already use. The backward was already device-generic. **The CPU path is untouched**, so every existing artifact keeps its exact numbers. |
| where | [#2175](https://github.com/UOR-Foundation/uor-r4/pull/2175), branch `deepseek/memory-cuda-20261010` |
| checks | CPU build clean; `cargo test -p uor-r4-training --lib stack_memory` **7 passed** including the finite-difference gradient check; the **CUDA-enabled pod build compiles and passes parity** (the check a CPU-only local build cannot perform) |
| measured effect | **7.4 s/step → 0.19 s/step** (2,000 steps: ~2.5 h on CPU → **387 s on the GPU**, per arm, both arms in parallel) |

## The pre-registered run (D22 cycle 2)
| | |
|---|---|
| base | `chat-29m-B-lr5e-4` (`d8a3c971…`) — the memory is inside the fine-tune, from step 0, installed before the optimizer |
| memory | `memory_layers=4` of the 10-layer `rrarrarrar` stack (width 576), `memory_sub_keys=64`, `memory_top_k=16`, `memory_heads=4`, `memory_key_dim=64`, `memory_score=dot`, no codebook |
| data | the adopted base's own store, cycle 1's `mix-10` (`tokens.u16` `888241261c378138…`), unchanged |
| recipe | `steps=2000 batch=16 lr=2e-4 warmup=100 protocol=2 context=384 policy=full_prefix`, `pointer=32`, `pointer_gate_supervision=0`, `device=cuda` — **D10's own recipe**, so the contrast is the memory operator |
| seeds | `20261010` (**G1**, model `b1aa24e1…`, dev 0.3946, 377 s) and `20261011` (**G2**, model `25cf22c7…`, dev 0.3865, 379 s) |
| fitness | [`TEST FITNESS: FIT`](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6101635064) with two declared deviations (the frozen panel has 64 rows, not the brief's 80; the tagger/read split belongs to the exact-key arm this run does not test) |

## Results

### Frozen v5 memory panel (40 rows, frozen checks, cap 64)
| arm | steps | device | memory `check_pass` | unknowable | **wrong-value** | derangement |
|---|---:|---|---:|---:|---:|---:|
| D10 (adopted base, **no memory**) | 2,000 | GPU | **20/40** | 0/24 | 17 | 0 |
| **G1** (memory, seed 20261010) | 2,000 | GPU | **19/40** | 0/24 | 17 | 1 |
| **G2** (memory, seed 20261011) | 2,000 | GPU | **19/40** | 0/24 | 15 | 2 |
| *cpuM1 (memory, seed 20261010, **half the steps**)* | 1,000 | CPU | *22/40* | *2/24* | ***11*** | *2* |

**Bar (pre-registered): memory ≥ 27/40 on both seeds and BPB within 0.01 of the anchor. NOT MET on both
halves** — the seeds sit at 19/40 (one row under the adopted base, eight under the brief's bar) and G1's BPB is
**+0.0276** against the anchor, outside the 0.01 the bar allows. **No headline movement.**

### The development reading that names the next step
The same memory configuration fitted at **1,000 steps on the CPU** reads **22/40** — *above* the adopted base's
20/40 — with **11 wrong-value failures**, the best on this line (read binding's best was 12, the token-identity
pointer's 12). It is **not a claim**: it differs from the GPU arms in step count *and* device (GPU and CPU
reductions order differently, so the arms are not expected to be bit-identical), and its own cycle's
pre-registered bar was 24/40. It is exactly the kind of reading that needs a step-matched control before it can
mean anything.

### Float likelihood on the chat held-out stream
| arm | reading |
|---|---|
| D10 (anchor) | **1.17425 BPB** (nll 2.30985 over 6,169,728 targets, the whole held-out stream at context 384) |
| **G1** | **1.20184 BPB** — **+0.0276** against the anchor, i.e. outside the bar's 0.01 |
| G2 | pending |
| cpuM1 | pending |

### Open reply panel (232 rows, `fluent_and_relevant`), primary arm — **pending**
G1's replies are generating and its grading is in the shared judge queue as this record is written; posted
on #2029 when it lands. It cannot change the verdict, which the memory half decides (19/40 against 27/40).

## Decision

**REJECT for this configuration; the mechanism stays open (D22 §1).** The pre-registered bar for the
product-key-memory configuration — **≥ 27/40 on both seeds with BPB within 0.01** — is not met: 19/40, 19/40.
That closes *this configuration* (layer 4, 64 sub-keys, 16 top-k, replacing that layer's MLP, fitted from the
chat base for 2,000 steps) and nothing else.

**What it does establish, and what it does not.** The device fix works and is measured (0.19 s/step, 387 s per
2,000-step arm, both arms in parallel, the op's values unchanged — the CUDA forward *is* the CPU forward). The
configuration is *not* a winner at the anchor's step count on this panel. Neither statement is evidence about
the exact-key (prime/semiprime addressed store) arm of the same brief, which this run did not test.

**Next cycle, named now (cheapest first):**
1. **The step-matched control**: the identical recipe with **no memory** at 1,000 steps, two seeds — it decides
   whether cpuM1's 22/40 and wrong-value 11 come from the memory or from the shorter fit. Cheap now: ~6.5 min
   per pair on the GPU.
2. **Configuration variation at the anchor's steps**: the memory at a **read** layer (`a` in `rrarrarrar`) and
   with a larger sub-key set, one seed each, against the same frozen panel and bar.
3. **The exact-key arm** of the brief (prime/semiprime addressed store) — a different mechanism path from the
   native learner, and the one whose "tagger accuracy reported separately from read accuracy" item this run
   declared not applicable.

## Limitations

- **Two seeds, one configuration**; the panel's memory half is 40 rows and its noise is ±3 rows, which is why
  the bar was set at 27/40 rather than at a one-row difference.
- **The brief's ≥ 80-row panel cannot be met on v5** (64 rows: 40 memory + 24 unknowable) — declared as a
  deviation before compute and an owner question, not a lab decision.
- **CPU and GPU arms are not bit-identical** (different reduction orders), so the 1,000-step CPU reading is a
  development observation, not a controlled comparison; the control in (1) is what makes it one.
- **The BPB instrument differs from cycle 5's** (the D11 export path refuses a memory model; this uses
  `evaluate` with the same 2.837427 B/token lens), so the bar's "within 0.01" is read within this instrument.

## Cost

| | |
|---|---|
| pod | `l8uv41elowab34`, 2 × RTX 5090, leased 16:22Z, renewed to 22:35Z; the GPU fits used **12.8 minutes** of it (two arms in parallel, 387 s + 389 s wall) |
| GPU work | two 2,000-step fits; the CPU fits that preceded them (~2.2 h of the pod's CPU for one completed arm, superseded and stopped as declared) |
| laptop CPU | v5 replies and grading for three arms, the reply panel, analysis |
| external | none beyond the pod; inside the ≤ 4 pods / ≤ $8/h caps |

## Evidence

- Arm reports, models and configs: pod volume `/workspace/uor-r4/deepseek/mem-in-20261010/runs/{G1,G2}-gpu/` and
  `runs/M1-e/` (CPU), pulled to the laptop; job logs in `/workspace/uor-r4/jobs/deepseek/`.
- Acceptance reports: `score/v5g-{G1,G2,cpuM1}/report.json`, `score/rpg-G1/report.json`.
- Bundle on cloud-store: `icloud:UOR-R4/results/deepseek/addressed-memory-2026-10-10.tar` (the object is
  re-written with the GPU arms, the CPU arm, the BPB reports and the job logs; its md5 is recorded in the
  delivery PR).
- Pre-registration and fitness review:
  [#2029 comment 6101635064](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6101635064);
  the previous cycle's record and its carry-forward:
  [#2029 comment 6101562834](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6101562834).

## Cycle 9: the step-matched control, and what the half-steps reading really was

The half-steps reading above (memory at 1,000 steps on CPU: 22/40, wrong-value 11) had two candidate
explanations — the memory, or the shorter fit. The control is the identical recipe with **no memory** at
**1,000 steps**, `device=cuda`, and it settles it:

| arm | memory | steps | device | v5 memory `check_pass` | unknowable | **wrong-value** | derangement |
|---|---|---|---:|---:|---:|---:|---:|
| **C1 / C2 (control, no memory)** | — | 1,000 | GPU | **22/40** | 3/24 | **12** | 0 |
| cpuM1 (memory) | yes | 1,000 | CPU | 22/40 | 2/24 | 11 | 2 |
| G1 / G2 (memory) | yes | 2,000 | GPU | 19/40 | 0/24 | 17 / 15 | 1 / 2 |
| D10 (adopted base, no memory) | — | 2,000 | GPU | 20/40 | 0/24 | 17 | 0 |

**The 22/40 was the shorter fit.** The matched control reads the same two rows and one wrong-value row away,
so the memory configuration contributes **nothing measurable on this panel at either step count** — and at the
anchor's steps it reads one row *below* the no-memory anchor while costing **+0.026 BPB** on the chat held-out
stream. The mechanism is untouched by that (D22 §1: a configuration, not a mechanism); the *configuration* is
what is rejected.

**A measured note on the control's two seeds.** C1 and C2 are **bit-identical** (`model.sha256`
`90c382d6…`, dev 0.4274, 63 s wall each): without a memory to draw, `seed=` has nothing to initialise, since
every weight comes from the checkpoint and the data order is fixed by `data_seed`. The memory arms' seed
variation therefore comes entirely from the added memory — the property [#2170](https://github.com/UOR-Foundation/uor-r4/pull/2170)
restored — and the control is one run reported twice.

**Cost of the memory's host bridge, measured:** 1,000 steps took **63 s** without the memory and **~190 s**
with it (the CUDA arms), i.e. **≈ 0.13 s/step** of transfer and host compute for the op — real, small next to
the trunk, and the reason the *kernel* remains the honest follow-up rather than a claim that the bridge is
free.

**What the next cycle does, decided by this reading:** the memory's *placement and size* at the anchor's steps
(a **read** layer `a` of `rrarrarrar`, and a larger sub-key set), one seed each against the same frozen panel
and bar; then the brief's **exact-key arm** (prime/semiprime addressed store) with the tagger-versus-read split
the brief asks for.

## Cycle 10: placement and size probes at the anchor's steps

The measured configuration sat at **layer 4** with **64 sub-keys**; the two things it never varied are *where* the
memory sits and *how big* it is. Both probes use the anchor's own recipe and one seed (`20261010`), and are
**probes**, not decisive runs — the pre-registration promotes one to a two-seed run only if it beats the no-memory
anchor's **20/40 by ≥ 4 rows** *and* stays within **0.01 BPB** of 1.17425.

| arm | placement | size | steps | params | v5 memory `check_pass` | unknowable | wrong-value | derangement | wall |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| G1 (measured configuration) | layer 4 (an `r` layer) | 64 sub-keys, 16 top-k | 2,000 | 32.6 M | 19/40 | 0/24 | 17 | 1 | 387 s |
| **P256** | layer 4 | **256 sub-keys, 32 top-k, key width 128** | 2,000 | 65.9 M | **22/40** | 1/24 | 14 | 3 | 2,007 s |
| **P8** | **layer 8** — the last **read** layer (`a`) | 256 sub-keys, 32 top-k, key width 128 | 2,000 | 65.9 M | **20/40** | **4/24** | **10** | 1 | 2,000 s |
| C1 (control, **no memory**) | — | — | 1,000 | 30.2 M | 22/40 | 3/24 | 12 | 0 | 63 s |
| anchor D10 (**no memory**) | — | — | 2,000 | 30.2 M | **20/40** | 0/24 | 17 | 0 | — |

**Both probes ran with the trainer's default memory sizes** (256 sub-keys, 32 top-k, key width 128, 65.9 M
parameters against the base's 30.2 M): the intended 64-sub-key P8 arm took the defaults too, so the pair varies
the **placement** (layer 8, a read layer, against layer 4, a recurrence layer) and both vary the **size**
against the previously measured 64-sub-key configuration. That is stated rather than glossed: the two
variations are not orthogonal in this pair.

**No promotion, and the first configuration that moves the failure mix without losing the head-line.** The
pre-registered promotion bar — **≥ 24/40** *and* BPB within 0.01 of 1.17425 — is met by neither probe: P256
reads **22/40**, which the **no-memory control** at the shorter fit also read, so it is not evidence of the
memory; P8 reads **20/40**, exactly the anchor. But P8 is the first memory configuration that does not *lose* on
the headline while moving what this line actually measures: **wrong-value failures 17 → 10** (the best of any
arm, against the control's 12 and the read-binding arm's 12) and **unknowable rows 0 → 4/24** (also the best),
at the cost of a new **distractor-key** pattern (3 rows) the layer-4 placements never showed. The read-layer
placement is therefore the configuration worth a two-seed run; the memory's *size* at layer 4 (64 → 256
sub-keys) buys 19 → 22/40 — exactly what the shorter fit buys — for **16× the parameters**.

**The bridge's cost scales with the memory's size, measured.** A 2,000-step fit of the 64-sub-key configuration
took **387 s** with both arms in parallel; the same fits with the 256-sub-key memory took **2,000 s and 2,007 s**
— **5.2× slower** — so the host bridge, not the GPU trunk, is now the cost. That is the concrete argument for
the kernel follow-up, and it also means a bigger memory is not free even when it is not better.

**What the next cycle does:** iterate on the **read-layer placement** with the two seeds the brief asks for, and then the ordered pieces D22
lists: the **identity pointer keyed on a slot/entity address** (order 3, wrong-value primary, ≥ 2 seeds) and
**powered read binding** (order 4, ≥ 3 seeds or ≥ 200 fresh rows). The probe evidence supports order 3's framing
directly: this line's failures are *value-selection* failures, and a memory on a read layer is the first thing
that halves them.
