# #745 — serving/#655-E3: root-cause the degenerate small-vocabulary token-loop generation on the best-available compiled bundle

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-16T21:23:12Z
- **Updated:** 2026-08-17T03:58:32Z
- **Closed:** 2026-08-17T02:31:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/745
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

This is the central open question for the project's stated goal: an LLM
that works without transformers or matrix multiplication. Found during a
coherence-measurement pass across locally compiled bundles (see the
comment on #655:
https://github.com/UOR-Foundation/uor-r4/issues/655#issuecomment-5309698590,
and the two sibling issues on the `R4Engine::load` panic and the
`instruction_eval_passed` gate disagreement filed alongside this one).

# Current evidence

With the caveat that the recorded quality metrics for this bundle are
themselves under question (see the sibling gate-honesty issue), the
`smollm2-1-7b-chat` manifest is the best-looking locally compiled bundle
available: `top1_accuracy_pct: 34.7`, `teacher_argmax_agreement_pct: 31.5`,
`bits_over_teacher_floor: 4.30` (its own `instruction-eval.json`),
`grounded_answer_rate: 0.85`, `repetition_rate: 0.01` (its manifest).

Live `r4 ask` output for two different real prompts on this bundle:

```
$ r4 ask --model smollm2-1-7b-chat "Hello, how are you?"
tetra fatile gop gru chall gulick caption tetrachact sul n that or grubag
low challPrevious d tetraationsforevenilesipedvery M b extid gru abopes
knocked Glacas tetra fat E Be gruillileact gulick used tetra any tetra
chall caption gch tetra s n dbag by Twenty .. revolves or bif ones low gru
tetra fatfore that thousoo Aeile gulickindact tetraorship challveniles E g
ext caption s tetra

$ r4 ask --model smollm2-1-7b-chat "Write one sentence about dogs."
gruill tetra n d chall support caption any or tetra s tetra gulick
tetraileascyact lowch fat bvenileshttps Squad tetra trip gru tetra willoo
det cultValues username Sumif anal caption or chall d n gulickile s
gruchact
```

This is not "weak but language-shaped" output. It's a small, repeating set
of subword fragments (`tetra`, `gru`, `gulick`, `chall`, `caption`, `fat`,
...) recombined in non-syntactic order, with essentially no sentence
structure, no grammatical agreement, and (as far as can be told without
deeper tracing) little to no relationship to the prompt. The same rough
handful of tokens recurs across both prompts, which is itself a clue: it
suggests something close to a context-independent or near-context-
independent token set dominating selection, rather than a context-sensitive
but low-accuracy signal.

README already discloses weak quality in aggregate terms ("~1% top-1 OOD,
~36% top-1 ID on the Gate C 500k fixture") but does not include an actual
generation transcript or diagnose *why* generation looks like this.

# Problem

The project's premise is that a table-native, zero-multiply/divide/float,
non-transformer compiled pipeline (source teacher trace -> transformerless
compiler -> R4G1 graph -> region/table scorer) can produce usable language
generation. The evidence gathered so far establishes that *something* is
being emitted (not a crash, not literally random bytes) and that quality
metrics improve with model scale (135M: 1.5% top1 vs 1.7B: 34.7% top1) --
both mildly encouraging signs that the architecture is not fundamentally
inert. But nobody has traced *why* live generation degenerates into a
small repeated nonsense-token set, so it's currently unknown whether this
is:

(a) a specific, fixable defect (e.g. the emission table falling back to a
    context-independent `root_prior` far more than it should, a
    tokenizer/detokenizer round-trip bug producing overlapping garbage
    fragments, a corrupted or mis-selected score table for this bundle, a
    broken top-k/sampling/argmax selection step, or the wrong resolution
    tier being served by mistake), or
(b) evidence that the current compiled-table/region approach, at this
    corpus/model scale, is not yet capable of coherent generation and
    needs a specific, named further research investment (more/better
    training corpus, a different region-splitting or scoring strategy,
    more capacity in the table representation, etc.) before it can work,
    still within the no-multiply/no-transformer constraint.

Distinguishing (a) from (b) is the single most important open question for
whether this project's core architecture can reach its goal.

# Goal

Determine root cause of the degenerate small-vocabulary-loop generation
failure mode, and either (a) fix a specific defect and get the pipeline
producing recognizably grammatical (even if still low-accuracy/weak)
English text, or (b) produce a rigorously evidenced conclusion that the
current architecture needs a specific, named further investment before it
can produce coherent text, with enough detail that the maintainer can
decide whether to fund that investment or reconsider the approach.

# Scope

- Depends on the sibling `R4Engine::load` panic fix landing first (or a
  documented workaround) so `r4 certify`'s full held-out evaluation can run
  cleanly against this bundle for quantitative grounding, not just spot-
  check prompts.
- Trace one full generation step by step for a real prompt: tokenize it,
  and at each generated position inspect the exact candidate distribution
  (`root_prior`, node-level `emissions`, `context_rows`/n-gram table
  entries -- see `GraphScorer::from_artifact` and its scoring path in
  `crates/uor-r4-graph-certify/src/score_runtime.rs`) that the scorer
  actually selects from, versus what the compiler intended for that
  context. Use or extend existing debug/audit tooling (`r4 audit`, the
  A-mode `r4 graph infill` skeleton-fill path, or ad hoc instrumentation)
  rather than treating the scorer as a black box.
- Check whether the recurring nonsense tokens (`tetra`, `gru`, `gulick`,
  `chall`, `caption`, ...) are dominating `root_prior` (context-independent
  root candidates) specifically -- if so, that's strong evidence for
  hypothesis (a)'s "context-independent fallback dominating" variant, and
  should be traced back to why the compiler produced that
  root-prior/region structure from the source corpus.
- Test whether the same small token set recurs across a wider set of
  prompts and/or across independent re-runs (determinism check) to
  distinguish "small dominant fallback set" from "different garbage each
  time."
- Check tokenizer/detokenizer round-trip fidelity for this bundle
  specifically -- encode then decode a range of known strings and confirm
  byte-for-byte round-trip; a detokenizer bug producing overlapping/merged
  garbage fragments is a distinct and plausible candidate root cause worth
  ruling in or out early since it's cheap to test.
- Compare generation against the teacher-oracle tier for the same prompts,
  if a teacher checkpoint can be obtained (none is present locally as of
  this writing -- `/tmp/ref/out/model.bin` does not exist; may require a
  download/compile step first). This isolates compiler/scorer-side defects
  (bad results even against teacher-quality source signal) from
  source-model-side ones (source model itself is too weak at this scale).
- Cross-reference against `docs/deferral_record_2026_08_05.md` and any
  other existing internal research notes on known generation-quality
  investigations, so this doesn't duplicate already-refuted hypotheses
  (e.g. "standalone two-pass generation is refuted, twice" per README).
- Document findings thoroughly regardless of outcome. A well-evidenced
  "here is specifically why this doesn't work yet, and here is what it
  would take" is an acceptable and valuable outcome, not a failure to
  close this issue -- but it must be evidenced, not speculative.

# Non-goals

- Do not introduce multiply, divide, floating-point, or matrix-
  multiplication operations into the deployed serving hot path to "fix"
  quality -- any fix must stay within the existing P-4 contract and the
  project's committed transformerless/table-native architecture. If the
  investigation concludes the architecture needs a fundamentally different
  compute primitive to reach coherent generation, say so explicitly as a
  finding for the maintainer to decide on -- do not unilaterally introduce
  one.
- Do not treat the teacher-oracle or geometric tiers as fixes for this --
  they are reference/research paths, not the production target (per #655's
  own scope).
- Do not weaken or bypass the `instruction_eval_passed` gate to make this
  bundle look like it's passing (that gate's correctness is the sibling
  issue's job, not this one's).
- Do not claim a fix "works" based on the quantitative top1/agreement
  metrics alone -- confirm with actual generation transcripts, since this
  issue exists precisely because those metrics and live output disagreed
  once already.

# Dependencies and blockers

- Sibling issue: `R4Engine::load` panic (blocks clean C-row measurement of
  the 135M bundle; the 1.7B bundle used for the transcripts above was
  reached via the `ModelStore`/`ask` path, not `R4Engine::load`, so it is
  not itself blocked by that panic, but a full before/after quantitative
  comparison across bundles benefits from that fix landing first).
- Sibling issue: `instruction_eval_passed` gate honesty (informs which
  recorded numbers, if any, can be trusted while investigating this).
- May require obtaining a teacher checkpoint (not currently present
  locally) for the teacher-oracle comparison step -- treat that as
  optional/best-effort rather than a hard blocker if it's costly to
  obtain.

# Proposed implementation shape

Start with the cheap, high-information checks first (tokenizer round-trip,
determinism across re-runs and prompts, whether the same tokens recur) before
instrumenting a full step-by-step scorer trace, since those can quickly
rule in/out entire classes of root cause. Escalate to full tracing only
once the cheap checks have narrowed the search space.

# Acceptance criteria

- [ ] A specific root cause (or a small, evidenced set of candidate causes
      with the investigation needed to distinguish them) is identified for
      the degenerate small-vocabulary-loop generation failure mode.
- [ ] Either a fix lands that produces recognizably grammatical text on a
      held-out probe set (with before/after transcripts, not just
      aggregate metrics), or a documented, evidenced conclusion is
      produced stating exactly what further investment the architecture
      needs and why.
- [ ] Findings are written up thoroughly (this is exactly the kind of
      result that belongs in `docs/RESEARCH.md` and/or a dedicated
      deferral-record-style document, matching this repo's existing
      pattern for documenting refuted/open hypotheses).
- [ ] If a fix lands, it stays within the P-4 no-multiply/divide/float
      contract and the existing table-native architecture.
- [ ] Workspace fmt/clippy/tests, and any relevant Gate C/serving-eval
      harnesses, pass.

# Verification

- Before/after generation transcripts on a fixed, documented set of
  prompts.
- Before/after `r4 certify` C-row and/or `instruction-eval.json`-style
  quantitative metrics, cross-checked against the transcripts (per the
  sibling gate-honesty issue's finding that these can disagree -- don't
  rely on metrics alone).
- Determinism check: same prompt, same bundle, same seed/config produces
  the same output.


## Comments (5)

### comment 5309741563 — 2026-08-16T21:23:20Z — Casey-allard

Sibling issues from the same investigation: #743 (R4Engine::load panic, likely worth landing first to unblock certify measurement), #744 (quality-gate honesty, informs which recorded numbers can be trusted while investigating this one).

### comment 5310827999 — 2026-08-17T01:37:29Z — Casey-allard

Starting on this one now, per the recommended sequencing from #747 and #744's follow-up (#750 filed, #744 left open pending that separate R4G1-path gap).

Following the issue's own proposed shape: starting with the cheap, high-information checks (tokenizer round-trip, determinism across re-runs/prompts, whether the same small token set recurs) before instrumenting a full scorer trace.

Working in `.worktrees/i745-degenerate-generation` off `main@20001570` (post-#749).

### comment 5310919799 — 2026-08-17T01:56:27Z — Casey-allard

Thorough progress update — two concrete hypotheses ruled out by direct empirical test, and a decisive new lead. Following the issue's own proposed shape (cheap checks before expensive tracing).

**Checked and ruled out #1: live prompt-encoding bug.** Found a real bug in `scenarios::Tokenizer::encode_into`: it unconditionally rewrites spaces to the GPT2 byte-remap character `'Ġ'` (0xC4 0xA0) before matching against the vocab, assuming the loaded vocab's own bytes use that same convention. `smollm2-1-7b-instruct`'s tokenizer.bin (49152 entries) actually stores **literal raw bytes** (zero vocab entries contain the `0xC4 0xA0` sequence anywhere) — so the encoder can never match a real merged word token and splits every space into two junk single-byte tokens instead. Confirmed: `"Hello, how are you?"` encoded to 12 tokens including spurious byte-pairs at every word boundary; round-trip testing doesn't catch this because `decode()` happens to reconstitute the same bytes by accident. I patched the encoder to auto-detect the vocab's actual convention (skip the remap if no entry uses it) and verified it fixes encoding: the same prompt now encodes to 6 tokens using real merged tokens (`" how"`, `" are"`, `" you"` as single ids). **Then re-ran the actual bundle with the fix applied — byte-identical word-salad output.** This rules out prompt encoding as the cause. The bug is real and worth fixing on its own merits regardless (filed as #751), but it isn't this issue's answer.

**Checked and ruled out #2: cross-vocabulary mismatch.** The recurring predicted token ids (29889, 278, 322, 471, 297, 304, 338, 29892, 29901…) looked suspiciously like well-known Llama-2/SentencePiece fixture values (29889="." in the Llama vocab, etc.), raising a hypothesis that the compiled artifact was built against a different (32k SentencePiece) vocabulary than the 49152-entry SmolLM2 tokenizer being used to decode it. I did not take this at face value — checked directly: `tless_artifacts.bin`'s own `TLA5` header declares `vocab=49152` (confirmed via hex dump, bytes 4-8 = `0x0000C000`), exactly matching the tokenizer's 49152 vocab entries. And decoding the *exact* predicted token ids through the actual SmolLM2 tokenizer reproduces the garbage precisely: id 29889 → `" tetra"`, id 29891 → `" gru"`, id 1497 → `" chall"`, id 29892 → `" caption"`. These are genuinely native SmolLM2 subword pieces at their real ids — not a foreign vocabulary. My initial pattern-match to memorized Llama fixture values was wrong; glad I verified against the actual data before writing it up as a finding.

**The decisive new lead.** `runtime::predict_witness_plain` already carries a resolution-depth witness (`Prediction.depth`, 0 = context-independent unigram fallback, 1-4 = real n-gram/table context match) that wasn't being surfaced anywhere. I instrumented it directly against a live `r4 ask` run. Result: generation is **not** falling back to depth-0 selection — the overwhelming majority of tokens resolve at depth 2-4 with substantial evidence counts (many in the thousands, several over 50,000). I also confirmed the output is near-context-independent empirically: `"Hello, how are you?"`, `"Quantum physics describes the behavior of subatomic particles."`, and `"1 2 3 4 5"` all produce output dominated by the same small token set.

Putting those two facts together: the compiled emission/count tables themselves have learned an extremely strong, high-confidence preference for this narrow token set as the top continuation across a wide variety of unrelated contexts. That's not a live-serving bug — serving is faithfully reporting what the tables actually contain. The cause has to be upstream, in what this bundle was observed against or how observations got aggregated into those tables. I've scoped that continuation as **#752** (decode and read the bundle's actual observed corpus text, then either confirm the corpus itself is anomalous or push further into the aggregation code) rather than open-endedly continuing to trace inside this same issue, since it's a distinct, well-defined next step with its own acceptance criteria.

Leaving #745 open — this is real progress (two hypotheses eliminated with evidence, not argument, and the search space meaningfully narrowed) but not yet the root cause. Findings recorded in `docs/RESEARCH.md` (PR #753). Given the depth this has already reached, I'm handing the corpus-inspection work to #752 as its own unit of work rather than open-endedly continuing here.

### comment 5311200549 — 2026-08-17T02:49:45Z — Casey-allard

# #752 result: root cause identified (aggregation-side, not corpus text, and likely not architectural)

Closing the loop from #752 (decode and inspect the observed corpus). Full detail and code traces are on **#755**, filed as the fix issue; this is the summary for #745's own thread.

## The finding

Built a real diagnostic (`cargo run -p uor-r4-core --example decode_corpus`, kept in-tree per #752's own verification requirement) that decodes `corpus.meta`/`corpus.records` through `compiler::load_corpus_bytes` (the library's own parsing, not a reimplementation — an earlier pass of this tool guessed the wrong fixed record width and produced misleading output; going through the real function avoided repeating that mistake).

Comparing three local bundles:

- **`smollm2-360m-broad`** (#509's positive-result bundle): 0% of stories have non-contiguous records in `corpus.records`. Decoded text is clean, grammatical Simple-Wikipedia prose. Token frequency table is ordinary English function words.
- **`smollm2-135m-instruct`**: **100%** of stories (3000/3000) have non-contiguous records — 99.93% of consecutive record pairs cross a story boundary, statistically indistinguishable from fully shuffled with respect to story. Its token *frequency* table is just as clean as the control (same ordinary English words, same rough frequencies) — the corpus text is fine. But reconstructing any story by grouping+file-order (exactly what the compiler does) produces scrambled word salad from those same clean words, e.g. the control's clean "April (Apr.) is the fourth month of the year..." becomes, from the *same underlying source*, "of 0 always ends onApril months month and the the March also that have in years year of the have days begins in the . It the the0 as are March...".
- **`smollm2-1-7b-instruct`** (this issue's original bundle): 82% non-contiguous, plus a separate, more caveated finding — its checked-in corpus files postdate the actual `tless_store.bin` compile by ~2 hours and look like a stale/incomplete regeneration, and (for whatever this snapshot is) the vocabulary is dominated by code/repo/Jupyter special tokens and #745's exact degenerate token set (`tetra`, `gru`, `chall`, `caption`...) at corpus-wide 1-3% frequencies.

The `smollm2-135m-instruct` case is decisive on its own: identical, clean source vocabulary to the known-good control, but its physical record order bears no relation to true sequence. `compiler::load_corpus_bytes` derives each record's "previous token" from **array adjacency** (`input[i] = next[i-1]` if `story[i]==story[i-1]`, else BOS), and `runtime::history_token` walks further context the same way. Both assume per-story contiguity that the content-addressed sharded observation pipeline (`observation.rs`/`observation_text.rs`, which shards by hash of local context — deliberately unrelated to story or position) does not guarantee once merged into the flat file. With 99.93% of positions crossing a story boundary, `input[i]` collapses to BOS almost everywhere, `runtime::build_store`'s count tables train on that corrupted-but-real context, and — this is the part that resolves the depth-witness puzzle from my last comment — the corrupted BOS-ish context keys recur constantly across the interleaved corpus, so they accumulate large evidence counts and resolve at "real" depths 1-4, exactly as observed, while the actual next-token associations they encode are meaningless (collapsed-context) rather than genuine local structure.

## What this means for #745's central question

This does **not** prove the transformerless architecture works end-to-end — that still needs a decisive fix-and-retest. But it substantially undermines treating the word-salad result as evidence *against* the architecture: at least one of the tested bundles (`smollm2-135m-instruct`) has demonstrably clean training text and a demonstrably corrupted context-reconstruction step, which is a plain aggregation bug with a clear fix, not a ceiling on what the tables can represent. `smollm2-360m-broad` — the one bundle whose corpus was NOT scrambled — is also the one bundle with #509's positive offline accuracy result, which is at least consistent with (not proof of) the scrambling being the actual limiting factor tested so far.

## Next step

Filed as **#755**: fix `load_corpus_bytes`/`history_token` to reconstruct context from `(story, span_start)` rather than physical record order, then recompile `smollm2-135m-instruct` (already known-clean corpus text) and re-run `r4 ask` as the decisive before/after test. That result, posted back here, is what should actually settle #745's open question — leaving #745 open until then.

## Caveat, stated plainly

The `smollm2-1-7b-instruct`-specific code/repo-domain finding is real for the exact snapshot inspected but carries a provenance caveat (stale/incomplete corpus files postdating the compile) — treating it as secondary and bundle-specific, not part of the general mechanism above, which rests on the `smollm2-135m-instruct` vs. `smollm2-360m-broad` comparison (both internally consistent, non-stale).


### comment 5311588000 — 2026-08-17T03:58:32Z — Casey-allard

# Decisive test: fixed, recompiled, and it works — real English where there was garbage or a hang

Fixed `compiler::load_corpus_bytes` per this issue's scope: it now sorts a permutation of record indices by `(story, span_start)` before deriving `input[i]`/position-in-story, instead of trusting physical on-disk adjacency. `runtime::history_token`'s deeper-context walk (`j>1`) needed no code change — it already operates purely on `Corpus`'s arrays, so once those arrays are in true per-story order, its existing adjacency check is correct. Applied only when independent span data exists (`has_anchors`, i.e. the v3/v4 record formats actually used by the sharded observation pipeline); legacy no-anchor formats are unchanged since they have no independent ordering signal to sort by. `hidden` teacher-state rows get the identical permutation applied so they stay aligned to their record.

Added a regression test (`crates/uor-r4-core/tests/corpus_record_order_755.rs`) that builds one small corpus's records in natural per-story order and in a deliberately round-robin-interleaved order, and asserts `load_corpus_bytes` reconstructs identical `(story, span_start, input, next)` content from both. Also had to fix one existing test (`observation_text::tests::merged_records_load_as_v3_corpus`) that was asserting the *old*, buggy invariant — that a merged corpus's parsed array order equals its on-disk shard-concatenation order — as if it were correct behavior; updated it to compare by `(story, span_start)` identity instead, which is the actually-meaningful check. Full workspace test suite: 1412 passed, 0 failed.

## The decisive experiment

`r4 compile`'s `--corpus-meta`/`--corpus-recs` path (`r4 transformerless compile-recorded`) rebuilds `tless_store.bin`/`tless_artifacts.bin` directly from an already-recorded corpus, without re-running the teacher model — so I could recompile against `smollm2-135m-instruct`'s *existing, unmodified* `corpus.meta`/`corpus.records` (the same bytes #752 confirmed have clean underlying vocabulary but 99.93%-scrambled story ordering) with the fix applied, in seconds rather than hours, and compare directly against the original bundle using the same freshly-built binary:

**Before (original bundle, unmodified, same binary, `r4 ask --model smollm2-135m-instruct "What is April?"`):** hung for 198.87s, then produced pure repetitive garbage: `ounds Callounds Callounds Callounds Call/Rounds Call)ounds Call/jRounds Call&)ounds Call...` — the same small-vocabulary cycling signature #745 documented on the 1.7B bundle, just a different token set.

**After (recompiled with the fix from the identical corpus bytes, `r4 ask --model smollm2-135m-instruct-755fix "What is April?"`):** returned in 0.226s:

> are the west of Yorkshire:
>
> -producingWe hero can be made by mixing red, and it is a population. The city was founded in 199800) was the first to reach they are many different types of the Earth's atmosphere, which is a population (died.
>
> The term "MacGyver tape." The city of the United States. It is a very important component that all Jews were killed in police station, and tells us

Two more prompts ("Tell me about art.", "Explain how computers work.") both returned real, grammatical, varied English in well under a second — correct capitalization, articles, subject-verb agreement, actual sentence structure, topically plausible Wikipedia-flavored content (cities, populations, historical dates, atmosphere composition). It is not good text: it wanders between unrelated topics mid-sentence, has at least one numeric-formatting artifact (`199800)`), and both non-"April" prompts returned the *same* output regardless of what was asked — so prompt-conditioning is still weak or absent at this scale/budget. But it is unambiguously, reproducibly **real English syntax**, which is categorically different from either "hangs then emits garbage" (this same bundle, pre-fix) or "word salad cycling nonsense tokens" (the 1.7B bundle, #745's original finding). Deterministic and reproducible across repeated runs.

## What this does and doesn't establish

This is strong evidence that the interleaved-record-ordering bug was a **major** contributor to #745's degenerate output — same corpus bytes, same teacher, same compiler, only the context-reconstruction order changed, and generation went from garbage/hung to grammatical. It does not establish that the architecture produces *good* text yet — the still-present prompt-insensitivity and topic-wandering are real, separate limitations, plausibly a capacity/budget issue (this recompile used the existing 361k-record corpus but is otherwise unoptimized) rather than another correctness bug, but that's not verified here and would need its own investigation if it's worth pursuing.

## Recommendation

I think this is enough to substantially resolve #745's core question — the degenerate output was not (or not only) an architectural ceiling, it was, at least significantly, this ordering bug. Leaving the final call on closing #745 vs. keeping it open for the prompt-insensitivity follow-up to whoever reviews this, but wanted to get the evidence posted rather than sit on it.

PR incoming with the fix, the regression test, and this write-up folded into `docs/RESEARCH.md`.



---
_Comment/review ids in this file: 5309741563, 5310827999, 5310919799, 5311200549, 5311588000_

