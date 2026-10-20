# #755 — compiler::load_corpus_bytes / runtime::history_token assume per-story-contiguous record order; violated by sharded observation output (#745 root cause)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-17T02:49:20Z
- **Updated:** 2026-08-17T04:33:30Z
- **Closed:** 2026-08-17T04:33:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/755
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

Direct continuation of #745/#752. #752 set out to decode and read the actual observed `corpus.records` content for a degenerate bundle to determine whether the corpus text itself was anomalous, or an observation→count-table aggregation bug was concentrating evidence on the wrong tokens. A new diagnostic tool (`cargo run -p uor-r4-core --example decode_corpus`, added by #752) makes this directly inspectable, and comparing three local bundles gives a clean, reproducible answer: **(b)**, an aggregation-side bug — specifically, a broken assumption about record ordering that corrupts every reconstructed local-context signal.

# The finding

`compiler::load_corpus_bytes` derives each record's "input" (previous) token from **physical adjacency in the record array**, not from any stored sequence field:

```rust
// crates/uor-r4-core/src/transformerless/compiler.rs, load_corpus_bytes
if i == 0 || story[i] != story[i - 1] {
    input.push(1u32); // BOS
} else {
    input.push(next[i - 1]);
}
```

`runtime::history_token` makes the identical assumption when walking further back for deeper context (used by `bundle_plain`/`build_store`, i.e. the actual per-depth count-table trainer):

```rust
// crates/uor-r4-core/src/transformerless/runtime.rs
pub fn history_token(c: &Corpus, i: usize, j: usize) -> Option<u32> {
    if j == 1 { return Some(c.input[i]); }
    let back = j - 1;
    if i >= back && c.story[i - back] == c.story[i] {
        Some(c.input[i - back])
    } else {
        None
    }
}
```

Both assume that a story's records are **physically contiguous and in position order** in the flat `corpus.records` stream. That assumption is false for at least two locally compiled bundles, and it directly matches #745's degenerate-output signature.

## Evidence: three-bundle comparison

Using #752's `decode_corpus` tool (`load_corpus_bytes`'s own record-size/parsing logic, not a reimplementation):

| Bundle | done | distinct stories | non-contiguous | decoded text | next-token frequency table |
|---|---|---|---|---|---|
| `smollm2-360m-broad` (#509's positive-result bundle) | 1 | 2994 | **0 (0%)** | Clean, grammatical Simple-Wikipedia prose (`" (Apr.) is the fourth month of the year in the Julian and Gregorian calendars..."`) | Ordinary English function words: `.`, ` the`, `\n`, `,`, ` of`, ` is`, ` a`, ` and`, ` in` |
| `smollm2-135m-instruct` | 1 | 3000 | **3000 (100%)** | **Scrambled word salad using the same clean vocabulary**: `"of 0 always ends onApril months month and the the March also that have in years year of the have days begins in the . It the the0 as are March..."` — same words as the clean control's April story, shuffled | Same ordinary English words as the clean control (`.`, ` the`, `\n`, `,`, ` of`, ` is`...) — the corpus **text** is fine |
| `smollm2-1-7b-instruct` (#745's bundle) | 0 (stale/incomplete snapshot, see caveat) | 1861 | 1532 (82%) | Scrambled, plus dominated by code/repo/Jupyter special tokens (`<filename>`, `<gh_stars>`, `<issue_start>`, `<reponame>`...) and exactly #745's degenerate vocabulary (` caption`, ` tetra`, ` gu`, ` chall`, ` gru`, ` fat`) at 1-3% frequency each, corpus-wide | Dominated by the same tokens #745 sees in live serving output |

`smollm2-135m-instruct` is the decisive case: its corpus **vocabulary and per-token frequency are indistinguishable from the clean control** (both are the same Simple-Wikipedia-style source), which rules out "bad corpus text" for this bundle entirely — yet its per-story record ordering is **totally scrambled** (measured directly: 361,451 of 361,693 consecutive record pairs, 99.93%, cross a story boundary — matching the ~1/3000 chance of two consecutive positions sharing a story if records were uniformly randomly ordered with respect to story, i.e. statistically indistinguishable from fully shuffled). Reconstructing "story 0" by grouping-and-file-order (exactly what `load_corpus_bytes`/`history_token` do) produces word salad from perfectly good source words, because the words are real but the order they were written in bears no relation to their real sequence.

Mechanistically: with 99.93% of consecutive array positions crossing a story boundary, `input[i]` is force-reset to BOS (`1`) for essentially every record, regardless of true depth into its source document. `history_token`'s deeper-context walk (`j>1`) then almost always returns `None` too, for the same reason. `build_store` (`crates/uor-r4-core/src/transformerless/runtime.rs`) trains its per-depth count tables from this corrupted, collapsed-to-near-BOS context — which explains **both** halves of #745's depth-witness finding at once: predictions resolve at "real" depths 1-4 with high evidence counts (the corrupted-but-real BOS-ish keys recur constantly across the interleaved corpus, so they accumulate huge counts), while the actual continuations are word salad (because that "context" was never genuine local sequence, just a collapsed reset applied almost everywhere).

## Why this doesn't show up on every bundle

`smollm2-360m-broad`'s records are written (or later merged) in perfect per-story order (0% non-contiguous); `smollm2-135m-instruct`'s are not (100%). Both use the v4/88-byte record format (`compiler::encode_v4_record`), which is written by the content-addressed sharded observation pipeline (`crates/uor-r4-graph-compiler/src/observation.rs` / `observation_text.rs`): each record's shard is `shard_of(sample_id(local_context_window), shard_bits)` — a function of the local context hash, with **no relationship to story or position**. That routing is a deliberate, documented design for parallel/resumable observation (see `observation.rs`'s module docs on content-addressed sample identity). What differs between the two bundles is whatever happens when shards get consolidated into the single flat `corpus.records` file `compiler::load_corpus_bytes` actually reads — I have not traced that merge/consolidation code path to find exactly where one bundle ends up ordered and the other doesn't; that's this issue's remaining scope (see below), not something #752 needed to resolve to identify the aggregation-side defect itself.

## Caveat on `smollm2-1-7b-instruct` specifically

That bundle's checked-in `corpus.meta`/`corpus.records` postdate `tless_store.bin`/`tless_artifacts.bin` by roughly two hours (filesystem mtimes: compile outputs at 11:04, corpus files at 12:57-13:06) and `corpus.meta`'s `done` byte is `0` with a story count that disagrees with the records actually present. That means this exact on-disk snapshot cannot be shown to be byte-identical to whatever corpus originally built the currently-served `tless_store.bin` — it looks like a later, incomplete regeneration into the same directory. The code/repo/Jupyter-domain skew found in it is real for *this snapshot*, but should be treated as a secondary, less certain observation about that one bundle rather than part of the general mechanism above, which rests entirely on the `smollm2-135m-instruct` vs. `smollm2-360m-broad` comparison (both of which have internally consistent, non-stale corpus files).

# Scope

- Trace the shard-consolidation/merge code path(s) that turn the sharded observation output into a flat `corpus.meta`/`corpus.records` pair, and determine which merge path `smollm2-360m-broad` went through vs. `smollm2-135m-instruct` (different call sites of `encode_v4_record`? different merge function? a resumed-run artifact?).
- Fix `compiler::load_corpus_bytes` (and/or whatever feeds it) so per-story context reconstruction does not depend on physical record adjacency. The corpus already carries `story` and `span_start`/`span_end` fields for exactly this purpose — the robust fix is to derive `input[i]`/position-in-story from records sorted/grouped by `(story, span_start)` rather than raw on-disk order, so correctness does not depend on the writer's/merger's ordering discipline.
- After the fix, recompile at least `smollm2-135m-instruct` (its underlying corpus text is already confirmed clean) and re-run `r4 ask`/`r4 certify` to check whether generation becomes coherent — this is the decisive confirmatory test #745 has been missing. Post the result on #745.
- Keep `crates/uor-r4-core/examples/decode_corpus.rs` (from #752) as the verification tool for before/after comparison; extend it with a `--check-order` style summary if useful, but the existing contiguity report already suffices for a before/after diff.

# Non-goals

- Not attempting the fix+recompile+retest cycle within this issue's initial filing — scoping it first so the fix can be reviewed on its own, since it touches a function (`load_corpus_bytes`) with several existing callers/tests.
- Not asserting this is #745's *only* defect — `smollm2-1-7b-instruct`'s separate corpus-domain-skew observation (see caveat above) may still matter for that specific bundle even after this fix.
- Not re-opening #751 — that was a real, independently-fixed bug, confirmed not to be #745's cause.

# Acceptance criteria

- [ ] Root cause of the ordering divergence between bundles is identified (which writer/merge path each went through).
- [ ] `load_corpus_bytes` (or an equivalent fix point) reconstructs per-story context correctly regardless of on-disk record order, verified by re-running #752's `decode_corpus` contiguity check productively (i.e., confirming the *reconstructed* context is now order-independent, not just that raw record order changed).
- [ ] `smollm2-135m-instruct` (or another already-clean-corpus bundle) is recompiled with the fix and `r4 ask` output is qualitatively assessed against the pre-fix word salad; result posted to #745 regardless of outcome.
- [ ] Existing tests covering `load_corpus_bytes`/`build_store`/`history_token` still pass, and new regression coverage is added for the interleaved-record case specifically (a corpus fixture with deliberately shuffled record order should reconstruct identically to the same corpus with records in story order).

# Verification

Full ladder: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`, `cargo run -p xtask -- audit-limits`, `cargo test --workspace --offline`, plus the decisive `r4 ask` before/after comparison described above.


## Comments (3)

### comment 5311410755 — 2026-08-17T03:24:48Z — Casey-allard

Starting on this now — it's the most direct path to actually answering #745's central question, so picking it up next in the same cadence rather than #653 phase 2 (the original roadmap's next item), which doesn't move generation quality.

Plan: fix `compiler::load_corpus_bytes` to derive `input[i]`/position-in-story from records grouped by `(story, span_start)` rather than physical on-disk array adjacency, so context reconstruction is correct regardless of how the observation pipeline ordered/sharded/merged records on disk. Add a regression fixture with deliberately shuffled record order that must reconstruct identically to the same corpus with records in story order. Then, if the local SmolLM2-135M-Instruct source model is still available, recompile that bundle (its corpus text is already confirmed clean per #752) and re-run `r4 ask` as the decisive before/after test — that result gets posted back on #745 regardless of outcome, since a negative result there is still valuable evidence.

Working in `.worktrees/i755-corpus-ordering` off latest `origin/main` (includes #752's `decode_corpus` diagnostic, useful for verifying the fix directly).

### comment 5311589012 — 2026-08-17T03:58:42Z — Casey-allard

Fixed, tested, and confirmed with a real before/after — full write-up posted on #745 (https://github.com/UOR-Foundation/uor-r4/issues/745#issuecomment-5311588000) since that's the tracking issue for this whole investigation. Summary here for #755's own record:

- `compiler::load_corpus_bytes` now sorts records by `(story, span_start)` before deriving `input[i]`/context, only when independent span data exists (`has_anchors`); `hidden` rows get the same permutation. `runtime::history_token` needed no change.
- New regression test `crates/uor-r4-core/tests/corpus_record_order_755.rs` (shuffled vs. natural record order must reconstruct identically).
- Fixed one existing test that had encoded the old buggy invariant as expected behavior (`observation_text::tests::merged_records_load_as_v3_corpus`).
- Full workspace: 1412 passed, 0 failed.
- Decisive test: recompiled `smollm2-135m-instruct` from its existing, unmodified corpus bytes via `r4 transformerless compile-recorded` (no re-running the teacher model needed). Same binary, same question, same corpus: pre-fix bundle hangs ~199s then emits repetitive garbage; post-fix bundle answers in ~0.2s with real, grammatical (if topically wandering and prompt-insensitive) English.

PR incoming.

### comment 5311696010 — 2026-08-17T04:13:55Z — Casey-allard

_edited 2026-08-17T04:14:21Z_

PR up: #757 — fix, regression test, the corrected `observation_text` test, and the RESEARCH.md write-up. Full verification ladder green (fmt, clippy -D warnings, xtask audit-limits R5, full workspace test suite, check_claim_wording.py). Auto-merge enabled, waiting on the merge queue.

Noticed #745 is already closed (closed at 2026-08-17T02:31:08Z) — consistent with my own recommendation on that thread that the decisive fix-and-retest was enough to substantially resolve its core question. Filed the residual prompt-insensitivity observation as its own distinctly-scoped follow-up: #758, since it is a materially different failure mode (does the model condition on the prompt at all) from #745's (was output grammatical at all), and deserves its own evidenced investigation rather than riding along as an unverified aside.



---
_Comment/review ids in this file: 5311410755, 5311589012, 5311696010_

