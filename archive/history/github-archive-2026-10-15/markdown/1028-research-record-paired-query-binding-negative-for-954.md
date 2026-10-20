# #1028 — research: record paired-query binding negative for #954

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-01T02:41:19Z
- **Updated:** 2026-10-09T14:16:57Z
- **Closed:** 2026-09-01T02:43:07Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1028
- **Merged:** 2026-09-01T02:43:07Z
- **Merge commit:** 61de4f73ee3e34bac90d29d3df9089acf05c9b88
- **Base / Head:** main / UOR-Foundation:issue-954-paired-query-binding
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implements and executes the independently frozen C1-SB5 `R4PairedQueryCandidateMatrixV1` rung for #954, then records its predeclared negative branch.

The sole MPS run fit all `56/56` construction pairs but transferred only `14/28` independently sealed pairs (`37/56` rows, `227/266` cells, `38/49` flips). It therefore terminates `FAIL_PAIRED_QUERY_BINDING_PREFLIGHT` and retires this rung without retry.

References #954

## What changed

- adds the paired-query construction/sealed/product population builder and tokenizer census;
- adds the all-layer rank-8 Q/K/V/O LoRA mechanism plus asymmetric rank-32 query/candidate binding head;
- adds deterministic one-shot preparation, immutable run contracts, exact training-view allowlists, exclusive start markers, MPS-only training, controls, replay, delta audits, and negative artifact suppression;
- adds focused mechanism/data/campaign tests;
- appends the C1-SB5 result and compact bound aggregate;
- updates the current project/roadmap/lifecycle/contributor surfaces to remove the stale paired-query proposal and preserve the source-free claim boundary.

## Frozen evidence

- implementation commit: `bd06cfaf761cfc4b2e61b38efaadc76e2c72c7c4`
- implementation tree CID: `blake3:ca3b1f0804af779151baa9e0875973aba9542cb219f6e876c85512c3365b1a29`
- run-contract CID: `blake3:992bc9bb61c3e7bb4481cee3b959db338aa68530ffe43e93ee0eac5c133d2103`
- result CID: `blake3:076242ab2bd379083ae55a22a272b3a0943b350fa301f65909e1b0ecc0d72571`
- result manifest CID: `blake3:c4a7ec5e4926cc5ede144d7f3c013940d104bb77ce26f8e201aa63c524c6a119`
- result tree CID: `blake3:0c3b0d869c45fca9924c1d76af19c9b411d8529c6ac8f9f7bf9d641d788dc188`

Controls remain informative but do not rescue qualification:

- identity-aligned row-swap trace and aggregates: bit-exact;
- mean-query ablation: `0/28`, paired rows identical `28/28`, higher loss;
- attention disabled: `0/28`, higher loss;
- deterministic replay: exact;
- intended parameter changes: 24/24 LoRA tensors and 3/3 head tensors, finite, zero non-target changes.

Product or forbidden reads were `0`. The four product pairs remain committed and unopened. No checkpoint, binding-head artifact, Rust parity, development expansion, product evaluation, or retry was emitted/run.

## Focused verification

- native unittest runner: `41/41` passed in 2.364 s;
- compact aggregate versus frozen result: `16/16` selected fields matched;
- Python compileall: passed;
- JSON parse: passed;
- diff whitespace check: passed;
- claim-wording gate: passed.

No broad workspace suite or corpus-scale run was launched. Queue CI remains the binding transport verification.

## Claim boundary

This preserves bounded evidence that ordinary source-backed causal softmax attention is query-sensitive: both query destruction and attention removal collapse the sealed score. It does not establish exact transferable binding, intrinsic geometry, a source-free runtime, transformerlessness, general generation, correctness, reasoning, browser/WASM readiness, or release readiness. #954 remains open behind #973; #955 remains blocked.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

