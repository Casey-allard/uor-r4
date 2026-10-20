# #268 — D3 teacher floor remains ~13.4 bits after the tokenizer fix — a second observation/eval confounder exists

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T15:53:23Z
- **Updated:** 2026-07-30T06:17:08Z
- **Closed:** 2026-07-30T06:17:08Z
- **Labels:** bug
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/268
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
Post-#253 measurement (#242, comment with the three-way table): with correct byte-level BPE end-to-end, artifact/teacher agreement jumped 2.6% → 23.1% and artifact overhead collapsed +22.3 → +3.0 bits — but the **teacher floor itself remains ~13.4 bits/token** against a ~15.6-bit uniform bound. A healthy 135M instruct model teacher-forced on Simple-Wikipedia text should plausibly sit around 4–7 bits. The tokenizer (#242) was *a* confounder, not the only one: something in the observation/evaluation path still depresses the teacher to near-chance.

### Candidates to check (in rough order)
1. **Sequence truncation**: observation caps at `--sequence-length 128`; 490/3000 articles truncated. Are floor positions early-in-window (short effective context) over-represented? Instrument floor-by-position-in-window.
2. **Context reset semantics**: does the eval replay give the teacher the full running window the observation gave it, or shorter effective contexts at window boundaries?
3. **Prompt framing**: raw encyclopedic text with no BOS/formatting may be off-distribution for an *instruct*-tuned checkpoint — compare floor for SmolLM2-135M base vs instruct on the same stream, and with/without BOS.
4. **Digits pre-tokenization interaction**: individual-digit tokens inflate entropy on number-heavy encyclopedic text; quantify the floor contribution of digit positions.
5. Refetched-corpus drift (~0.5%) — unlikely to explain multiple bits, but re-measure once the corpus is re-pinned (see the reproducibility issue).

### DoD
The floor is decomposed (by position-in-window, token class, article) sufficient to name the dominant cause; either a fix lands (with before/after floor) or the floor's meaning is re-documented as a bound under stated conditions. Until then the D3 teacher floor should not be quoted as a model-quality bound.


## Comments (6)

### comment 5123513942 — 2026-07-29T21:24:08Z — Casey-allard

Candidate 5 (refetched-corpus drift, ~0.5%) is **eliminated**: the #267 investigation found the drift never existed — the manifest's `text_bytes` was a character count (11,972,624 chars, exact match against the refetch) while the refetch compared utf-8 bytes (12,038,579). blake3 of the refetched articles.jsonl equals the pinned corpus_cid bit-for-bit; upstream is stable. The pinned corpus is now also retrievable directly (release tag `d3-corpus-simple-wiki-20231101`, asset keyed by corpus_cid).

Also possibly relevant to candidates 2-3: the first serving-surface held-out row (PR #288) measured 12.6% teacher agreement where the score-time eval reports 31.7% — the serving path derives signatures from token windows rather than observation vectors. If the observation/eval mismatch being hunted here has a window-boundary component, that gap is a second instrument for it.

### comment 5123863505 — 2026-07-29T22:08:11Z — Casey-allard

## Dominant cause named — instrumented run (2026-07-29, smollm2-135m-instruct bundle, schema-2 report, /tmp/i268-eval-report.json blake3:8feb64a4…)

Headline metrics reproduce #242 (top1 2.9% | agreement 23.1% | WB 16.49 | **floor 13.446**). The new decomposition:

| slice | tokens | floor bits/token |
|---|---|---|
| position-in-story 0-7 | 63,116 (**87%**) | 13.54 |
| position-in-story 8-15 | 9,104 | 12.83 |
| position-in-story 16-31 | 38 | 12.50 |
| next-token: digit | 6,392 | 10.48 |
| next-token: alpha | 52,150 | **15.04** |
| next-token: other | 13,716 | 8.77 |
| worst articles | top-10 | 14.9-16.2 |

## Reading

1. **The dominant cause is candidate 2 — replay/order semantics, with a measured mechanism.** 87% of held-out tokens sit at story-positions 0-7 and the mean story *run length* in record order is **9.2 tokens** (72,258 / (63,116/8)). Per observation_text's own doc, story ids are *article ordinals* (~120 tokens each) — so the record stream must interleave stories in ~9-token runs (≈ WINDOW+1; consistent with sharded observation routing by content-addressed sample id). The eval loop resets the oracle on every story-id *change in stream order*, so the teacher is re-cold-started every ~9 tokens: **mean effective context ≈ 4.6 tokens** on encyclopedic text. A 13.4-bit floor under those conditions is unsurprising; it is not a model-quality bound.
2. Corollary confirming the mechanism: **alpha tokens sit at 15.04 bits — statistically at chance** (uniform over the 49,152 vocab ≈ 15.6). With ~4 tokens of context, content words are unguessable; punctuation/function tokens (8.8) and digit continuations (10.5) survive on local cues.
3. **Candidate 4 (digits) is exonerated** — digit positions are *below* the mean floor, not above. Candidate 'article pathology' is a mild tail (worst article 16.2 vs 13.4 mean), not a driver.
4. Position bands within runs trend the right way (13.54 → 12.83 → 12.50), quantifying the short-context penalty directly.

## Next (the fix path)

1. One code check: confirm the shard-routing/write order in observe/observe_text produces story-interleaved corpus.records (the 9.2-token run length says it does; the doc's 'story = article ordinal' says ids are right and *order* is the issue).
2. Fix the replay: evaluate in story-contiguous order (stable sort by (story, in-story position) before stepping, resetting only at true article boundaries) — or fix observation to write contiguous records. Note the same stream-order assumption appears in transition compilation ('consecutive train positions in one story') and the 32-token repetition probe — both are silently degraded by ~9-token runs and should be re-measured after the fix.
3. Re-measure: the floor should move toward the 4-7-bit range expected at 128-token context. **Until then, per the DoD's re-documentation branch: the 13.4-bit floor is a bound under ~4.6-token mean effective context, and must not be quoted as a model-quality bound.**

Candidates 3 (prompt framing/BOS) and 1 (window truncation) stay open but are now second-order: the measured mechanism above accounts for the bulk of the anomaly. Corpus-drift (candidate 5) was already eliminated via #267.

### comment 5124270416 — 2026-07-29T23:03:45Z — Casey-allard

Instrumentation merged in batch-2 (PR #289, merge c187ef0): every `evaluate-report` run now emits the schema-2 floor decomposition (position-in-story bands / next-token class / worst articles) in the report JSON and stdout. The dominant-cause analysis posted above stands on the first instrumented run. **This issue stays open for the fix**: story-contiguous replay in the eval (reset only at true article boundaries), plus re-measurement of the two other consumers of the stream-order assumption — transition compilation ('consecutive train positions in one story') and the 32-token repetition probe — and a re-measured floor, expected to move from 13.4 toward the 4-7-bit range. Until then the 13.4-bit figure is quotable only as a bound under ~4.6-token mean effective context.

### comment 5125324488 — 2026-07-30T01:36:18Z — Casey-allard

## Replay fix measured (2026-07-29 late, /tmp/i268-eval-fixed.json blake3:531aabc5…; fix on branch batch-3, commit bed698e)

Story-contiguous replay (stable sort by story, reset only at article boundaries) — the decomposition verifies the reconstruction directly:

| position-in-story | tokens | floor bits/token |
|---|---|---|
| 0-7 | 4,800 | 14.23 |
| 8-15 | 4,798 | 14.13 |
| 16-31 | 9,544 | 12.24 |
| 32-63 | 18,787 | 11.41 |
| 64-127 | 34,329 | 10.91 |

Band counts now match article structure (~600 held-out articles × 8 ≈ 4,800 in band one, vs 63,116 before), and the context gradient is clean and monotone. **Floor: 13.446 → 11.651 bits/token** (−1.8). Interleaved replay was real and is fixed.

## But the issue's thesis survives: a further confounder remains

At full 64-127-token context the floor is still ~10.9 bits — implausibly high for a 135M model on Simple Wikipedia. With candidates 2 (replay order — fixed here), 4 (digits — now 8.44, *below* the mean) and 5 (corpus drift — eliminated in #267) all retired, **candidate 3 (prompt framing) is the front-runner**: an instruct-tuned checkpoint teacher-forced on raw encyclopedic text with no BOS/formatting. The issue's original experiment stands as next: same stream, SmolLM2-135M base vs instruct, with/without BOS — four cheap arms, one run.

Two side observations for the record: (1) live-teacher agreement fell 23.1% → 11.0% — expected: the store's predictions were built from short-effective-context observation evidence, and the properly-contexted teacher argmax diverges from them; this quantifies how much the *observation* side (not just eval) was affected, which feeds the transition-compilation re-measure. (2) Interim quoting rule updates: **11.65 bits under 128-token story-contiguous replay, still not a model-quality bound pending candidate 3.**

### comment 5126264316 — 2026-07-30T04:00:34Z — Casey-allard

## Candidate 3 measured and retired (2026-07-30, four arms, one run; branch batch-3 commit c987e73, pinned binary; reports /tmp/i268-c3-*.json)

Same D3 story-contiguous stream, SmolLM2-135M base (93efa2f) vs instruct × ±BOS at every held-out story boundary:

| arm | floor bits/token | bands 0-7 / 8-15 / 16-31 / 32-63 / 64-127 |
|---|---|---|
| instruct, no BOS | **11.6511** (reproduces the replay-fix measurement exactly — identical-binary re-baseline ✓) | 14.23 / 14.13 / 12.24 / 11.41 / 10.91 |
| instruct, +BOS | 11.6955 | 14.29 / 14.34 / 12.37 / 11.44 / 10.91 |
| base, no BOS | **11.1744** | 13.08 / 11.98 / 11.35 / 11.06 / 10.81 |
| base, +BOS | 11.2158 | 12.82 / 12.13 / 11.44 / 11.13 / 10.85 |

Store metrics bit-identical across arms (top1 2.9%, WB 16.4875) — correct invariant, the arms only move the teacher. Two findings:

1. **BOS: nothing.** +0.04 bits on both checkpoints. Retired.
2. **Instruct-tuning damage is real but small and early-band-concentrated**: −0.48 bits overall for base, almost all of it in bands 0-15 (14.2→13.1, 14.1→12.0); at full context the checkpoints converge (10.91 vs 10.81). Prompt framing was a minor contributor, not the confounder.

**Quoting rule update:** the D3 teacher floor is **11.17 bits/token (base checkpoint, 128-token story-contiguous replay)**, full-context band **~10.8**, checkpoint- and framing-robust.

**The thesis still survives.** ~10.8 bits at 64-127 tokens of real context remains implausibly high for a 135M on Simple Wikipedia (plausible model-quality bound ~4-6 bits). With candidates 2/3/4/5 all retired, the remaining suspects are pipeline-internal: the observation-side text→stream reconstruction (tokenization/segmentation of the corpus, post-#242 id space) or the teacher-forcing surface itself. Proposed next experiment — the external referee: run the same held-out articles' raw text through the same checkpoint OUTSIDE the r4 pipeline (HF transformers/candle, 128-token context, same positions) and measure bits/token. ~4-6 ⇒ the confounder is ours (tokenization/stream); ~11 ⇒ the model is simply that weak on this stream and the floor stands as a model property. One run, decides the issue's remaining scope.

### comment 5127310206 — 2026-07-30T06:17:08Z — Casey-allard

Closing on the measured record; the one remaining experiment moves to its own issue. Findings: the 13.4-bit "teacher floor" was stream interleaving (fixed, #295: story-contiguous replay, 13.446→11.651); candidates 2/3/4/5 all measured and retired (BOS +0.04; base-vs-instruct −0.48 concentrated in bands 0-15; digits below mean; corpus drift never existed). **Quoting rule: D3 floor 11.17 bits/token (base checkpoint, 128-token story-contiguous), full-context band ~10.8, checkpoint- and framing-robust.** The residual question — whether ~10.8 at full context is pipeline-internal or the model's true quality on this stream — is exactly one experiment (external referee: same text, same checkpoint, outside the pipeline) and is filed separately rather than holding this issue open.


---
_Comment/review ids in this file: 5123513942, 5123863505, 5124270416, 5125324488, 5126264316, 5127310206_

