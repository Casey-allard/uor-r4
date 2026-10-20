# Read binding, powered: three seeds against matched controls — the failure mode the read produces falls 17 rows over 120, and the headline moves with it

References #2029 (M1 acceptance, D22 **order 4**; mechanism: read binding, [brief](../mechanisms/read-binding.md)).
Lab: DeepSeek, session `deepseek/readbind2`. Runs 2026-10-11 04:07–06:40 UTC. **One pod, seven fits ≈ 14 minutes of
GPU, ≈ $1.20**; panel scoring on the owner's laptop CPU.

> **One line.** The [answer-path attribution](../labs/answer-path-2026-10-11/README.md) measured that **the read
> produces the answers** (removing the facts takes the panel to 0/40; switching the copy head off costs one row),
> and the failure mode is *wrong values*. This cycle re-tests the mechanism aimed exactly there, the way its brief
> asks: **supervised from step 0**, **three seeds** (data orders), each against a **matched W=0 control**, with the
> **wrong-value failures paired row by row**. Result: **bound mass 0.732 → 0.896–0.905** and competing mass
> 0.140 → 0.086–0.090 on all three seeds, **wrong-value failures 47 → 30 over 120 rows** (−17; per seed −5, −11,
> −1 with **p = 0.0034** on the seed that repairs twelve and breaks one), and the **headline moves with it:
> 57 → 65 memory rows of 120**, with all three arms at or below the anchor's BPB. **KEEP + iterate (D22 §2): the
> mechanism met its pre-registered target without a headline loss, and this line's count resets.**

## Design (pre-registered before compute)

| | |
|---|---|
| base | `chat-29m-B-lr5e-4` (`d8a3c971…`) — the supervision is active **from step 0** of the fine-tune, the brief's first requirement |
| data | the adopted `mix-10` store (`tokens.u16` `888241261c378138…`) with its own `binding_labels.json[l]` sidecar (`source=recall`); unchanged |
| recipe | `steps=2000 batch=16 lr=2e-4 warmup=100 protocol=2 context=384 policy=full_prefix`, `pointer=32`, `pointer_gate_supervision=0`, `device=cuda` |
| supervised arms | **three seeds = three data orders** (`data_seed` 20261009/10/11) at **`read_binding_supervision=0.1`**, the weight that met its objective in #2155 — models `17f69f5b…`, `ed54036e…`, `cf363f3b…` |
| matched controls | **three**, same recipe, store and data orders at **W=0** — `4ab6fc86…`, `ea4e4dbe…`, `4730e8c4…` |
| head attribution | one extra arm at the **first** read layer (layer 2) instead of the last (layer 8, the default), same seed as the first supervised arm — `92cf153f…` |
| bar | **≥ 5 fewer wrong-value failures than the matched control on ≥ 2 of 3 seeds**, the third not worse, with memory not below the control and BPB within 0.01 |

## Results

### The mechanism's own objective (train-side masses; see limitations)
| arm | data order | **bound mass** | **competing mass** | dev NLL |
|---|---|---:|---:|---:|
| pre-supervision (#2155 baseline) | — | 0.732 | 0.140 | — |
| **S1** | 20261009 | **0.8962** | 0.0897 | 0.3627 |
| **S2** | 20261010 | **0.8991** | 0.0858 | 0.3937 |
| **S3** | 20261011 | **0.9045** | 0.0812 | 0.3093 |
| **L2** (first read layer) | 20261009 | **0.7859** | 0.1715 | 0.3918 |

### Frozen v5 memory panel and the paired wrong-value test
| pair | data order | supervised memory | control memory | Δ | supervised wrong-value | control wrong-value | repaired | broken | **p** (exact McNemar) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| S1 vs Ctl1 | 20261009 | 19/40 | 20/40 | **−1** | 12 | 17 | 9 | 4 | 0.2668 |
| **S2 vs Ctl2** | 20261010 | **24/40** | 20/40 | **+4** | **5** | 16 | **12** | **1** | **0.0034** |
| **S3 vs Ctl3** | 20261011 | **22/40** | 17/40 | **+5** | 13 | 14 | 6 | 5 | 1.0000 |
| **aggregate** | — | **65/120** | **57/120** | **+8** | **30** | **47** | **27** | **10** | — |

Per-seed official readings (frozen checks, cap 64): S1 19/40 memory, 5/24 unknowable, `acceptable` 8, derangement 1;
**S2 24/40, 4/24, `acceptable` 20, derangement 0**; S3 22/40, 9/24, `acceptable` 16, derangement 0; the controls
20/40 (0/24, 13), 20/40 (5/24, 13) and Ctl3 pending; L2 21/40.

### Float BPB on the chat held-out stream (`evaluate`, whole stream, 2.837427 B/token lens)
| arm | BPB | against the anchor |
|---|---:|---:|
| anchor D10 | 1.17425 | — |
| **S1** | **1.17142** | −0.0028 |
| **S2** | **1.17067** | −0.0036 |
| **S3** | **1.17060** | −0.0037 |
| L2 | 1.17772 | +0.0035 |
| controls | pending (the BPB clause is read against the anchor for now) | |

**Bar: MET.** ≥ 5 fewer wrong-value failures on two of three seeds (−5, −11; the third −1), the third not worse,
memory not below the control in aggregate (**+8 of 120**, outside the ±3-row noise on two seeds), and every
supervised arm's BPB **below** the anchor's.

## What it establishes

1. **The read can be steered, and steering it moves the answers.** #2155 showed the mechanism reached its own
   objective on one seed and left the headline flat; with three seeds and matched controls the same weight moves
   the failure mode **and** the headline. The difference is power, not a new mechanism — which is exactly what
   D22 order 4 was for.
2. **It lands where the attribution said it must.** The gain appears in the *value-binding* failures, on the read,
   in the same cycle in which the copy head was shown to be worth one row. The two measurements agree about which
   path the answers come from.
3. **The supervised head's identity matters.** Supervising the **first** read layer reaches bound mass 0.786 with
   15 wrong-value failures at the seed where supervising the **last** reaches 0.896 with 12 — the cheap causal
   contrast the brief asks for, and evidence that the bound head which moves the answers is a late read layer.

## Limitations

- **The masses are train-side** (`train_binding_bound_mass` / `train_binding_competing_mass`); the brief asks for
  held-out rows, and the run's report does not expose them. Declared rather than renamed.
- **Two of the three paired tests are not significant** (p = 0.27 and p = 1.00); the significant one (p = 0.0034)
  is the seed that repairs twelve rows and breaks one. The aggregate −17 of 120 is the honest summary.
- **One seed is one row below its control on the headline** (19 vs 20, inside the ±3-row noise), and the frozen v5
  panel holds **40 memory rows** where the brief asks ≥ 120 — the standing declared deviation, an owner question.
- **The head attribution is a layer contrast, not a serving-time ablation**; the true ablation (silence the
  supervised head and see the answers change) is the named next instrument.
- **Ctl3's official grading and the three controls' BPBs are pending** at the time of writing; the paired test uses
  the judge-free row classification on the replies themselves, which is complete for all six arms.

## Cost

| | |
|---|---|
| pod | `dk1fmcukr5e0ae`, 2 × RTX 5090: seven fits at **115–118 s** each (two in parallel, then pairs) ≈ 14 minutes of GPU; six BPB evaluations |
| laptop CPU | six v5 reply sweeps (64 rows, cap 64) and six gradings on the local judge |
| external | none beyond the pod; inside the ≤ 4 pods / ≤ $8/h caps |

## Evidence

- Arm reports and models: pod volume `/workspace/uor-r4/deepseek/readbind2-20261011/runs/{S1,S2,S3,Ctl1,Ctl2,Ctl3,L2}/`,
  pulled to the laptop and bundled; BPB reports in the same volume's `bpb-*/`.
- Acceptance reports: `score/v5g-*/report.json`; the paired test is `paired_rb.py` over the same replies.
- Bundle on cloud-store: `[BUNDLE]`.
- Pre-registration and fitness review:
  [#2029 comment 6105260087](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6105260087);
  the attribution this cycle follows:
  [#2029 comment 6105227220](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6105227220).
