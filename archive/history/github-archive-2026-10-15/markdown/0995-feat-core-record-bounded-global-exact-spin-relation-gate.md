# #995 — feat(core): record bounded-global exact-spin relation gate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-28T19:45:49Z
- **Updated:** 2026-10-09T14:17:23Z
- **Closed:** 2026-08-28T19:46:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/995
- **Merged:** 2026-08-28T19:46:59Z
- **Merge commit:** 8fea81b3e5943bae39b6f621af11eb93494e5a84
- **Base / Head:** main / UOR-Foundation:issue-973-bounded-global-exact-spin-v1
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- add a read-only canonical global exact-spin snapshot view and detached lower/snapshot artifact boundary
- implement the frozen bounded-global exact stored-spin left-fold operator with exact class reuse and matched controls
- preserve the binding target-free negative instead of opening decoded targets or tuning the fixture
- reconcile current #973 programme claims and append the full evidence record

## Observed decision

The frozen pair failed the target-free relation gate. `Pavel` and `helix` map to the same exact H4 root and `prism` maps to identity, so both swapped orders finish at the same complete `-1` state with identical fiber/torsion. Real roles are `helix/helix`; permuted roles are `prism/prism`.

Terminal:

`RETAIN_CONVERSATION_ONLY_REDESIGN_BOUNDED_GLOBAL_EXACT_SPIN_RELATION`

Target preimage loads are zero and decoded evaluation is `NOT_RUN`. This preserves the retained paragraph/conversation results but does not establish bounded-global attention. #973 remains open for a newly frozen noncommuting relation/population repair; corpus induction is not authorized and #954 remains blocked.

## Evidence

- operator, 3,228 bytes: `blake3:f6b36cdf3e6cf96e1e9a345980843ee9eaffd25f5b864d4b4ed45a30ae6f746f`
- target-free census: `blake3:6c0a9f89a29584a09d917ae427a494b53c06b76e56482f665870ae86c1cd130a`
- focused test: 1 passed, 0 failed, 210.11s
- support mismatches: 0
- work mismatches: 0
- target loads: 0

## Verification

- `cargo test -p uor-r4-core --test bounded_global_exact_spin_attention_973 --offline -- --nocapture`
- `cargo check -p uor-r4-core --offline`
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib --offline`
- `cargo fmt --all -- --check`
- `python3 scripts/check_claim_wording.py`
- `git diff --check`

The full strict clippy command was attempted and stopped on pre-existing warnings in untouched model-source, graph-runtime, and core modules; no diagnostic named the new bounded-global module or test. Workspace-wide, BDD, model, no-std, Gate C, kappa, audit, fuzz, conformance, corpus, product, performance, formal, and release suites remain `NOT_RUN` under the frozen decision boundary.

References #973

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

