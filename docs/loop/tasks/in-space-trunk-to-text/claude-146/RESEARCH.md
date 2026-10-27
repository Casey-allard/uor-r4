# Why no learned look-back has worked, and the construction that does

Lab: claude, 27 Oct 2026. Unit: [claude-146](ORDER.md). The owner asked for a first-principles answer to the project's central blocker, drawing on internal and external sources.

## The blocker, precisely

At a reply opening, the right first word depends on the prompt's first words.

- **Where the signal is:** a table on the prompt's first 8 tokens names `607` of DEV's 1,226 openings ([claude-145 ORDER](../claude-145/ORDER.md), premise count). The last 1–4 tokens before the opening name only `161`, the floor of always guessing the commonest opening.
- **What the trunk can reach:** it is a selective quaternion recurrence that remembers about 8–12 tokens. Replacing the token 16 back moves its prediction by 1–2% of what replacing the last token does (`R16`).
- **The gap:** something has to bring tokens 15–190 back, and it has to be learnable, cheap, and free of softmax and dense matrix products.

## The evidence

### Ours

| What | Result | Source |
|---|---|---|
| A minimal retrieval head: key = code of the **previous** token, query = code of the current token, readout tied to the value embeddings | **100.0%** recall at 512 pairs (1,024 candidates). This held for 64-bit Hamming codes and for E8 block codes, **even with hard top-1 selection**. A single superposed binding state, with no look-back, stayed at 0.4–0.9% | [geometric-attention-2026-09-26](../../../../../archive/integration/geometric-attention-2026-09-26.md) §3 |
| The same note's byte language model (128-byte windows) | No attention layer beat a third recurrent layer there. The note's reading: this short-window task barely needs recall | the same note, §8 |
| Exact memory with the address written by a fixed rule | Held-out `0.82` and `0.72` (two seeds), against `0.0098` for recurrence alone | [exact-memory](../../../../mechanisms/exact-memory.md) |
| Starling read (600-cell, kept-7) | The source entered the kept set 73% of the time at peak, but recall stayed at chance | `docs/labs/attention-state-2026-10-18/README.md:19` |
| Pointer arms | 0.28–0.44, about what an untrained "copy the latest" rule gets (0.36) | [pointer-key](../../../../mechanisms/pointer-key.md) |
| Today's three trunk reads: the Flock read keyed on state (claude-144), the content read keyed on the position's own token (claude-145), and Hamming attention keyed on state (claude-146) | None moved openings. Hamming attention, after its scale fix, matched its twins on next-word accuracy but sat at the 161 floor through step 8,192 | claude-144, claude-145 REPORTs; this unit |
| Today's first recall probe, on the state-keyed form `9d4ca83b` (seed 2, 4,096 steps; the later induction-keyed form's results are in the [REPORT](REPORT.md)) | **Every arm at chance in every bucket, including the no-read arm on values 1–7 tokens back** (key distance 2–8; the value sits one after the key) | `caseyallard/uor-r4-store:results/claude/hamming-read-2026-10-27/` |

### Outside (each checked against the paper's page)

| Work | What it shows |
|---|---|
| Zoology, Arora et al. 2023, [2312.04927](https://arxiv.org/abs/2312.04927) | 82% of the quality gap between attention and attention-free models is in-context recall. A 70M attention model beats a 1.4B gated-convolution model at it. Input-dependent sparse attention closes 97.4% of the gap |
| Repeat After Me, Jelassi et al. 2024, [2402.01032](https://arxiv.org/abs/2402.01032) | Fixed-state models are provably limited at copying from context |
| Based, Arora et al. 2024, [2402.18668](https://arxiv.org/abs/2402.18668) | Recall trades off against state size. Linear attention plus a small sliding window recovers attention's quality |
| Induction heads, Olsson et al. 2022, [2209.11895](https://arxiv.org/abs/2209.11895) | In-context copying, [A][B] … [A] → [B], appears in a sudden phase change in training |
| Attention approximates SDM, Bricken & Pehlevan 2021, [2111.05498](https://arxiv.org/abs/2111.05498) | Attention's weight matches Kanerva memory's: the size of the intersection of two Hamming balls, which falls about exponentially with Hamming distance |
| Hamming Attention Distillation, Horton et al. 2025, [2502.01770](https://arxiv.org/abs/2502.01770) | Queries and keys binarised and compared by Hamming distance: 1.78% loss on GLUE |
| ReLU attention: Wortsman et al. 2023, [2309.08586](https://arxiv.org/abs/2309.08586); ReLA, Zhang et al. 2021, [2104.07012](https://arxiv.org/abs/2104.07012) | Non-exponential weights learn. A head can read nothing at all. Stability needs a normalisation or a gate |

## First-principles diagnosis

A learned look-back needs four things:
1. **Reach:** it can get at any earlier position.
2. **A match that can be found:** the training signal has to be able to discover the right position.
3. **A usable payload.**
4. **A bounded contribution.**

Requirement 2 is the one every failed read broke, and the evidence separates two ways to meet it:

- **(a) Credit to positions not picked.** Softmax gives every candidate a slice of the gradient. Hard top-K, all three of our reads, gives credit only to its picks. So a position that is never picked is never learned, and at the start, with keys built from the model's own state, the right position is no nearer than any other.
- **(b) A match that is already right at the start.** The 26 Sep head keys on the *previous token's* code and queries with the *current token's* code. With the two code maps equal at the start, "the position after the earlier copy of my token" is an exact match, Hamming distance 0, from the first step. Hard top-1 is enough, because nothing needs to be discovered. This is the induction circuit Olsson et al. describe, built into one head instead of grown in a phase change.

Our reads keyed on context-dependent states, or on the position's own token (claude-145, whose payload was that same position, not what followed it). None had (a) or (b).

The recall probe confirms it from the other side: trained with this objective at this length, the trunk learns no token-identity matching at all, not even inside its own reach.

## The construction this points to: an induction-keyed geometric read

Per head, after a trunk layer:
- **Key** at position `s`: the code of token `x_{s−1}`. That is the shared table's point, rotated per lane by a learned unit quaternion, with its sign bits as the code.
- **Query** at position `t`:
  - **copy heads:** the code of token `x_t`, through a rotation that starts equal to the key's;
  - **type heads:** the code of the residual at `t`, so the head can learn to ask, for example, "the word after the turn marker".
- **Match:** Hamming distance by `popcount(q XOR k)`. The query and key codes are per-vocabulary tables, 4,096 entries of 64 bits, so at serving a key costs nothing to build.
- **Choose:** the nearest earlier positions. An exact repeat is distance 0.
- **Weigh:** the bounded `max(0, m − d)` weights this unit already uses.
- **Payload:** the token at `s`, which is what followed the earlier copy, or its residual.

**What it should do by construction.** Without learning a selection:
- solve the recall task;
- copy names and numbers from the prompt, as the DEV replies need ("Okay, kandgind is the code word.", "Got it, your lucky number is 5246.").

**What it must learn:** the type heads, which are where openings come from. Their credit problem (a) remains. A wide candidate set and graded weights are the cheap mitigation.

**Cost at serving per token, per head:**
- one table read for the query;
- a window's worth of XOR + popcount;
- a gather of the chosen positions.

## Decisive tests, in order

1. **The recall probe with the copy heads.** The 26 Sep head reached 100% at 512 pairs.
   - If the far buckets reach 0.5 or more quickly, retrieval is solved, and so is training it without softmax.
   - If not, the fault is in the training path, not the mechanism: an instrument fault, to be found first.
2. **The language run against claude-145's twins:**
   - reply openings (type heads);
   - copied prompt tokens in the replies (copy heads);
   - the replies themselves, shown.

## Honest unknowns

- **How the 26 Sep head was trained.** Its scratch code is not archived, and its table reports recall "with softmax / with hard top-1". Training probably used softmax, before the 12 Oct ban. Here the copy heads need no learned selection, so a softmax-free training path is not at stake for them.
- **Copying alone does not fix openings.** The type heads must learn which prompt word matters. That is the part of the problem no read here has solved yet.
