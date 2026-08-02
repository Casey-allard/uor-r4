# VP-tree routing cutoff (#277)

The exact VP-tree adapter remains available for larger ROUT indexes, but the
current serving graph is below its measured useful range.

On the checked-in SmolLM2-135M graph (363 nodes), release-mode measurements
with 32 deterministic queries and 1,000 iterations were:

| ROUT lookup | ns/query |
| --- | ---: |
| linear masked-Hamming scan | 2,503.8 |
| exact VP-tree | 2,654.6 |

A repeat measured 2,575.5 ns/query versus 2,758.9 ns/query. The tree was
therefore approximately 6% slower across the stabilized runs. Both paths
returned identical nearest nodes, distances, active sets, and checksums.

`R4G1Runtime::parse` now enables the tree only for graphs with at least 512
nodes. Smaller graphs keep the linear path, avoiding the measured regression;
the cutoff is a serving heuristic and should be revisited if graph sizes or
the ROUT layout change. The tree remains exact and contract-legal when used.

Reproduce:

```bash
cargo bench -p uor-r4-graph-runtime --bench vp_tree --offline -- \
  .uor-models/compiled/SmolLM2-135M-Instruct-7e27bd9f9532/graph/score.r4g1 1000
```
