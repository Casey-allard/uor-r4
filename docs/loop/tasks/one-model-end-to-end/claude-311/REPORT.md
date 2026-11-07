# REPORT: one-model-end-to-end, claude-311

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no training, no GPU, no spend.

## What did not work, first

- **No model answers the panel: 0 of 256 rows complete and 0 of 128 pairs, for all six artifacts.** The bar is 104 rows and 52 pairs. These are claude-259's (b) and (h) arms at seeds 1–3, trained 8,192 steps on `mw-balp/train`.
- **The replies collapse onto one canned line.** At every seed, 128 to 152 of the 256 replies are the same text: " I'm sorry you feel nervous. I hope it gets better soon." Each artifact gives only 7 to 24 distinct replies across 256 prompts. The replies stop, but they are not about the prompt.
- **Tokens per second is not comparable to (b64L)'s.** Each run took 195 to 236 s, against 18 to 21 s for (b64L) (claude-309). These artifacts are 51,310,784 bytes against (b64L)'s 2,156,416. The bin's timer re-parses the artifact for every row (`served-reply-panel.rs:204`). The runs step 14,004 prompt tokens and 3,261–4,871 reply tokens: about 14 s at claude-259's served decode of about 1,300 tok/s for (b). Most of the wall time is most likely the 256 parses of a 51 MB artifact. The bin's 15 to 23 tok/s reflects the timer's scope, not decode speed.
- **This does not show that the corpus is what makes a model stop.** The arms differ from (b64L) in recipe as well as corpus (claude-259's sheet), so the ORDER pinned the reading as descriptive.

## What changed

- **No code.** Two readings with the tools already merged: `corpus-eos-exposure` (#3829) on `mw-balp/train`, and `served-reply-panel` (#3828, bin `be3d4cb0…`) on the six artifacts in claude-259's seal (`results/claude/one-model-end-to-end-capacity-2026-11-02/served/`).
- **Cancel probe:** no open PR or claim scores these artifacts. All six hash to the seal's `served/SHA256SUMS`, and the panel files and tokenizer pass the bin's own sha checks against the manifest.

## Measurements

**The corpus.** `mw-balp/train` (`3dee858f…`): 212,968 scored EOS in 2,425,925 scored targets, **87,788 per million**, median reply 11 (max 58). DEV has 85,812 per million, median 11. D3 has 756 (claude-310). Output `out/mwbalp.json` `d7179a4bad9689cb9f017dd81b3b964e5d459138b00b343e7d0257453d4a3057`.

**The served replies on `m2-fresh256`** (EOS, BOS, cap and context stops; rows complete; pairs both complete and distinct):

| Artifact | Stops EOS / BOS / cap / context | Rows complete | Pairs | Distinct replies | Most common reply | Output sha256 |
|---|---|---|---|---|---|---|
| (b) s1 | 242 / 0 / 14 / 0 | 0 | 0 | 24 | 129 | `19849a18116dc689dad34e3e2dfc85f52c3f40efd582730cbaa2969fcb3d9715` |
| (b) s2 | 254 / 0 / 2 / 0 | 0 | 0 | 16 | 128 | `3ea422091188051e8171c29c2e4e05af23c4fa32937c4bbe1c790226ced1fb81` |
| (b) s3 | 248 / 0 / 8 / 0 | 0 | 0 | 10 | 128 | `2b31b5b1ce9962e6498aa8d38829c77751f34f0d162bb837c1c5527f4a6522c3` |
| (h) s1 | 227 / 0 / 29 / 0 | 0 | 0 | 16 | 152 | `4ebc45ef733e971d1d6fbb84e05a017823ddf5c1bbd05e3c390a41b6d685202e` |
| (h) s2 | 252 / 0 / 4 / 0 | 0 | 0 | 13 | 146 | `873452fd6af19f60b977a4f465a3054ba11b0912208c9dc0f95068b64595bffb` |
| (h) s3 | 256 / 0 / 0 / 0 | 0 | 0 | 7 | 128 | `9a580ceec9b488a50979f89ba7b3c59da34815729fb570ffa4b0fdc0f03f89bb` |
| (b64L) s1, s2 (claude-309) | 0 / 0 / 256 / 0 | 0 | 0 | | | |

The median reply is 14 tokens for every artifact.

## Checks run at the head

- Each artifact's sha256 against the seal's `SHA256SUMS`: six of six equal.
- `served-reply-panel` checked the panel's `inputs`, `labels`, `plan` and tokenizer against the manifest on each run, and the strata against `strata_rows` (exit 0 each).
- The distinct and most-common counts are from `jq` over each output's `rows[].reply_text`. The median length is from `rows[].reply_ids`.
- `loop report --check` on this REPORT.

## Skills used

None beyond the loop's own steps.

## Still open

- **Stopping and answering are separate problems.** Both corpora fail on recall. D3's model never stops; the `mw-balp` models stop after about 14 tokens, but with a stock line. A run that moves this score needs both: a corpus with a reply-length EOS rate, and replies that depend on the prompt's facts.
- **`served-reply-panel`'s timer includes a parse of the artifact per row.** Its tokens-per-second is a decode figure only for a small artifact. Parsing once outside the timer would make it one for any artifact (an instrument change for a later unit).
- **Not separated:** corpus against recipe. A run that changes only the corpus, (b64L)'s recipe on a dialogue corpus with DEV's EOS rate, is the clean test. It needs its own prep sheet and pre-run review.

## Plain summary

The model we serve now never stops talking. This unit asked whether a model trained on conversations with short replies stops. Six such models already existed, so no training was needed. They do stop: 227 to 256 of 256 replies end on their own after about 14 tokens. But none answers the question asked. Most say the same sentence, "I'm sorry you feel nervous", whatever the prompt. So stopping is something these models learned, which fits their short-reply training data, though the recipes also differ, so this does not prove the data is the cause. Getting a model to answer is the harder part, and it is still open.
