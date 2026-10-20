# The exact content-route pointer: the address key raises the pointer's hit rate 0.377 → 0.54 at no likelihood cost, and the answers do not follow

References #2029 (M1 acceptance, D22 **order 3**; mechanism: pointer-copy head with an exact identity key).
Lab: DeepSeek, session `deepseek/ptr-addr`. Runs 2026-10-10 23:53 – 2026-10-11 00:10 UTC.

> **One line.** Order 3 reopens the identity pointer **with a real key** — the slot/entity address, not the
> six tokens around the value that #2161 used and that a question never resembles. The cheapest faithful
> implementation of that address is the trainer's own route with a **one-atom window** (`pointer_route=prime-ranked:1`:
> admit any source sharing a single prime atom with the query, ranked by `ln gcd` so rare atoms weigh more) and the
> surrounding-window term **off** (`pointer_identity=none`). Measured on three genuine seeds: the pointer's own
> **hit rate rises 0.377 → 0.54** and its reachability is a stable 0.668, the **float BPB is unchanged**
> (1.17387 against the anchor's 1.17425 — the memory arms cost +0.026, this costs nothing), and the **v5 memory
> half reads 19/21/23 of 40 with wrong-value failures 17/15/9** against the matched control's **20/40 and 17**.
> The key works as a key — it finds the value's source far more often — and **the answers do not follow**, which
> is the brief's own "what failure looks like" test carried one step further: not *"an oracle address still fails
> to raise the hit rate"* but *"a raised hit rate still does not produce the value"*. **The pre-registered bar is
> NOT MET on all three seeds, so the configuration is not kept** (D22 §2) — and the mechanism is very much open,
> with the bottleneck now located **downstream of the pointer**.

## What was run

| | |
|---|---|
| base | `chat-29m-B-lr5e-4` (`d8a3c971…`) — the pointer present from the start of the fine-tune, the route key active from step 0 |
| key | **`pointer_route=prime-ranked:1`**, `pointer_identity=none`: admission on a single shared prime atom (the entity/attribute word), ranked by the shared atom's weight; the six-token window that failed in #2161 is off |
| data | the adopted `mix-10` store (`tokens.u16` `888241261c378138…`), unchanged |
| recipe | `steps=2000 batch=16 lr=2e-4 warmup=100 protocol=2 context=384 policy=full_prefix`, `pointer=32`, `pointer_gate_supervision=0`, `device=cuda` — D10's own recipe |
| seeds | **three genuine seeds**: three **data orders** (`data_seed` 20261009 / 20261010 / 20261011), models `71409eac…`, `70fef0ba…`, `09e9dd3d…` |
| control | **D10**, the same recipe and store with an **unrouted, learned** pointer: memory 20/40, wrong-value 17, reply 37/232, BPB 1.17425 |

**A measurement about seeds that this cycle had to make first.** Two arms run with different `seed=` and the
same `data_seed=` produced **bit-identical models** (`71409eacea886cee` twice, 605 s and 572 s wall): with
`init=` from a trained checkpoint and no fresh operator to initialise, `seed=` has nothing to vary. For a
fine-tune from a checkpoint the **data order** is the seed that exists, so the three arms differ there — and
that is also why the memory arms of cycles 8–10 (whose added memory *did* draw from `seed=`) could use `seed=`
meaningfully while the no-memory controls could not.

## Results

### The pointer's own metric — the brief's "own metric" line
| arm | data order | dev pointer **hit rate** | reachable rate | dev NLL |
|---|---|---:|---:|---:|
| D10 (control, learned keys) | 20261009 | 0.377 | 0.800 | 0.3773 |
| **A10** | 20261009 | **0.5452** | 0.6677 | 0.4002 |
| **S1** | 20261010 | **0.5419** | 0.6677 | 0.4476 |
| **S2** | 20261011 | **0.5419** | 0.6677 | 0.3642 |

The hit rate is the metric #2161's key destroyed (0.003 at weight 1.0) and the one this key **raises by 44 %**
over the learned pointer, stably across all three data orders. Reachability is identical on every arm (0.6677),
which is exactly what the frozen-route evidence predicts — the admission rule is deterministic, so reach is a
property of the key, not of learning — but at **0.67 rather than the 0.182** the 4-token n-gram route produced:
a one-atom address admits roughly four times as often.

### The frozen v5 memory panel (40 rows, frozen checks, cap 64)
| arm | data order | **memory `check_pass`** | unknowable | **wrong-value** | derangement | float BPB |
|---|---|---:|---:|---:|---:|---:|
| **control D10** (learned keys) | 20261009 | **20/40** | 0/24 | **17** | 0 | **1.17425** |
| A10 | 20261009 | 19/40 | 0/24 | 17 | 0 | **1.17387** |
| S1 | 20261010 | 21/40 | 3/24 | 15 | 0 | **1.16947** |
| S2 | 20261011 | **23/40** | 8/24 | **9** | 2 | **1.17109** |

**Bar (pre-registered): wrong-value ≤ 12/40 on all three seeds and memory ≥ 25/40 on all three, BPB within
0.01 of 1.17425. NOT MET** — the wrong-value counts are 17, 15 and 9 (only the third clears 12) and the memory
readings are 19, 21 and 23. The BPB clause is met with room to spare, and then some: **all three arms sit at or BELOW the anchor** — 1.17387,
1.16947 and 1.17109 against 1.17425 — where the memory arms of cycles 8–10 cost +0.026.

## Decision

**REJECT for this configuration, and the mechanism is now located precisely (D22 §1).** The configuration is
the one-atom ranked route with the surrounding-window key off; it is not kept, because it does not clear its
own pre-registered bar on all seeds.

**What it establishes.** This is the first test on this line where the **pointer's own metric moved
decisively**: an entity-address key *does* find the value's source far more often (0.377 → 0.54) at **no
likelihood cost**, in all three data orders. And the answers still do not follow: the best seed reaches the
**line's best wrong-value count (9/40)** while another reads exactly the control's 17, and the headline stays
inside the panel's noise of 20/40. The brief's failure condition was *"an oracle-supplied correct slot address
still fails to raise hit rate or memory"*; the measured state is one step further along — **the hit rate rises
and the memory does not**, so **the bottleneck is not the pointer's source selection but what happens to a copy
once the source is found**: the gate, the register, or the decoder that has to place the copied value in the
answer rather than continue the frame.

**Next, and it follows from that location:**
1. **The oracle/forced-copy instrument the brief asks for** (a serving-time hook that supplies the true span to
   the pointer op) — it bounds the gain with no training and would say directly whether a *perfect* address
   yields the value. It is the piece this cycle declared missing rather than faked.
2. **The copy-side levers, now with a reason**: `pointer_gate_supervision` and `pointer_gate_floor` act on
   exactly the "found it but did not copy it" step, and the read-layer memory of cycle 10 (wrong-value 10,
   abstention 4/24) moved the same metric from the other side.
3. A **combined** arm — the address key plus the copy-side lever — is the natural KEEP candidate: the key is
   free in likelihood, so there is room to spend.

## Limitations

- **Three seeds, three data orders**: the panel half is 40 rows and its noise is ±3 rows, which the readings
  (19/21/23) sit inside; the wrong-value spread (17/15/9) is why the pre-registered bar demanded all three.
- **The frozen v5 panel holds 64 rows** where the brief asks ≥ 120 memory rows for a learned address key — the
  same declared deviation as the previous cycles, an owner question rather than a lab decision.
- **The oracle ablation (brief step 1) was not run**: it needs a forced-attention hook the pointer op does not
  have; it is named as the next instrument instead of being approximated.
- **`prime-ranked:1` is one reading of "entity address"**: the trainer's route over prime atoms, not a parsed
  entity×attribute address written when the fact is stated (the brief's open design question 1). A parsed
  address is a different configuration, and this result does not close it.
- Reachability being constant at 0.6677 means a fifth of the value spans are **not addressable at all** under
  this key; that ceiling is a property of the admission rule and would need the parsed address to move.

## Cost

| | |
|---|---|
| pod | `aipnvd9edej0ao`, 2 × RTX 5090, 23:35Z → released after the BPB evaluations; three fits at **605 s / 572 s / ~590 s** wall (two in parallel, then one) |
| GPU work | three 2,000-step fits with no memory operator |
| laptop CPU | v5 replies and grading for three arms, the reply panel, analysis |
| external | none beyond the pod; inside the ≤ 4 pods / ≤ $8/h caps |

## Evidence

- Arm reports and models: pod volume `/workspace/uor-r4/deepseek/ptr-addr-20261010/runs/{A10,S1,S2}/`, pulled to
  the laptop and bundled; BPB reports in the same volume's `bpb-*/`.
- Acceptance reports: `score/v5g-{A10,S1,S2}/report.json`.
- Bundle on cloud-store: `icloud:UOR-R4/results/deepseek/ptr-addr-2026-10-10.tar` (arm reports and models, v5
  acceptance reports, BPB reports, job logs; the object's md5 is recorded in the delivery PR).
- Pre-registration and fitness review:
  [#2029 comment 6103332464](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6103332464).
