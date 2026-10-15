# #752 — serving/#655-E3 continuation: inspect observed corpus.records content for the smollm2-1-7b-instruct bundle (#745)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-17T01:54:44Z
- **Updated:** 2026-08-17T03:23:03Z
- **Closed:** 2026-08-17T03:23:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/752
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

Direct continuation of #745. That issue's latest comment documents ruling out two hypotheses with direct empirical tests (live prompt tokenizer encoding bug; cross-model vocabulary mismatch) and a decisive new finding via prediction-depth tracing: the degenerate `tetra`/`gru`/`gulick`/`chall`/`caption` word-salad is **not** context-independent root-prior fallback. `runtime::predict_witness_plain`'s resolution-depth witness shows most generated tokens resolve at depth 1-4 (real n-gram/table context matches) with substantial evidence counts (some over 50,000), not depth 0 (unigram fallback). This means the compiled emission tables themselves have learned extremely strong, high-confidence associations favoring this narrow token set as the highest-count continuation across a wide variety of unrelated prompts (verified: "Hello, how are you?", "Quantum physics describes...", "1 2 3 4 5" all produce near-identical output vocabularies).

This rules the root cause **into** the observe/corpus/training side of the pipeline (what this specific bundle was actually observed against, or a systematic aggregation bug turning observations into count tables) rather than the live serving/decode side, which is behaving exactly as designed given what it learned.

# Problem

It is currently unknown whether:
(a) the corpus text actually observed to build `smollm2-1-7b-instruct`'s `tless_store.bin` was itself garbled/non-representative (e.g. a corpus-generation bug upstream of compilation), or
(b) the corpus was fine but the observation→count-table aggregation step has a bug that concentrates evidence on a narrow, wrong token set, or
(c) something about this specific corpus (source, size, domain) genuinely, legitimately produces training statistics dominated by these fragments (e.g. a highly repetitive/templated corpus), which would be a real but different finding — not a bug, but a corpus-selection problem for anyone re-running the broad-corpus program.

# Scope

- Decode a representative sample of `.uor-models/compiled/smollm2-1-7b-instruct/corpus.records` (9.2MB) back into text using the bundle's own tokenizer (`.uor-models/compiled/smollm2-1-7b-instruct/tokenizer.bin`) and read it. This requires understanding `corpus.records`' exact binary layout — start from `crates/uor-r4-graph-compiler/src/observation.rs`'s writer side (search for where per-record token sequences are serialized) and/or `corpus.meta`'s header (25 bytes; first 8 bytes look like a little-endian u64 count, next 8 bytes another u64 — worth confirming against the writer, not just guessing from the hex dump).
- Directly check whether "tetra"/"gru"/"gulick"/"chall"/"caption" (or their token ids 29889/29891/5779+1213/1497/29892) appear anomalously often in the decoded corpus text itself, versus appearing rarely in the corpus but somehow dominating the compiled count tables (which would point at an aggregation-side bug instead).
- Cross-check `compile_report.json`'s `inputs.corpus_kappa`/`inputs.artifact_kappa` and `space_manifest.json`'s `learning_origin: {"kind": "native-corpus", "teacher_model": null}` — the null `teacher_model` for what should be an SmolLM2-1.7B-Instruct broad-teacher bundle per #509's methodology is itself worth explaining; confirm whether this bundle was actually compiled via the broad-teacher pipeline or some other (possibly older/simpler) path that might explain low-quality or unrepresentative corpus content.
- If the corpus itself looks fine (grammatical, on-topic, no unusual repetition of these specific fragments), move the investigation to the observation→count aggregation code (`crates/uor-r4-graph-compiler/src/observation.rs`, the compile/cover/score pipeline that turns records into `tless_store.bin`'s per-depth count tables) and look for a bug that could cause evidence to concentrate on the wrong tokens (e.g. an off-by-one in n-gram key construction, a hashing collision in the `code: &[u8; STAGES]` region/context encoding that's colliding unrelated contexts onto the same table entry, or similar).
- Document findings on #745 directly (do not fork the write-up into a separate document; #745 is the tracking issue for this whole investigation).

# Non-goals

- Do not attempt a full re-compile/re-observe of a fresh bundle as the first step — that's expensive and the cheaper corpus-inspection check should narrow the search space first, per #745's own "cheap checks before expensive tracing" proposed shape.
- Do not close #745 based on this alone unless it actually identifies the mechanism; a negative/inconclusive result here is still valuable and should be documented as such, then the investigation should escalate to whatever the results point toward.

# Acceptance criteria

- [ ] A representative sample of the actual observed corpus text for this bundle is decoded and read.
- [ ] A determination is reached and documented: is the corpus itself anomalous, or is the aggregation step suspect, or is this a legitimate-but-unhelpful corpus-selection issue?
- [ ] Findings are added to #745's thread and, once #745 concludes, folded into `docs/RESEARCH.md` per that issue's own acceptance criteria.

# Verification

- Whatever tooling is written to decode `corpus.records` should be a real, reusable diagnostic (worth keeping, per #745's own suggestion to extend existing debug/audit tooling rather than treat the pipeline as a black box) — not one-off scratch code.

## Comments (1)

### comment 5311113902 — 2026-08-17T02:32:56Z — Casey-allard

Starting on this now, directly continuing #745's investigation thread per the standing sequencing (#744 landed, #745 hypotheses narrowed, #751 fixed independently).

Plan: read `observation.rs`'s writer side first to nail down `corpus.records`'/`corpus.meta`'s exact binary layout (not guess from the hex dump), write a small reusable decoder that walks records and decodes token sequences through the bundle's own tokenizer, sample and read a representative chunk of the actual observed text for `smollm2-1-7b-instruct`, and check whether the degenerate token set appears anomalously in the corpus itself vs. only in the compiled count tables. Will also check `space_manifest.json`'s `teacher_model: null` against the compile pipeline that actually produced this bundle, since that's a live open question about whether this bundle even went through the #509 broad-teacher path.

Findings go on #745's thread per #752's own acceptance criteria (not forked into a separate doc). Working in `.worktrees/i752-corpus-inspection` off latest `origin/main` (includes #744/#750-adjacent, #751, and #745's docs update).


---
_Comment/review ids in this file: 5311113902_

