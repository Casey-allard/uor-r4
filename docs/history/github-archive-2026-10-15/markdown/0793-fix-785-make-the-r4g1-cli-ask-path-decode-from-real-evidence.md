# #793 — fix(#785): make the R4G1 CLI ask path decode from real evidence

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T17:48:47Z
- **Updated:** 2026-10-09T13:08:57Z
- **Closed:** 2026-08-18T18:10:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/793
- **Merged:** 2026-08-18T18:10:09Z
- **Merge commit:** 2576572c32abf1e97e38e2e4ef24af43b13d917b
- **Base / Head:** main / UOR-Foundation:issue-785-ngram-row-candidates
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #785 (C1b + C1c). References #783 (feeds the A4 quality read).

## What was wrong

`r4 ask --model smollm2-135m-instruct` produced byte-identical
degenerate output (`regarding regarding ...`) even on the freshly
recompiled post-#755 bundle (#783-A1), while the same bundle's plain
transformerless path produced grammatical English. Instrumenting the
beam showed one candidate per step carrying a near-saturated score
(~2,147,356,480 raw ~= i32::MAX), decaying by exactly the 350x recency
step. Dumping the artifact explained it:

- `compiled.r4g1` (converter carryover) has **zero per-node emission
  entries** across all 1,025 nodes, and its EXCT section is the raw
  TLS1 store container (31 MB, magic `TLS1` at the descriptor
  boundary) -- by design ("EXCT: descriptor + the raw TLS1
  carryover", `convert_r4g1.rs`).
- The engine's node-0 fallback preferred EXCT and read those container
  bytes as (token, score_q) pairs; ~20% of 3.9M pseudo-pairs pass the
  token filter, several at near-i32::MAX raw. The suffix-DFA's
  emission oracle (`check_node_emits`) did the same whole-section scan
  per query. Garbage drowned every real tier and cost ~35 s per ask.
- The scored graph `graph/score.r4g1` (36/37 nodes with real
  emissions, real context rows, RX1 residual EXCT) sat next to it,
  already preferred by the HTTP server (`server.rs` `primary_graph`)
  and the release packager (`GRAPH_RELATIVE_PATH`) -- but the CLI
  loader never looked at it.

## The change

- **C1c (loader)**: `read_preferred_chat_graph` -- both CLI chat
  loaders (manifested + local-compiled) now prefer
  `graph/score.r4g1`, falling back to `compiled.r4g1`. This aligns
  the CLI with the server/packager artifact preference; it does not
  build #655-C1e's shared loader stack.
- **C1c (engine)**: EXCT is no longer read anywhere in the runtime
  engine -- both writers emit a storage descriptor + store container
  there, never a pair list. The whole-EMIT-remainder fallbacks
  (node-0 source, last-resort emit) are gated to the converter flavor
  (`has_per_node_emissions == false`), where the remainder really is
  the global root-prior table. On a scored graph an unmatched query
  now returns an honest miss instead of scored header bytes.
- **C1b (engine)**: a context-row hit surfaces the winning row's
  alternative entries into the candidate walk, so beam search is real
  on the n-gram tier (previously: single winner, greedy in effect).

## Evidence

- Seeded-violation tests (`exct_container_bytes_are_never_scored_as_
  emission_pairs`, `scored_flavor_reads_only_per_node_emission_
  ranges`) poison EXCT/EMIT with valid-looking near-MAX pseudo-pairs;
  against the previous engine both fail by emitting the poison
  (4242 / 49000); with this change the poison cannot surface.
  `ngram_hit_surfaces_row_alternatives_as_candidates` pins C1b (also
  fails pre-change: `got 1` candidate).
- 15-prompt protocol (#758 set), canonical refreshed 135m bundle,
  default R4G1 ask path, deterministic double pass (30/30
  byte-identical):
  - pre-fix: 15/15 prompts -> one byte-identical `regarding`
    repetition, ~35 s per ask;
  - post-fix: 5/15 distinct outputs with grammatical, prompt-form
    openers ("What is the capital of France?" -> "It is the capital
    of the 19...", "hello" -> "What is known as the 19...",
    "Once upon a time, there was a little" -> "more than 2000..."),
    ~1 s per ask; 11/15 converge to "The word comes from the 19..."
    and every trajectory ends in a "19"-digit/comma attractor.
- The remaining attractor is the #759-corrected collapse on real
  evidence (year/number n-grams dominating continuations); it is
  #784's instrument target and #785-C4's decode-policy experiment,
  deliberately not patched ad hoc here. The beam's consecutive-only
  repeat penalty never fires on the digit/comma cycle -- same evasion
  class as the earlier whitespace alternation; that observation is
  recorded for C4.

Gates run locally: fmt, clippy --workspace --all-targets
--all-features -D warnings, workspace tests (--no-fail-fast), wasm32
lib build, R-register conformance. Latency on the canonical ask path:
~35 s -> ~1 s.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

