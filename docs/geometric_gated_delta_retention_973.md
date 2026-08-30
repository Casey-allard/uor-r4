# #973 geometric gated-delta retention construction smoke

- **Date:** 2026-08-28
- **Issue:** #973
- **Programme root:** #820
- **Decision:** [ADR-0005](adr/0005-predictive-geometric-connection-memory.md)
- **Deliverable:** `GeometricGatedDeltaRetentionR4V1` focused qualification
- **Mechanism scope:** bounded multirate last-context core
- **Evidence status:** `EXERCISED_CONSTRUCTION_SMOKE_ONLY`
- **Structural result:** `PASS_STRUCTURAL_SMOKE`
- **Geometry-specific result:** `NO_ADVANTAGE_ON_THIS_FIXTURE`
- **Decision-bearing corpus qualification:** `NOT_RUN`

## Result first

The bounded multirate last-context core is executable and its declared
structural mechanism is present. Independent compiles are byte-identical;
learned K/V/Q placements remain distinct; both learned arms apply nonzero
K/V/Q updates; prediction precedes target observation; every target belongs to
its caller-supplied support; support is unchanged; four 4x4 banks remain a
fixed 64-scalar matrix state; and the artifact binds construction-partition,
support, exact-H4, leaf-map, and policy identities.

This smoke is **not positive evidence for geometry-specific retention**. On
the independently frozen synthetic construction fixture, the full geometric
arm made 16/28 correct next-token choices and 55/112 positive deployed-readout
association margins. The matched plain-delta arm made 23/28 and 98/112,
respectively. The full arm also tied the `left_fold_route` intervention on both
aggregate counts. These are small, seen-construction measurements rather than
a transfer verdict, but they do not justify promoting a geometry advantage.

This cell is not yet the full hierarchy-fed rung described by ADR-0005. Its
four banks use different retention rates, but the read key is derived from the
last causal context. Separately typed local, short, scope, and long hierarchy
inputs have not been constructed or qualified.

No typed #953 support origin, protected held-out document, decoded autonomous
text, teacher/provider output, or runtime lowering entered this qualification.

## Frozen fixture and construction binding

The fixture contains four sorted synthetic construction sequences, 28 causal
events, and token IDs 1 through 8 within a namespace ending at 8. Each event
supplies one of four strictly sorted two-candidate supports:

```text
[2,6]  [3,7]  [4,8]  [1,5]
```

The two repeated cycles provide multiple context-dependent queries over the
same supports. Every prediction is recorded before the event's observed token
is passed to `observe`. Compilation rejects an observed target that is absent
from its admitted support.

- Fixture kappa:
  `blake3:b32a94caaa60c97f2f3df346b65ddf3d7d0e7bab81d786f599fb62c32e2762f5`
- Construction partition identity:
  `uor-r4.ggdr-973.synthetic-construction-partition/1`
- Bound construction-population kappa:
  `blake3:f2d07a091936e0619ad29db6053ac097c1469d3b9bd2ef64ff038d5dbe1e51c4`
- Synthetic matched-table binding:
  `blake3:bc94083a1632d2b262ed6e975b2140ffd21e3e2138225629e2e13c71aa0b39e8`
- Synthetic matched-overlay binding:
  `blake3:27c7cd79df7ce7bdc6a9f9c4ea62439c076077eb7dd916c983bd4cbd881f141e`

The construction-population kappa binds the partition identity, support
bindings, sorted document identities, initial tokens, ordered supports, and
observed targets. Changing only the partition identity changes both the
population kappa and model artifact.

The two support CIDs are fixture-only provenance stand-ins. They demonstrate
that the model binds and preserves caller-supplied support identity; they are
**not** typed #953 `SourceFreeTable` or `MultiscaleCountRadiusR4V1` artifact
identities.

## Artifact and exact geometry bindings

| Field | Observed value |
| --- | --- |
| Artifact bytes | 4,233 |
| Artifact CID | `blake3:6bf22c9c5283b971d8a9e5e7f4bce067424064a49394f5c7b02a2174e6f38973` |
| H4 root-table kappa | `blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76` |
| H4 product-table kappa | `blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759` |
| Exact prime-spin leaf-map kappa | `blake3:283e39f34837b574c0dd407ee74287095efce4191ca30796857f90604fdd3abb` |
| Smoke report CID | `blake3:f99b9815044f139ec0380b5a82502aaf4e159e25761626c0c25eee39173816e1` |

Two independent compiles and a compile with reversed input-document order
produced identical artifact bytes. Extending the token namespace preserved
the canonical H4 root/product identities but changed the exact leaf-map kappa
and artifact CID. Canonical bytes contain the construction population,
partition, H4, leaf-map, support, and policy identities.

The current public reference API exposes canonical `to_bytes` and artifact
CID, but no `from_bytes` decoder. Therefore byte reload and tamper-rejection
evidence is `NOT_AVAILABLE_API`, not PASS. Independent deterministic
recompilation is the replay evidence qualified here.

## Learned mechanism checks

| Check | Result |
| --- | --- |
| Geometric K/V/Q update counts | `[96, 112, 96]` |
| Plain-delta K/V/Q update counts | `[96, 112, 96]` |
| Tokens with pairwise-distinct learned K/V/Q | 9/9 |
| Recurrent matrix state | four 4x4 banks, 64 scalars |
| Prefix/corpus stored in recurrent state | none |
| Pre-observation counterfactual target mutation | choice byte-identical |
| Post-observation distinct targets | state checksums differ |
| Support mismatches across all arms/queries | 0 |
| Target outside support | rejected at compile |
| Duplicate, unsorted, or out-of-namespace support | rejected |

The fixed 64-scalar count covers recurrent matrix memory. Exact route frame,
last-token identity, and counters are additional bounded metadata; the claim
is not that the entire Rust struct contains only 64 machine scalars.

## Multi-query construction measurements

Each row answered 28 pre-observation next-token queries. Association
measurement then tested each of the 28 observed key/candidate pairs in all four
banks, yielding 112 margins per row. The measured score is the deployed
candidate readout for one bank:

```text
dot(Q(candidate), S_bank * K(key))
```

`V` is write-only in this measurement and is never substituted for candidate
`Q`.

| Arm | Event order | Next-token correct | Accuracy | Q-readout association wins | Win rate | Support mismatches |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `full_geometric` | frozen natural | 16/28 | 57.14% | 55/112 | 49.11% | 0 |
| `plain_delta` | frozen natural | 23/28 | 82.14% | 98/112 | 87.50% | 0 |
| `no_delta_overwrite` | frozen natural | 15/28 | 53.57% | 58/112 | 51.79% | 0 |
| `transport_permuted` | frozen natural | 15/28 | 53.57% | 62/112 | 55.36% | 0 |
| `left_fold_route` | frozen natural | 16/28 | 57.14% | 55/112 | 49.11% | 0 |
| `last_only` | frozen natural | 15/28 | 53.57% | 52/112 | 46.43% | 0 |
| `order_shuffled_events` | pre-bound permutation | 17/28 | 60.71% | 54/112 | 48.21% | 0 |

Relative to the full geometric arm, plain delta is +7 correct choices out of
28 (+25.00 percentage points) and +43 Q-readout association wins out of 112
(+38.39 percentage points). Because this is a small construction fixture, the
result does not falsify transfer on an independent natural population. It does
show that this execution supplies no geometry-specific advantage and that
plain recurrence remains a binding comparator for the next qualification.

`left_fold_route` is a route-composition-law intervention. It is not an event-
order shuffle. The separate harness-only `order_shuffled_events` row applies
the pre-bound within-document permutation `[2,5,0,3,6,1,4]`, then rebuilds
causal state in that permuted order. Its frozen permutation kappa is:

`blake3:1ec330fbff9d8b644c7c1f69afc48dcb6343b38665946412be18454528a0db11`

All seven rows produced distinct final-state checksum vectors, and all four
documents within each row produced distinct final states. Aggregate equality
between full geometry and `left_fold_route` is therefore a measured outcome,
not an identical execution.

## Decision and next gate

This record qualifies mechanism presence, boundedness, causal ordering,
construction binding, deterministic compilation, and matched support for the
bounded multirate last-context core. It does **not** qualify predictive
transfer or the full hierarchy-fed connection-memory rung.

The next decision-bearing qualification is not a larger or tuned recurrence
run. It is `DirectCausalGeometricAttentionR4V1`, the missing dense offline
reference with learned Q/K/V/O, S3 tangent projection, causal H4-frame
transport, stable softmax, and transported value aggregation. This synthetic
fixture must not be tuned after observing the plain-delta advantage.

Softmax is an oracle, not the target runtime. If direct geometric attention
qualifies on an independently frozen construction-validation population, the
next rung replaces only its weighting law with a fiber-preserving
multi-resonance sieve. Only after that replacement preserves the reference
effect is this bounded gated-delta core revised and measured as its recurrent
factorization.

## Reproduction

```bash
cargo test -p uor-r4-core --offline \
  --test geometric_gated_delta_retention_973 --no-fail-fast -- --nocapture
```

Observed focused result: 3 passed, 0 failed, 0 ignored. The test prints the
canonical JSON smoke report and its BLAKE3 CID.

## Evidence ledger

| Evidence item | Status |
| --- | --- |
| Bounded multirate last-context core | `EXERCISED_CONSTRUCTION_SMOKE_ONLY` |
| Separate hierarchy inputs | `NOT_RUN` |
| Typed #953 support origin | `NOT_RUN`; synthetic matched supports only |
| Independent natural construction validation | `NOT_RUN` |
| Direct causal geometric-attention reference | `NOT_RUN` |
| Multi-resonance softmax replacement | `NOT_RUN` |
| Protected held-out language-model evaluation | `NOT_RUN` |
| Decoded generation | `NOT_RUN` |
| Integer/table runtime lowering | `NOT_RUN` |
| Artifact byte decoder/tamper rejection | `NOT_AVAILABLE_API` |

## Claim boundary

This smoke does not establish geometric attention, hierarchy-fed retention,
natural-language transfer, syntax or semantics, paragraph/conversation/global
attention, autonomous generation, ChatGPT-level capability, held-out
correctness, reasoning, integer/table runtime legality, allocation behavior,
CPU or energy advantage, formal proof, product readiness, or release
readiness. It establishes only that the present compiler-side bounded
multirate last-context core and its declared controls can execute causally and
deterministically over one frozen synthetic construction fixture while
preserving caller support and bounded recurrent matrix state.

## Subsequent direct-reference result (append-only update)

The evidence ledger above records the state when this recurrent smoke was
sealed. `DirectCausalGeometricAttentionR4V1` has since run. Its V2 result is
`NON_PROMOTABLE_BUDGET_MISMATCH`; fresh equal-manifold-budget V3 returned full H4 3/12,
matched plain 12/12, current-only 6/12, and a coherent alternative tangent
connection 10/12. The binding V3 verdict is
`FAIL_EQUAL_DOF_H4_DIRECT_ATTENTION_NOT_LOAD_BEARING_ON_FRESH_V3`.

This does not revise the recurrent counts above. `ConnectionGaugeCovarianceV4`
Phase I subsequently passed construction covariance; #973 has frozen its
separate target-free held-out population and salted commitment in PR #1001
before paired-E8 binding,
multi-resonance replacement, or another recurrent experiment. The
multi-resonance sieve remains `NOT_RUN`.

## Successor direction (2026-08-29)

V4 subsequently completed terminal-negative at `13/24`, without adequate
separation from its destructive controls. V4 will not be rerun or retuned. The
HELM-D-R4 became the full-decoder, gauge-equivalent ordinary-causal-softmax
reference with R4/Spin frame transport. Its parity gate now passes; the verdict
and scope are authoritative only in the
[HELM-D-R4 result](helm_d_r4_softmax_decoder_result_973.json). The active #973
successor is intrinsic R4 distance and normalized-centroid attention, followed
conditionally by multi-resonance replacement and recurrent lowering.

## Attempt 02 successor update — 2026-08-29

This record's bounded evidence and the `HELM-D-R4` gauge-equivalent ordinary-
softmax PASS remain unchanged. The separately trained intrinsic Lorentz-R4
successor stopped before D3 at
`UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT` (result CID
`blake3:da2a63323d6211b8d581e5a4ed75d788eb919ff0f210d2e3beb8a749ee1bc64f`):
normalized-barycenter covariance was `9.1214e-8` against the frozen `1e-8`
limit, and construction-validation NLL was diagnostically worse than the donor
by `1.2531` and the matched flat control by `0.20893` nats/token. No reveal
marker or held-out result exists. No Attempt 03 is authorized under this freeze;
any further intrinsic work must be a newly frozen, source-faithful
learned-manifold successor. Multi-resonance, recurrence, lowering, scale, and
#954 remain blocked. See the
[owning intrinsic record](intrinsic_lorentz_r4_attention_973.md) and the
[compact result summary](intrinsic_lorentz_r4_attention_attempt_02_summary_973.json).

## Learned-manifold V2 outcome and current successor — 2026-08-29

Source-faithful learned-manifold V2 completed a valid non-D3
construction-validation run. Donor/gauge parity and all destructive-control
separations passed, but learned Lorentz failed donor retention and matched
Euclidean parity; the controls establish sensitivity only. The sole current
#973 action is the frozen 8/8
[score-by-readout localization](helm_d_score_centroid_localization_973.md).
D3 remains `NOT_RUN`; resonance, recurrence, lowering, scale, and #954 remain
blocked.
