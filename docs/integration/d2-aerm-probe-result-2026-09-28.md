# D2: architectural exact relational memory — result

September 28, 2026. References #973 and #962 under #820.
- **Lab:** Lab 1 (Claude).
- **Plan:** [pre-registered](d2-aerm-probe-plan-2026-09-28.md) at `8b8f4ef5`, fix `fb57c4ee`.
- **Evidence:** [`d2-aerm-probe-2026-09-28.json`](../evidence/d2-aerm-probe-2026-09-28.json), assembled from the sealed roots.

**Status.** Attempt 2 was executed with 3 seeds × 2 arms, all six runs exited 0 and the roots are sealed. **The frozen gate FAILS on the margin in every seed.** The measurements beside the gate locate the mechanism's bottleneck precisely: the store and its read work, and the learned parser does not generalise.

## Frozen gate

Applied by `aerm-probe summarize` to the sealed roots.

| Seed | Memory arm, Updated | Control, Updated | Margin (≥ 0.30) | Text NLL, memory − control (≤ 0.05) |
|---:|---:|---:|---:|---:|
| 1 | **1.000** | 0.854 | 0.146 ✗ | −0.027 ✓ |
| 2 | **1.000** | 0.878 | 0.122 ✗ | −0.039 ✓ |
| 3 | **1.000** | 0.898 | 0.102 ✗ | −0.038 ✓ |

The memory arm clears 0.90 and the text gate in every seed. The control's recency reaches 0.85–0.90 on the gated class, so the margin falls short. **Outcome: FAIL.** As pre-registered, it is recorded, not extended, and no dose, seed or scale follows automatically.

## Measured beside the gate

These are reported, not gated. The runs are at equal parameters (1,411,790 against 1,411,660) on identical batches.

**Fresh dialogues** (training templates and names; 512 per seed, about 1,190 queries), accuracy per class:

| Class | Memory arm (every seed) | Control, seeds 1 / 2 / 3 |
|---|---:|---|
| First | 1.000 | 0.507 / 0.507 / 0.543 |
| Updated | 1.000 | 0.854 / 0.878 / 0.898 |
| Updated, recency trap | 1.000 (36/36) | 0.25 / 0.25 / 0.42 |
| Reasserted | 1.000 | 0.77 / 0.73 / 0.69 |
| Previous | 1.000 | 0.60 / 0.63 / 0.58 |
| PreviousAbsent ("I do not know.") | 1.000 | 0.51 / 0.37 / 0.45 |
| Absent ("I do not know.") | 1.000 | 0.00 / 0.02 / 0.07 |
| Free-running exact final answers | 32/32 in every seed | 20 / 16 / 23 of 32 |

- **The memory arm answers every class perfectly**, including whole answers, abstentions and recency traps.
- **The dense control mostly answers with the latest mention.** It reaches 0.85–0.90 where the latest mention is right, but falls to about 0.5 on older facts and to 0.25–0.42 on recency traps, and almost never abstains when a fact is absent.
- **No misfires on stories:** the memory arm's heads fired no memory operation on the 131,072 development story tokens (0.0 per 1,000).

**Held-out templates and names** (512 per seed): **the memory arm collapses.**

| | Memory arm | Memory arm with the gold register | Control |
|---|---|---|---|
| Updated | 0.000 / 0.000 / 0.015 | **1.000** | 0.41 / 0.39 / 0.47 |
| First | 0.000 / 0.000 / 0.035 | **1.000** | 0.26 / 0.31 / 0.28 |

- **With a perfect parser the model answers every held-out class correctly** (the gold-register column).
- **Every memory-arm failure traces to the parser.** The four-class trace counts 1,125, 791 and 629 failures per seed, all *Unavailable* or *NotSelected*, and **zero** *Emission*.
- **The read trigger never fires on the unseen query phrasing** ("Which pet does Mia have now?"): 0 of 826 gold reads in seeds 1 and 2, and 158 of 826 in seed 3.
- **Write triggers transfer** to the unseen assertion phrasings: 2,525–2,612 of 2,614.
- Role-tag accuracy falls from 1.000 to 0.971–0.973. Unseen names are part of the cause.

## Reading

This interprets the result; it changes no gate.

1. **Exact version-ordered memory and its read path work.**
   - In distribution, the memory arm is perfect on every class, including absent facts, previous values and recency traps.
   - The equal-parameter dense model fails on these, and cannot abstain.
   - Held out, the gold register would give 1.000.
2. **The bottleneck is learned addressing from language, specifically when to read.**
   - A trigger-gated read learned from three query phrasings does not recognise a fourth.
   - This is the "learned interface to exact structure" failure of synthesis §0 finding 4, now measured with a clean trace.
3. **The pre-registered gated class was the one most favourable to a recency-biased control.**
   - The frozen outcome stands.
   - The lesson for later gates: name, before the run, the class that separates the mechanisms (older facts, recency traps, absent facts), and gate generalisation to held-out phrasing.

## Scope and limits

- **Probe scale:** a 1.4M-parameter probe on synthetic single-token slots with five template families.
- **Gate metric:** a teacher-forced value slot. Free-running answers are reported separately.
- **Memory semantics are a subset:**
  - current value and **previous distinct value** only; the scoped-memory contract also has previous-record and initial views;
  - a fresh store per sequence, with no persistence across windows or sessions.
- **No trained weights were saved.** The runner writes reports only, so no D2 model can be loaded or served.
- A D2 pass would not have qualified conversation, cross-window memory or native serving, and this result does not either.

## What it changes

With the owner's decisions of 15:27 UTC:
- **The geometric read/address operator goes on the main path**, and D2 names its first job.
  - Writes transfer; reads do not.
  - So the operator is an **always-on learned addressed read into the exact store**, replacing the brittle trigger-gated read, with the current read as fallback.
  - D6 and D3 shape its representation and cost claims.
- **Before the next fit:**
  - checkpoint save and reload for the stack plus store;
  - an identified consumer;
  - a decision on memory semantics (previous-record versus previous-distinct, and persistence).
- **Cost:** 5,887 s wall for three parallel processes of two threads each, peak sampled RSS 1.64–1.73 GiB per process. The void attempt 1's processes ran about 3.5 minutes. About 1 MB of new SSD storage.
