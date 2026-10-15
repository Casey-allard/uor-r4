# #2067 — Answer-span supervision: the training lever's target population is empty (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T18:18:00Z
- **Updated:** 2026-10-09T18:19:53Z
- **Closed:** 2026-10-09T18:19:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2067
- **Merged:** 2026-10-09T18:19:14Z
- **Merge commit:** 483e69037e37770d44b3b96eb8aed1d267c98e3f
- **Base / Head:** main / UOR-Foundation:answer-span-delivery
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this is

The pre-registered **answer-span supervision** round (deepseek lab, #2029) — the first training-side
lever in the pointer/binding line — **recon'd to a negative and was not run**: on the corpus the arm
checkpoints were trained on, the intervention's target population does not exist. No training run, no
panel run, no new panel number. The training lever is closed as inapplicable to the recorded corpus and
the read-out line is declared finished with it.

Record: [`docs/labs/answer-span-supervision-2026-10-09/README.md`](../blob/answer-span-delivery/docs/labs/answer-span-supervision-2026-10-09/README.md)
· current state: newest-first entry in `docs/integration/current-state.md` ending in `Next:`.

## Measured (from the bytes, not from a manifest)

Training store `mixed-cp` = `chatstore` (label `chat`, 82,248,461 tokens) + `dcopy` (label `copy`,
57,229 tokens). `dcopy` decodes to 1,006 documents and **2,012 supervised answer runs whose content is
the value itself** (` cat, dog.`, ` 3.`, ` first.`, ` 2500.`, ` 25 metres.`):

| quantity | measured |
|---|---:|
| content tokens per answer (EOS excluded) | min 2, mean 4.35, max 10 |
| single-word answers | 1,386 of 2,012 = 0.6889 |
| answers containing `is/are/was/were/my/your/the/that/did/does` | **0 of 2,012** |
| occurrences of `My name is` / `my name` / `name is` / `is called` | **0 / 0 / 0 / 0** |

So on this corpus the answer span **is** the response run — the mask would zero approximately nothing.
And the failing class-C prefix is not supervised at all: re-read from the sealed trace
(`natural/n_ithmar`), window ids `4..12` = ` My| name| is| It|h|m|ar|.` and the reply is exactly that
plus EOS, with the value's span starting at window[7]. It is three ids of the PROMPT read back through
the copy channel.

Recon also established: no `binding_labels.json[l]` sidecar exists for either source (`find` over the
volume → nothing), and the recorded arm recipe is not the round's brief — `qat=true` refuses a pointer
head, so it cannot have produced these arms; arm-ptr was trained from scratch on `mixed-cp` (exact argv
in the record). Owner has recorded that correction publicly as their own error.

Reproduce with `docs/labs/answer-span-supervision-2026-10-09/measure_answer_source.py` (no model, no
GPU, ~1 s).

## What is in the diff

- **The record** (`docs/labs/answer-span-supervision-2026-10-09/`): README with the pre-registration,
  the recon commands and numbers, measured vs inferred, the decision and the limitations, plus the
  measurement script.
- **The current-state entry** (newest-first, ends in `Next:`).
- **The instrument, PAUSED/UNACTIVATED**, as
  `docs/history/branch-archive/deepseek_answer-span-supervision-20261009.patch` + an INDEX row:
  `answer_span_supervision=0|1` (default off), `AnswerSpanMask` +
  `ReadBindingLabels::restrict_to_answer_spans`, validation/record/lineage, window counts and two
  focused tests. It is **not** live machinery; the INDEX row says plainly that it is inapplicable to
  the recorded corpus by measurement. Applies cleanly to `fa786e4ef` (`git apply --check` PASS).
- **One pre-existing main defect repaired on the path, reported separately**: at
  `dialogue_episodes.rs:613` the subtraction `position + 1 - prefix_positions` was evaluated before the
  `supervised` guard, so **any debug build panicked** on a response with a non-trivial prefix. Guard
  ordered first; release behaviour is provably identical (`commitment` is `false` either way when
  `!supervised`). Pristine-main baseline: **798 passed / 14 failed**, 11 of them this subtraction; after:
  **809 passed / 3 failed**, the 3 being pre-existing `joint_campaign` failures that are untouched and
  out of scope for this round.
- The owner's own `Reply.span_extract` compile fix (#2066, `fa786e4ef`) is this branch's rebase base and
  is **not** carried here.

`STATUS.md`, `ROADMAP.md` and the #2028 table are unchanged: no served model, BPB headline or milestone
moved. Nothing here is a serving or D11 result; the 84-cell numbers quoted are the sealed ones from
earlier rounds, re-read rather than re-run.

## Checks at exact head `8e89e478d`

```
cargo test -p uor-r4-training --lib --offline
  → 809 passed; 3 failed; 7 ignored  (225.45s)
    FAILED: joint_campaign::tests::{calibrated_shadow_preserves_fractional_weights_clocks_and_packed_metadata,
            radial_initialization_resume_preserves_evolved_beta_and_next_update,
            radial_transfer_checkpoint_resume_preserves_receipt_and_next_update}
    (pre-existing on pristine main; untouched by this PR)
cargo test -p uor-r4-training --lib --offline dialogue_episodes
  → 10 passed; 0 failed        (these are the tests that panicked on pristine main)
rustfmt --edition 2021 --check crates/uor-r4-training/src/dialogue_episodes.rs
  → clean                        (4 pre-existing fmt diffs elsewhere in the crate are untouched)
python3 scripts/check_claim_wording.py
  → Claim-wording gate passed: no unlinked machine-verified/exact-equivalence wording
git apply --check docs/history/branch-archive/deepseek_answer-span-supervision-20261009.patch
  → PASS (applies to fa786e4ef)
```

## Cost and teardown

Pod `eknm8hhsq398od`, EU-RO-1 (`UOR_POD_VOLUME_DCS=EU-RO-1`), 2×4090, recon and one data pull only —
**no GPU job**. Lease 18:00:13Z → release 18:10:20Z → `down` 18:10:41Z with `--confirm-archived`;
ledger `uptime_s 605` at $1.78/h = **$0.30**. All CPU work (tests, decode, counts) on the laptop.

References #2029.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

