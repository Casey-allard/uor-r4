# Production full-prefix memory canary (#944)

- **Status:** `INERT` for the exact schema-2 production envelope ratified by #933.
- **Scope:** one-step, teacher-free calls through `NormativeServingEngine` and the
  sole token-authoritative `R4G1Runtime` selector. This is not a coherence benchmark.
- **Result:** [`production_prefix_memory_canary_944_result.json`](production_prefix_memory_canary_944_result.json).
- **Instrument:** `crates/uor-r4-api/tests/production_prefix_memory_canary_944.rs`.

## Predeclared decision

The canary inspected at most 512 deterministic held-out positions. Complete-prefix and
suffix-only histories supplied distinct 288-bit token-history signatures while retaining
the same newest eight-token scoring window. A positive verdict required all of the
following in one exact production case:

1. the primary context signature was probed and admitted no calibrated node;
2. the secondary session signature admitted at least one calibrated node; and
3. the complete-prefix and suffix-only signatures changed the normative candidate list or
   served token.

The run stopped after the first such effect or after 512 cases. `EFFECT_ESTABLISHED` would
authorize only a small production-scope slice of the frozen #841 protocol. `INERT` blocks
that rerun and sends the next change to the observation/compiler representation.

## Exact result

The admitted graph was
`blake3:ff82dfd5f04eac7e944443b1ea4cc9fe93a007b3b8f07286876d52709a98bc49`;
the schema-2 release manifest was
`blake3:c2025e9e507e8367993d78bd83ef099ce5851c838d3cc5cf01eda5560986ad33`.
Production admission succeeded before any case was inspected.

Of 512 cases:

- 492 resolved through an explicit context row;
- 20 reached the primary context-signature probe, and all 20 admitted a node;
- 0 primary context probes missed;
- 0 secondary session probes were attempted or admitted;
- 0 candidate lists changed; and
- 0 served tokens changed.

The tested-position CID is
`blake3:f108e9caab5edc0b7b4d312cf5ae187608d9653ea3d4e24bfa00ef68a07507b7`;
the ordered-observation CID is
`blake3:c6876af09d4c416f10f7098b18eda94cf8e6e4217e9cf28bb66a73f8f86748fe`.

## Verdict and next decision

**Empirical Criterion. Status: Empirical. Verdict: `INERT`.** The #942 full-prefix state is
structurally connected, but the exact admitted artifact never reaches its secondary
session-routing probe on this bounded population and shows no candidate or token effect.
Signature-byte inequality alone is not credited as product behavior.

Per the run contract, the broader #841 rerun is **NOT_RUN**. The next representation change
must make complete-prefix information behaviorally reachable—for example through
signature-addressable compiler records or a separately pre-registered routing composition—
before re-entering this same canary. The historical S3 `LIMIT` verdict and frozen #841 bar
remain unchanged.
