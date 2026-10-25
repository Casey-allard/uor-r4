# ORDER: m3-nonrelation-prose-panel, claude-94

Lab: claude. Item: [m3-nonrelation-prose-panel](../../../../roadmap/items/m3-nonrelation-prose-panel.md). Base: `61d6cb0f3`.

## Goal

The item's done-when reads:

> A pre-registered non-relation prose panel with grading, plus the baseline false-assert and false-overwrite rates on world=v2. The pass bar for those rates is not yet registered; registering it is step 2 below.

The panel (`data/panels/nonrelation-prose-v1.txt`, `7c29c665…`), the grader and the owner's bar (false_overwrite 0/50, false_assert 0/50, clean 50/50; `#2028` comment `6356034555`) already exist. What is missing is a lawful compiler to measure. This unit builds one for the world=v2 schema and measures it on the GPU.

## Cancel probe (run first)

**Question:** does a lawful turn compiler for world=v2 already exist in the non-test build, so that only a run is missing?

**Result (25 Oct, at `730967519`):** no.
- The pinned world=v2 compiler (`compiler-save-op-v25-rawtable`) is `relation_compiler::SavedCompiler`. Its relation and act heads are `SparseSoftmax` fits (`relation_compiler.rs:497-623`, fitted at `738-747`), with a normalized exponential at lines 464 and 578, over a float trunk. CHARTER §3.2 bans softmax anywhere, including losses and training. The module is `#[cfg(test)]` in `lib.rs:190-192`, so no non-test binary can run it.
- Every other `TurnCompiler` is banned or not built: `temporal_compiler` and `geometric_turn_compiler` are `#[cfg(any())]` and fitted with sparse softmax or cross-entropy; the rest are test stubs.
- The grader (`nonrelation_prose_panel.rs`) and the session types it uses are `#[cfg(test)]` too (`lib.rs:179-181`, `206-208`).

**Class:** instrument. The measurement has no lawful object to measure. This says nothing about the exact-memory mechanism.

## The fix: a prime-sieve frame compiler

A lawful compiler for the world=v2 schema, built from world=v2's own training phrasings, with its decision computed on the GPU. It follows the owner's prime-router direction (data are primes, gcd sieves shared factors, semiprimes are the routing experts).

- **Words are primes.** Every distinct word of the world=v2 training phrasings gets its own prime, in sorted word order. A word set is the squarefree product of its primes. The kernel holds that product in its exact bitset form: a squarefree integer is the set of its prime factors, gcd is AND, and divisibility is subset.
- **Frames are exact memory.** Each training phrasing of each relation (assert, update, query) is one frame: the words before the `{v}` slot and the words after it. Its expert is the semiprime `p_relation · q_act`.
- **Compile = sieve, align, agree.**
  1. Sieve: keep the frames whose signature divides the turn's signature (gcd equals the frame).
  2. Align: the turn's words must be exactly prefix, then a non-empty value of at most four words, then suffix (or exactly the frame, for a query).
  3. Agree: the turn compiles only if every surviving frame names the same expert and the same value span. Otherwise it is `Unresolved`. This is the abstention gate, and it is an exact uniqueness test, not a probability.
- **No softmax, no float in the decision, no matmul.** Integer compare, AND and a bounded loop.
- **Where it runs.** Tokenizing turns and building the frame table is data preparation on the host. The sieve, alignment and agreement run in one CUDA kernel (nvcc ahead of time, its own PTX module). Grading the GPU's decisions is host grading of stored GPU output.

**Controls, measured in the same run.**
- The panel: the three registered counts, overall, per category and per split.
- Recall on true statements, so that a compiler that refuses everything cannot pass: training phrasings filled with development values (expected near complete), and development phrasings (unseen wordings, reported as they come, no bar).
- Finite CUDA controls with hand-built frames and literal expected answers, run on the pod first.

## Fixed before the run (decision review, 25 Oct)

- **Recall floor.** At least 95% of the training phrasings filled with development values must compile to exactly the expected act, relation and value span. The panel bar counts as met only together with this floor.
- **No panel leakage.** The frame table is built only from the world=v2 training phrasings. Its digest goes in the prep sheet before the panel runs. No edit to the frames, the tokenizer or the alignment rule may be informed by panel output. The panel text is not read while building.
- **Tokenizer, fixed now.** A word is a maximal run of alphanumeric characters and apostrophes (`'` and `’`), lowercased. Everything else separates words. Each word keeps its source byte span, and a value span runs from its first word's first byte to its last word's last byte. A value is 1 to 4 words.
- **Scope of the claim.** The result is a baseline for the sieve compiler on the world=v2 schema. It is never reported as closing or passing `m3-memory-turn-discriminator-gate`, whose own gate and bar stay open.
- **Notes.** The discriminator-gate item gets a note that its `SavedCompiler` references are superseded by this decision. The draft pre-registration gets a note that its v25 pins are replaced by the frame-table and PTX digests.
- **Host steps.** Tokenizing and building the frame table are data preparation, and grading reads the GPU's stored decisions. No compile decision runs on the host (CHARTER §3.5).

## Files in scope

- `crates/uor-r4-training/**`
- `data/**`
- `docs/labs/m3-nonrelation-prose-panel/**` (the prep sheet is `preregistration-sieve-2026-10-25.md` here)
- `docs/loop/tasks/m3-nonrelation-prose-panel/**`
- `docs/research/notebook/m3-durable-memory.md`
- `docs/roadmap/items/m3-nonrelation-prose-panel.md` and the rendered roadmap files
- The discriminator-gate note and this item's notebook scope land first on `main`, in claude-96 (#3191), because `loop guard` reads item scope from `origin/main`.

## Done when

1. The session's action types and the grader are in the non-test build, moved without change of behaviour.
2. `sieve_frame_compiler` is in the non-test build, with finite host tests for the frame table, tokenizer and decoding, and finite CUDA controls that pass on a pod.
3. The prep sheet `docs/labs/m3-nonrelation-prose-panel/preregistration-sieve-2026-10-25.md` registers the run, with its own fresh FIT and pre-run review posted on `#2856`.
4. The panel is graded on the GPU through the sieve compiler, and the record holds the three counts, both recall controls, tokens/s, the panel digest, the frame-table digest and the PTX digest. The outputs are saved in the HF store with a SHA256SUMS.
5. One row in `docs/research/notebook/m3-durable-memory.md`, the item set to `status: done`, and the REPORT says `Closes #2856`.

Nonzero false counts do not block done. The done-when asks for the baseline rates, not a pass, and a miss closes this configuration for `m3-memory-turn-discriminator-gate`.

## Lab decision (D68, logged on #2856)

The baseline is measured on the lawful sieve compiler for the world=v2 schema, not on the softmax-fitted saved compiler. The panel, grader and owner bar are unchanged. Casey can reverse this.

## Stop and decide (never stop and wait)

- If moving a module into the non-test build would bring banned code with it, move only the lawful types or data it needs.
- A failed finite CUDA control is an instrument fault: repair it, never count it.
- If the frames cannot be read without the test-only world modules, bring only their phrasing data into the non-test build.
