# Proof-carrying inference (#266)

R4G1 generation can include an optional, independently replayable witness.
The feature is enabled by sending `"include_witness": true` to
`POST /api/r4g1/generate`:

```json
{
  "window": [5],
  "max_tokens": 4,
  "include_witness": true
}
```

The response adds one `witness` row per emitted token:

```json
{
  "token": 10,
  "region_kappa": "kappa:blake3:…",
  "region_id": 0,
  "depth": 1,
  "resolution_status": "graph",
  "engine": "r4g1",
  "widened": false
}
```

`depth` is the length of the covered refinement chain. `region_id` is the
zero-based NODE-section region id; exact-context and novel results have no
covered region and therefore use `null` for both region fields. The compact
κ is derived from the artifact κ, the canonical NODE-section κ, and the
region id. A changed artifact, section, or node cannot reuse that claim.
Standalone exported region objects and resolver-backed manifests remain the
follow-up tracked by #263.

To verify a response, post its `seed`, emitted `tokens`, and `witness` array
to `POST /api/uor/verify`:

```json
{
  "seed": [5],
  "tokens": [10],
  "witness": [
    {
      "token": 10,
      "region_kappa": "kappa:blake3:…",
      "region_id": 0,
      "depth": 1,
      "resolution_status": "graph",
      "engine": "r4g1",
      "widened": false
    }
  ]
}
```

The verifier recomputes the signature, scorer result, selected chain, status,
and token against the loaded artifact. Tampering produces a typed reason such
as `witness_region_mismatch`, `witness_depth_mismatch`, or
`witness_status_mismatch`. Witness generation and verification use the
allocating reference scorer server-side; the default inference path and its
allocation-free runtime kernel are unchanged.

## Conformance vectors

The status-policy fixture covers the minimum positive and negative vectors:

| vector | expected result |
|---|---|
| valid graph witness | `verified: true` |
| depth incremented by one | `witness_depth_mismatch` |
| replaced region κ | `witness_region_mismatch` |

Run it with:

```bash
cargo test -p uor-r4-wasm-router --test status_policy \
  proof_witness_roundtrips_and_rejects_tampering --offline
```
