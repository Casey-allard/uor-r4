# ORDER: one-model-end-to-end, claude-357

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `ab718955ff8cddda7f597776e07b3b5b522c8c3f`.

## Goal

**Build what a retrain with the `longest-after-opening` copy rule needs: the rule in the float trainer, its way into the exported artifact, and the sheet for the run. Laptop only.**

**Why now.** [claude-356](../claude-356/REPORT.md) served the rule on the three stage 3 place-read models and read "better on recall" on a fresh panel (245 to 268, 278 to 304, 237 to 270 of 560). Those models were trained with the one-token rule and only read with the new one. The fall-back rule was counted on data on 3 Nov ([claude-267](../claude-267/REPORT.md), +13.7% over the one-token key) and has never been trained: claude-266, claude-323 and claude-345's arm rk trained two-token keys with no fall-back.

On 12 Nov 2026 (UTC), in chat, the lab asked "Retrain the place-read model with this copy rule on one pod?" and Casey answered "Plan the retrain". The plan was given in chat, and to "Build the retrain's laptop part (no pod, no training)?" Casey answered **"Yes, build it"**.

## What is built

Laptop only. Nothing is trained and no pod is launched.

| Step | What |
|---|---|
| 1 | **The float read** (`flock_read_hamming.rs`). `HammingReads::with_copy_longest(marker)`: a copy head under the latest-exact rule orders its exact matches by "one more token agrees" before "latest", except on an opening row. The choice is the GPU sort it already makes, with one more term in the key and one more equality table per copy head. Saved in the checkpoint beside `copy_latest`. Needs `copy_latest` and one-token copy keys. |
| 2 | **The trainer** (`in_space_long.rs`, `train-in-space`). `--hamming-copy-longest A,B`, the two token ids of the reply marker. At start it refuses a split whose reply mask and marker disagree at any position. |
| 3 | **The artifact** (`stack/format.rs`, `stack/hamming.rs`, `fbs-export`). The export reads `hamming_copy_longest` from the run's `config.json` and sets a header field that is left out when off. An artifact that sets it serves `longest-after-opening` as its own rule and finds opening rows by the marker. |
| 4 | **Tests.** Laptop: the opening rows, the sort key against the integer reader's choice, save and load, the header round trip, the trainer's check. Pod only, written here and compiled, run as the pod's first step: the tensor choice against the host's on random codes. |
| 5 | **The sheet** for the run (`docs/harness/prep/`), with a fresh fitness read by a reviewer who did not write it. |

## Decisions of this unit (claude's own decision review; logged for Casey, who can reverse them)

1. **An opening row is found from the tokens, not passed in beside them.** The row that chooses a reply's first token is the row whose own token and the one before it are the reply marker (the end of `Assistant` and `:`, ids 714 and 28 under this tokenizer). The plan said the trainer would pass opening rows from the reply mask at every forward. Counted before this ORDER on four local corpora: the pair stands before every reply's first token and nowhere else (fresh panel 3,553 of 3,553; old panel 3,586; TRAIN's oracle twin 215,017; the DEV split 1,226; no stray occurrence in any). Why the change: the float read has many forward paths (training, scoring, sampling, the trace, the decode step), and a value passed beside the tokens has to reach each one; a test on the tokens reaches all of them, and the float read and the integer reader then use the same test. The trainer still checks the marker against the mask on every split it loads and refuses on any difference.
2. **So no reader changes.** The plan had `raw-id-replies` and `uor-chat` mark every reply's opening. With the marker in the artifact, the reader finds every opening itself, in a prompt and in a re-fed conversation.
3. **An artifact without the field serves exactly as before.** A caller's `--copy-rule longest-after-opening` on such an artifact still reads the session's reply mark (claude-356's rule, unchanged).
4. **The rule is the one claude-356 tested:** the first token of a reply only, one-token copy keys only. The second token after a bare-space opening is not spared.
5. **Turn memory's leave rule is untouched.** It reads the latest one-token match at reply openings, where this rule makes the same choice. Arm rk fell at this point; the build reads the leave rule's rows first and stops if it acts on any other row.
6. **The sheet's float-against-integer check is row by row** (`row-picks`, at least 0.98 of scored rows, read first), in place of stage 3's whole-reply bar of 0.10, which no model passes. [claude-354](../claude-354/REPORT.md) left this to the owner; it was put to Casey in chat with the plan and is the sheet's to carry. Casey can refuse it before the run.
7. **No pod and no training.** The run (unit claude-358 in the plan) waits for Casey's word.

## What a pass is

| # | Gate | Check |
|---|---|---|
| G1 | Each new laptop test fails when the lines it pins are set aside, and passes at the head: the integer reader's marker opening and own rule, the header field, the float read's opening rows and sort key, save and load, the trainer's marker check. | the test runs |
| G2 | Nothing served changes for an existing artifact: the four stage 3 artifacts read at the head on claude-356's fresh prompts, under their own rule and under `--copy-rule longest-after-opening`, give reply files byte-identical to claude-356's. | `cmp` |
| G3 | The float crate's `cuda` build of the new code and its pod-only test compile on the laptop (stub compiler, no GPU); `uor-r4-training` checks; the integer stack suite and the instrument tests pass; clippy and the ratchets are clean. | the builds and test runs |
| G4 | The sheet passes `uor-harness prep-check` and carries a fresh non-author fitness read that ends FIT. | the sheet |
| G5 | The REPORT says what is built, what could not be run on the laptop, and what the pod must do first. | the REPORT |

## Not established by this unit

- Whether a model trained with the rule states remembered values more often. Nothing is trained.
- That the float read's new tensor code computes what its host twin does: the laptop cannot run it. It is the pod's first step.
- Stopping after a copied string, the value's first token, questions about things never told. No milestone moves.

## Cancel probe (run first)

- **What would make this unit unnecessary:** a float read that already orders exact matches by a longer match, or a trained model that already used one.
- **Result (at claim):** none. The float read's copy rules are the mean and `copy_latest`; `--hamming-copy-key 2` and `--hamming-copy-keys` have no fall-back; the notebook has no training with one.

## Claim

Written by hand, as claude-347's to claude-356's were: claude-346 (PR #3913, a draft under the hold) holds the item for `uor-harness loop claim`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-357/**`
- `crates/uor-r4-float-train/src/flock_read_hamming.rs`, `crates/uor-r4-float-train/src/in_space_long.rs`, `crates/uor-r4-float-train/src/bin/train-in-space.rs`
- `crates/uor-r4-integer/src/stack/**`, `crates/uor-r4-integer/src/bin/fbs-export.rs`
- `docs/harness/prep/**` (the sheet)
- `docs/research/notebook/**` (one row a result)

## Inputs

- claude-356's kit, both panels and reply files, and claude-352's four stage 3 artifacts, on the laptop: `local/claude/claude-356/`, `local/claude/claude-352/`.
- The plan Casey read: `local/claude/claude-357/PLAN.md` (not committed).

## Addendum, 12 Nov 2026 (claude's own decision review; logged for Casey, who can reverse it)

**Decision 5 was wrong, and what was built in its place.** Decision 5 said turn memory's leave rule acts only at reply openings and would stay untouched. Read before any trainer line was written, as decision 5 required: with `--turn-memory-reply 1`, which stage 3's recipe carries, the leave rule also acts at every later reply row (stage 3 logged `reply keys left to the copy read 160416 of 327257`). Left at the latest rule, a later reply row where the latest match names the target and the longer match does not would lose its key from the reply table while the copy read named another token: nobody would supply the target. That is the kind of fault arm rk fell to.

- **Built instead:** with the flag on, the leave rule asks the copy read by the run's rule (`copy_names_ruled`): the longer match first at later reply rows, the latest match at an opening row. With the flag off it is the old rule, unchanged. A laptop test compares it with a position-by-position spelling over every six-token window of three tokens.
- **Consequence for the run:** the flag changes two things in training, the copy read's choice and the keys the reply table holds. The sheet says so and gives the run a guard on the logged count (I-k).

Three smaller departures from the plan:

- **The scoring pass checks each `--panel` split for the marker,** as the trainer checks TRAIN and DEV at start. The pinned fresh-v2 split is not checked.
- **A resumed run (`--init-from`) is refused when its checkpoint's marker differs from the run's.**
- **The exporter does not compare the checkpoint's rule with the config's,** which the plan named: the snapshot does not carry the rule. The trainer's refusal above and the sheet's row check cover it.
