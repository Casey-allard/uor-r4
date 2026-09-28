# D2: architectural exact relational memory — pre-registered plan

September 28, 2026. References #973 and #962 under #820.
- **Lab:** Lab 1 (Claude).
- **Status:** frozen before any fit. The code, gates and resource limits below are committed before the runs launch.
- **Result (attempt 2, 09:00 UTC):** the frozen gate **FAILS on the margin**. The store is perfect in distribution, and the learned read does not generalise to held-out phrasing. See [the result](d2-aerm-probe-result-2026-09-28.md).
- **Track:** [whole-project synthesis](whole-project-synthesis-2026-09-28.md), §3 M1 and §4 D2 (owner-approved).

## Question

Does architectural exact memory give a small recurrent model updated relations?

The comparison is the recurrence-primary stack with an exact relation store against the same stack with an equal-size MLP and no store. Both train on identical data.

## What is built

The code is `crates/uor-r4-training/src/stack_aerm.rs` and `examples/aerm-probe.rs`.

**Host.** The B1 host shape:
- the `rrar` stack: width 128, 4 heads, MLP 384, context 256, vocabulary 4,096 (#1017), Dot read, rotation;
- 1,393,732 stack parameters.

**Heads.** The stack is split after layer 2. Two learned heads read the RMS-normalised residual stream there:
- a **role tag** per token: other, entity, relation or value;
- a **trigger** per token: none, write, read current or read previous.

The layers below the split never see the store, so tags and triggers are causal and can be computed in one pass.

**The store** (integer, exact):
- A trigger closes the open clause: the latest tagged entity, relation and value since the previous trigger or EOS.
- **Writes** store `(entity, relation) → value`. A hit overwrites, keeps the previous distinct value and increments the version. **Version order decides "current"; no score does.**
- **Reads** set a register `(status, value)`, with status None, Hit, Absent or Evicted. The register persists until the next read.
- Records keep their exact key, and there are 64 slots with two-choice placement by xor-shift mixes. A collision can evict, never alias.

**Re-entry.** The register re-enters the residual stream through two zero-initialised paths (the branch starts silent; a unit test checks the logits equal the plain stack's):
- a status embedding plus a value projection;
- a copy boost on the value token's logit.

**Training and evaluation:**
- **Training** drives the store from the gold tags and triggers (teacher forcing). The heads train by auxiliary cross-entropy; no gradient passes through the store.
- **Evaluation** drives the store from **the model's own tags and triggers only**.

**Control.** The same stack, heads and auxiliary losses, with the MLP widened by 11 units. That gives 1,411,660 parameters against the memory arm's 1,411,790, and no store.

### Deviations from the synthesis's M1 text

These were decided before any fit:

1. **Pointers.** Per-token role tags, plus "the latest tagged token since the last trigger or EOS", replace pointer heads over the last W tokens.
2. **Gate training.** The gates train by gold supervision, not KVAR's soft surrogate. The code survey found that no annealed-to-hard recipe has ever worked in this project.
3. **Store addressing.** A power-of-two table with xor-shift placement. A prime modulus would be a division when served.
4. **Copy path.** A zero-initialised logit boost, not a mixture distribution.
5. **Slots.** Single-token slots only: names, relation words and values are single tokens with a leading space. No city name is a single token in #1017.

## Data

**Synthetic relation dialogues** in the shared literal-role protocol (`uor-r4.literal-role-dialogue/1`, identity `blake3:0099a613…`, the dialogue child's).
- Every run first verifies 256 generated episodes token for token, and mask for mask, against the protocol encoder.
- **Cast:** 2–3 people and 2–3 relations per episode, from pet, color, toy, job, place and friend. Friend values are names, so it is a role test.
- **Events:**
  - 2–4 assertions;
  - 2–5 middle events: at least one update, plus reassertions and chit-chat;
  - 2–3 final queries.
- **Chit-chat includes hard negatives** that mention relation or value words ("I saw a cat in the park.").
- **Query classes:**
  - **First**: asserted once;
  - **Updated**: last event changed it. This is the **gated class**;
  - **Reasserted**: last event restated the same value;
  - **Previous**: the previous distinct value;
  - **PreviousAbsent** and **Absent**: the reply abstains with "I do not know.".
- **Recency traps.** Updated queries are also flagged when another person's value of the same relation was written later.
- **Text.** Beside the dialogues, #1017 TinyStories text: the first 16,777,216 training tokens; development is the first 249,000 tokens of `dev.u16`.

## Fixed conditions

- **Seeds** 1, 2 and 3. Each seed trains both arms on identical batches (data seed 5,000 + seed).
- **Schedule:**
  - 1,500 updates;
  - each update is 16 text windows plus 16 dialogues;
  - learning rate 0.003, 100 warm-up updates, then cosine decay to 10%;
  - AdamW with betas (0.9, 0.95) and weight decay 0.1;
  - global gradient clip 1.0.
- **Loss** (all weights 1):
  - text cross-entropy;
  - response cross-entropy (assistant content and EOS);
  - tag and trigger cross-entropy on both batches. Stories are all "other"/"none", and a trigger-positive position counts 5×.
- **Evaluation seeds:** dialogues 9,001, held-out templates 9,002, free-running 9,003, protocol check 9,004.
- **Evaluation sizes:**
  - 512 fresh dialogues with training templates and names;
  - 512 dialogues with held-out templates and names;
  - 512 development text windows;
  - 32 free-running final answers.

## Gates and decisions

The synthesis's gate applies to **every seed**:

| Gate | Threshold |
|---|---|
| Memory arm, **Updated** value-slot accuracy (teacher-forced argmax at the value token) on the 512 fresh dialogues | **≥ 0.90** |
| Memory arm minus control, same metric | **≥ 0.30** |
| Development text NLL, memory arm minus control | **≤ 0.05** |

`aerm-probe summarize` applies these gates mechanically to the sealed roots.

**Reported, not gated:**
- the other classes (abstaining classes score the whole answer);
- recency-trap Updated queries;
- held-out-template and held-out-name accuracy;
- oracle-register accuracy, meaning the memory arm with a perfect parser;
- tag accuracy and trigger confusion;
- trigger firings on story text;
- free-running exact answers;
- **the four-class trace of every memory-arm failure**: unavailable, not selected, wrong value, emission.

**Decisions:**
- **PASS in every seed.** AERM enters D5 as M1. The next unit covers natural paraphrase and the milestone panel's format.
- **Accuracy below 0.90.** The trace decides the reading:
  - mostly unavailable or not selected: the language-to-address mapping is the bottleneck at 1.4M;
  - mostly emission: the read works but goes unused.

  AERM is parked at this scale either way (synthesis §4).
- **Margin below 0.30, accuracy at or above 0.90.** In-window dense reads with recency suffice for this population, and AERM's in-window advantage is not shown. Its remaining case is beyond-window memory, which needs state carry. Recorded, not extended.
- **Text gate fails.** The injection costs language; recorded.
- **Every failure is preserved**, with no automatic dose, seed or scale extension (D9).

## Checks already run

- `cargo fmt --check` passes.
- The `stack_aerm` unit tests pass (5):
  - store overwrite, versions and previous value;
  - eviction reported, never aliased;
  - episodes' tags, protocol reconstruction and gold answers, over 300 episodes;
  - the branch starts silent and the split equals the plain stack;
  - a training step reaches every memory parameter.
- The `geometric_stack` tests pass (12).
- **A 20-update smoke run (sealed, not a result):**
  - the protocol check is identical on 256 episodes;
  - both arms train, evaluate and seal;
  - 1.1 s/update for the memory arm and 0.85 s/update for the control, at 2 threads.

## Resources

- **Processes:** three, one per seed, each at 2 threads. The model slot is claimed (6 threads).
- **Projected wall time:** 75–90 minutes for both arms of every seed.
- **Hard caps:** 3 GiB RSS per process and 150 minutes of wall time.
- **Storage:** SSD roots under `/Volumes/UOR-Workspace/uor-r4-lab/claude-d2-aerm/`, projected below 10 MB. No internal-drive writes beyond the shared build cache.

## Attempt 1 (`d2-1`, 07:10 UTC): void, instrument failure

**What happened.** All three seeds stopped after about 3.5 minutes (roughly 3,000 generated episodes) with the generator's own consistency check: "query class disagrees with the gold store". No arm finished, and nothing was evaluated.

**Cause.** The store's second placement mix left its low bits nearly independent of small token ids. Every key's second choice fell in almost the same slot, so two-choice placement degenerated into one choice. An episode's few facts could then evict each other, and the gold answer became Evicted while the generator's bookkeeping still held the fact.

**Fix:**
- one xor-shift mixer (three rounds, no multiply) that spreads both ids over the word, with the two choices drawn from different halves;
- an 8-entry overflow stash, so a record is evicted only when its two slots and the stash are all taken.

**New tests:**
- the placement spreads keys, and fewer than 1 in 16 keys repeat a choice;
- 30,000 episodes produce no eviction. This test failed before the fix and passes after it.

**What did not change.** The semantics are unchanged: exact keys, version order, typed statuses. So are the generator's random stream and the intended data. The gates, seeds and conditions above are unchanged.

**Other changes:**
- The branch merged `main` (#1437, #1454). `StackConfig` gained `memory: None` in D2's literals, which does not change behaviour.
- The zsh launcher failed to record PIDs, released the slot for about a minute and sealed a premature summary root; this is disclosed on #973. The retry uses a bash launcher.
- Attempt 1's roots (`d2-1-s1`–`s3`, `d2-1-summary`, `d2-1-summary-2`) are preserved and sealed. Their "FAIL" summary reflects incomplete runs, not a model result.

**Observed peak RSS:** 1.6–2.0 GB per process, within the 3 GiB cap.

**Retry:** attempt 2 (`d2-2-*`), with fresh roots.
