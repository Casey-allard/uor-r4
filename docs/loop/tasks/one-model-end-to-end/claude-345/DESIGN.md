# UOR-R4: the straight answer, one aligned design, and the plan

Lab: claude (Fable 5.2 session, takeover from the Opus 5.5 handoff). Date: 9 Nov 2026. Read base: `origin/main` at `6fcf57f78`.
Status: Casey accepted this design on 9 Nov ("Yes, start"). DeepSeek reviewed it the same day and set five conditions; all five are adopted in section 10, which governs wherever it differs from sections 4 to 9. A fresh-context review of this document on 10 Nov set further corrections; they are made in place and listed in section 10.11, with T0's result. Nothing here is a capability claim. Labels: **measurement** = a recorded run, **proof** = follows from code, **hypothesis** = untested.

---

## 1. Are we on the right track?

**The engine is on track. The model is not.**

- **On track (measurement):** text in, integer artifact out, served on a device, byte-identical on rerun, export gap 6 and −8 against a bar of 24, hundreds to thousands of tokens per second. The geometry is real and lawful. None of that needs redoing.
- **Off track (my reading, a hypothesis; the two numbered points under it are what was checked):** the thing inside the artifact will not become a chatbot by adding flags, data shares or question-specific heads. Two parts every language model needs are missing, and the last two weeks tuned around them:
  1. **It has almost nothing that learns to transform meaning.** Of 1,112,576 learned values, 1,048,576 are the token table and 256 the final norm. Each of the 6 layers has 8,448 routing values and 2,176 ordinary values (**proof**: 264 routed blocks × 4 stages × 8, from `StackConfig::shapes`; 1,048,576 + 256 + 6 × 10,624 = 1,112,576). A route picks one input block per output block and permutes it; no coefficient exists for any input-output pair (**proof**, mechanism page). In the one controlled comparison, trained routes lost to frozen random routes 8 of 8, and identity transport had the lowest loss in 4 of 4 (**measurement**, 26 Oct). A transformer layer of the same width has about 790,000 learned mixing weights.
  2. **Nothing it reads is chosen by what it has understood so far.** Every working read is keyed by token identity: the copy heads match the current token to an earlier copy; the ask heads use a per-token question table. The source says so itself: "Reads keyed on the trunk's own state never learned where to look" (`flock_read_hamming.rs`, header). The state barely remembers: swapping one token moves the output 5.97 at distance 0, 0.37 at 6, 0.04 at 12 (**measurement**).
- **Also off track:** data and scoring. 72.5% of scored rows are Wikipedia with no question; the most common first reply tokens are " It", " Your", " That", " You"; the panel wants bare values, which were 1.4% of targets. And progress was read from a slot-recall panel, not from replies. That is "testing for the tests".

**The rules are not the obstacle (hypothesis; T1 and T2 are its tests).** Both missing parts have forms in the project's own record that I read as lawful and geometric, parked before they were combined (section 4). So: keep the engine, keep the rules, change the model.

**What I cannot promise:** nobody has shown a model with no dense multiply and no normalized-exponential read at frontier level. The plan below finds out fast, in steps where each one either shows you replies or tells us exactly which part failed.

---

## 2. Goal

> A model you open on a laptop and hold a coherent, grounded, multi-turn conversation with: trained in float on the GPU, exported to one integer artifact, built only from the project's geometric mechanisms, at thousands of tokens per second.

Progress is judged by reading replies served through `uor-chat`. Every number is secondary to the transcript. Lead metric after that: tokens per second.

Milestones in order: (a) it reproduces replies it was trained on, complete and stopping; (b) it answers a fresh question about its own prompt; (c) it writes grammatical, on-topic sentences on simple fresh dialogue; (d) it holds several turns; (e) it scales.

---

## 3. What I checked, and what I did not

- **Read in full:** the handoff, `CHARTER.md`, `PLAN.md`, the pipeline map, the mechanism board, the archive catalog, the flock-read, E8 orbit memory and SpiralCore/FBS pages, the Flock read source, and the 8–9 Nov notebook rows.
- **Checked in code:** the trunk's parameter shapes; the trunk forward's order (recurrence, feed-forward, optional reads per layer); the Hamming read's key, query, choice, weights, payload and placement (depths 2 and 6); the content read's design; that `hidden` runs with no carry.
- **Confirmed handoff claims:** two model paths that do not join; 94% of values in the table; previous-token keys; no state-keyed read that works; the integer runtime serves the Hamming read; `uor-chat` runs text turns with EOS, cycle and cap stops.
- **Not checked:** the integer session internals line by line; the discrete M1 composition in code; most archive lab records (I read the mechanism pages that summarize them); the loss code itself; notebook rows 34–137 beyond a partial digest. Four scouting subagents each started at about 137,000 tokens of injected context and stopped after two or three reads, so I did the reading myself and narrowed it.
- **Pods:** the pod tool shows 0 running Runpod pods. It cannot list Vast pods from my worktree (its adapter is not built there). I am running no evaluation.

---

## 4. The one design

**One principle across every part:**

> Everything that is **learned** is a table addressed by geometry. Everything that **moves** is an exact rotation or route. Everything that is **chosen** is a hard nearest-neighbour choice with margin credit.

That is your 29 Oct line, made exact: **token → vector → vector → integer → token.**

| Step | What happens | Mechanism | Train (float, GPU) | Serve (integer, device) |
|---|---|---|---|---|
| text → token | cut text into pieces | byte-BPE, 4,096 pieces (pinned) | u16 ids in a UORT file with a response mask | u16 ids |
| token → vector | look up the token's point | one shared table, 4,096 × 256 (32 E8 blocks = 64 quaternion lanes) | f32 | i16, Q12 |
| vector → vector: **order** | carry the last few tokens | quaternion recurrence, 16 retention rates, 4-tap causal convolution, Hamilton gate, FBS routes. **Kept as is.** Also carries position (transport spin, no rotary code) | f32, state carried across windows | integer per-layer carry |
| vector → vector: **knowledge** | turn what is known into what follows | **E8 address memory (new).** Decode each 8-block of the state to its nearest E8 root (exact, 8 steps), read that address's row, add it | rows f32, exact gradient; address hard, credited through the wall rule (section 10.2) | rows i16; lookup and add |
| vector → vector: **attention** | fetch what matters from earlier | **state-keyed margin read (changed)**, at two depths, plus today's copy read at the readout | ±1 sign codes with the identity backward the copy read uses today (no squashing surrogate); weights max(0, m − d)/m over the 16 nearest | u64 codes, popcount, integer margin |
| vector → integer | name the nearest token | mean L1 to every table row, argmin | hinge against the 8 nearest wrong tokens, plus an EOS margin | integer L1, argmin |
| integer → token → text | emit and decode | tokenizer byte decode | | same |

**The two new parts, exactly.**

**A. E8 address memory: the lawful place for knowledge.** In a transformer, about two thirds of the weights are the feed-forward layers, and that is where facts and phrasing live. We cannot multiply by a dense grid, so we look the answer up.
- For each 8-block of the stream (32 at width 256): find its nearest E8 root by the exact 8-step rule (two largest magnitudes, or all signs with a parity fix; checked on 20,000 queries on 11 Oct). That root, one of 240, is the address.
- Read row `V[block][root]` (256 wide) and add it. The rows start at 0, so step 0 is today's model. There is no gate and no weight on the row: a weight that also started at 0 would leave both the weight and the row with no gradient.
- Size, rung 1: 32 × 240 × 256 = 1,966,080 values per layer, 32 lookups per token per layer. Rung 2: pair addresses (240 × 240 slots per head) for conjunctions. Rung 3: several shells scaled by powers of φ, the golden radial bridge already in the record, for coarse and fine cells.
- Learning: a row that was read gets its true gradient. The address is a hard choice, so the state that produced it gets its credit through the wall rule of section 10.2: a piecewise-linear secant across the wall between the nearest root's cell and the runner-up's. (The first draft said the address took no credit at all. DeepSeek's review showed that would train the rows and never the addressing; corrected 9 Nov.)
- Why believe it: the 11 Oct E8 orbit memory replaced half the feed-forward layers at a cost of +0.033 bits per byte after only 2,000 steps (**measurement**). It was parked for a sigmoid gate and a dense query map; this form has neither.

**B. State-keyed margin read: the lawful attention.** Flock's idea (look at a few neighbours, move toward their consensus) in the Hamming read's working arithmetic.
- **Key** of earlier position s: the sign code of the state there, taken *after* that layer's address memory, turned lane by lane by a learned unit quaternion. **Query** at t: the same, with its own quaternion. **Distance**: Hamming, by popcount.
- **Weight**: max(0, m − d)/m, margin m learned per head. Exactly 0 outside the margin.
- **Credit (the missing piece until now), bounded from the first step.** Training measures the distance from the query to every earlier key in the window, as the copy read on `main` does today and as serving does by popcount. That is a distance scan with the identity backward; it sums nothing. The payload is summed over the 16 nearest only, at every step: a weighted sum over the whole window would be a dense product written as a loop, which §3.3 forbids. (The first draft opened with a wide margin over every position. The fresh-context review of this document caught it; corrected 10 Nov.) Keys outside the 16 get credit from terms that sum no payload: the found-copy pull and the span pull, each a margin hinge on the distance to the position the data names against the nearest wrong positions; and an edge band at the 16th place, the read's twin of the address memory's wall rule (section 10.2), where the 17th-nearest takes a piecewise-linear share of credit while it is within a band of the 16th. No exponential anywhere.
- **Payload**: the state at s, turned by a value quaternion (transport into the reader's frame). The next layer's address memory transforms what arrived.
- **Combine**: added to the stream through the read's own weight row, which starts at 0, as the copy read's does today. That is one zero factor, not two: the rotations and the margin do not start at 0, so the row gets a gradient at step 0 and everything else from step 1, and step 0 is today's model.
- **Two depths** (after layers 2 and 4). The first read can find the question; the second, now holding the question, can find the value. If T2 shows it, that removes the need for the question carry and the ask table, which are hand-built single cases of this. Until T2 shows it, both stay (section 10.6).
- **Extra credit, where the data offers it:** the found-copy pull (the only signal that made B1 exact: 512 of 512 with it, 2 of 512 without) and a source-span pull on generated dialogues.
- Why the earlier state-keyed reads failed (**hypothesis**): they keyed on a state with no learned transform (256 rotation values), kept 7 by hard choice with credit only to the kept ones, at one depth, and added a token point after the last layer where nothing could use it.

**Decisions the design makes.**
- **Which path is the model (G10):** the float-trained FBS trunk. The discrete M1 path stays as the record of M1, the source of integer kernels, and a later legal-move repair step after export. One artifact family, `lut-stack`.
- **Position (G6):** the recurrence's transport spin. No rotary code. The type-spin frames stay available if reads need more.
- **Decode (G9):** greedy with the cycle stop, plus the EOS margin. If loops remain, a margin-ball choice (uniform among tokens within a fixed integer distance of the nearest), by decision review.
- **Context (G5):** train with carried state and a carried key ring, so training equals serving. 192 → 1,024 → 8,192. Later, the base-3 retention tree as the index for long context.
- **Durable memory (G8):** the same key function reading a persistent store. Not before the model talks.

---

## 5. Every gap: how a transformer closes it, and how we will

| Gap | Transformer | Ours today | This design | First evidence |
|---|---|---|---|---|
| G1 selection by meaning | soft read over all earlier states, stacked | reads keyed by token identity only | state-keyed margin read at 2 depths | B1 and B2 pass with no ask table and no carry |
| G2 training signal | cross-entropy over all tokens | hinge against 8 nearest wrong tokens (this change took top-1 from 18,402 to 216,675) | keep; add EOS margin and read pulls | memorize test |
| G3 capacity | hundreds of thousands of mixing weights per layer | 2,176 ordinary + 8,448 routing values per layer | E8 address memory, about 2M values per layer | held-out top-1 against the same run without it |
| G4 data | billions of tokens | 73M, 72.5% of scored rows with no question | simple dialogue and stories first; one reply grammar; binding dialogues with spans | transcripts |
| G5 context | 8k to 128k | 192, state reset each window | carried state and key ring | recall by distance bucket |
| G6 position | position codes | none beyond the recurrence | transport spin | binding at long distance |
| G7 prompt conditioning | attention to the prompt | opening tokens right 14–19% | follows from G1 and G3 | first-token rank |
| G8 exact memory | none (context only) | exact with a true address, 0 of 40 with its own | same read over a durable store | later |
| G9 generation | sampling | greedy; loops on 130–140 of 256 | EOS margin, cycle stop, margin-ball | EOS stops |
| G10 two paths | one model | two that do not join | float trunk is the model | decided |
| G11 tokenizer | role tokens | literal `User:` and `Assistant:` | atomic turn tokens at the corpus rebuild | — |
| G12 training cost | — | backward is 54–62% of a step | address memory is lookups; fix the width-512 memory defect | tokens per second |
| G13 evaluation | — | floors on one panel | transcripts first, through `uor-chat` | — |
| G14 export | one file | base copy, re-export, splice | one exporter; artifact binds tokenizer and protocol | — |
| G15 scale | known curves | unknown | doubling ladder | capacity curve |

---

## 6. Every mechanism, placed

- **In the model, unchanged:** tokenizer; shared table and one vector writer; SpiralCore/FBS routes; quaternion recurrence and retention rates; Hamilton gate; Hamming copy read (induction, one- and two-token keys); rank-margin hinge; integer LUT serving.
- **In the model, new or changed:** E8 address memory (from e8-orbit-memory, exact-memory, address-cells); state-keyed margin read (from flock-read, the starling record, and the Hamming read); EOS margin; carried context.
- **Training signals reused:** read binding, as a margin hinge (its softmax-era form cut wrong answers from 47 to 30 of 120); found-copy pull.
- **Held for a named later step:** base-3 retention tree (long context; hit rate 0.99, 0.68, 0.90 on 12 Oct); turn memory, point key and object store (durable memory); prime-zeta codebook and semiprime keys (exact identity channel for stored facts; table initialisation); type spin (position); golden shells (address memory rung 3); discrete legal-move learning (repair after export); kept-fibre Hopf map (a readout view, never tested on language).
- **Experimental, out of the base model:** ask table and question carry (they pass the synthetic tasks, did not move language, and cost stopping: EOS fell from 109 and 86 to 2 and 0); VSA binding; icosian spin weights (lawful only block by block).
- **Not used:** Hopf sector routing (parked by owner ruling); every softmax-era read, gate and dense map; neural donors, teachers and graders.

---

## 7. DeepSeek's audit, reconciled

I am not running the evaluation that note addresses, and I changed nothing it protects. I treat it as hypotheses.

1. **Supported by what I checked:** the code path exists (1, 2); claude-322 and claude-339 numbers (3, 4); the size (5), which I recomputed from the shape table; the format mismatch (6); the read's description (7); carry is not durable memory (8); the hinge objective (10); session restore refuses type heads (part of 11); the geometry is credible (12). **Dated:** the 1/410 output-side table gradient (9) is a 27 Oct reading under the clamp; unmeasured for the current recipe. **Not checked by me:** PRLL, tokenizer binding in the artifact.
2. **Where I go further:** the audit reads the problem as under-assembly. Assembly is needed and cheap, and I adopt all of it. But an integrated model built only from today's parts can at best pass the slot-recall panel, because the ask table, pull and carry are shaped to that panel. My reading (**hypothesis**) is that it would not converse: there is no learned transformation and no selection by state. The contract has two required slots that are empty today.
3. **Blockers in order:** learned transformation capacity; selection by state; data and reply grammar; EOS; context carry; the joined artifact.
4. **Cause-ladder pod work (claude-344, PR #3908):** hold and close. It is another sweep of one configuration.
5. **Contract `r4-chat-v1`:** *mandatory* = section 6's first two lists, one tokenizer, one protocol, one exporter, scored only through `uor-chat`. *Experimental* and *excluded* = as in section 6.
6. **Corpus format:** UORT ids and response mask as today, plus training-only side channels that never enter the artifact: a per-token span byte (question, answer value, reply end) and, per scored token, the position it copies from when one exists. One reply grammar for training and panel: a full sentence containing the value, ending with EOS; bare-value dialogues at a real share, not 1.4%.
7. **Training sequence:** one run from scratch on one mix, not a chain of fine-tunes (the chain is how stopping was lost). Each new part opens with exactly one zero factor, so step 0 is today's model and no value is left without a gradient: the address memory's rows are 0 and carry no gate (section 4A); a read's output weight row is 0 while its rotations and margin are not (section 4B). Checkpoint gates: held-out top-1 above the bigram table; EOS stops on at least 80% of replies; asked value in the first line far above the other value.
8. **Exporter and session:** the artifact carries the tokenizer's bytes or hash, the protocol and role tokens, the EOS id, the context length, every read and memory section; the session serializes its key rings and carries so restore works; `uor-chat` refuses a mismatched tokenizer.
9. **The one experiment (E1):** one model, two seeds, the contract above, scored only through `uor-chat --stack`: a fresh 256-row binding panel, 64 fixed multi-turn simple prompts whose transcripts go to Casey, tokens per second, export gap. **Scale if:** both seeds reach the existing bar (104 of 256 complete), held-out top-1 beats the bigram table by more than the two-seed band, and Casey judges the transcripts readable.

---

## 8. Plan

**Phase 1: can it learn, and do the two new parts work? (float side, minutes to an hour each, one RTX 5090.)** One pre-registration sheet covers all three, with one fresh-context FIT. These are go/no-go reads for two slots, never capability claims.
- **T0, memorize.** 1,000 short dialogues, today's recipe, trained until the loss stops. Pass: at least 90% of replies reproduced exactly, each ending with EOS. Needs little or no new code. *You read the replies.* A fail here means loss, readout, EOS or decode is broken, and that is fixed before anything else. Re-measures the output-side table gradient.
- **T1, capacity.** Same data and steps, with and without E8 address memory. Its cells, arms and pass and fail branches are in section 10.3, which governs: the first cell is a fit test on T0's own set with T0's readings, and held-out top-1 is the second cell.
- **T2, selection.** B1 and B2 with no ask table and no question carry: today's type heads against two state-keyed margin reads plus address memory. Pass: asked value at least 90% in the far bucket, other value at most 5%, both seeds; with the read cut, chance.

**Phase 2: E1, the integrated run** (section 7, item 9). Build the integer forms of A and B, one exporter, the corpus with spans. *You read transcripts from `uor-chat`.*

**Phase 3: scale.** Doubling ladder on width (after fixing the width-512 memory defect), depth, address slots, context and corpus. Each rung: transcripts, held-out top-1, tokens per second. A rung that moves neither seed beyond the band stops the ladder for a rethink.

**Phase 4: memory and long context.** Durable store through the same key function; retention tree.

**Phase 5: the milestone bars M2–M8 and device builds.**

**Kill criteria (what tells us the route is failing, not just unfinished).** A miss closes a configuration, never a mechanism; three misses on one slot go to Casey with evidence and a specific proposal.
- *Capacity slot:* the three rungs of section 4A (single-block, pair, φ-shells) all fail to move held-out top-1 beyond the band at equal steps. My reading then (still a **hypothesis**) would be that the no-dense-multiply rule is blocking learned transformation, and I bring that to you with the numbers and the narrowest change I can find.
- *Selection slot:* state-keyed reads fail T2's bar (asked value at least 90% in the far bucket, other value at most 5%, both seeds) under margin credit with the edge band, under the pull, and under span supervision. Then selection stays keyed by token identity (which passes the synthetic tasks), and I bring you that limit.
- *Whole route:* E1 plus one rung up (four times the values and data), on two counts together. The countable one: held-out top-1 does not rise beyond the two-seed band from E1 to the rung above. The judged one, which is Casey's: reading the 64 fixed transcripts at each rung, Casey finds no grammatical, on-topic replies.

**Roadmap items to add:** aligned design (this document; it lands with claude-345 in that unit's folder, and moves to `docs/model/` when the first of its two new parts merges); memorize test; address memory; state-keyed read; `r4-chat-v1` contract and exporter; corpus v2; capacity ladder.

---

## 9. Open with Casey

1. **Go on Phase 1 as designed?** It needs one or two GPUs inside the standing caps. *Answered 9 Nov: "Yes, start".*
2. **Training text for Phase 2.** The only text on which models this small are known to write coherent prose is simple-English story and dialogue text. Two such sets are already in your data repository (TinyStories, TinyDialogues); both are machine-written. My recommendation is to use them as plain token text. If you want human-written text only, I will build the mix from child-directed speech, film subtitles, spoken transcripts, children's books and Simple English Wikipedia instead. This only matters at Phase 2.
3. **Leftovers from the last session.** *Done 9 Nov, with one change:* draft PR #3908 is closed unmerged and its worktree is removed. Its ORDER and sheet stay at the PR's head on branch `claude/ladder`, as the record. The pin branch `claude/ladder-pin` is kept, because removing it would orphan the commit the sheet pins. `local/claude/claude-344/control` (about 1.8 GB) stays until its seal, file list and hashes are read back from the store.
4. **Subagents.** Each one this session starts with about 137,000 tokens of plugin and connector instructions, so fresh-context reviewers and scouts can do almost nothing. Turning off connectors this project does not use would make the charter's review step workable. That setting is yours.

---

## 10. Amendments after DeepSeek's review (9 Nov)

Casey relayed DeepSeek's review of this design on 9 Nov. It accepts the diagnosis as directionally right and sets five conditions. All five are adopted here. Where this section differs from sections 4 to 9, this section governs. Every item below is a **hypothesis** or a **design rule** until a run reads it.

### 10.1 The address memory's exact shapes (rung 1)

| What | Value |
|---|---|
| State at one position | 256 values, read as 32 blocks of 8 |
| Rows per layer | `V[32 blocks][240 roots][256]`, f32 in training |
| Values per layer | 32 × 240 × 256 = 1,966,080 |
| Gate or weight on the row | none; the rows start at 0, so step 0 is today's model |
| Six layers | 11,796,480 values |
| Model total | 1,112,576 today + 11,796,480 = 12,909,056 values |
| Where it sits | after each layer's ordinary values: the state becomes the state plus the sum of its 32 rows, then the usual rounding |
| Work per token per layer, serving | 32 decodes and 32 row adds of 256 values (8,192 additions); no multiplication |
| One decode | 8 absolute values, the three largest and two smallest magnitudes, one parity check, four compares |
| Artifact | one section per layer, 16-bit integers, shape [32, 240, 256], 3,932,160 bytes; 23,592,960 bytes for six layers |

DeepSeek asked for a model nearer 8 to 30 million values than 1.1 million. Rung 1 on six layers is 12.9 million.

### 10.2 Credit to the address (condition 1)

The first draft trained the rows and gave the address no credit. Then a state could never learn to sit in a better cell. The rule that replaces it:

- **Two addresses, both exact, no scoring of all 240 roots.** All roots have the same length, so nearest means largest inner product with the block `b`. With `m1 ≥ m2 ≥ … ≥ m8` the sorted magnitudes of `b` and `S` their sum:
  - integer roots: the best is `m1 + m2` (the two largest coordinates, with their signs); the next is `m1 + m3`;
  - half roots: the best is `S/2` when the count of negative coordinates is even, and `S/2 − m8` when it is odd (flip the smallest); the next is `S/2 − m8 − m7` in the even case and `S/2 − m7` in the odd case;
  - the nearest root `a` and the runner-up `a′` are the top two of those four candidates.
- **The gap** is `μ = ⟨b, a⟩ − ⟨b, a′⟩`, zero exactly on the wall between the two cells. Its scale-free form is `μ̂ = μ / ⟨b, a⟩`.
- **The wall weight** is `λ = ½ · max(0, 1 − μ̂/τ)`: one half on the wall, zero once the block is deeper than `τ` inside its cell. It is piecewise linear; there is no exponential.
- **Training forward:** `y = V[a] + (λ − stop(λ)) · (V[a′] − V[a])`, where `stop` holds a value fixed for the gradient. The value of `y` is exactly `V[a]`, which is what serving computes with one lookup. The gradient reaches the state only through `λ`, only inside the band `τ`, and it points toward the side of the wall whose row lowers the loss.
- **If the surrogate moves no addresses,** the second form uses the blend as the real forward, `y = (1 − λ)·V[a] + λ·V[a′]`, with `τ` tightening to 0 by the end of training, so the last phase equals serving. This is the same discipline as the read's margin.
- **No dense multiply.** `⟨b, a⟩` uses the one selected root: eight products and a sum per block. The decode is absolute values, a partial order and compares.
- **Measured in every address-memory run:** roots in use per block (of 240) and how evenly; address mobility (the share of probe positions whose address changed since the last checkpoint); address stability at the end; the share of blocks inside the band.
- **Test before any run:** on a fixed batch, the gradient that reaches the selected rows is non-zero, the gradient that reaches the address-producing state is non-zero, and with `τ = 0` the second is exactly zero.

### 10.3 Controls for T1 (condition 5)

The 11 Oct orbit memory stayed within +0.033 bits per byte of the layers it replaced, and a learned-key twin beat it by 0.023. So E8 addressing is a hypothesis, not a recovered answer. T1 runs these arms at matched steps and seeds. The first cell is a fit test on T0's own set, read with T0's three readings and bars; T0 is its no-memory arm, and nothing in it is held out (the DEV count is recorded only). The second cell, run only if the first shows a gain, is the same arms scored on held-out text.

| Arm | Address | What it separates |
|---|---|---|
| no memory | none | today's model (T0) |
| E8 | nearest root of each block | the proposal |
| collapse | one fixed row per block | a learned bias, with no addressing |
| token | a seeded hash of the position's own token, 240 rows per block | the same table size, with a consistent address that is not the state |
| sign code | the block's 8 signs, 256 rows | a simpler lawful address of nearly the same size |

The `token` arm replaces the first draft's `random` arm (10 Nov, with claude-346's build). An address drawn afresh at each position is noise no row can learn from: every row would drift to one average, which the collapse arm already measures.

T1's sheet fixes the margins. The branches it must carry, each read at the lower seed:

- **E8 does not beat no memory.** The first miss on the capacity slot. The next cell (more steps, or the pair rung) is chosen from the address census, not guessed.
- **E8 beats no memory but not collapse or token.** The gain is a bias or table size, not the state's address. E8 addressing does not go forward on that evidence.
- **E8 beats all three and still misses T0's bars.** The address carries signal and the rung or the steps are short. The next cell is chosen from the census.
- **E8 meets T0's bars.** The held-out cell, then T2.

If the sign code matches E8, the simpler address is the one that goes forward.

### 10.4 The same support in training and serving (condition 2)

The state-keyed read sums payloads over exactly the set the server uses, at every training step and not only in a last phase: the 16 nearest earlier positions by popcount, ties broken by the server's rule, with the weights `max(0, m − d)/m` inside that set. There is no wide-margin phase (section 4B). A test on a fixed probe compares the trainer's selected set with the integer server's; they must be identical at every probed position.

### 10.5 The copy read stays (condition 3)

Today's token-keyed Hamming copy read stays at the readout, as sections 4 and 6 already say. The state-keyed read is added beside it and replaces nothing until it wins a controlled comparison.

### 10.6 Ask heads and question carry stay until T2 decides (condition 4)

They stay in the code and in the artifact format. T2 runs three arms on the same data, seeds, steps and scorer:

- (a) the copy read alone;
- (b) the copy read plus the state-keyed read (and the address memory, if T1 passed);
- (c) today's full model with the ask heads, the pull and the carry, as the reference.

The deciding comparison is (b) against (a). On B1 and B2, arm (c) is already at its ceiling (512 of 512 with the pull), so adding a read to it cannot show a gain. The ask heads and the carry leave the E1 contract only if (b) reaches (c)'s level and keeps stopping; otherwise they stay. T2 also reports stopping, as DeepSeek asks: a read that finds the value and loses the end marker has not passed.

### 10.7 What the missing capacity is

DeepSeek's wording is the better one and replaces mine wherever they differ: the model needs an additive learned table read, such as the address memory. Nothing in this design asks for dense multiplication. Section 8's capacity kill criterion is the only place the rule is questioned, and only with three recorded misses first.

### 10.8 Data

Taken as the default unless Casey says otherwise:

- **T0:** the current controlled dialogues, with no new data. Done in claude-345.
- **T1 and T2:** fixed narrow sets. T1's first cell is T0's own set.
- **E1:** a hybrid: simple story and dialogue text (TinyStories, TinyDialogues, labelled synthetic) with D3 and OASST2, and a human-written held-out set. The simple sets are a development scaffold, not the final corpus.

### 10.9 T0 as it is run

The sheet fixes what section 8 left loose: 4,096 steps (about 231 passes over the set's 1,133 trained windows), the 192-token window, two seeds, and three bars: exact replies, token fit and reply fit under teacher forcing. Each bar is read at its lower seed. The output-side table gradient is not re-measured, because the gradient-norm mode does not take this recipe's width and read flags. A T0 miss is classified as an instrument defect or a model defect before any memory is added; the sheet's rows 0, 2 and 3 are that classification.

### 10.10 What E1 alone may claim

T0, T1 and T2 are reads of single slots. Only E1 may claim end-to-end capability: two seeds, one frozen artifact format, prose in and prose out only through `uor-chat --stack`, with complete replies, the end marker, fresh transfer and tokens per second reported together.

### 10.11 T0's result, and the corrections from the review of this document (10 Nov)

**T0 (measurement; [REPORT](REPORT.md)).** Row 3 of the sheet at both seeds: 0 of 1,072 replies reproduced, token fit 0.602 and 0.607 under teacher forcing against a bar of 0.95, and the first reply token right 36 times in 1,117. At each seed every reply opens with the same words. Every instrument check passed, so it is a model result for this recipe at 4,096 steps on this set, and nothing wider: the loss was still falling. It is the row the sheet said would send the work to the address memory on the same set.

**Corrections.** A fresh-context reviewer read this document against the charter and the repository at `1fb08d4e5`. What changed:

- **Sections 4B and 10.4:** the state-keyed read never sums payloads over the whole window. The first draft's wide-margin phase would have been a dense product written as a loop (§3.3).
- **Sections 4B and 7 (item 7):** how each new part opens, with one zero factor. The first draft said it two ways.
- **Section 1:** the final norm's 256 values were missing from the breakdown. The total was right.
- **Section 8:** the capacity kill criterion names the rungs section 4A defines; the selection criterion names T2's bar; the whole-route criterion says which half is a count and which is Casey's judgement.
- **Section 10.3:** T1's two cells and its fail branches; the `token` arm in place of `random`.
- **Sections 1, 7 and 8:** four statements now carry the **hypothesis** label they should have had.
- **Section 10.9:** each bar is read at its lower seed.

Not verified by that review, and still resting on the earlier session's reading: the per-layer split (8,448 routing and 2,176 ordinary values), the 1.4% bare-value share, and five figures the reviewer did not reach: the EOS counts, the read-binding counts and the retention-tree hit rates in section 6, the learned-key twin's 0.023 in section 10.3, and the gradient-norm limit in section 10.9.
