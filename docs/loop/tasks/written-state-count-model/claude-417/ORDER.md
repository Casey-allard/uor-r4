# ORDER — written-state-count-model (claude-417)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-417. Task: `written-state-count-model`, decision item 2 of
[claude-416's REPORT](../claude-416/REPORT.md): the span dedup keyed on multiplicity.

## Goal

claude-416 dropped every TRAIN record holding a 100-token DEV-test span. That cut 245 per mille of
TRAIN tokens and still left 1,331 of 5,321 DEV-test records contaminated at 50 tokens, 1,261 of
them copied. Long spans turned out to be stock template text, and the copies are short.

This unit keys the dedup on how many TRAIN records hold a span, not on its length. It drops only
the holders of copied spans, removes the stock-only residue from DEV-test, and reports what each
costs.

## The rule (deterministic; no grader, no reading of text to judge it)

1. **Copied spans.** A 50-token DEV-test reasoning+answer span shared with TRAIN is *copied* when
   at most `COPIED_MAX` (2) distinct kept TRAIN records of split v1 hold it. This is claude-414's
   rule and its counting set, unchanged.
2. **TRAIN drop.** A kept TRAIN record of split v1 is dropped when it holds any copied span. The
   comparison is the census's, token for token.
   - Multiplicity is known only after a full TRAIN pass. So the finder is a second TRAIN pass,
     restricted to the copied spans of the first.
3. **DEV-test drop.** After the TRAIN drop, the DEV-test records still contaminated at 50 tokens
   are dropped from DEV-test. They can hold only stock spans: every holder of a copied span is
   gone.
4. **Split v3** is split v1 with both lists removed. DEV-tune is unchanged.
5. Both lists hold record ids, not text, and stay laptop-side beside the run. Their counts and
   SHA-256 digests are committed.

## Bar

Each cost is reported first and set against 5%, as claude-416's was:
- the TRAIN tokens dropped;
- the DEV-test records dropped.

Split v3 is adopted only if both are at most 5%. By construction, the 50-token gate then reads 0
contaminated. The run checks this.

Whatever the result, novel-subset scoring stays the guard for bar Q, as claude-411 decided.

## Scope

- `docs/labs/d87-split-audit/instrument/`:
  - a finder over the copied spans of a counting set, after the census pass;
  - a DEV-test drop list, applied in both passes, which refuses ids that are not DEV-test
    records;
  - `d87-audit --write-drop-copied <file>` and `--drop-dev-test <file>`;
  - the existing outputs keep their meaning.
- Fixture tests:
  - a holder of a span held by one or two TRAIN records is listed;
  - a holder of a span held by three TRAIN records is not listed;
  - with both lists applied, the 50-token contamination is 0;
  - a bad DEV-test drop id is refused;
  - the lists do not depend on the thread count.
- Runs on wick's container (CPU only, 12 CPUs) over the `/workspace/in/d87` shard copies and
  claude-411's split:
  - one run writes the copied-holder list;
  - one re-audits with it applied and writes the DEV-test residue;
  - one re-audits with both applied, with `--classify-spans`.
- Nothing on `/mnt/plexpool` is touched.
- This is a data-preparation census: it trains nothing, scores no model and uses no GPU. So it
  needs no pre-registration sheet (§4.5).

## Gates

- G1: `cargo test --release --manifest-path docs/labs/d87-split-audit/instrument/Cargo.toml`
  passes, with the new tests.
- G2: set-aside negatives, each passing again when restored:
  - with the multiplicity restriction set aside, the three-holder test fails;
  - with the DEV-test drop set aside, the both-lists test fails.
- G3: `uor-harness loop pr-check --local` at the committed head reads `verdict: PASS`.
- G4: every wick run returns rc 0. The first run's audit equals claude-411's, at 1,936. The last
  run shows both lists applied and the 50-token result.
- G5: the lists exist laptop-side and are not tracked by git.

No GPU and no pod. No corpus text, record id list, manifest over 1 MB or binary is committed.
