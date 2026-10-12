# Why starling learning is a lottery, and a dense bootstrap that removes it (12 October 2026)

**Milestone:** M4 [#2032](https://github.com/UOR-Foundation/uor-r4/issues/2032). Owner-requested investigation into why some seeds never learn, followed by fix 1 and fix 1b.
**Line:** starling read · count 1/3 → 2/3 (no headline move; best result row 2) · headline "softmax at runtime: yes" → yes.
**Pre-registrations** (each reviewed, FIT, before compute):
- step 3c: [6118203486](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6118203486);
- the diagnostic: [6118671477](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6118671477), with its amendment;
- fix 1: [6119228664](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6119228664);
- fix 1b: [6119887475](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6119887475).

## The question

With the persistent tree and the readout distillation ([#2241](https://github.com/UOR-Foundation/uor-r4/pull/2241)), the starling read reaches softmax-level recall at 1536 **when a seed learns** (0.964 at about 69 candidates per query). But only 1/3 seeds learned, and seeds flipped between schedules. Why is learning random, and how can it be made reliable?

## 1. Step 3c: the helper schedule is not the lever

The arms used WTR's flags at 1536, seeds 1–3:
- **A75:** the helper is kept to step 2700, then the distillation runs.
- **S72:** the budget is doubled, to 7,200 steps.

| Arm | Recall, seeds 1–3 | Learned (≥ 0.85) |
|---|---|---:|
| WTR (base) | 0.063, 0.964, 0.104 | 1/3 |
| A75 | 0.428, 0.023, 0.009 | 0/3 |
| S72 | 0.413, 0.264, 0.502 | 0/3 |

**Row 3 for both.** Seeds also flipped: WTR seed 2 learned, while A75 seed 2 failed. Outputs: `claude-s3c-20261012` (md5 e2199a3e…).

## 2. The bootstrap diagnostic: what makes it random

**Instrument** (observation only): at every evaluation, the dense helper's weight on the true source, the source's **share** of the helper's mass, and recall with the source forced in. **Runs:** WTR at 512 and at 1536, 8 fresh seeds each (11–18). Escape step E is the first evaluation with in-class accuracy ≥ 0.20.

| Context | Escaped | Median E | Learned | Early helper share on source, escapers vs others |
|---|---:|---|---:|---|
| 512 | 6/8 | 2950 | 2/8 | 0.0098 vs 0.0072 |
| 1536 | 3/8 | never | 1/8 | 0.0058 vs 0.0033 |

- **P1 holds:** escape is slower and rarer at 1536.
- **P2 holds at both contexts:** seeds whose helper happened to concentrate more on the true source early escaped sooner. At 1536, every seed with a share ≤ 0.0032 never escaped.
- **P3 fails, so by the amended letter this is row D.** Only one seed per context escaped before the anneal ended, and several escaped **after** the helper ended and still learned (512 seeds 13 and 18 at step 1900, giving 0.99; 1536 seed 18 at step 2000, giving 0.86).
- **Recall tracks the budget left after escape.** At 512: escape at 1900 → 0.99; at 2900–3000 → 0.41–0.54; at 3600 → 0.28; never → about 0.
- **For comparison,** the L2 softmax read escapes at steps 500–1100 on every seed at both contexts ([step 2 logs](../starling-context-2026-10-12/README.md)).

**Mechanism, as measured:**
- Learning waits for a heavy-tailed escape from the chance plateau, where selection and the value path each need the other's signal.
- The escape is driven by how much the **dense helper concentrates on the true source**. The helper is a 1/(t+1) causal mean gated by the address dot, so its concentration falls with context (the share roughly halves from 512 to 1536).
- Within a fixed budget, escape timing becomes the seed-to-seed lottery. Softmax escapes early because its read concentrates by construction.

Outputs: `claude-diag-20261012` (md5 9cb96fac…).

## 3. Fix 1: a dense bootstrap

**Change** (training only):
- Until the training NLL with the helper on reaches 1.5 (at the latest half the steps), the dense helper is the **content teacher read** at full weight, with no KL.
- Then it anneals out over 25% of the steps, while the KL, salience and readout terms move selection into the 600-cell addresses.

| Seed (1536) | Without (diagnostic) | **Fix 1 (DB)** | Handover | Forced-in |
|---|---:|---:|---|---:|
| 11 | 0.800 | 0.366 | cap 1800 | 0.42 |
| 12 | 0.015 | **0.968** | cap 1800 | 0.98 |
| 13 | 0.486 | 0.727 | cap 1800 | 0.93 |
| 14 | 0.009 | 0.731 | NLL at 1200 | 0.86 |
| 15 | 0.009 | 0.673 | NLL at 1100 | 0.84 |
| Mean | 0.264 | **0.693** | | |

**Row 4 by the bars:** 1/5 learn, and the dense phase escaped by NLL on 2/5.
- **Beyond the bars:** no seed is left at chance (3/5 were), and 4/5 reach 0.67–0.97. The lottery is largely gone.
- **Two failure points remain:**
  1. **The dense phase is slow** on 3/5 seeds. The teacher scores q·k/√d with no learned scale, while the L2 control scores −β(|q − k| − offset) with a learned scale.
  2. **The handover loses accuracy.** Seed 14 was at NLL 0.08 with the helper on and ended at 0.73: in phase 1 the addresses get no gradient, so selection is not ready when the helper leaves.

Outputs: `claude-fix1-20261012` (md5 5f26d0da…).

## 4. Fix 1b: the L2-score teacher, and the KL during the bootstrap

**Change** (training only):
- **DBR:** the dense teacher scores with the L2 read's own learned score −β(|q − k| − offset) (`starling_teacher_read`).
- **DBRK:** DBR, plus the distillation KL (with the salience and readout terms) on during phase 1 as well (`starling_bootstrap_kl`), so the addresses train before the handover.

Seeds 11–15 at 1536, otherwise as fix 1.

| Seed | Without | Fix 1 (DB) | DBR | **DBRK** | DBRK handover |
|---|---:|---:|---:|---:|---|
| 11 | 0.800 | 0.366 | 0.011 | **0.971** | cap 1800 |
| 12 | 0.015 | 0.968 | 0.677 | **0.959** | NLL at 1600 |
| 13 | 0.486 | 0.727 | 0.744 | **0.999** | NLL at 1300 |
| 14 | 0.009 | 0.731 | **0.988** | 0.792 | cap 1800 |
| 15 | 0.009 | 0.673 | 0.803 | 0.008 | cap 1800 |
| **Learned (≥ 0.85)** | 1/5 | 1/5 | 1/5 | **3/5** | |
| Mean | 0.264 | 0.693 | 0.645 | **0.746** | |

**Decision: row 2 for DBRK, the best arm** (3/5 learn). DBR is row 3: the dense phase escapes by NLL on 4/5 seeds, but only 1/5 learn.

**What the arms show:**
- **The learned-score teacher fixes the bootstrap** (DBR escapes on 4/5, against fix 1's 2/5). The handover then loses accuracy. For example, DBR seed 15 kept the source 99.8% of the time, but natural and forced-in recall were both 0.80.
- **Training the addresses during the bootstrap fixes most handovers** (DBRK). Three seeds reach 0.96–0.999, flat to d1000 (0.94–0.996), with held-out class 0.96–0.998. That is the L2 softmax read's level (0.999) from a read that is softmax-free at serving time.
- **One DBRK seed collapsed** (seed 15: hit rate 0.16, recall 0.008). This is the risk the pre-registration named: the KL from step 0 against an early teacher can collapse the addresses, as in SD.
- **Handover timing matters but is not decisive:** DBRK seed 11 learned despite hitting the cap.

Outputs: `claude-fix1b-20261012` (md5 24ad4488…).

## Conclusions

1. **Why learning was a lottery:** a heavy-tailed escape from the chance plateau, driven by how concentrated the dense helper's signal on the true source happened to be. The 1/t helper dilutes that concentration as context grows, and a fixed budget turns the escape time into a seed lottery.
2. **What removes it:** a concentrated dense bootstrap, scored like the L2 control, with the KL training the 600-cell addresses during the bootstrap. 3/5 seeds then match softmax at 1536, against 1/3 for WTR and 1/8 for the diagnostic seeds.
3. **What remains:**
   - one collapse in five (the early KL);
   - one seed short (0.79).

   The next step, per row 2, is 5 more DBRK seeds. Two other levers are ready: the teacher's L2 **age table** (`starling_teacher_age`, implemented and untested; the L2 control also uses it), and starting the KL only once the teacher has sharpened.

## Next

DBRK with 5 more seeds (16–20) at 1536, alongside DBRK + `starling_teacher_age`, scored on learn count. If either reaches ≥ 4/5 of 10, then 4096.

## Code (training only; the served read is unchanged and softmax-free)

- **The `track_bootstrap` probe** (helper weight and share on the true source, forced-in recall over training).
- **`set_starling_teacher_helper`:** the dense teacher read as the helper, without the KL.
- **`mqar-bench`:**
  - `starling_bootstrap=NLL`, a phase-1 teacher read with an NLL trigger and a cap;
  - `starling_handover=F`, the anneal length;
  - `starling_teacher_read`, the teacher scored with the L2 read's learned score;
  - `starling_bootstrap_kl`, the KL during phase 1.
- **Reviews:** CODE OK at each step after fixes (probe accumulation across layers; anneal conflict rejected).
