# #763 — Add opt-in weighted sampling to break the #759 attractor-basin collapse (#762)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T13:25:07Z
- **Updated:** 2026-10-09T14:17:10Z
- **Closed:** 2026-08-17T13:45:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/763
- **Merged:** 2026-08-17T13:45:39Z
- **Merge commit:** 29f8cd99c58a91ad3bdbf8cb2aa58d287c2c10d9
- **Base / Head:** main / UOR-Foundation:i762-lever2-sampling
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Add opt-in weighted sampling to break the #759 attractor-basin collapse (#762)

#759 found that greedy argmax decoding on the legacy plain-TLA path
locks onto the single highest-count continuation whenever multiple
prompts' trailing-token windows quantize to the same (or a
sufficiently similar) compiled code -- not a fallback-tier bug, but a
genuine argmax "attractor basin" effect (#460's already-open
codebook-fit thread is the deeper root; this is the decoding-side
lever). This lands lever 2 from that issue: weighted sampling over the
stored count distribution, as an opt-in alternative to always taking
the argmax winner.

- `SampleRng` (xorshift32) and `reduce_into_range` (restoring binary
  long division) in runtime.rs give a division-free, P-4-clean
  selection primitive -- zero `*`/`/`/`%` tokens, verified against the
  real `%` operator by an external 200k-iteration random sweep plus
  exhaustive small-value and edge-case tests (outside the P-4 scan
  file, so they're free to use real modulo as the oracle).
- `Runtime::predict_witness_sampled`/`generate_sampled_into` are
  additive siblings of `predict_witness`/`generate_greedy_into`:
  same repetition-penalized scoring, but draw proportionally from the
  resolved depth's full candidate distribution instead of always
  taking the max. Every existing caller is untouched -- nothing reaches
  the new functions unless a caller explicitly opts in.
- `ChatEngineBuilder::sample_seed`/`r4 ask --sample <SEED>`/
  `r4 chat --sample <SEED>` thread an opt-in, reproducible seed into
  the legacy generation path only. The R4G1 beam-search path (used
  automatically whenever a manifest carries `compiled.r4g1`) is
  unaffected -- out of this issue's scope, noted explicitly rather than
  silently expanded.

Measured effect (the decisive test, following #758's own
methodology): 15-prompt comparison (12 real questions + 3 edge cases)
against `smollm2-135m-instruct-tla6` (no `compiled.r4g1`, so this
exercises the legacy path end to end). Deterministic baseline: 15
prompts collapse to 13 distinct outputs (one group of 3 byte-identical
answers). With `--sample 42`: 15 prompts produce 15 distinct outputs --
zero exact-match collisions. Sampling does not fix the underlying
codebook-quantization root cause (#460's open thread), and doesn't
improve grammaticality on this already-degenerate bundle, but it does
eliminate the exact-repeat collapse #758/#759 measured, which is what
this lever set out to do.

Verification:
- cargo fmt --check
- cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
- cargo run -p xtask -- audit-limits
- cargo test --workspace --offline (full suite, no failures)
- allocation_census.rs re-verified passing: the new sampling functions
  use Vec/heap allocation but are never called from within the
  default (measured) generate_greedy_into path, so the allocation-free
  guarantee on the default path is untouched
- p4_runtime_source_scan / p4_contract_owned_graph_runtime_source_scan
  both pass with the new code in place
- Real 15-prompt before/after collapse-rate measurement against a live
  local bundle (see above)

New tests:
- crates/uor-r4-core/tests/sampling_reduction_762.rs (6 tests):
  reduce_into_range exhaustively verified against real `%`
- crates/uor-r4-core/tests/sampling_prediction_762.rs (6 tests):
  predict_witness_sampled/generate_sampled_into determinism-per-seed,
  variation-across-seeds, evidence-set containment, and non-effect on
  the existing default path

Closes #762

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

