# REPORT: one-model-end-to-end, claude-320

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Laptop only, no spend.

**No stored dialogue corpus is both varied and short-replied.** The census counted the two candidates in the store:
- **OASST2's `train/`** is far more varied than `mw-balp/train`: 99.8% of its replies are distinct, against 28.9%.
- **But OASST2's replies are long.** The median is 349 tokens against 11, so it carries about 36 times fewer EOS targets per scored token. Its typical reply also runs past the panel's 64-token cap and past half the 256-token context.

By the ORDER's measure, distinct replies per scored EOS, OASST2 is the most varied, and its licence is recorded (Apache-2.0), so the ORDER's stop rule does not fire. But the pick for the next sheet is neither corpus as it stands. It is `mw-balp/train` with its repeated replies' scoring capped (decision review below).

## What did not work, first

- **OASST2 cannot simply replace mw-balp as the stopping signal.**
  - It has 7,784 EOS targets over 3,234,805 scored targets: 2,406 per million. mw-balp has 212,968 over 2,425,925: 87,788 per million. D3 alone has 756 per million (claude-318's mix table). Per-million figures here are rounded down.
  - A D3 + OASST2 mix would therefore carry a weaker stopping signal than claude-315's × 1 mix, which already missed stopping at seed 2 (122 of 256).
- **OASST2's replies are the wrong shape for the panel.**
  - The median reply is 349 tokens; the longest is 6,135.
  - The served panel caps replies at 64 tokens, and TRAIN windows are 256 tokens.
  - A model that learned OASST2's reply length would mostly stop on the cap, not on EOS.
- **OASST2's held-out split is one of the no-memory panel's five splits** (`oasst2`; claude-319's REPORT). TRAIN and held-out are split by source id. 1 of the 427 held-out replies also appears in TRAIN, and 4,343 per million of TRAIN's distinct reply 8-grams appear in the held-out split. But training on OASST2 would put that panel split in-distribution, so a gain there would not compare with earlier units.
- **Adding OASST2 to mw-balp barely dilutes the repeats by count.** mw-balp has 212,968 replies against OASST2's 7,784. In their union, at least 69.8% of the EOS targets would still close a repeated reply ((154,143 + 23) of 220,752), against mw-balp's 72.4%. It is a lower bound because the 1,763 mw-balp replies that match an OASST2 reply would make those OASST2 copies repeats too.
- **`m2-fresh256` is not covered.** Its replies live in `labels.json`, not in a UORT split, and `reply-census` reads splits only. The overlap between the served panel's labels and mw-balp's replies is not measured here.
- **mw-balp's licence and generator are still recorded nowhere.** Its manifest says so (claude-314).

## What changed

- **`reply-census`**, a standalone Rust instrument in [`docs/labs/one-model-end-to-end/reply-census-2026-11-08/`](../../../../labs/one-model-end-to-end/reply-census-2026-11-08/README.md). It counts one split's scored replies (each maximal mask-1 run), optionally against a second split. It uses integer arithmetic only and refuses any inconsistency.
- **The census**, committed beside the instrument in `census/`:
  - `mw-balp-train.json`, against OASST2 train;
  - `oasst2-train.json`, against OASST2 held-out;
  - `oasst2-heldout.json`, against OASST2 train.

## Measurements

| Split | Replies (all end on EOS) | Distinct | EOS in a repeated reply | Most common reply | Median length (min, max) | Distinct reply 8-grams / windows | EOS per million scored |
|---|---|---|---|---|---|---|---|
| `mw-balp/train` | 212,968 | 61,616 (28.9%) | 154,143 (72.4%) | 4,470 (2.1%) | 11 (3, 58) | 395,021 / 991,235 | 87,788 |
| OASST2 `train/` | 7,784 | 7,770 (99.8%) | 23 (0.30%) | 5 | 349 (2, 6,135) | 2,764,621 / 3,180,389 | 2,406 |
| OASST2 `heldout/` | 427 | 427 | 0 | 1 | 339 (5, 4,704) | 161,863 / 181,557 | 2,313 |

The median is the lower median.

- **mw-balp's top five replies** occur 4,470, 4,429, 4,069, 3,776 and 3,653 times, at lengths 12, 9, 33, 5 and 5 tokens. They alone close 20,397 of its 212,968 EOS targets.
- **mw-balp against OASST2 train:** 1,763 of mw-balp's replies (0.83%) are an OASST2 reply, from just 2 distinct texts. 41 of its distinct 8-grams appear there.
- **Inputs:**
  - `mw-balp/train` was read from claude-314's local copy: tokens `3dee858f…`, mask `b5e7352d…`.
  - OASST2 came from `results/claude/real-text-corpus-2026-11-05/oasst2/`: train tokens `8bdcabfb…`, mask `a2b0b2a3…`. Each file passed the root's `SHA256SUMS`.
  - Full hashes are in each census JSON.
- **Census outputs:** `mw-balp-train.json` `4ad5dc89…`, `oasst2-train.json` `19e82c5a…`, `oasst2-heldout.json` `8aa17b89…`.

## Checks run at the head

- **`reply-census` tests:** `cargo test --offline --locked` reads 8 passed, 0 failed.
  - The fixture's counts were worked out by hand before the code: replies, EOS ends, a mid-reply EOS, an EOS with mask 0, duplicates, 8-grams, the median, an `against=` split with no `bos_id`, and refusals.
  - The refusals cover a mask-length mismatch, a missing `eos_id`, a non-0/1 mask byte, a wrong declared `mask_sha256` and bad arguments.
- **Three mutations fail the tests:**
  - Taking the upper median instead of the lower fails `against_split_b_without_bos`, whose four replies have an even count. The in-unit review found that no test pinned this; the session added the assertion and ran the mutation: 7 passed, 1 failed, then 8 passed once restored.
  - Merging reply runs across a single mask-0 gap fails `split_a_counts_exactly` and `against_split_b_without_bos` (the builder's run).
  - Counting EOS regardless of the mask fails `split_a_counts_exactly`. The session re-ran this one itself: 7 passed and 1 failed under the mutation, then 8 passed once restored.
- **Release build:** `cargo build --release --offline --locked` exits 0.
- **No floats, and no banned words,** in the instrument's source (grep).
- **`loop report --check`** on this REPORT.

## Decision review

The pick is `mw-balp/train` with each distinct reply scored at most **k** times. Every later copy stays in the text as context, but its mask is set to 0. Mixed with D3 as claude-319's × 3 was, it goes under the next unit's sheet. The claude lab's reasons:

1. **The stopping signal and the stock replies come from one place.** mw-balp's short replies are what made (b64L) stop (claude-313, 315, 319). Its repeats are what it collapsed onto: 72.4% of its EOS close a reply seen more than once. Capping the copies that are scored keeps the first kind and cuts the second.
2. **OASST2 gives variety at the wrong length and at about a thirty-sixth of the EOS density.** Adding it would also put the no-memory panel's `oasst2` split in-distribution. It stays a candidate for a later sheet with a longer context and reply cap.
3. **This is a corpus lever, not a decode lever.** claude-319's REPORT said that collapse again on a varied corpus would put the fault in how the trunk decodes. A capped mw-balp is the cheapest test of that: if (b64L) still collapses once no reply is scored more than k times, the next step is a decode-time lever.
4. **The next sheet must fix before any run:**
   - k, and the capped split's EOS count;
   - its distinct share and its top reply's share. `reply-census` reads all three on the capped split;
   - the stopping (128) and collapse (64) bars, unchanged;
   - the pair against (b64L-D3), with claude-319's × 3 beside it.

   The capping tool is new code, under that unit with its own tests.

What a pass and a fail would each unlock:
- **Stopping met with no collapse:** the served replies' content (complete replies, recall) becomes the remaining gap.
- **Stopping met, still collapsed:** the decode-time lever.
- **Stopping lost:** k was too low. The capped split's EOS count, fixed in advance, says by how much.

This is the claude lab's own decision review (§5.1), logged for Casey, who can reverse it.

## Skills used

- None beyond the loop's own commands. The instrument was written by a `delegate:builder` subagent under suborder `local/claude/claude-320/orders/1.md`, scoped to the instrument's folder. The first builder stopped before writing, because the guard needs a suborder.

## Still open

- **The capped-mw-balp sheet,** with the capping tool, under its own unit.
- **`m2-fresh256` against mw-balp's replies:** not measured, since the panel is not a UORT split.
- **mw-balp's licence and generator** (claude-314).
- **The scorer's timer** still measures re-parse plus decode (claude-311).

## Plain summary

We counted the replies in the two dialogue corpora we have. The one the model learned to stop from (mw-balp) repeats itself: only 29% of its replies are different from each other, and the model's stuck replies come from that. The human-written one (OASST2) almost never repeats, but its replies are about 30 times longer, so it teaches the model to stop far less often, and at lengths the test never reaches. So the next step keeps the short corpus but stops training on its repeats after a few copies each.
