# #249 — Dormant algebra modules (Bott-Fock, Cayley-Dickson, Lie-Jordan, quantum_cover): activate behind flags or record dated deferrals — no removal

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T03:53:02Z
- **Updated:** 2026-07-28T18:05:23Z
- **Closed:** 2026-07-28T18:05:23Z
- **Labels:** documentation, enhancement
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/249
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
Several algebra modules compile, pass their own unit tests, and feed nothing in the inference path:
- `uor-r4-core/src/transformerless/bott_fock.rs` — O(1) unbounded-context fold (16×16 decaying state): the one existing candidate for principled context beyond the hard 8-token window; referenced only by smoke tests.
- `cd_space.rs` / `endomorphism.rs` / `lie_jordan.rs` — Cayley-Dickson/octonion and Lie-Jordan machinery from the Furey plan (`docs/r4_furey_quantum_geometric_plan.md`), self-contained, uncalled.
- `uor-r4-graph-compiler/src/quantum_cover.rs` — Phase 5 density-operator clustering, zero callers outside its own file/tests; present in the stage DAG registry but not the compile path.

These modules are **kept deliberately** — they are planned capability, not cruft, and are not to be removed. The problem is documentation honesty: the Furey plan reads as committed architecture while its phases are dormant, which misleads readers about what is load-bearing.

### Scope (activation-or-deferral, no removal)
1. **Bott-Fock**: wire behind a feature flag as an alternative context source and measure vs the 8-token baseline on the D3 corpus — a natural companion to the #234 eval-design work, since long-range context is exactly what exact-context lookup cannot fake.
2. **Cayley-Dickson / Lie-Jordan / quantum_cover**: for each, record a dated deferral note in the Furey plan doc (which plan phase activates it, what gate it waits on), so docs match code reality.
3. Disambiguate the two "Cayley-Dickson"s in the codebase: the dormant real algebra (above) vs the live golden-ratio hash tie-breaker (`uor-r4-graph-runtime/src/cayley_dickson.rs`, used in `syntactic_morphism_score`) — a naming/doc note preventing confusion.

### DoD
Each module has either (a) a caller behind a flag plus a measurement, or (b) a dated deferral note in the plan doc. Nothing deleted.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

