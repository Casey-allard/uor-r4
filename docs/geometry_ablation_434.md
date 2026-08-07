# Spectral vs VSA: the geometry switch is not a like-for-like choice

Issue #434 item 2. Measured 2026-08-07 on the committed 500k fixture corpus,
2,000 construction stories → 46,342 distinct eight-token windows, 500 held-out
probes, through `get_top_resonances_native` — the production retrieval surface.

Reproduce with:

```
cargo test --release -p uor-r4-router --test geometry_ablation -- --ignored --nocapture
```

About four minutes; no teacher, no checkpoint.

## Result

| arm | top-1 | MRR | recall@20 | deranged-key top-1 | deranged-key MRR |
|---|---:|---:|---:|---:|---:|
| `Spectral` (default) | 0.6240 | **0.7179** | 0.9720 | 0.0000 | 0.0000 |
| `Vsa` | 0.0000 | **0.0000** | 0.0000 | 0.0000 | 0.0000 |

The deranged-key null — every probe graded against its neighbour's target,
same store, same queries, same retrieval calls — is dead for both arms, so the
Spectral number is content matching and not an artefact of the answer key.

**Spectral wins by 0.7179 MRR against a pre-declared 0.05 margin.** But the
interesting part is *why* the VSA arm is exactly zero rather than merely worse.

## The mechanism: the two geometries read different stores

`get_top_resonances_native` dispatches on `geometry_type`:

- `Spectral` → route, then `retrieve_geometric_resonance`, which reads
  `corpus_index_by_identity[identity_key(identity)]`;
- `Vsa` → `retrieve_vsa_multi_facet_resonance`, which reads
  `facet_store.{type,entity,relation,temporal,intent,provenance}_index`.

`index_corpus` — the production bulk ingestion surface, and the only one used
to load a corpus — populates `corpus_index_by_identity` and **never touches
`facet_store`**. The facet store is populated by a separate object-indexing
API.

So the two arms are not two ways of searching one store. They are two disjoint
subsystems, and the config switch between them is presented as a like-for-like
choice. Under corpus ingestion, `Vsa` returns entries with relevance `0.0000`
and no relation to the query.

This is stated as a wiring fact, not as a judgement on the VSA design: nothing
here says VSA grounding is a bad idea, only that **a caller who sets
`set_geometry_type("vsa")` after `index_corpus` silently loses retrieval.**

## What should follow

Two defensible dispositions, and this record does not pick between them
because the choice is a design decision rather than a measurement:

1. **Connect them** — have `index_corpus` populate the facet store, and
   re-run this harness. Then the switch means what it appears to mean and VSA
   gets a real number.
2. **Make the switch honest** — if VSA is only intended for the object-indexing
   path, `set_geometry_type("vsa")` should say so, and a `Vsa` router that has
   only ever seen `index_corpus` input should refuse or warn rather than return
   zero-relevance results.

What is not defensible is leaving a config switch that silently disables
retrieval with no test covering it. That is the state this issue found.

## The instrument that was there before

`benchmark::run_ablation_benchmark` and `tests/ablation_benchmarks.rs` were the
only coverage these types had, and they could not fail:

- `migration_agreement: 0.98` was a hard-coded literal (`// high consistency
  score`); the test asserted `migration_agreement == 0.98`;
- `unlearning_time_ns` was introduced by `// Measure unlearning latency
  (deleting a route)` but timed `geometry.ground(obj)` on the first query.
  Nothing was deleted;
- `recall_at_3` and `hits_at_3` were the same quantity — both incremented in
  the same branch, both divided by the query count;
- the remaining assertions were `>= 0.0` on ratios of non-negative counts.

Trimmed to what is actually computed (`recall_at_3` plus a `queries` count, so
an empty run cannot masquerade as a measured zero), re-scoped in its docs as a
*wiring check* rather than a retrieval measurement, and given a test that can
fail.

## Two traps in building the replacement, recorded because both look like findings

Both produced a clean, plausible, entirely false "measured indifferent" verdict
over two zeros:

- **Retrieval is identity-scoped.** Querying under fresh per-probe identities
  returns an empty index, so both arms score zero. A probe must query under the
  identity the corpus was indexed with.
- **`ResonanceResult::window_index` is not a store index.** It is a
  routing-window identifier and several distinct stored sentences share one
  value, so ranking ground truth by it also scores zero for both arms. Ground
  truth is matched on the stored sentence, as `aligned_vectors` does in
  `zeta_state_retrieval.rs`.

The general rule this session keeps re-learning: **an all-zero result across
every arm is a harness bug until proven otherwise**, and an instrument that
cannot fail is indistinguishable from one that passes.

## Scope limits and one thing found on the way

- Measured on synthetic word renderings of token ids (`t00042`), the standing
  #422/#423 law for this stack — the corpus carries no text.
- Spectral's 0.7179 MRR here is below the 0.8948 content-full-width reference
  from #440. Part of that gap is a different probe/window regime, and part is
  likely the query-side banding below.
- **`retrieve_geometric_resonance` builds a band-only query projection**
  (`full_state[start..end].copy_from_slice(...)`, zero elsewhere) and compares
  it against stored vectors that are now full-width by default since PR #465.
  The cosine therefore only ever sees the band, so the adopted de-banding gain
  (MRR 0.2348 → 0.8948, router anchor accuracy 9.3% → 11.4%) is not reaching
  the production retrieval surface. Flagged in the #465 scope note, confirmed
  here, and filed separately — it is a larger prize than this issue was.
