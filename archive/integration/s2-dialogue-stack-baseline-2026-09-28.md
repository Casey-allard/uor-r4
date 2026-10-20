# S2: the first dialogue-trained stack — result

September 28, 2026. References #973 under #820.
- **Lab:** Lab 1 (Claude main).
- **Pre-registration:** [#973](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5875334772), posted before the run.
- **Evidence:** [`s2-dialogue-stack-baseline-2026-09-28.json`](../evidence/s2-dialogue-stack-baseline-2026-09-28.json), assembled by script from the four sealed roots.

**Status.** The run is complete. All four phases exited 0 with no hard cap hit: 6,323 s wall, at 4 threads (the integer-reply phase at 2), peak sampled RSS 1.72 GB (1,676,384 KiB).

**The pre-registered primary metric passes decision rule 1.** On the identical 161-response panel, the stack's pooled development response NLL is **2.520916**. The pinned native full-prefix child scores 2.773887, and its parent 3.022131.

**The complete replies are poor.** Only 2 of the 38 requests' final turns answer the request, and none of the 10 memory recalls succeeds. This is development evidence, not qualification: a loss win is not conversation.

## What ran

**Core:** `rrarra`, Lorentz reads, quaternion transport, width 288, MLP 749: 7,153,860 parameters. Executable `52a20eeb…` (the Rust on `main` `ccd19cb6`).

**Data:** chat-v0, #1017 tokenizer `d36d3e87…`. The tokenizer is recorded in the language-model, dialogue and chat reports; the export report does not record it.

**Phases:**
- **A, language modeling:** 2,237 updates of 16 × 256 tokens, with windows drawn from the whole chat-v0 train split: 9,162,752 positions, 11.1% of its 82,529,690 tokens, so not a full pass. Heldout NLL is 2.535171 on 512 windows. Model `84f46ac9…`; 4,526 s at 2,054 tok/s.
- **B, responses:** 1,024 full-prefix updates, 1,165,549 supervised target visits. It reuses the study's seeds, and its visit count equals the study's full-prefix arm. There is no schedule hash to compare, and its tensor positions (4,063,504) differ from the study's (4,194,304). Model **`8cb11d8f…`**; 1,777 s.
- **C, integer export and replies:** GPTQ export `f14d6c13…`, then D10 `lut-chat` replies to the 38 requests.

## Development result on the study's frozen panel

The panel is 161 responses and 13,353 targets, with the same ordered response ids as the study.

| Source | Native parent | Native full-prefix child `98aca5ab…` | Stack after A | **Stack after B (S2)** |
|---|---:|---:|---:|---:|
| Magpie (23) | 3.317055 | 3.260219 | 2.936853 | **2.939346** |
| Constraints (32) | 3.206600 | 2.709090 | 2.437294 | **1.899336** |
| Rewrite (10) | 1.962991 | 1.939291 | 1.945239 | **2.075107** |
| Summarize (32) | 2.052222 | 1.623803 | 2.122431 | **1.928926** |
| UltraChat (32) | 3.125430 | 3.023602 | 3.165046 | **3.227190** |
| Everyday (32) | 3.126874 | 2.791712 | 2.746506 | **2.358719** |
| **Pooled** | **3.022131** | **2.773887** | **2.698077** | **2.520916** |
| First-four, pooled | 2.302482 | 1.852403 | 2.458803 | 2.031832 |

The native columns are copied from the [study's record](dialogue-prefix-paired-result-2026-09-27.md). The stack columns are read from the roots.

**Reading:**
- Against the native child, the target-weighted shares of the −0.2530 pooled gain are Constraints −0.231, Magpie −0.079 and Everyday −0.031. The other three sources move the other way.
- The stack is **worse than the native child on Rewrite, Summarize and UltraChat**, and on the first-four targets.
- The pooled mean weights the sources by target count, so it is not an equal-source mean.
- B's curve falls monotonically, from 2.7029 at step 128 to 2.5209 at step 1,024, and is still falling at the end.

## Complete replies

These are 58 greedy turns with a 32-token cap, the study's stops, and exact generated history. The director read them as text. A reply counts only if it answers its own turn.

| Category | Requests | Final turns answered |
|---|---:|---:|
| Greeting | 5 | 1 ("Hi there!" → "Hello! How can I help you today?") |
| Factual | 8 | 1 ("The capital of France is Paris.") |
| Instruction | 7 | 0 |
| Memory: 3-turn recall | 10 | 0 |
| Ambiguity / refusal | 8 | 0 |

**Typical failures:**
- generic filler ("I'm so glad I could help you with that. I'm here to help with your dog, and I'm here to help with your dog.");
- mismatched stock replies ("Good morning!" → "You're welcome! …");
- "My answer is maybe." for every memory turn about Momo.

**No memory recall succeeds.** The stated fact is always within the 256-token context, so every memory failure is *available but not selected*, or a failed emission. The two cannot be separated without read diagnostics.

By the study's record, the native child recalled Alex and green, and partly Tokyo. **On memory, S2's complete replies are worse than the native child's.**

**Integer replies** (D10, GPTQ export): identical to float on **14 of 58 turns**. The 4-bit representation flips greedy decisions, and each flip derails what follows. S1.0 measured the size of this gap on the code model `geometric_s1` (+0.0257 nats with GPTQ); it was not measured on S2. QAT (#1466) targets it.

**D11 scoring** (pre-registered, run after S1.1 merged; sealed root `d11-eval-1`, executable `bc08ece3…`): on 16 evenly spaced heldout windows, the D11 engine's logits equal D10's at every position (maximum difference 0). The NLL is 2.980479 for both. This is all tokens, not only responses. D11 ran at 75.6 tok/s on one thread and D10 at 233.2 tok/s on 2 NEON threads, under machine load.

## Decision (pre-registered rule 1) and what it changes

1. **S2 `8cb11d8f…` becomes the pinned dialogue baseline for the main line**, for loss-level comparisons on the chat-v0 panel. The native child `98aca5ab…` stays the **reference for complete replies and memory recall**, where it is better.
2. **Memory is the named gap.** The stack has no exact store in its path, and it recalls nothing. The pre-registered four-class trace is **not reported**: it needs read diagnostics, meaning the read mass on the stated fact, which this run did not capture. The next learned-behavior step is:
   - I1's product store (six views; its PR is pending) wired into the stack;
   - G's learned address (Lab 2) behind the read interface;
   - measured on this panel's 10 memory trajectories and on D2's relation world.
3. **Integer replies need a QAT phase:** 14 of 58 turns match float. The QAT validation, and then a QAT phase on S2, are pre-registered before use.
4. **Response quality** (instructions, factual answers) is not addressed by more of the same objective at this scale without new causal evidence (D9). The 10–30M milestone shape (§2b) is the lever after the store integration.

## Scope and limits

- One seed.
- One development panel, exposed during development, so there is no held-out claim.
- The director read the replies alone; there was no second reader.
- The native comparisons are the study's recorded numbers, not a re-run.
- The export and chat phases were too short for the 10 s RSS sampler.

**Cost:** 6,323 s of wall time on the model slot, plus 73 s for the D11 scoring. It is charged to the ledger (`charge-2026-09-28-lab1-s2.json`), which now stands at 756.7M of 780M ms. **Storage:** 141.6 MiB (148.5 MB) of new SSD space, about 84 MiB of it smoke-test roots.
