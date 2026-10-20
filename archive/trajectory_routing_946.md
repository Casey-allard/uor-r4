# Full-prefix trajectory routing regions (#946)

- **Implementation status:** complete for compiler, R4G1 admission, and normative runtime.
- **Product verdict:** `INERT` on the exact #933-bound schema-2 candidate.
- **Scope:** teacher-free one-step production selection. This does not establish coherence,
  semantic relevance, planning, or clearance of the frozen #841 generation bar.
- **Result:** [`trajectory_routing_946_product_result.json`](trajectory_routing_946_product_result.json).

## Implemented mechanism

Compiler observations now carry a 288-bit full-prefix `TokenHistorySignature`, updated once
per recorded token and reset at story boundaries with the same algorithm production uses.
`cover --trajectory-routing` attaches a deterministic prototype and calibrated radius to each
induced region without increasing the region ceiling. R4G1 node flag `0x01` declares that ROUT
extension; stage-2 admission rejects unknown flags, invalid extents, invalid radii, nonzero
reserved bytes, and nonzero signature padding.

The normative runtime builds separate context and trajectory route indexes at admission.
Fallback probes both lanes, composes at most four context plus four trajectory nodes in stable
distance/node-ID order, removes duplicates, and scores each node with its declared signature
lane. The active-set ceiling remains eight. Legacy flag-zero artifacts and calls without a
trajectory signature retain their previous behavior. The hot path uses fixed arrays and remains
allocation-free; runtime admission remains `no_std` compatible.

## Binding cheap instrument

The teacher-free preflight used 4,096 training and 4,096 held-out positions from the exact
#933 input. Treatment and context-only control had identical ceilings and sizes: 64 regions,
65 nodes, 1,232 edges, 8,280 ROUT bytes, 30,796 EMIT bytes, 66,508 total bytes, and a 4+4
active-node split. It observed 1,549 fallback positions, 1,549 trajectory admissions, and
1,549 candidate-list differences. Repeated treatment emission was byte-identical.

The treatment CID was
`blake3:53ab5e4098bd06db1c7183d4e529059f160209f906a52101c359cbb1e16e2353`;
the control CID was
`blake3:a30c9beed023b53ce8f008498e77986cce8c4f616f7ac38759d4567e3401dc07`.
Total preflight wall time was 8.386 seconds. No teacher was loaded. The measured-stage
projection and 15-minute hard wall were posted before the candidate build; the actual cover
build completed in 17 seconds.

## Exact candidate and admission

The candidate reused the #933-bound teacher artifact
`blake3:6324aabec22fca5af371333cefc206f9b6762bfb52dccfb8efa0dc8fe5a1efaa`
and corpus stream
`blake3:7db27ffb488ad996f2317c99f3eb627ca964b28c3e730d050d1e51136c7a335e`.
It contains 34 induced regions, 35 scored nodes, and 439 scored edges. The scored graph is
77,672,656 bytes, 1,664 bytes larger than the historical #933 context-only graph while keeping
the same node, edge, emission-entry, candidate, and active-node ceilings. This byte delta is
reported explicitly; the full artifact is not described as byte-equal to the historical
control.

The graph CID is
`blake3:b97b5834799968d34c319a03df9a9dd217bf89a3ec2fd0b53f06abd2e03c38ba`.
The schema-2 instruction-chat release manifest CID is
`blake3:987b60167c6f1b1a57a646e8f5685051062680113b202fdcd6c4559b5b8d77d0`.
Its compiler authority is implementation commit
`45d293181c0db35ea343a761d3fb1a19425c5ad7`.

Production admission's full 72,130-position census passed: selector top-1 was 21,293/72,130
(29.5203%) versus the same-position TLA comparator at 20,284/72,130 (28.1214%). Witness replay
passed 64/64; internal and cross-surface mismatches were zero. This repeats the #933 quality
scope and is admission evidence, not a new quality claim.

## Product canary

The bounded product canary admitted the complete schema-2 envelope before inspecting the same
first 512 eligible held-out positions used by #944. Of those, 492 resolved through exact context
rows. All remaining 20 attempted both route lanes and admitted trajectory nodes, establishing
that the new representation is reachable in production. However, changing the earlier prefix
changed zero normative candidate lists and zero served tokens. Tested-position CID:
`blake3:f108e9caab5edc0b7b4d312cf5ae187608d9653ea3d4e24bfa00ef68a07507b7`;
ordered-observation CID:
`blake3:f87a37bf1888371c6822236abb558f9d34b8b41485cd1870dd6915fb33416dda`.

**Empirical Criterion. Status: Empirical. Verdict: `INERT`.** Trajectory routing is compiled,
admitted, and reached, but the exact bounded product comparison found no earlier-prefix-caused
normative behavior change. Per the predeclared contract, #841 is **NOT_RUN** and no later
production-scope slice is authorized from this result.
