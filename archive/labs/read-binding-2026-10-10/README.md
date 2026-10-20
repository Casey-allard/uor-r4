# Read-binding supervision reaches its own objective (bound mass 0.73 → 0.90) and leaves the frozen panel flat — it repairs 9 wrong-value rows and breaks 4

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek, session `deepseek/bind-sup`, **cycle 2** of the
standing goal adopted from `docs/labs/session-goal.md` @ `8421e605f…`. Run 2026-10-10 10:22–10:31 UTC.
**One pod, 2 × RTX 5090, ~9 minutes, ≈ $0.45**; acceptance scoring on the owner's laptop CPU.

> **One line.** Cycle 1 exposed the remaining memory deficit as a **wrong-value (binding) failure**: 15 of 40
> rows fail for both new doses, every one of them naming the *distractor's* value rather than the asked key's.
> This cycle trains the mechanism that says what those rows need — `read_binding_supervision`, whose labels
> mark the asked value's history positions as `bound` and the other stated values as `competing`. On its own
> terms it works: the binding head's mass on the bound positions rises **0.73 → 0.90** (W = 0.1) and **0.93**
> (W = 0.5) and the binding NLL falls **0.67 → 0.13–0.18**. On the frozen panel it moves the failure mode and
> not the total: **wrong-value rows 17 → 12** (9 repaired, 4 broken), memory `check_pass` **20/40 → 19/40**
> (W = 0.1) and **17/40** (W = 0.5). The pre-registered bar needs memory ≥ 21/40 with reply ≥ 29/232, so the
> verdict is **REJECT**; the line's count goes to **2/3**. The reply-panel guard is clean: **39/232**
> `fluent_and_relevant`, statistically indistinguishable from the base's 43 (p = 0.62) and from cycle 1's
> anchor (p = 0.86), so unlike the mixture doses this objective costs the chat register nothing measurable.

## The pre-registration, and what was actually run

[Pre-registered on #2029](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6096484704)
before any pod existed: bar, base artifact, data split, arms, stop rule and cost. Two things not in it are
declared on the issue **before the affected numbers existed**: the reply-panel ordering under the shared judge,
and the wrong-value sub-reading as the mechanism check.

| | |
|---|---|
| base artifact | `chat-29m-B-lr5e-4` `d8a3c971ef07a82ca0db0704b34821e24782c7734baaafb504fd754f82988068`, its own run `report.json` as `init=` provenance |
| the one change | `read_binding_supervision=W` (`StackModel::read_supervised_loss`): at each labelled query `W · mean(−log(m + 1e-6))`, `m` the binding head's attention mass on the **bound** value's history positions; the head is picked without gradient as the one with most mass on bound + competing together |
| data split | **cycle 1's 10 % mixture, unchanged** (`mix-10`: base store `8794ad27…` + `recall-11k`, `tokens.u16` `888241261c378138…`, 163,446 response episodes) **plus** the `binding_labels=1` sidecar regenerated with the identical seed and dialogue count |
| labels | `read_binding_labels=<recall-11k-b>/train`, `read_binding_source=recall`; the regenerated store is **byte-identical** to cycle 1's (`tokens.u16` `28f7bacccd099b18…`, `response_mask.u8` `444eaae67eb4ef98…`), so the only difference from cycle 1's anchor is the objective |
| arms | **B1 = 0.1 (primary)**, **B5 = 0.5**; the **W = 0 anchor is cycle 1's D10** (same mixture, seed and recipe, so no re-run) |
| steps / stop rule | `steps=2000 batch=16 lr=2e-4 warmup=100 data_seed=20261009 policy=full_prefix context=384 device=cuda qat=false`, `eval_every=250 checkpoint_every=500`, `max_seconds=3600` — no arm came near it (≈ 105 s each) |
| pod | `ydz1z30jlvgsnr`, `--ref d9f883fcf16a7eb7328ed944db8395c08ae654c6`, CUDA parity PASS; lease released and pod deleted at 10:31 UTC |

## Results

### The objective reaches its own target
| arm | `read_binding_supervision` | binding NLL (train) 250 → 2000 | bound mass | competing mass | `dev_response_nll` |
|---|---:|---:|---:|---:|---:|
| D10 (anchor) | 0.0 | — | — | — | 0.3773 |
| **B1** | **0.1** | 0.673 → **0.183** | 0.732 → **0.896** | 0.140 → 0.090 | 0.3627 |
| **B5** | **0.5** | 0.695 → **0.132** | 0.739 → **0.934** | 0.150 → 0.056 | 0.4218 |

At both weights the read's binding head moves onto the asked value's positions and off the competing ones,
monotonically, on the training stream. **The objective is not inert and it is not misaimed.**

### The frozen v5 panel (cap 64, the frozen `conversational-v5-checks.tsv`, deterministic `check_pass`)
| arm | memory `check_pass` | unknowable `check_pass` | panel `check_pass` / `acceptable` | derangement |
|---|---:|---:|---:|---:|
| base | 10/40 | 1/24 | 11 / 4 of 64 | 0 |
| D10 (anchor, W = 0) | **20/40** | 0/24 | 20 / 13 of 64 | 0 |
| **B1 (W = 0.1)** | **19/40** | 5/24 | 24 / 8 of 64 | 1 |
| **B5 (W = 0.5)** | **17/40** | 4/24 | 21 / 12 of 64 | 2 |

### The mechanism reading: the targeted failure mode moves, the total does not
Judge-free classification of every memory-row failure by *why* it fails (wrong value / distractor key / no
value), from the same replies the official report grades:

| arm | `check_pass` | failures: **wrong-value** | distractor-key | no/other |
|---|---:|---:|---:|---:|
| D10 (anchor) | 20/40 | 17 | 0 | 3 |
| **B1** | **19/40** | **12** | 2 | 7 |
| B5 | 17/40 | 14 | 3 | 6 |
| D25 (25 % mixture, no supervision) | 20/40 | 15 | 0 | 5 |

Row by row, B1 against the anchor: **nine of the anchor's seventeen wrong-value rows are repaired**
(`conv-v5-mem-002`, `-014`, `-018`, `-022`, `-027`, `-030`, `-032`, `-034`, `-040`) and **four rows that were
correct become wrong-value** (`-007`, `-010`, `-013`, `-028`). Net 17 → 12, while the pass total goes
20 → 19 and the dose response runs *downward* (20 → 19 → 17 as the weight rises).

**So the cycle's negative is precise:** the objective does what its labels say on the read, and the panel
trades one failure mode for another at this scale. Neither supervision path tried on this line — the copy
gate (#2145) nor the read-binding objective (this cycle) — has moved the memory total; the only thing that
has is the **mixture** (cycle 1: 10/40 → 20/40).

### Open reply panel (the guard), the primary arm
| arm | `fluent_and_relevant` (the pre-registered metric) | `acceptable` incl. the 88 row checks | fluent | relevant | derangement | vs base |
|---|---:|---:|---:|---:|---:|---|
| base, sealed (re-graded in #2145) | **43/232** | 27/232 | 70 | 53 | 4 | — |
| D10 (anchor, W = 0) | **37/232** | 16/232 | 61 | 46 | 4 | 22 / 16, p = 0.42 |
| **B1 (W = 0.1)** | **39/232** | 18/232 | 64 | 48 | 4 | 20 / 16, **p = 0.62** |
| D25 (25 % mixture, no supervision) | 28/232 | 17/232 | 56 | 38 | 1 | 22 / 7, p = 0.0081 |
| D55 (55 % mixture, no supervision) | 22/232 | 12/232 | 36 | 31 | 3 | 26 / 5, p = 0.0002 |

Per category (`fluent_and_relevant`): base → D10 → B1: `follow_up` 1 → 1 → 0; **`heldout_first_turn`
34 → 28 → 35**; `simple_instruction` 1 → 1 → 0; `simple_question` 1 → 2 → 0; `smalltalk` 6 → 5 → 4.
**B1's reply panel is statistically indistinguishable from the base's (43 → 39, p = 0.62) and from
cycle 1's anchor (37 → 39, 14 / 16, p = 0.86)**, and it is the best `heldout_first_turn` count of any
fine-tuned artifact on this panel (35 against the base's 34). The read-binding objective therefore costs
the chat register **nothing measurable**, unlike the 25 % and 55 % mixture doses.

## Decision

**REJECT — the pre-registered bar is not met on the memory half by either arm, and the cycle's value is the
mechanism result plus the closed door.**

- **Bar:** memory ≥ 21/40 **and** reply ≥ 29/232 at the same time. B1 reaches **19/40** and B5 **17/40**, so
  the bar cannot be met whatever the reply half says; no threshold is moved to fit.
- **Count: 2/3.** Under the rule stated on #2029 (the count increments unless both halves are at least at
  their last-cycle levels), the memory half is below the 20/40 this line already had. **One more merged PR
  on this line without a moved headline makes 3/3 and requires a pivot card (D21 §1)** — stated here so it is
  not a surprise.
- **What the cycle establishes.** (1) `read_binding_supervision` reaches its objective (bound mass 0.73 →
  0.90/0.93; competing mass halved) and **does not transfer to the acceptance panel**; (2) it *does* move the
  failure mode the panel is made of — 9 wrong-value rows repaired, 4 broken — so the binding failure is
  reachable by a training objective and this one is not yet the right shape; (3) with both supervision paths
  now measured as flat, **the data mixture remains the only lever this line has moved**, which is what the
  next cycle must use.
- **The artifact is not better than the one this line already had.** B1 trades one memory row (20 → 19)
  for two reply cells (37 → 39) and five fewer wrong-value rows (17 → 12); the reply difference is inside
  noise (p = 0.86 against the anchor) and so is the memory difference, so the honest summary is **a wash on
  the headline with a real mechanism movement underneath it** — not a new best.
- **Criterion 1 remains NOT MET on both halves** and is not claimed: memory 19/40 against ≥ 34/40, reply
  39/232 against ≥ 116/232. The base artifact's 10/40 and 43/232 are unchanged.

## Limitations

- **One seed per arm.** The 20 → 19 → 17 memory sequence is 1–3 rows on a 40-row panel; the wrong-value
  counts (17 → 12 → 14) are the sharper signal and they are row-paired, not aggregate.
- **The objective's own metrics are training-stream metrics**, not panel metrics: "bound mass 0.90" means the
  read put its mass on the labelled value's positions **on the recall training data**, which is exactly what
  the labels asked for and not evidence about the panel.
- **The wrong-value classification is judge-free but not the official check**; it uses the same frozen row
  checks and separates failures by cause, and it agrees with the official totals it is derived from.
- **The reply-panel guard covers the primary arm only**, under the shared judge ordering declared before the
  numbers existed; B5's reply half is not scored.
- **Nothing here is evidence about addressed memory**: the artifact still carries a copy pointer and no memory
  operator.

## Cost

| | |
|---|---|
| pod | `ydz1z30jlvgsnr`, 2 × RTX 5090, 10:22:08Z → 10:31Z (~9 min, ≈ $0.45); lease released, pod deleted |
| GPU work | two arms × ~105 s = 3.5 min; corpus regeneration 5 s |
| laptop CPU | v5 replies 2 × ~2 min, v5 grading 2 × ~12 min, wrong-value analysis seconds; reply-panel replies ~10 min and grading ~35 min for the primary arm on the shared judge |
| external | none; 1 pod and $1.19/h of the ≤ 4 pods / ≤ $8/h caps while running (another lab's pod was the only other) |

## Evidence

- Arm reports, curves, settings and the label sidecar: pod volume `/workspace/uor-r4/deepseek/pointer-ft-20261009/`
  (`runs3/bind-B{1,5}/`, `recall-11k-b/train/binding_labels.json[l]`, `mix-10`), pulled to the laptop.
- Acceptance reports: `score/v5g-{B1,B5}/report.json`, `score/v5-{B1,B5}/replies.json`, reply-panel
  `score/rpg-B1/report.json`.
- Bundle on cloud-store: `icloud:UOR-R4/results/deepseek/read-binding-2026-10-10.tar`, **2,241,536 bytes,
  md5 `04172efbba05bdfadf60e9848c3a8b28`** (object `read-binding-2026-10-10`, uploaded and MD5-verified by
  `cloud-store put`): both arm reports and curves, both v5 acceptance reports and reply files, the primary
  arm's reply-panel report, the anchor and base reply-panel reports for the paired tests, the
  `binding_labels.json[l]` sidecar and its generator/leak manifests, the failure-cause analysis script and
  the scoring logs.
- Pre-registration and cards: [#2029 comment 6096484704](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6096484704)
  (pre-registration), [6096645284](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6096645284)
  (mechanism card), [6096682658](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6096682658)
  (cycle 1's zero-dose reply reading, closing the merged record's declared gap).
