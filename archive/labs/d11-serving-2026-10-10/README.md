# Criterion 2, measured for the first time on this line's artifacts: the D11 engine is bit-exact and the charted model sits at 1.20–1.24 BPB against the ≤ 0.90 target

References #2029 (M1 acceptance criterion 2). Lab: DeepSeek, session `deepseek/d11-serve`, **cycle 5** of the
standing goal adopted from `docs/labs/session-goal.md` @ `8421e605f…`. Run 2026-10-10 14:29–15:10 UTC.
**One 2 × RTX 5090 pod, ≈ 45 minutes, ≈ $1.80.** Pre-registered on #2029 before any compute.

> **One line.** Criterion 2 — *"the same model exports and serves under D11 at ≤ 0.90 BPB on the chat held-out
> stream"* — had **never been read on this line's artifacts**, every one of which now carries a copy pointer.
> It is read here for the 29M base and the two best fine-tunes: **all three export cleanly, the integer engine
> is bit-exact with the float path (`d11 − d10 nll = 0.0`, `max |Δlogit| = 0` on all 64 windows), and the served
> readings are `**1.20138**`, `**1.24402**` and `**1.24233**` BPB against the target of ≤ 0.90.** The criterion is
> **NOT MET**, and because the engine is exact the gap is the **model**, not the export — the same attribution
> the earlier [chat-served gap round](../labs/chat-served-gap-attribution-2026-10-09/README.md) reached for the
> 19.9M family, now measured on the 29M line.

## The pre-registration, and what was actually run

[Pre-registered on #2029](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6098537341) before
any pod existed: artifacts, export recipe, evaluation, byte basis, bar, stop rule and cost.

| | |
|---|---|
| artifacts | **A0** `chat-29m-B-lr5e-4` (the base: reply 43/232, memory 10/40); **A1** cycle 1's D10 (memory 20/40, reply 37/232); **A2** cycle 2's B1 (memory 19/40, reply 39/232) — all float f32 geometric stacks with a 32-wide unselected pointer |
| export | `geometric-stack export model=ROOT/model out=NEW_REPORT_ROOT calibration=<chat-v0-p2 train stream> calibration_windows=64 damp=0.01` — the GPTQ damped quantiser, calibration 24,576 positions, `mlp_padded_from 732 → to 736` |
| the served reading | `geometric-stack d11-evaluate artifact=ROOT/model.lut valid=<chat held-out stream> model=ROOT/model windows=64 threads=8`, which runs **both** engines on the same 64 windows and compares logits and mixtures |
| held-out stream | `/workspace/uor-r4/deepseek/chat-cuda-20261008/data/heldout.u16` — **6,194,589 tokens, 17,576,697 bytes, 2.837427 bytes/token, 0 out-of-vocab**, computed with `make_lens.py`'s byte accounting, which reproduces the recorded basis exactly |
| bits per byte | `nll_nats × log2(e) ÷ 2.837427` on the same windows, i.e. the repository's own exact byte basis rather than an assumed one |
| stop rule | evaluation only, nothing trained; if the export refused a pointer model that would have been the finding |
| pod | `0rkk8hgawtig8k`, `--ref e094c5675…` (main at run time), CUDA parity PASS; lease released and pod deleted after the run |

## Results

### The engine is exact on every artifact
| artifact | export sha256 (`model.lut`) | `d11 − d10 nll` | `max |Δlogit|` | D11 tokens/s (8 threads) | D10 tokens/s |
|---|---|---:|---:|---:|---:|
| A0 base | `b55bdf6bfb49a54bc3a5200aa8c4a2bbd2aebd53e091ef0de5756f3327bc2e00` (16,871,620 B) | **0.0** | **0** | 68.0 | 127.5 |
| A1 D10 | `bbbd1ff99fd976ab…` (16,871,620 B) | **0.0** | **0** | 67.9 | 130.5 |
| A2 B1 | `3f39a91aa2fb3ac7…` (16,871,620 B) | **0.0** | **0** | 80.0 | 136.9 |

`first_difference` is `null` on every window: the multiplier-free integer engine reproduces the float
distribution exactly, so **no part of the gap below is a serving artefact**.

### The criterion reading
| artifact | served nll (nats/token) | **served BPB** | float nll | criterion 2 (≤ 0.90) |
|---|---:|---:|---:|---|
| A0 base | 2.362828 | ****1.20138**** | 2.362828 | **NOT MET** |
| A1 D10 | 2.446684 | ****1.24402**** | 2.446684 | **NOT MET** |
| A2 B1 | 2.443367 | ****1.24233**** | 2.443367 | **NOT MET** |
| reference: the 19.9M chat-v0-p2 stack (recorded, not this run) | — | **0.933** | 0.877550 | NOT MET by 0.033 |

**Two facts from that table, and one of them is actionable.** (1) The fine-tunes are **worse** on this stream
than the base they came from (1.2441 against 1.2014 BPB): the recall mixture that buys the memory half costs
in-distribution chat likelihood, which is the same trade the reply panel shows. (2) The recorded **0.933 BPB**
belongs to a **19.9M** artifact fine-tuned on **chat-v0-p2 alone** — in-distribution with this held-out split —
while every artifact here was fine-tuned on a **mixed** stream (M-world ×3 + chat-v0-p2, then the recall
mixture). Scale is not what separates them: the *distribution* is. That names the next piece precisely: an
**in-distribution chat-only fine-tune of the 29M base**, the recipe behind the 0.933 reading, scored on both
this stream and the criterion-1 panels.

## Decision

**Criterion 2: NOT MET, with the gap attributed to the model and the next lever named.** No threshold, panel or
stream is changed, and nothing about the criterion is reinterpreted: `≤ 0.90 BPB` is read as written, on the
stream the acceptance names.

- **The serving path is not the problem**: bit-exact engine, zero logit difference, and a 16.9 MB artifact that
  loads in 19 ms.
- **The model is**: 1.20–1.24 BPB served, i.e. **33 % above** the target, and *worse* than the 19.9M
  in-distribution artifact's recorded 0.933.
- **Criterion 1 is unaffected** and remains blocked on the owner as the [blocked card](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6098512039)
  states; this cycle changes nothing there.

## Limitations

- **64 windows**, the recorded ladder protocol, so the reading is comparable to the 0.933 cell but not to a
  512-window count; the same artifact would read slightly differently at another window count (the pinned
  protocol record measured 0.046 BPB between 64- and 512-window counts on the 19.9M family).
- **The 0.933 reference is a recorded number from another artifact** (19.9M, chat-only fine-tune), not a run in
  this cycle; it is cited as the comparison the acceptance implies, and its artifact is not on this volume.
- The byte basis is exact for the **whole stream** (17,576,697 bytes over 6,194,589 tokens); the evaluation
  scores a fixed window subset of it, as the ladder's own readings did.
- Nothing here is a claim about served *quality* in conversation: BPB on a chat held-out stream is a
  likelihood reading, and the criterion-1 panels remain the acceptance instrument for behaviour.

## Cost

| | |
|---|---|
| pod | `0rkk8hgawtig8k`, 2 × RTX 5090, 14:29Z → 15:12Z (~43 min incl. bootstrap, ≈ $1.70); lease released, pod deleted |
| GPU work | three exports (37 / 32 / 29 s) and three evaluations (9 / 9 / 9 min, D10 and D11 engines interleaved) |
| laptop CPU | the byte accounting seconds; no local build (the pod's cached binary was used) |
| external | none; inside the ≤ 4 pods / ≤ $8/h caps |

## Evidence

- Export and evaluation reports, the three LUTs and the lens table: pod volume
  `/workspace/uor-r4/deepseek/d11-20261010/` (`export-{base,d10,b1}/`, `eval-{base,d10,b1}/`, `lens.u16`,
  `run-crit2.sh`), pulled to the laptop and bundled.
- Byte basis: `make_lens.py` on the held-out stream — 6,194,589 tokens, 17,576,697 bytes, 2.837427 bytes/token,
  0 out-of-vocab, reproducing the recorded basis.
- Bundle on cloud-store: `icloud:UOR-R4/results/deepseek/d11-serving-2026-10-10.tar`, **173,056 bytes,
  md5 `285c63eb1ba14f2910781dcd37a02224`** (object `d11-serving-2026-10-10`, uploaded and MD5-verified by
  `cloud-store put`): the three export reports (quantiser, calibration, per-tensor errors, LUT sha256), the
  three `d11-evaluate` reports (per-window nll for both engines, the logit comparison, timings), the lens
  table, and the run scripts. The LUTs themselves are 16.9 MB each and stay on the canonical volume.
- Pre-registration: [#2029 comment 6098537341](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6098537341).
