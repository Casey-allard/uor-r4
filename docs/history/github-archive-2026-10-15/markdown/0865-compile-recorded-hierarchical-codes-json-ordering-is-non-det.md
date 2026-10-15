# #865 — compile-recorded: hierarchical_codes.json ordering is non-deterministic (HashMap tie-break) — breaks byte reproducibility

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-20T06:53:37Z
- **Updated:** 2026-08-20T07:33:33Z
- **Closed:** 2026-08-20T07:33:33Z
- **Labels:** graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/865
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

`compile-recorded` (and the HF `compile` path) emit a **non-deterministic**
`hierarchical_codes.json`, violating the repository determinism invariant
("identical pinned inputs ⇒ identical artifact bytes; no HashMap-iteration-order
dependence in compiler outputs", `AGENTS.md`). Two fresh recompiles from the *same*
corpus and binary produce byte-differing `hierarchical_codes.json`, while every
other compiled artifact (`tless_artifacts.bin`, `tless_store.bin`,
`hamming_calibration.json`) is byte-identical.

Surfaced by the #833 preflight (`docs/attested_broad_baseline_833.md`): it blocks
#833's acceptance criterion "two deterministic builds reproduce the claimed byte
identities."

## Reproduction

```sh
r4 transformerless compile-recorded \
  --corpus-meta .uor-models/compiled/smollm2-360m-broad/corpus.meta \
  --corpus-recs .uor-models/compiled/smollm2-360m-broad/corpus.records \
  --vocab-size 49152 --out /tmp/a
# repeat into /tmp/b
cmp /tmp/a/hierarchical_codes.json /tmp/b/hierarchical_codes.json   # differ
cmp /tmp/a/tless_artifacts.bin     /tmp/b/tless_artifacts.bin       # identical
```

## Root cause

`crates/uor-r4-core/src/transformerless/compiler.rs`, `induce_hierarchical_codes`:
`transition_counts` is a `HashMap<Vec<u32>, usize>`; `frequent_paths` is sorted by
`sort_by_key(|e| Reverse(e.1))` — count only. Count is not a total order, so tied
paths retain HashMap iteration order (Rust `RandomState` is seeded per process), and
`.take(100)` then selects a run-dependent subset/order. `token_type_prefixes` already
uses a `BTreeMap` and is fine; only `relational_prefixes` is affected.

## Fix

Impose a total order on the tie-break — sort by count descending, then by path bytes
ascending — so the top-100 selection is a pure function of the input multiset:

```rust
frequent_paths.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
```

Plus a regression test pinning the tie-break, and a re-verification that two fresh
`compile-recorded` runs now produce a byte-identical `hierarchical_codes.json`.

## Scope / non-goals

Compiler-side determinism only. No runtime-kernel, format, or serving-semantics
change; no numeric/quality change (the *set* of frequent paths is unchanged — only
the previously-ambiguous ordering among ties becomes fixed).


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

