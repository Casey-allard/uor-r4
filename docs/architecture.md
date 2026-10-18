# Current architecture and its sources

The current M1 implementation is [`uor-r4-discrete`](../crates/uor-r4-discrete/README.md).
This is a map of implemented data flow, not a claim of useful language quality or a
completed unified serving system. [Status and results](../STATUS.md) name the measured scope.

<picture>
  <source media="(max-width: 600px)" srcset="figures/architecture-narrow.svg">
  <img src="figures/architecture.svg" width="1200" alt="Discrete M1 data flow with exact geometric learning; M2 and unified serving remain separate obligations">
</picture>

## Follow one token

| Figure arrow or component | Primary source and meaning |
|---|---|
| Token IDs + shared table → encoded blocks | [`discrete_encode` in discrete.cu](../crates/uor-r4-discrete/src/discrete.cu) performs the token-table lookup into encoded input blocks. `input_block` reads that already encoded buffer and optionally applies the virtual candidate move; it is not the encoder. The writer uses the same `Snapshot.table` for output vectors. |
| Encoded blocks + ordered context plan → read | [`metadata.rs`](../crates/uor-r4-discrete/src/metadata.rs) builds ordered ternary plans; `route_vector` / `flock_vector` in [`discrete.cu`](../crates/uor-r4-discrete/src/discrete.cu) consume the plans. |
| Ordered context → read vector | `read_vector` dispatches Route, Flock or Relation. `route_vector` follows one selected carrier per block and applies the composed signed transport, without summing or snapping. Flock ranks eligible prior nodes, transports payloads into the query frame, sums, snaps and applies ternary gates. Relation reranks with finite graph-edge sweeps before that selection. |
| Read vector → shared writer → selected token | `writer` in [`discrete.cu`](../crates/uor-r4-discrete/src/discrete.cu) admits occupied product-address cells and scans tokens in admitted cells by exact integer distance, with deterministic ties. No probability normalization is formed. |
| Snapshot + fixed batch → virtual moves → accepted snapshot | [`device.rs`](../crates/uor-r4-discrete/src/device.rs), [`training.rs`](../crates/uor-r4-discrete/src/training.rs) and [`controller.rs`](../crates/uor-r4-discrete/src/controller.rs): proposals compare exact fixed-batch objectives; updates require improvement and applicable admission. [`rows_device.rs`](../crates/uor-r4-discrete/src/rows_device.rs) provides opt-in row-parallel proposals. |
| Prime-Zeta constructor → shared table (optional initialization) | [`prime_zeta.rs`](../crates/uor-r4-discrete/src/prime_zeta.rs): four complex channels per block → two normalized Q30 quaternion lanes → fixed coordinate permutation → exact alignment to the carrier, under writer-cell quotas and full-vector uniqueness controls. This constructor is distinct from the integer kernel. |

## Dimensions and cost

[`build.rs`](../crates/uor-r4-discrete/build.rs) constructs **64 signed-axis actions**
and the anchor orbit of **16 roots in eight coordinates**. These are different
counts. Each model block is 8D; four blocks have width32. The current orbit does
not cover all 240 E8 roots, nor does its dimension imply full E8 integration.

A retained K bounds the selected list, not the work needed to select it.
`flock_vector` scans candidate nodes at each level and inserts into a bounded
list; Relation includes additional node/edge scans. `writer` scans the vocabulary
to mark occupied cells, then scans it again for admitted cells. Fixed context,
K or finite tables alone do not establish context-independent cost or laptop latency.

## Research and serving boundaries

Quaternion/S³, full icosian/H4, reduced-metric E8, Hopf/fibre and wider prime-addressed
memory research are explained in the [geometry guide](geometry.md). Hopf fibre tracking
uses float-backed offline geometry helpers with Q30 storage; it is not an
integer execution path or a step in the current discrete M1 read.

Cargo selects core [`native_support.rs`](../crates/uor-r4-core/src/native_support.rs)
and integer [`native_geometry.rs`](../crates/uor-r4-integer/src/native_geometry.rs)
through their manifests. Retained historical core/integer model executors are not
selected APIs. M2 compiler/emitter research is held; the separate
[D19 object store](../crates/uor-r4-object-store/README.md) has no model-integration arrow.
A single D11 served model joining these lines remains an obligation.

[D37](integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised)
allows offline float training, preserves matmul/GEMM and softmax/squashing bans,
and leaves integer serving as a separately measured export boundary. A source diagram,
finite fixture or scoped pilot does not qualify a general conversation model.
