# ORDER: one-model-end-to-end, claude-311

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `b80933c4b`.

## Goal

**Does a served model trained on a corpus with DEV's EOS rate stop its replies?** claude-310 (#3829) found (b64L)'s TRAIN D3 scores EOS at 756 per million, against DEV's 85,812. claude-309 (#3828) found (b64L) served stops on 0 of 256 `m2-fresh256` rows. `mw-balp/train` (`3dee858f…`), the earlier pinned TRAIN, scores EOS at 87,788 per million with a median reply of 11 (`corpus-eos-exposure`, this unit's first count). claude-259 (#3558) trained width-256 arms on it for 8,192 steps, and its seal holds six served `UORLUT01` artifacts: (b) and (h) at seeds 1–3.

This unit scores those six with `served-reply-panel`, unchanged, on `m2-fresh256`. Laptop only, no training, no spend.

Claim check: (a) the step is "pipeline first" (#2028, 6709947526): every training choice is judged by this score. (b) It changes no model. It says, before any spend, whether a DEV-rate corpus is enough for a served model to stop.

**Scope of the reading:** descriptive. The arms differ from (b64L) in more than the corpus (claude-259's recipe), so a difference cannot be credited to the corpus alone.

## Cancel probe (run first)

1. No open PR or claim scores claude-259's served artifacts on `m2-fresh256`.
2. The six artifacts hash to the seal's `served/SHA256SUMS`, and the panel files to claude-309's shas.

## The reading, pinned before any score

- **Per artifact:** stops by kind (EOS, BOS, cap, context), rows complete, pairs both complete and distinct, tokens per second, and the output sha. The bar stays the manifest's: 104 rows, 52 pairs.
- **The question's answer:** the share of rows stopped on EOS, per arm and seed, against (b64L)'s 0 of 256.
- **If the bin refuses an artifact** (for example, turn memory it does not drive), that is an instrument result, recorded as such, not a reading.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-311/**`
- `docs/research/notebook/float-training.md`

## Done when

1. The six artifacts are scored, or each refusal is recorded with its message, and the outputs and their shas are recorded.
2. The mw-balp EOS count is recorded with its output sha.
3. The REPORT, negatives first, and one notebook row.

## Stop and ask if

Nothing waits on Casey (§5.1). Laptop only, no spend. Bound: one unit.
