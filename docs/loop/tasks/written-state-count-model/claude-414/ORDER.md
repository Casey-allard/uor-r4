# ORDER — written-state-count-model (claude-414)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-414. Task: `written-state-count-model`, decision item 3 of
[claude-411's REPORT](../claude-411/REPORT.md): classify the TRAIN/DEV-test shared spans of the
D87 donor corpus as stock teacher wording or copied content.

## Goal

claude-411 found that 1,936 of 5,321 D87 DEV-test records (split v1, the prompt split) share an
exact span of at least 50 reasoning or answer tokens with TRAIN. The spans were not decoded, so
the overlap is not yet known to be the teacher's stock wording rather than copied answers. This
unit classifies every shared 50-token span by a deterministic rule and measures how many
contaminated records hold copied content.

## The rule (deterministic; no neural or LLM grader, no reading of text to judge it)

1. For each distinct 50-token DEV-test span that occurs in TRAIN, its **TRAIN multiplicity** is
   the number of distinct TRAIN records that contain it. A span repeated inside one TRAIN record
   counts once for that record.
2. A span is **copied** when its multiplicity is 1 or 2 (a specific answer duplicated), and
   **stock** when it is at least 3 (recurring wording across unrelated prompts).
3. A contaminated DEV-test record is **copied-contaminated** when any of its shared spans is
   copied; otherwise it is stock-only.
4. Histograms of multiplicity in buckets 1, 2, 3–9, 10–99 and ≥ 100: over distinct shared spans,
   and over contaminated records by each record's minimum multiplicity.
5. The top 200 shared spans, ranked by the number of DEV-test records that hold them, are decoded
   with their multiplicity and class to a laptop-side file under `local/claude/claude-414/`,
   never committed (D87 text is never committed).

## Bar

- Copied-contaminated records × 10 > contaminated records (copied above 10%): the next step is
  the span dedup of claude-411's decision item 3 (remove the TRAIN records that share a span of
  100 tokens or more with DEV-test, then re-audit).
- Otherwise split v1 stands.
- A copied share of 8–12% is borderline: reported as such, and the call is left to the top
  session.

## Scope

- `docs/labs/d87-split-audit/instrument/`: per-span TRAIN record multiplicity in `SpanSet`, a
  `--classify-spans <file>` flag on `d87-audit` that writes `classify-summary.json` beside the
  audit summary and the decoded top 200 to `<file>`. The existing outputs keep their meaning.
- Fixture tests: a span planted in one TRAIN record is copied; in three distinct TRAIN records,
  stock; three times inside one TRAIN record, multiplicity 1; multiplicity independent of the
  thread count.
- One run on wick's container (CPU only, 12 CPUs) over the `/workspace/in/d87` shard copies and
  claude-411's split; nothing on `/mnt/plexpool` is touched.
- A data-preparation census: it trains nothing, scores no model and uses no GPU, so it needs no
  pre-registration sheet (§4.5).

## Gates

- G1: `cargo test --release --manifest-path docs/labs/d87-split-audit/instrument/Cargo.toml`
  passes, with the four new tests.
- G2: with the within-record dedup set aside, the within-one-record test fails; restored, it
  passes.
- G3: `uor-harness loop pr-check --local` at the committed head: `verdict: PASS`.
- G4: the wick run returns rc 0 and `classify-summary.json` holds the copied share, both
  histograms and the contaminated total, which equals 1,936 (consistency with claude-411).
- G5: the decoded file exists laptop-side and is not tracked by git.

No GPU; no pod; no corpus text, manifest over 1 MB or binary is committed.
