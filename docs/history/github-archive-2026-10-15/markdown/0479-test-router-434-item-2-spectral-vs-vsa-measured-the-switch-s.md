# #479 — test(router): #434 item 2 — Spectral vs VSA measured; the switch silently disables retrieval

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-07T14:38:47Z
- **Updated:** 2026-08-07T15:07:01Z
- **Closed:** 2026-08-07T15:02:57Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/479
- **Merged:** 2026-08-07T15:02:57Z
- **Merge commit:** 3cfa8d701856236cbd6ab7789284417ab44fe912
- **Base / Head:** main / UOR-Foundation:measure/434-geometry-ablation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #434 (item 1 shipped as full-width storage in #465; this is item 2, the last one).

## What was there

`run_ablation_benchmark` + `tests/ablation_benchmarks.rs` were the only coverage these types had, and they **could not fail**:

- `migration_agreement: 0.98` was a hard-coded literal (`// high consistency score`); the test asserted `migration_agreement == 0.98`
- `unlearning_time_ns` was introduced by `// Measure unlearning latency (deleting a route)` but timed `geometry.ground(obj)`. Nothing was deleted
- `recall_at_3` and `hits_at_3` were the same quantity — same branch, same divisor
- the remaining assertions were `>= 0.0` on ratios of non-negative counts

Two synthetic queries, no corpus, no ground truth, no null.

## Result

500k fixture, 2,000 construction stories → 46,342 windows, 500 held-out probes, through `get_top_resonances_native` — the production retrieval surface. Corpus/window/probe/null laws taken from `zeta_state_retrieval.rs` (#439–#440).

| arm | top-1 | MRR | recall@20 | deranged-key MRR |
|---|---:|---:|---:|---:|
| `Spectral` (default) | 0.6240 | **0.7179** | 0.9720 | 0.0000 |
| `Vsa` | 0.0000 | **0.0000** | 0.0000 | 0.0000 |

Null dead for both arms, so Spectral's number is content matching. **Spectral wins by 0.7179 MRR against a pre-declared 0.05 margin.**

## Why VSA is exactly zero rather than merely worse

The arms read **different stores**:

- `Spectral` → `retrieve_geometric_resonance` → `corpus_index_by_identity`
- `Vsa` → `retrieve_vsa_multi_facet_resonance` → `facet_store.*_index`

`index_corpus` — the production bulk ingestion surface — populates the former and **never touches the latter**. They are disjoint subsystems presented as a like-for-like config switch, so **a caller who sets `set_geometry_type("vsa")` after `index_corpus` silently loses retrieval**.

Stated as a wiring fact, not a judgement on VSA grounding. Two dispositions follow and this record deliberately picks neither, because it is a design decision rather than a measurement:

1. **Connect them** — have `index_corpus` populate the facet store, re-run this harness, and VSA gets a real number.
2. **Make the switch honest** — if VSA is only for the object-indexing path, say so, and refuse or warn rather than return zero-relevance results.

What is not defensible is leaving a config switch that silently disables retrieval with no test covering it.

## Two traps in building the replacement

Both produced a clean, plausible, entirely false *"measured indifferent"* verdict over two zeros:

- **Retrieval is identity-scoped** — querying under fresh per-probe identities returns an empty index, so both arms score zero.
- **`ResonanceResult::window_index` is not a store index** — it is a routing-window identifier and several stored sentences share one value, so ranking by it also scores zero for both arms. Ground truth is matched on the stored sentence, as `aligned_vectors` does.

The rule worth keeping: **an all-zero result across every arm is a harness bug until proven otherwise**, and an instrument that cannot fail is indistinguishable from one that passes.

## Found on the way — filed separately

`retrieve_geometric_resonance` builds a **band-only query projection** (`full_state[start..end].copy_from_slice(...)`, zero elsewhere) and compares it against stored vectors that are **full-width by default since #465**. So the adopted de-banding gain (MRR 0.2348 → 0.8948, router anchor accuracy 9.3% → 11.4%) is not reaching the production retrieval surface. Flagged in the #465 scope note, confirmed here, filed as its own issue — it is a larger prize than this one was.

## Gates

`cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `check_claim_wording.py`, `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`, full `uor-r4-router` suite, and dependent builds all green. The new harness is `--ignored` (~4 min); the repaired wiring check runs in CI.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

