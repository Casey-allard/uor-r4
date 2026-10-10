# Binding-dense mixture: doubling the binding families at the same recall dose makes the memory panel worse (20/40 → 17/40), and the line reaches 3/3

References #2029 (M1 acceptance criterion 1). Lab: DeepSeek, session `deepseek/bind-dense`, **cycle 3** of
the standing goal adopted from `docs/labs/session-goal.md` @ `8421e605f…`. Run 2026-10-10 12:00–12:06 UTC.
**One pod, 2 × RTX 5090, ~6 minutes, ≈ $0.30**; acceptance scoring on the owner's laptop CPU.

> **One line.** Cycle 2 left the deficit named — the failing memory rows are **selection between two stated
> values** — and the recall generator's `generator=v2` stream dilutes the families that teach that selection
> with six added families. This cycle ran the same recipe on **`generator=v1`**, whose dialogues are *only*
> those families (binding 37.5 % of dialogues against ~15 % in v2), at the two shares whose reply cost is
> acceptable. **It fails**: v5 memory `check_pass` **20/40 (v2, 10 % dose) → 17/40 (v1, 10.2 %)**, and 17/40
> at 25.5 %, with the failure mix getting *worse* (wrong-value 19 of 40 at V10; at V25 a **new** failure mode
> appears — 7 rows name the distractor's *key*). The pre-registered bar is not met, the line's count reaches
> **3/3**, and this delivery carries the **pivot card** D21 requires.

## The pre-registration, and what was actually run

[Pre-registered on #2029](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6097241219)
before any compute: bar, base artifact, split, arms, sizing rule, stop rule and cost, including the statement
that a cycle which does not move the headline **is** the 3/3 pivot.

| | |
|---|---|
| base artifact | `chat-29m-B-lr5e-4` `d8a3c971…`, its own run `report.json` as `init=` provenance |
| the one change | the recall stream: **`generator=v1`** instead of `v2`; `pointer_gate_supervision=0` and no read-binding supervision on either arm |
| arms | **V10 = 10.2 %** recall share, **V25 = 25.5 %** — the shares are the measured mask-1 response ratio over the mixed store, and they are the doses whose v2 counterparts are cycle 1's D10 (memory 20/40, reply 37/232) and D25 (20/40, 28/232), so the generator is the only difference |
| data split | the base's own fine-tune store `ft-mixed-mworld-chatv0` (`tokens.u16` `8794ad27…`, 147,104 episodes) **plus** `dialogue-recall-corpus generate generator=v1 seed=1 … train_on=answers` at 12,150 and 36,450 dialogues (`leak pass` against `data/panels` and the three open-reply-panel request files); **v5 never in training** |
| what v1 is | the probe (`probe-v1`, 200 dialogues) reports exactly the six v1 families — **binding 80 of 200 (40 %)**, update 23, assistant-stated 24, abstain 23, cross-relation 23, reverse 27 — against v2's 15 % binding dialogues; the 12,150-dialogue arm carries **4,561 binding dialogues (37.5 %)** |
| steps / stop rule | `steps=2000 batch=16 lr=2e-4 warmup=100 data_seed=20261009 policy=full_prefix context=384 device=cuda`, `eval_every=250 checkpoint_every=500`, `max_seconds=3600` — each arm ≈ 105 s, none came near the cap |
| pod | `ky2773n3m9wi3h`, `--ref 61a0dcce6…`, parity PASS; lease released and pod deleted at 12:10 UTC |

## Results

### Frozen v5 acceptance panel (cap 64, frozen `conversational-v5-checks.tsv`, deterministic `check_pass`)
| arm | recall share | generator | memory `check_pass` | failures: wrong-value / distractor-key / no-other | v2 counterpart |
|---|---:|---|---:|---|---|
| base | — | — | 10/40 | — | — |
| D10 (cycle 1) | 10.00 % | v2 | **20/40** | 17 / 0 / 3 | — |
| D25 (cycle 1) | 25.00 % | v2 | **20/40** | 15 / 0 / 5 | — |
| **V10** | 10.21 % | **v1** | **17/40** | **19 / 2 / 2** | 20/40 at the same dose |
| **V25** | 25.48 % | **v1** | **17/40** | 16 / **7** / 0 | 20/40 at the same dose |
| D55 (cycle 1) | 55.06 % | v2 | 22/40 | — | — |

Official `chat-grade grade-replies` on both arms: **V10** memory `check_pass` **17/40**, unknowable
2/24, panel 19/64, `acceptable` 5/64, derangement 1; **V25** memory **17/40**, unknowable **18/24**
(the best abstention reading of any arm at that dose — the v1 stream keeps the abstain family), panel
35/64, `acceptable` 16/64, derangement 0.

**Doubling the binding density does not buy binding.** At the same response share, the v1 stream is **three
rows worse** than the v2 stream (20/40 → 17/40), and it does not even preserve the *kind* of failure the
mixture was chosen to fix: at V25 **seven rows name the distractor's key** (`"Your kitten is named …"` for a
question about the brother), a failure mode the v2 arms never produced (0 rows at both D10 and D25).

### Open reply panel (the guard), the primary arm — **pending, declared**
The pre-registration named the reply panel for the primary arm and said that anything the shared judge
does not reach is reported **pending, never guessed**. The 232 replies were generated (10,913 ids,
1,215 s at ~9 ids/s under contention) and their grading is still running as this record is written; the
number will be posted on #2029 when it lands and carried into the next cycle's bundle. It cannot change
this cycle's verdict, which is decided on the memory half (17/40 against a bar of 21/40).

## Decision

**REJECT, and the line reaches 3/3 — the pivot card D21 requires is delivered with this cycle.**

- The pre-registered bar (**memory ≥ 21/40 and reply ≥ 29/232 at the same time**) is not met: both arms sit at
  **17/40** on the memory half.
- The count is **3/3**: three merged PRs on this line — the mixture dose ([#2151](https://github.com/UOR-Foundation/uor-r4/pull/2151)),
  the read-binding objective ([#2155](https://github.com/UOR-Foundation/uor-r4/pull/2155)) and this one — with
  the headline below its level at the start of the window (memory 20/40 → 17/40, reply 37 → [REPLY HEADLINE]).
- **What the three cycles establish together**, and this is the pivot's evidence:
  1. **The mixture moves the panel** and saturates: base 10/40 → 20/40 at a 10 % recall share, 20/40 at 25 %,
     22/40 at 55 %; the reply cost is nil at 10 % (43 → 37, p = 0.42) and significant from 25 % (28, p = 0.0081).
  2. **Neither supervision path moves it**: the copy gate is flat (#2145) and the read-binding objective
     reaches its training target (bound mass 0.73 → 0.90) while the panel goes 20 → 19 → 17 (#2155).
  3. **Nor does re-weighting the mixture's own families**: binding density ×2.5 makes the panel worse.
  4. The residual failures are **value selection** — the model retrieves a value and picks the wrong one —
     and no data or loss lever tried so far reduces that.
- **Criterion 1 remains NOT MET** on both halves and is not claimed.

## Limitations

- **One seed per arm**, 40-row memory panel: 17 vs 20 is three rows, and the wrong-value/discriminator-key
  split is the sharper signal — it moves in the *worse* direction and it produces a failure mode the v2 arms
  did not have.
- **The generator is one argument, the curriculum is one document** (`generator=v1` draws only the v1
  families); a different binding emphasis (family weights inside v2) is *not* what this cycle tested.
- The reply-panel guard covers V10 only and is **pending** at merge time (declared above and in the
  pre-registration); it cannot change the verdict, which the memory half decides.
- Nothing here is evidence about addressed memory: the artifact still carries a copy pointer and **no memory
  operator**, which is precisely what the pivot below names.

## Cost

| | |
|---|---|
| pod | `ky2773n3m9wi3h`, 2 × RTX 5090, 12:00:14Z → 12:10Z (~10 min incl. bootstrap, ≈ $0.40); lease released, pod deleted |
| GPU work | two arms × ~105 s = 3.5 min; two corpus generations ~3 s; two mixes ~4 s |
| laptop CPU | v5 replies 2 × ~2 min; v5 grading 2 × ~12 min; failure-cause analysis seconds; reply-panel replies + grading for V10 on the shared judge |
| external | none; inside the ≤ 4 pods / ≤ $8/h caps |

## Evidence

- Arm reports, curves and settings: pod volume `/workspace/uor-r4/deepseek/pointer-ft-20261009/`
  (`runs4/dense-V{10,25}/`, `recall-v1-12k`, `recall-v1-36k`, `mixv1-10`, `mixv1-25`, `probe-v1`), pulled to the
  laptop and bundled.
- Acceptance reports: `score/v5g-{V10,V25}/report.json`, `score/v5-{V10,V25}/replies.json`, reply panel
  `score/rpg-V10/report.json`.
- Bundle on cloud-store: `icloud:UOR-R4/results/deepseek/binding-dense-2026-10-10.tar`, **1,600,512 bytes,
  md5 `0cc1b5ef952306971d218bab5dd11f3c`** (object `binding-dense-2026-10-10`, uploaded and MD5-verified by
  `cloud-store put`; it holds both arm reports and curves, both v5 acceptance reports, the primary arm's v5
  and reply-panel reply files, the corpus generator/leak/manifests for both v1 streams and mixes, the probe
  that establishes the v1 family composition, the failure-cause script and the scoring logs).
- Pre-registration and cards: [#2029 comment 6097241219](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6097241219).
