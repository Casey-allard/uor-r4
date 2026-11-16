# ORDER: one-model-end-to-end, claude-384 (V3: serve the pair rows as integer rows)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Base: `origin/main` at `2a2a1c3d2`. Follows claude-370 (PR #3961), claude-373 (PR #3964), claude-374 (PR #3965) and claude-376 (V0, PR #3970). This is stage V3 of the 14 Nov roadmap (`local/claude/one-vector-roadmap-2026-11-14.md`, §4). The kit folder is `local/claude/claude-384/`.

## Goal

**Export claude-370's pair rows into the integer artifact as i16 rows at 2^-12 (Q12), addressed by the prime product of the token pair, and have the served stack read them. Then read whether the copying gain claude-370 measured in float survives serving.**

**Why now.** claude-370's rows lift exact replies by 96 and 108 of 2,218 in float (G = 4.60 points, "Adds"). They are the only knowledge step that has passed a bar. But today the integer export refuses a checkpoint that holds them (`fbs_export.rs`, `a_checkpoint_with_pair_rows_is_refused`), so the gain never reaches `uor-chat`. claude-373 (1/R) and claude-374 (1 + 1/R) both read "Does not add", so claude-370's unweighted rows (`--pair-rows 2`) are the rows to serve.

## Cancel probe (run first)

- **What would make this unit unnecessary:** an integer form of the pair rows in the artifact, or a served read of them.
- **Result (at base):** none. `rg pair_rows crates/uor-r4-integer` finds nothing. `crates/uor-r4-integer/src/pair_experts.rs` (claude-367) uses the same prime-product address but holds counted TRAIN replies, not learned rows. `stack/address.rs` and `fbs_address.rs` (claude-366) serve learned rows addressed by a position's own block, not by a pair of turn tokens; they are this unit's template.
- **What earlier work warns of:** claude-366's address rows were exported at the residual's 2^-16; their row check measured what the quantization cost. Here the rows are stored coarser (Q12), so the cost may be larger. The I-8 control of claude-370 shows the trunk leans on its rows: a served form that drops or damages them reads below the plain `b` model.

## Claim

Written by hand, as claude-359 to claude-376 were. No code of this unit exists at this order's commit. A draft PR is pushed at this commit.

## The mechanism (as claude-370 trains it, now served)

- **Address.** Token `t` stands for the prime `primes[t]`, the first `vocab` primes. An unordered pair of different tokens `a`, `b` has the address `primes[a] · primes[b]` (31 bits). The dictionary holds the addresses of the 187,413 pairs seen in at least 2 TRAIN user turns, ascending; row `r` is the `r`-th address.
- **Picks.** The rows a reply reads: every pair of different tokens of the user turn it answers, kept when the dictionary holds it, sorted and unique. A turn is the tokens between the user marker and the assistant header; the reply runs from the header's last token to the position before the next user marker.
- **Read.** At every reply position, after layer 0's Hamilton block (after its recurrence and MLP, after any address rows, before the Hamming read), the bag (the sum of the picked rows) is added to the residual.

## Native-space items (D85, `.agents/skills/native-space-cycle/SKILL.md`)

- **State schema.** The pair rows are a new carried field of the served state: the dictionary (u32 addresses), the marks (user marker and assistant header ids), and `pairs × width` i16 values at 2^-12. The session carries the open turn's picks between steps.
- **Shared operator path.** The address rule, the dictionary, the turn scan and the picks move into one plain-data module of `uor-r4-integer` (`fbs_pair_rows.rs`), built by both the float trainer and the served stack, as `fbs_address.rs` is for claude-366. The float trainer's `pair_rows.rs` calls it for every pick; the served session calls it for every pick. There is one source definition of the picks; the only implementations are host Rust.
- **Replay oracle.** For each panel prompt, the trainer's own plan (`pair_rows::plan` on the window's ids, the turn open at the last position) and the served session's picks are compared as raw u32 row numbers and raw u32 addresses, prompt by prompt. Replies are compared token id by token id against the float scoring pass's replies.
- **Forward, gradient, update.** Not in this unit's reach: the rows are trained in float by claude-370 and are not retrained. **This unit does not meet D85's default path for the row values**: they are rounded from an F32 master into Q12, an export gap. It is historical float/export evidence and is reported as such. The picks and addresses are bit-identical by one shared operator; the values are not, and the REPORT measures that gap (largest rounding, served-against-float reply agreement on `pair` beside the same agreement on `b`).
- **Serialization.** The rows file and the artifact tables hold the i16 values and u32 addresses byte for byte; the artifact adds them without conversion. The only numerical conversion is the one rounding from F32 to Q12 in the snapshot tool, which refuses any value outside i16.

## Inputs

- **Models:** claude-370's four checkpoints on `wick`: `/workspace/out/c370/try-1/{pair-s2,pair-s3,b-s2,b-s3}/run/<run>/models/step-016384`, trained by build `00bd4c415839956e163cfcba78d5b66a3c3c6d78` (claude-358's `p` recipe, 16,384 steps; `pair` adds `--pair-rows 2`).
- **Snapshots:** `fbs-snapshot` at this unit's head, on `wick`'s GPU (an export step: no training, no evaluation). Then `fbs-export --legacy-artifact --raw-gates --raw-transition --run-config <snap>/config.json --context 8256` on the laptop, as claude-376 did.
- **Panels:** P14 (1,108 rows) and P23 (1,110 rows), claude-374's `samples-p-greedy-memory.json` and their `[prompt, reference, position]` identity hash.
- **Float replies:** claude-370's scoring passes (`local/claude/claude-370/got-1/score-*`), the replies the served replies are compared with.

## Settings

Every artifact is served on the laptop by `raw-id-replies` at this unit's head, greedy, 48 tokens, `--mark-reply 1`, turn memory as exported (claude-376's VM setting). No counted asks table, no serve-again, **no pair experts**.

- **S-pair:** `pair-s2`, `pair-s3` exported with their rows.
- **S-b:** `b-s2`, `b-s3` exported as before (the recipe without rows).
- **S-cut (read without a bar):** `pair-s2`, `pair-s3` served with the rows dropped, the served form of claude-370's I-8.

## Files in scope

- `crates/uor-r4-integer/src/fbs_pair_rows.rs` (new), declared in both crate roots, `src/native_geometry.rs` and `src/serving_root.rs` (corrected at the REPORT: the order named `lib.rs`, which is not the crate's root)
- `crates/uor-r4-integer/src/stack/pair_rows.rs` (new), `stack/mod.rs`, `stack/format.rs`, `stack/session.rs`, `stack/tests.rs`, `stack/tests/pair_rows.rs` (new), `stack/q31_read.rs` (after the merge of `main`: its adder refuses pair rows, and one test of the two together)
- `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `crates/uor-r4-float-train/src/pair_rows.rs`, `src/fbs_export.rs`, `src/bin/fbs-snapshot.rs`, `src/bin/pair-picks.rs` (new: the trainer's plan for the replay check), `src/bin/pair-positions.rs` (new, added at the review: the per-position replay check)
- `docs/labs/one-model-end-to-end/instrument/src/bin/raw-id-replies.rs`, and the instrument's `Cargo.lock` (its build adds `uor-r4-native-vector`, a dependency `main` gave the integer crate without refreshing this lockfile)
- `docs/loop/tasks/one-model-end-to-end/claude-384/**`, `docs/research/notebook/float-training.md`

## Done when

1. The export no longer refuses pair rows: a checkpoint with them snapshots into a rows file plus a note, and `fbs-export` adds them to the artifact under their own schema, so a reader from before refuses it by schema instead of serving without the rows.
2. Tests on the laptop, each failing without the rule it pins:
   - the shared picks equal the trainer's plan on streams with several turns, cut turns, repeated tokens and pairs outside the dictionary;
   - the rows file and the artifact tables round-trip bit for bit, and a damaged file (length, order, magic, digest) is refused;
   - served with all rows zero, every reply id is the stack's without rows; served with rows, only reply positions change;
   - the served session's picks, fed token by token, equal the picks of the whole stream at every position;
   - the snapshot's rounding refuses a value outside i16 at 2^-12.
3. The replay check: on every P14 and P23 prompt of both `pair` seeds, the served picks equal the trainer's plan, as raw u32 row numbers and addresses (2 × 2,218 prompts). Replies are compared with the float replies, id by id.
4. The REPORT gives, per artifact: exact replies over P14 + P23, fact and number rows ended with the answer word (of 188), replies that did not end, served tokens a second (decode and prompt, both arms of one seed timed one after the other), artifact bytes; `uor-chat` transcripts with pair experts off for S-pair against S-b; negatives first. The result is one notebook row.

## Bars

Fixed here, before any artifact of this unit is served on a panel.

- **The replay (an instrument bar).** The picks agree on every prompt (4,436 of 4,436). Any disagreement is an instrument failure: it is repaired, and nothing is read until it passes.
- **The number.** For each seed, served exact replies of S-pair minus served exact replies of S-b over P14 + P23 (2,218). G_s is the mean of the two seeds' gains, as a share of 2,218.
- **Served (the gain reaches serving):** G_s of 2.3 points or more (half of claude-370's float G of 4.60) and each seed's gain above zero.
- **Not served:** G_s under 1.0 point.
- **Not called:** anything between.
- Read without a bar: S-cut; reply agreement with float; tokens a second; artifact bytes; the fact rows by kind.

## Stop rules

- The settings, panels and bars are not moved after an artifact of this unit is served on a panel.
- The rows are not retrained, re-weighed or re-counted in this unit.
- Compression (rows shared by the pair's E8 address, or a higher MIN) is a separate step: it runs in this unit only if items 1–4 are done and the unit is still small; otherwise it is the next unit.
- On `wick`, only the snapshot runs, beside the V1 session's runs (the same lab); nothing of another lab is touched. No rented pod.

## Rules check (CHARTER §3)

- Geometry only: the address is the project's prime route (the semiprime of two token primes); the rows live in the residual stream's space.
- The read is a gather and a sum of i16 rows into the i32 residual: no matrix product, no squashing.
- No training and no evaluation on a GPU. The GPU runs only the snapshot, an export step. Serving the integer artifact on the laptop is the product path (§3.6).
- Rust only; no Python; no neural comparator.
