# Auditable geometric-coordinate provenance (#259)

Issue #246 made the geometric state content-bearing and replaced the spectral
window/eigenvalue stubs with the sparse QR projection. This change makes each
stored coordinate auditable without adding work to the prediction kernel.

Every newly indexed `CorpusItem` records three UOR JSON addresses:

- `source_kappa`: the canonical sentence-text axis;
- `projection_kappa`: the versioned 16-window sparse-QR configuration,
  including dimensions, sample count, zeta-zero set, and query blend; and
- `vocabulary_kappa`: the sorted word-to-prime assignments used for that
  sentence's content vector.

The addresses are produced through `uor-addr::json`, never from a raw Blake3
digest. `verify_corpus_provenance` recomputes all three axes for an identity
scope and fails closed on missing or mismatched evidence. Geometric resonance
results expose the stored chain through their optional `provenance` field.

The fields are optional on deserialization so old exported router states remain
loadable. Verification intentionally rejects those legacy entries as missing
evidence; re-indexing is required to obtain an auditable store.

Validation:

```bash
cargo test -p uor-r4-router --offline --test provenance
```
