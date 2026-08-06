# Issue #434 — consumer audit of band-sparsity assumptions

Step (1) of the adopted de-banding order (issue #434, decision recorded on
PR #442: free top-1 8.17% full-width vs 7.64% banded, anchor accuracy
11.4% vs 9.3%, all three pre-declared conditions PASS). This document
enumerates every consumer of the stored `CorpusItem.state_vector` and
records whether it assumes band sparsity, whether full-width storage
breaks it or merely enlarges it, and the accommodation where one is
needed.

## What was banded

`UorR4Router::index_sentence_routed` (`crates/uor-r4-router/src/lib.rs`,
the issue-#245 content-bearing storage path) built the stored vector as a
512-zero buffer with only the routed window's `active_range` filled from
`RoutedResult::state_vector`:

    let mut state_vector = vec![0.0; 512];
    let s_idx = best.active_range[0] as usize;
    let e_idx = best.active_range[1] as usize;
    state_vector[s_idx..e_idx].copy_from_slice(&best.state_vector[..e_idx - s_idx]);

`active_range` is produced in `route_query_to_manifold_internal`
(same file): it is the soft-route path's `[path[1], path[2]]` when the
path has three or more entries, otherwise the hard fallback
`[(routed_idx - 1) * 32, routed_idx * 32]`. The fallback constant is the
banding constant: width 32 of 512, one sixteenth of the width, sixteen
window channels (`soft_route(&coords, 16)`, and the `"windows": 16`
capability record). On the D3 natural corpus the issue-#434 band-matched
controls (`crates/uor-r4-router/tests/zeta_state_retrieval.rs`) assert
that the realized `active_range` is always one of those sixteen 32-wide
channel ranges, so the natural-corpus store kept 32 of 512 coefficients
per sentence. On a synthetic probe corpus the soft-route path can widen
the band (measured mean span 152 of 512); the audit numbers below give
both.

The content vector itself was never banded: `content_state_vector`
returns the full 512-d L2-normalized zeta word-sum, and it was passed to
the router only to steer window choice — the storage step then threw
away fifteen sixteenths of it.

## Consumer table

| Consumer (file) | Assumes band sparsity? | Break or just bigger? | Accommodation |
| --- | --- | --- | --- |
| `sparse_vector_serde` (`router/src/lib.rs`) | Yes, implicitly: it encodes a contiguous run from first to last nonzero as `{start_idx, values}` | Just bigger. Full width yields `start_idx = 0`, `values.len() = 512`; the deserializer's `end_idx <= 512` guard still holds and round-trips exactly | None needed |
| `get_top_resonances_native` cosine retrieval (`router/src/lib.rs`) | Partially: the QUERY projection is still assembled band-only (`full_state[start..end]` from `all_routes`), then cosined against the stored item | Just bigger — no panic, both sides are 512-d. Scores shift: the stored norm now spans the full width, so `sim` shrinks uniformly within a window. Ranking is dominated by `shared_count * 100.0` anyway, with `sim * slice_norm` a secondary term | None required. Follow-up (out of #434 scope): supplying a full-width query projection here would recover the ARM BF geometry inside the production retrieval surface too |
| `get_sentence_projection` (u, v) and `get_state_4d_projection` (v_4d) on the stored vector | No | Values change (computed over 512 instead of 32 coefficients); no sparsity assumption | None needed |
| Semantic-map point serving (`corpus_index_by_identity` walks in `router/src/lib.rs`) and `index.html` | No — the browser reads `routed.state_vector` / `routed.active_range` from the ROUTING result, not stored items | Unaffected | None needed |
| `get_suggested_token_limit` stratum count, `generate_geometric_response` trajectory stratum | No — both count nonzeros of the ROUTING result's banded slice, not of stored items | Unaffected | None needed |
| `verify_corpus_provenance` | No — provenance is recomputed from sentence text, words and word-primes | Unaffected | None needed |
| `export_state` / `import_state` / `import_state_native` (JSON persistence) | No, but size is proportional to the stored run length | Just bigger, ~16x on the vector payload (numbers below) | Flagged, not blocked; the new `banded_storage` flag is serialized so an exported store records its own shape |
| wasm-bindgen API surface | No: `ResonanceResult` carries no state vector, and `CorpusItem` reaches JS only through `export_state` JSON | Just bigger | None needed |
| `tests/content_bearing_index.rs` (non-ignored) | No — asserts non-degeneracy, session independence, and a cosine inequality | Passes unchanged | None needed |
| `tests/memory_lift_ab.rs` (non-ignored) and `tests/memory_lift_corpus.rs` — ARM ONE (content-free) | **YES, indirectly.** Arm one's STORE is content-free (always banded), but its probe QUERIES are built through `index_sentence` on the same router, which is full width by default. Query and store then have different shapes and the arm's retrieval collapses | Does not panic, but silently **INVALIDATES the control**: measured arm-one MRR fell from the recorded 0.1674 to 0.0006 purely from the shape mismatch | Applied: both harnesses call `set_banded_storage(true)` on the arm-one router only. Arm one is back to exactly 0.1674 (see gate results) |
| `tests/zeta_projection_quality.rs`, `tests/hopf_retrieval_quality.rs` | No — they read stored vectors as opaque 512-d vectors (the Hopf harness slices 128-blocks, which is well defined at full width) | Just different numbers, which is the point of the change | None needed |
| `tests/zeta_state_retrieval.rs` — `content_full_and_band` | **YES.** Asserts `stored[i] == 0.0` outside `active_range`, and arm ZB is defined as "re-band the full vector" | **BREAKS** under full-width storage | Applied: the harness now calls `router.set_banded_storage(true)` before ingestion. It is the band-matched control set; its arms are only meaningful against the banded shape |
| `crates/uor-r4-graph-certify/tests/router_reconnect.rs` | No | ARM B's stored and query vectors become full width under the new default, so ARM B converges on ARM BF; ARM BF still builds its own vectors test-side and is unchanged | None needed |
| `router/src/lib.rs` unit test pinning `active_range: [64, 96]` | No — pins `RoutedResult` serde, not stored items | Unaffected | None needed |

Nothing in the shipping (non-test) code breaks. Two consumers need an
accommodation, and both are measurement harnesses whose CONTROL arms are
defined against the banded shape: the `#434` band-matched control set
(hard assertion failure) and the content-free arm one of the two
memory-lift harnesses (silent control invalidation). Both are pinned to
`set_banded_storage(true)`, which is exactly what they were always
measuring.

## Size impact

Measured on a 500-sentence synthetic store via `export_state` (serde_json
f64 formatting, ~20.3 bytes per coefficient):

| Quantity | Banded (32-wide, natural corpus) | Banded (152-wide, synthetic probe) | Full width (512) |
| --- | --- | --- | --- |
| `state_vector` JSON, one copy | ~0.65 KB | 3.06 KB (measured) | 10.78 KB (measured) |
| Per stored item, both copies (`corpus_index` and `corpus_index_by_identity` are both serialized) | ~1.3 KB | 6.1 KB | 21.6 KB |
| Whole-store `export_state` per item (vectors plus sentence, words, provenance) | ~3.5 KB | 8.69 KB (measured) | 24.19 KB (measured) |
| 46k-window store, total export | ~160 MB | ~400 MB | ~1.11 GB (measured basis) |
| 46k-window store, vector payload only | ~60 MB | ~280 MB | ~992 MB |

In-memory footprint is UNCHANGED: `CorpusItem.state_vector` was always a
512-element `Vec<f64>` (4 KB, 8 KB counting the duplicate index) in both
modes — banding only wrote zeros into it. The cost is entirely in JSON
persistence, roughly 16x on the vector payload against the natural
32-wide band. That is a real operational cost for large on-disk stores
and is the reason the banded path is retained behind a flag rather than
deleted.

## Storage change (step 2)

`UorR4Router` gains a serialized `banded_storage: bool` field, default
`false` (full width), with `banded_storage()` / `set_banded_storage()`
accessors on the wasm-bindgen impl. `index_sentence_routed` stores the
full-width content vector when one is available and the flag is off, and
otherwise falls back to the pre-#434 banded copy.

The content-free path (issue #255, arm one) supplies no content vector,
so it keeps the banded shape in both modes and its pre-#245
reconstruction stays byte-identical.

Determinism and canonical ordering are preserved: the stored vector is
`content_state_vector(sentence)`, which depends only on the sentence's
words and the arrival-ordered word-to-prime assignment — the same
quantity that already selected the window. Window bucketing, item order
within a window, and the corpus-item id sequence are untouched.

## Regression gates (step 3)

### memory-lift at corpus scale

`crates/uor-r4-router/tests/memory_lift_corpus.rs`, 46,438 windows, 500
probes, natural stack (`/tmp/c_meta.bin`, `/tmp/c_recs.bin`,
`/tmp/wiki-obs/stories.jsonl`). Log: `/tmp/deband_memlift.log`.

| Arm | Baseline (banded) | De-banded | Delta |
| --- | --- | --- | --- |
| arm 2 content-derived, MRR | 0.2348 | **0.8948** | +0.6600 |
| arm 2 content-derived, top-1 | 0.208 | **0.832** | +0.624 |
| arm 1 content-free, MRR | 0.1674 | 0.1674 | unchanged (banded by construction) |
| arm 3 shuffled control, MRR | 0.0002 | 0.0001 | flat at the noise floor |

Exit rule PASSES: arm 2 beats arm 1 by +0.7274 (need +0.020) and beats
arm 3. The content-derived MRR lands exactly on the 0.8948 the #434
band-matched controls predicted for full width.

### router reconnection

`crates/uor-r4-graph-certify/tests/router_reconnect.rs`, 91,966 store
contexts, 19,083 queries, 9,867 anchors, 29,841 free positions. Inputs:
`R4_ARTIFACTS=/tmp/old_fixture_artifacts.bin`,
`R4_SCORED_R4G1=/tmp/strict_score/score.r4g1` (the container/scored pair
whose teacher CIDs match; `/tmp/tless_artifacts.bin` has since been
overwritten by a later corpus and no longer pairs with `strict_score`).
Log: `/tmp/deband_reconnect.log`.

| Arm | Baseline free top-1 | De-banded free top-1 | Anchor accuracy |
| --- | --- | --- | --- |
| ARM A true-anchor (ceiling) | 31.5 | 31.5 | 100.0% |
| ARM B router-anchor | 7.64 (banded) | **8.17** | **11.4%** (was 9.3%, +2.1pp) |
| ARM B2 router-draft | 4.9 | 5.08 | 3.3% |
| ARM BF full-width (test-side) | 8.17 | 8.17 | 11.4% |
| ARM C unigram null | 2.14 | 2.14 | 6.0% |
| ARM D shuffled null | 3.09 | 3.07 | 1.3% |

ARM B — the PRODUCTION path — now reproduces ARM BF exactly (8.17 free
top-1, 11.43% vs 11.42% anchor accuracy, a one-anchor difference from
the second renormalization in `evolve_state`). That is the adopted
payoff landing in the shipping code: the storage surface itself now
supplies what ARM BF had to construct test-side.

Consequence for the harness's own printout: the #434 pre-declared rule
"ARM BF must exceed ARM B's same-run free top-one" compared BF against a
BANDED ARM B. With banding removed by default, ARM B is full width, so
BF minus B is +0.00pp and the harness prints "recorded NEGATIVE". That
line is now VACUOUS, not a regression — it is the two arms agreeing. The
decision-relevant comparison is the de-banded ARM B (8.17 / 11.4%)
against the recorded banded ARM B baseline (7.64 / 9.3%), which is the
+0.53pp / +2.1pp the adoption was based on. Setting
`set_banded_storage(true)` reproduces the 7.64 / 9.3 baseline.
