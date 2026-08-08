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

## Correction (appended 2026-08-08, #487) — the Spectral row is lexical, not geometry

The figures above stand as measured; the *interpretation* of the Spectral row
does not. This block is appended rather than rewritten so the record shows what
was believed and when.

**What the 0.7179 actually measures.** `retrieve_geometric_resonance` ranks by
`shared_count * 100 + sim * slice_norm + scope_boost`. #484 and #486
subsequently established, on this same corpus, the same caps (2,000 construction
stories, 500 probes, `TOP_N = 20`), the same probe form, and through this same
function, that the cosine term `sim` is **at chance** on this path — the target
sits at median rank 21,082 of 46,342 even when the probe is the exact stored
sentence, against a random median of 23,171. #486 traced the cause: the query
vector is built from the *routing* path while stored vectors are the *content*
vector, so the two sides are different objects and their cosine is noise by
construction. With the lexical `shared_count` term dominating by a factor of
100, the Spectral arm's 0.7179 is **word overlap, not spectral geometry.**

**So this ablation compared (lexical ranking + a dead cosine) against (an empty
index).** It isolated the `set_geometry_type` switch correctly — that part is
sound — but it told us nothing about either geometry's retrieval quality,
because neither geometry's cosine was contributing. The VSA arm's `0.0000` is
still a real wiring fact (`index_corpus` never populates `facet_store`); the
Spectral arm's non-zero score is the lexical term both arms would have shared
had VSA's index not been empty.

**The tell we missed.** #484 and #480 each re-derived the identical triple
`0.6240 / 0.7179 / 0.9720` under their own framings without anyone noticing it
was one measurement three times. Identical metric triples across supposedly
different arms are a signal, not a coincidence; the cheap guard is to pin a
known reference row and assert against it, as #484's harness now does.

**Consequence for the disposition above.** Choosing between Spectral and VSA
(the "What should follow" section) is premature: the deployed geometry's own
cosine term is at chance, so there is no geometry-quality signal to compare yet.
The `set_content_query_vector` fix from #486 (adoption gated as #490) is the
prerequisite — once the cosine is comparing like objects, this ablation is
worth re-running. The VSA-wiring disposition is re-opened with this corrected
framing as a follow-up issue.
See [geometry_selfmatch_486.md](geometry_selfmatch_486.md) and
[lexical_weight_484.md](lexical_weight_484.md).

## Correction (appended 2026-08-08, #493) — the VSA zero is a scoring mismatch, not an empty store

The "wiring fact" repeated above — *`index_corpus` never populates `facet_store`* —
is **wrong on current code**, and #493 measured it directly. Both the original
#434 record and the #487 correction block restated it; this block supersedes
that claim. The figures still stand; the *cause* of the VSA `0.0000` was
misattributed.

**`index_corpus` DOES populate the facet store.** It calls
`index_sentence_internal` → `index_sentence_routed`, which grounds every
sentence through `VsaGeometry` and calls `index_semantic_object`
(`crates/uor-r4-router/src/lib.rs`). Probed on a five-sentence corpus in VSA
mode: `facet_store.type_index` has five keys, `entity_index` two — populated, not
empty. So VSA retrieval returns the correct candidate **set** from the facet
intersection.

**The zero comes from the scorer, not the store.** `retrieve_vsa_multi_facet_resonance`
ranks each candidate with `cosine_similarity(query, stored)` where the query is
the **1024-dim VSA hypervector** and `stored` is the item's **512-dim spectral
content vector**. `cosine_similarity` returns exactly `0.0` on a length mismatch
(`crates/uor-r4-core/src/lib.rs`), so **every candidate scores `0.0`** and the
ranking is dead — the same category error as #486 (comparing two different kinds
of object), here guaranteed exactly zero by a dimension check rather than left
at chance.

**And a commensurable comparison does not rescue it.** Re-grounding each
candidate's text to its own 1024-dim hypervector and comparing like with like
makes the relevances non-zero — but the ranking is still at chance: a query of
*"the quick brown fox jumps over the lazy dog"* ranked the fox sentence **last**.
`VsaGeometry::ground` builds the hypervector by hashing the exact content string
(`expand_atom`), so two related sentences get unrelated random ±1 vectors — the
grounding is a **content-hash placeholder, not a semantic encoder**. There is
nothing semantic to rank by.

**Disposition (#493).** Option (a) from "What should follow" — connect the store
and re-run — is already half-done (the store is connected) and the remaining
half does not pay: VSA has no semantic encoder, so its retrieval cannot rank
regardless of the scoring wiring. Option (b) — make the switch honest — is the
right call and is the cheap one: `set_geometry_type("vsa")` now warns loudly that
VSA retrieval does not rank by content similarity, and
`tests/vsa_scoring_honesty.rs` pins both corrected facts (facets populated,
scoring degenerate). Whether VSA should get a real encoder or be deprecated is an
engine-design decision, filed as #496 for the engine owners. Until then,
Spectral (now with the #490 content-vector query) is the semantic retrieval path.
