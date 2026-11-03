# ORDER: one-model-end-to-end, claude-267 (the slot key for the Hamming read's copy heads: measured on the data, then built in training, export and serving)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `9e0403b132da34752499c657def95dfce3235538` (main with claude-266 merged). This ORDER is hand-written under the umbrella exception (cancel probe 2).

## Goal

claude-266's Bar (part B, row 4 (iii)) keeps (b), copy key 1, as the base and names the next copy-key unit: "the slot key (entity × attribute atoms), since an ordered two-token key also matches a shared two-token frame".

This unit defines that key, measures it on the data before any training, and, if the measurement allows, builds it through the trained read, the export and the integer serve. The GPU run is the next unit (claude-268), with its own sheet and FIT.

**Plan step (the claim check):** order step 4, one model trained as served end to end. **What it changes in the model:** the key the copy heads of the Hamming read match on, in training and in serving alike.

## The slot key (my design, reversible)

The two-token key (claude-261) keys a copy head on `x_{s−1}` and `x_{s−2}`. It still matches any shared two-token frame: "my friend's name is" and "my dog's name is". The slot key keeps `x_{s−1}` and replaces the second token with the clause's **atom**: its rarest token so far.

- **Rank table.** Each vocabulary token gets a rank from its TRAIN count: rarer is lower, ties broken by token id. The table is one `u16` per token. Rank 0 marks a **boundary token**: one whose text holds `.`, `!`, `?` or a newline, or a special or role token, read from the pinned tokenizer.
  - The trainer computes the table from TRAIN and the tokenizer at start, and saves it with the read.
  - The export writes it into the artifact.
  - The integer serve reads it from the artifact, so the serve needs no tokenizer and no TRAIN.
- **The atom.** `A(i)` is the token of lowest nonzero rank among the clause's tokens up to and including `i`. A clause starts after the last boundary token. At a boundary token, `A(i)` is that token. This is one causal scan with a reset, in integers.
- **Key and query.**
  - A copy head's key at `s` is `[x_{s−1} | A(s−1)]`, half and half, through the same rotation as claude-261's pairs (`pair_halves`).
  - Its query at `t` is `[x_t | A(t)]`.
  - The distance is one popcount. It is the sum of the two half-distances, so an exact match needs the same last token and the same clause atom.
- **Unchanged:** type heads, weights, margins, the payload and the tie rule. Copy key 1 and copy key 2 artifacts stay byte-identical.

In "My best friend is Maya.", the key at "Maya" is `[" is" | rarest of {My, best, friend, is}]`. In the reply "Your best friend is", the query at " is" is `[" is" | rarest of {Your, best, friend, is}]`. "friend's name is" and "dog's name is" now differ in their atom.

## Step 1: the key measured on the data (decides whether step 2 runs)

The rule is an exact-match count, with no training. Over every reply position `t` of DEV, and of p-w, whose next token `x_{t+1}` occurs earlier in its window (a copyable position):
1. Take the candidates `1 ≤ s < t` (the read's own range) whose key under the rule equals the query at `t` exactly.
2. Count a position **right** when the latest such candidate's token `x_s` is `x_{t+1}`.
3. Report, for copy key 1, copy key 2 and the slot key:
   - the copyable positions;
   - the positions with an exact candidate;
   - right;
   - right among the positions with an exact candidate (purity).

This is a key-quality reading of the data, not a model result. The trained read keeps 16 graded candidates, not one exact match.

**Rule:**
- Step 2 runs if the slot key is right on more positions than copy key 2, on DEV and on p-w.
- Otherwise the REPORT records the negative for this atom rule only (a configuration, CHARTER §4.1). The next unit tries the next rule: the two rarest atoms, unordered.

## Step 2: the slot key built (only if step 1 allows)

- **Training:** `--hamming-copy-key slot` in `train-in-space`, through `flock_read_hamming.rs`, with the table saved in `hamming_read.safetensors`.
- **Export and format:** the table goes into the artifact's Hamming record. Copy keys 1 and 2 keep their bytes.
- **The integer read** (`stack/hamming.rs`): the atom scan and the paired query.
- **Tests:**
  - (a) the float and integer atom scans give the same `A(i)` on a synthetic sequence with boundaries;
  - (b) a slot-key read served exactly as trained, as claude-261's copy-key-2 tests do;
  - (c) every existing copy-key-1 and copy-key-2 test still passes, bytes included.

## Cancel probe

1. **Already done?** `git grep -i -e 'slot key' -e slot_key -e SlotKey -- crates/uor-r4-float-train crates/uor-r4-integer` finds only `tests/dialogue_state_m1.rs`, a memory-capacity test with no Hamming read. The lawful pointer's slot-address key (`crates/uor-r4-training/src/lawful_pointer.rs`) is another head, on registered primes, outside the trunk. Not cancelled.
2. **The lock.** `uor-harness loop claim one-model-end-to-end --lab-topic slot-key` refused at about 21:05Z: the item file overlaps `docs/roadmap/items/*.md` of item infra-loop-harness, PR #3622 (deepseek-104, draft). That PR's files are `docs/harness/claims.json`, its own task folder, `docs/roadmap/issues.json` and `docs/roadmap/items/geometric-image-input.md`. None is in this ORDER's scope, so the umbrella exception applies.
3. **Inputs.** TRAIN, DEV and the tokenizer are on the laptop at claude-261's pins (`local/claude/data/w2-inputs/2026-10-25/driver-data/`). p-w is in the store at `inputs/claude/heldout-panel-p-w1200-2026-10-31`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-267/**`
- `docs/research/notebook/float-training.md` (added rows only)
- `docs/roadmap/items/one-model-end-to-end.md`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`
- `crates/uor-r4-float-train/src/bin/train-in-space.rs` (the flag, and the step-1 count as a mode that reads DEV or a panel and exits)
- `crates/uor-r4-float-train/src/in_space_long.rs`
- `crates/uor-r4-float-train/src/fbs_export.rs`
- `crates/uor-r4-integer/src/fbs_snapshot.rs`
- `crates/uor-r4-integer/src/stack/format.rs`
- `crates/uor-r4-integer/src/stack/hamming.rs`
- `crates/uor-r4-integer/src/stack/synthetic.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`

## Done when

1. Step 1's counts for the three keys on DEV and p-w are in the REPORT, read by the rule above.
2. Step 1 decides:
   - if it allows, step 2 is built, its tests (a) to (c) pass, and the touched crates' full suites pass with `--no-fail-fast`;
   - if it does not, no code beyond step 1's count is merged.
3. `uor-harness loop gates --full`, `loop report --check` and `loop pr-check --local` pass.
4. A fresh-context review approves at the exact head.

## Stop and decide (never stop and wait)

- If a structure outside this scope must change to compile (`crates/uor-r4-cpu-train` builds Hamming records), add the field so that existing struct literals keep compiling. If that is impossible, record it and decide with a decision card. Never edit another lab's crate without one.
- If the rank table cannot be computed identically on the pod and the laptop, the table is computed once and pinned as an input instead.

## Amendment (after step 1's slot counts; my decision, reversible)

The slot key failed the rule on DEV and on p-w. Three diagnostic rows were then added to the same count, because each costs about two seconds to measure, and a unit of its own would cost a PR round:
- `atom2`, the next rule this ORDER names (the clause's two rarest tokens, unordered);
- `win4`, a suffix-local atom (the rarest of the four tokens ending at the position);
- `k2>k1`, copy key 2's candidate where it has one and copy key 1's elsewhere.

They are diagnostics, outside the rule above. They do not change step 1's decision, and step 2 does not run for any of them in this unit. They choose the next copy-key unit, whose own sheet and FIT decide it.
