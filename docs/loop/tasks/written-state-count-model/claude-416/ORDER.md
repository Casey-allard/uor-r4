# ORDER — written-state-count-model (claude-416)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-416. Task: `written-state-count-model`, decision item 2 of
[claude-414's REPORT](../claude-414/REPORT.md): the span dedup of the D87 donor corpus.

## Goal

claude-414 found that 1,868 of the 1,936 contaminated D87 DEV-test records (964 per mille) share
a 50-token reasoning or answer span with only one or two TRAIN records: copied content, not stock
wording. Its decision: remove the TRAIN records that share a span of 100 tokens or more with
DEV-test, then re-audit, including the classification. This unit does both and reports where the
50-token bar stands after the dedup.

## The rule (deterministic; no grader, no reading of text to judge it)

1. A kept TRAIN record of split v1 is **dropped** when any 100 consecutive tokens of it equal a
   100-token DEV-test span whose tokens are all reasoning or answer: the same span rule and the
   same token-for-token comparison as the census, on the TRAIN side over the whole record.
2. The drop list is the sorted ids of the dropped records. It holds record ids, not text, and
   stays laptop-side beside the run; its count and SHA-256 are committed.
3. **Split v2** is split v1 with the drop list removed from TRAIN. DEV-tune and DEV-test are
   unchanged.
4. The re-audit runs the census and the classification over split v2. By construction no
   DEV-test record is contaminated at 100 tokens after the dedup; the run checks this.

## Bar

- After the dedup, the 50-token gate of claude-411 (at most 1% of DEV-test records
  contaminated) is reported as it stands: PASS or FAIL, with the copied and stock shares.
- The dedup removes TRAIN text only. If it drops more than 5% of TRAIN tokens, that is reported
  first, as a cost to the count model's training data.
- Whatever the 50-token verdict, novel-subset scoring of claude-411's decision item 2 stays the
  guard for bar Q until a later unit closes it.

## Scope

- `docs/labs/d87-split-audit/instrument/`:
  - `SpanSet::holds`, true when a TRAIN record holds any DEV span (no early skip on spans
    already marked);
  - audit options that list the dropped records at one span length and that skip a given drop
    list in the TRAIN pass, refusing ids that are not kept TRAIN records;
  - `d87-audit --write-drop <file>` (span 100) and `--drop-train <file>`.
  - The existing outputs keep their meaning.
- Fixture tests:
  - a TRAIN record holding a planted 100-token span is listed, and one holding only 60 tokens of
    it is not;
  - with the list applied, the 100-token contamination is 0 and the 50-token one remains;
  - the TRAIN counts fall by the dropped record;
  - a drop id that is not a kept TRAIN record is refused;
  - the list does not depend on the thread count.
- Two runs on wick's container (CPU only, 12 CPUs) over the `/workspace/in/d87` shard copies
  and claude-411's split: one writes the drop list, the other re-audits with it applied, with
  `--classify-spans`. Nothing on `/mnt/plexpool` is touched.
- A data-preparation census: it trains nothing, scores no model and uses no GPU, so it needs no
  pre-registration sheet (§4.5).

## Gates

- G1: `cargo test --release --manifest-path docs/labs/d87-split-audit/instrument/Cargo.toml`
  passes, with the new tests.
- G2: with the `holds` call set aside, the listing test fails; with the drop skip set aside, the
  apply test fails; each passes when restored.
- G3: `uor-harness loop pr-check --local` at the committed head: `verdict: PASS`.
- G4: both wick runs return rc 0. Run 1's audit equals claude-411's at 1,936 contaminated
  records. Run 2's summary shows the drop applied, 0 contaminated at 100 tokens, and the
  50-token result with its classification.
- G5: the drop list and the decoded spans exist laptop-side and are not tracked by git.

No GPU; no pod; no corpus text, record id list, manifest over 1 MB or binary is committed.
