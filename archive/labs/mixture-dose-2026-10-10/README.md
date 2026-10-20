# The mixture dose: a 10 % recall share doubles the memory half at no measurable reply cost, while abstention needs 25 %

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek, session `deepseek/mixture-dose`, **cycle 1** of
the standing goal adopted from `docs/labs/session-goal.md` @ `8421e605f677175957ee3a268439bf8d62bfa311`.
Run 2026-10-10 07:22–07:33 UTC. **One pod, 2 × RTX 5090, ~11 minutes, ≈ $0.52** including a short
evidence-fetch pod; every acceptance reading on the owner's laptop CPU.

> **One line.** The last cycle measured the lever (the generated recall-dialogue mixture) but not its
> shape. Four recall shares, one recipe, supervision off: the **memory gain saturates by 10 %**
> (10/40 → 20/40, against 22/40 at 55 %), the **abstention gain does not** (unknowable rows 1/24 → 0/24
> at 10 %, 15/24 at 25 %, 19/24 at 55 %), and the **reply cost appears between 10 % and 25 %** (43 → 37
> at 10 %, *p = 0.42*, not significant; 43 → 28 at 25 %, *p = 0.008*; 43 → 22 at 55 %, *p = 0.0002*). The
> pre-registered KEEP bar needs one arm to hold memory ≥ 21/40 **and** reply ≥ 29/232, and the best new
> arm misses the memory half by **one row** — so the verdict is **REJECT (no keeper)** and the frontier is
> the result.

## The pre-registration, and what was actually run

[Pre-registered on #2029](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6095074135)
before any pod existed: bar, base artifact, data split, steps, stop rule and cost. Nothing below departs
from it except the reply-panel *ordering*, which was declared on the issue **before the affected numbers
existed** (#2029 comments 6095764623 and 6096059506).

| | |
|---|---|
| base artifact | `chat-29m-B-lr5e-4` `d8a3c971ef07a82ca0db0704b34821e24782c7734baaafb504fd754f82988068`, with its own run `report.json` as `init=` provenance (tokenizer `d36d3e87…`, `merges: null`), unchanged from #2145 |
| the one change | the **recall share of the training mixture**; `pointer_gate_supervision=0` on every arm |
| data split | the base's own fine-tune store `ft-mixed-mworld-chatv0` (`tokens.u16` `8794ad2710aa9e2a34772235d2c966d03bd672bf7d3f97385365e77b1a20c3b8`, 147,104 response episodes, 165,759,852 B) **plus** freshly generated `dialogue-recall-corpus generate generator=v2 seed=1 … train_on=answers` corpora, concatenated by `mix-chat-corpus`; both generators report **`leak pass`** against `data/panels` **and** the three open-reply-panel request files; **v5 is never in training** |
| steps / stop rule | `steps=2000 batch=16 lr=2e-4 warmup=100 data_seed=20261009 policy=full_prefix context=384 device=cuda qat=false`, `eval_every=250 checkpoint_every=500`, `max_seconds=3600` — a self-set estimate no arm came near: each took ≈ 105 s |
| pod | `hnzcla6nig6h1q`, `--ref 8421e605f677175957ee3a268439bf8d62bfa311`, bootstrap cache hit, CUDA parity PASS; lease released and the pod deleted at 07:33 UTC |

### The four arms, with their stores

| arm | recall share of response episodes | recall corpus | mixed store | `tokens.u16` sha256 | `model.safetensors` sha256 |
|---|---:|---|---|---|---|
| **D0** | **0 %** | — (the base store alone) | 147,104 runs / 165,759,852 B | `8794ad2710aa9e2a…` (the base store) | `7bc5a1f86636219a…` |
| **D10** | **10.00 %** | `recall-11k`: 11,250 dialogues, 16,342 runs, 2,390,416 tokens, sha `28f7bacccd099b18…` | 163,446 runs / 170,540,684 B | `888241261c378138…` | `4ab6fc86a3fb8592…` |
| **D25** | **25.00 %** | `recall-34k`: 33,750 dialogues, 49,039 runs, 7,127,638 tokens, sha `560a37cfeba6c512…` | 196,143 runs / 180,015,128 B | `0d17cffcfab63fd5…` | `fb8890bdb25db1fa…` |
| **D55** | **55.06 %** | `recall-124k`: 124,000 dialogues, 180,200 runs, 26,212,659 tokens, sha `dd61295eebd308e6…` | 327,304 runs / 218,185,170 B | `f6ae278dfed8b6e5…` | `f76c8bc0f53baf9b…` |

The shares are the pre-declared ones (0.1000, 0.2500, 0.5506), measured as the arm's mask-1 response runs
over the mixed store's, which is the sampler's own ratio. Response *tokens* are 53.9–66.9 % of the mixed
stores, i.e. the ratio is not a token ratio.

**D55 is a re-run, and it is bit-identical to #2145's matched control arm** —
`f76c8bc0f53baf9bf0aebc87ce5e588cf8089aeba295d5645a57838e522ed893`, the same weights, from a different
pod at a different commit. That is why D55's already-measured panels apply here without re-scoring, and it
is a **measured** statement about this trainer's reproducibility at fixed data and seed.

## Results

### The development instrument is monotone in dose — and it is not the acceptance panel

| arm | `dev_response_nll` | `dev_pointer_hit_rate` |
|---|---:|---:|
| D0 | 2.2461 | 0.194 |
| D10 | 0.3773 | 0.316 |
| D25 | 0.2651 | 0.323 |
| D55 | 0.2221 | 0.377 |

More recall in the mixture means a better recall-dev score, exactly as the dose says. The acceptance panel
below is what decides, and it does **not** follow this curve.

### Frozen v5 acceptance panel (cap 64, the frozen `conversational-v5-checks.tsv`, deterministic `check_pass`)

| arm | recall share | memory `check_pass` | unknowable `check_pass` | panel `check_pass` | panel `acceptable` | derangement `check_pass` |
|---|---:|---:|---:|---:|---:|---:|
| base (saved artifact) | — | **10/40** | 1/24 | 11/64 | 4 | 0 |
| D0 | 0 % | **5/40** | **0/24** | 5/64 | 0 | 0 |
| **D10** | 10 % | **20/40** | **0/24** | 20/64 | 13 | 0 |
| **D25** | 25 % | **20/40** | **15/24** | 35/64 | 13 | 0 |
| D55 (#2145 control) | 55.06 % | **22/40** | 19/24 | 41/64 | 20 | 0 |

**Three readings fall out of that table.** (1) **The recall mixture is necessary, not incidental**: two
thousand more steps on the base's *own* store alone (D0) *lose* five memory rows (10 → 5) and take the
abstention behaviour to zero (1/24 → 0/24), so #2145's +10–12 rows were the recall data and not "more
training on the same data". (2) **The memory gain saturates by 10 %**: D10 and D25 both land on 20/40
against D55's 22/40 — a fifth of the recall share buys 91 % of the movement. (3) **The abstention gain
does not saturate, and it starts later**: 0/24 at 10 %, 15/24 at 25 %, 19/24 at 55 %, against the base's
1/24. The mixture teaches two behaviours and they are dosed differently.

### Open reply panel (the guard) — 232 rows, cap 64, `fluent_and_relevant` is the pre-registered metric

| arm | recall share | `fluent_and_relevant` | `acceptable` incl. the 88 row checks | fluent | relevant | derangement `fluent_and_relevant` |
|---|---:|---:|---:|---:|---:|---:|
| base, sealed (re-graded on this build in #2145) | — | **43/232** | 27/232 | 70 | 53 | 4 |
| D10 | 10 % | **37/232** | 16/232 | 61 | 46 | 4 |
| D25 | 25 % | **28/232** | 17/232 | 56 | 38 | 1 |
| D55 (#2145 control) | 55.06 % | **22/232** | 12/232 | 36 | 31 | 3 |
| D0 | 0 % | not scored — see the scope note | | | | |

Per category (`fluent_and_relevant`), base → D10 → D25 → D55: `follow_up` 1 → 1 → 0 → 1;
**`heldout_first_turn` 34 → 28 → 23 → 17**; `simple_instruction` 1 → 1 → 0 → 1; `simple_question`
1 → 2 → 0 → 0; `smalltalk` 6 → 5 → 5 → 3.

Paired exact McNemar on the 232 rows: **base vs D10 — 22 base-only / 16 D10-only, `p = 0.42`, i.e. the
10 % arm's reply panel is statistically indistinguishable from the base's**; base vs D25 — 22 / 7,
`p = 0.0081`; base vs D55 — 26 / 5, `p = 0.0002`; D10 vs D25 — 19 / 10, `p = 0.136`; D10 vs D55 — 24 / 9,
`p = 0.0135`. Every arm still discriminates its own derangement control (D10 35 vs 2, D25 18 vs 1).

**So the reply cost appears between the 10 % and 25 % doses, and it is not linear**: a fifth of the 55 %
mixture's recall share keeps the whole reply panel and the whole memory gain; five halves of it give up
15 reply cells (p = 0.008) to buy 15 abstention rows.

## Decision

**REJECT under the pre-registered rule; the frontier, not a keeper, is the result — and the frontier is
what the next cycle needs.**

- The pre-registered KEEP bar needs one arm to hold **reply ≥ 29/232 and memory ≥ 21/40 at the same
  time**. Only D55 reaches memory ≥ 21 (22/40), and its reply half is 22/232; the new doses sit at
  **20/40 — one row short** — so the bar is not met and the artifact is **not kept as the base**. No
  threshold, panel or grader is moved to make it fit.
- **The line's count increments to 1/3**, on the rule stated on #2029 before the reply numbers existed:
  the count increments unless **both** halves are at least at their last-cycle levels (reply ≥ 28/232 and
  memory ≥ 21/40). The reply half clears it (37 ≥ 28) and the memory half does not (20 < 21).
- **What the cycle actually establishes, in the order the next decision needs it.**
  1. **A 10 % dose is a keeper-grade trade in everything but one row**: memory **10/40 → 20/40** at a
     reply panel that is **statistically indistinguishable from the base's** (43 → 37, p = 0.42) and a
     base artifact that is otherwise untouched.
  2. **Abstention is bought separately**: it appears at 25 % (15/24) and 55 % (19/24), and it costs reply
     cells there (p = 0.008 and p = 0.0002). `answer:abstain` is 24,428 of the recall corpus's 180,200
     trained replies (13.6 %), so a slice dosed on its own is the obvious next test rather than a bigger
     mixture.
  3. **More training on the same data is destructive, not neutral** (D0: −5 memory rows, −1 abstention,
     from the base's own store), which is why the mixture — not the step budget — is the lever.
- **Criterion 1 remains NOT MET on both halves** and is not claimed: memory 20/40 against ≥ 34/40, reply
  37/232 against ≥ 116/232. The base artifact's own 43/232 and 10/40 are unchanged.
- **The next cycle this designs**: the same four-arm pattern on an **abstention slice** (the abstention
  family at a fixed share of episodes on top of a 10 % recall mixture), with the reply panel
  pre-registered as the guard, aiming for the first artifact that holds memory ≥ 21/40 **and** reply
  ≥ 29/232 at the same time.

## Limitations

- **One seed per arm, four doses.** The one-row difference between D10/D25 (20/40) and D55 (22/40) is
  inside one seed's spread; the saturation reading rests on 10 % and 25 % agreeing at 20/40 against 55 %
  at 22/40, not on a single row.
- **The memory panel is 40 rows** (one row = 2.5 points) and the abstention reading is 24 rows.
- **The reply panel for the zero-dose arm was not scored.** The pre-registration named three arms for it;
  the shared single-slot judge carries up to four clients across labs, and this cycle declared on #2029
  (**before any of the affected numbers existed**) that it would score D10 first, then D25, and drop D0
  last — D0 already fails the memory half at 5/40 and so cannot change the verdict. Its v5 half is
  reported; its reply half is **pending, not guessed**.
- **D55's panels are inherited from #2145** on the strength of a byte-identical checkpoint, not by
  re-scoring; that cycle measured them on the same laptop build and grader digest.
- **The judge is the shared local `qwen2.5:7b`**; the like-for-like metric is `fluent_and_relevant`, the
  same field the sealed 43/232 baseline was measured on, and all comparisons above are paired on rows.
- **The mixture ratio is a response-episode ratio**, because the sampler draws responses uniformly.
- **Nothing here is evidence about addressed memory**: the artifact still carries a copy pointer and no
  memory operator, so `memory_read_diagnostic` returns `None` by construction.

## Cost

| | |
|---|---|
| pod | `hnzcla6nig6h1q`, 2 × RTX 5090, 07:22:08Z → 07:33Z (~11 min, ≈ $0.44); plus a `--no-bootstrap` pod for the corpus manifests (~4 min, ≈ $0.08); both leases released and both pods deleted |
| GPU work | four arms × ~105 s = 7 min of training; two corpus generations 3 s; three mixes 6 s |
| laptop CPU | v5 replies 4 × ~2 min, v5 grading 4 × ~10 min; reply-panel replies 3 × ~10–20 min; reply-panel grading 3 × ~40–80 min serialised on the shared judge |
| external | none; at most 3 pods and ≈ $5.35/h against the ≤ 4 pods / ≤ $8/h caps while running |

## Evidence

- Arm reports, curves and stores: pod volume `/workspace/uor-r4/deepseek/pointer-ft-20261009/`
  (`runs2/dose-*/`, `mix-10`, `mix-25`, `mix-55`, `recall-11k`, `recall-34k`, `recall-124k`); the four
  `report.json` files and the corpus `generator.json`/`leak.json`/manifests were pulled to the laptop and
  bundled.
- Acceptance reports: `score/v5g-{D0,D10,D25}/report.json`, `score/rpg-{D10b,D25}/report.json`,
  `score/{v5,rp}-*/replies.json`.
- Bundle on cloud-store: `icloud:UOR-R4/results/deepseek/mixture-dose-2026-10-10.tar`, **5,058,048 bytes,
  md5 `d31b25ddabaa6c1a161cfd7ae3de0104`** (object `mixture-dose-2026-10-10`, uploaded and MD5-verified by
  `cloud-store put`). It holds the four arm reports and curves, every v5 acceptance report, the reply-panel
  reports and reply files that completed, the corpus `generator.json`/`leak.json` and manifests, the
  pod-side run scripts and the scoring logs.
- Pre-registration and cards: [#2029 comment 6095074135](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6095074135)
  (pre-registration), [6095066546](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6095066546)
  (cycle 1 step 1), [6095478458](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6095478458)
  (card 2), [6095764623](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6095764623)
  (card 3, the reply-panel scope note), [6096059506](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6096059506)
  (card 4, D10's frontier point).
