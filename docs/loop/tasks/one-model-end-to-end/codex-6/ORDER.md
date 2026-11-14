# ORDER: one-model-end-to-end, codex-6

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parent contract: [claude-371](../claude-371/PARITY.md). Base: `d66d2d0e5`.

## Owner direction

> "please make this your next immediate roadmap, plan it thoroughly, and execute it
> completely autonomously"

## Goal

Complete step 1 of the native CUDA kernel roadmap: expose the internal route and active-leaf
read decisions as exact words on CUDA, the CPU training oracle and integer serving.

The trace is a qualification observation over the same production read. It must expose:

1. each Route block's configured frame, parent root, selected port, goal root, composed action
   and next binder node at every stage;
2. each Flock/Relation block's query/source roots, output root, residual term count, level gate,
   selected kept node and its transport/carry roots, at every stage, including gate-zero stages.
3. the Relation score tail in CUDA's context-padded batch layout, separately from each backend's
   compact score accessor.

The same artifact bytes and prompt prefixes must produce the same trace words on all three
backends. Normal production calls retain no trace allocation.

## Trace contract

- Route stride: `4 + stages * 6` `u32` words per `(row, block)`.
- Route header: `[frontier_count, output_root, completed_stages, terminal_atom]`.
- Route stage: `[frame_action, parent_root, selected_port, goal_root, composed_action, next_node]`.
- Flock/Relation stride: the existing `4 + stages * 28` `u32` words per `(row, block)`.
- Flock/Relation header and levels retain the production `discrete_forward` ABI:
  `[frame_root, output_root, residual_terms, query_payload_root]`, then per level
  `[gate+1, kept_count, consensus_root, eligible_count, kept_node_ids[8],
  transport_actions[8], carried_root_ids[8]]`.
- Relation padded scores contain `max_nodes` words per `(row, block)`, with `ABSENT` padding;
  compact `relation_scores()` contains only each row's actual node count.
- CUDA has one production Route body. Bucket generation may remove explicitly marked trace-only
  blocks from that body; a decision-line mutation must reach generated source and invalidate the
  retained source receipt rather than compile a second body.
- `ABSENT` (`u32::MAX`) marks unused route fields and level slots.

## Files in scope

- `crates/uor-r4-discrete/Cargo.toml`
- `crates/uor-r4-cpu-train/Cargo.toml`
- `crates/uor-r4-discrete/src/discrete.cu`
- `crates/uor-r4-discrete/src/bucket_source.rs`
- `crates/uor-r4-discrete/src/device.rs`
- `crates/uor-r4-discrete/src/relation_qualification.rs`
- `crates/uor-r4-discrete/src/writer_bucket_device.rs`
- `crates/uor-r4-discrete/src/driver_source_pin.rs`
- `crates/uor-r4-cpu-train/src/discrete_route.rs`
- `crates/uor-r4-cpu-train/src/lib.rs`
- `crates/uor-r4-integer/src/discrete_d11.rs`
- `docs/loop/tasks/one-model-end-to-end/codex-6/**`
- `docs/loop/tasks/one-model-end-to-end/claude-371/PARITY.md`
- `docs/loop/tasks/one-model-end-to-end/codex-5/ROADMAP.md`
- `docs/roadmap/items/one-model-end-to-end.md`
- `docs/research/notebook/cpu-training.md`, one result row

## Cancel probe

Cancel this unit if:

1. tracing changes a read vector, admitted cell, top-8, token, loss or update word;
2. a backend cannot expose a trace without recomputing a host copy of its own read;
3. an artifact mutation that changes a route or level gate can still pass with an identical
   trace on every prefix;
4. the trace requires a production allocation in the untraced path;
5. GPU parity cannot be run on the local RTX 4090.

## Done when

1. Route and Flock/Relation traced and untraced calls return identical public evaluations.
2. All 39 fixture prefixes produce byte-identical raw trace words through CUDA, CPU and serving.
3. Every alternative value of each Route frame word changes at least one serving trace word.
4. Every alternative level-gate trit changes at least one serving trace word.
5. Every single kept-node or transport-action mutation that can occur in a fixture prefix changes
   at least one traced word, and the test pins the observed count rather than accepting silence.
6. The existing planted carry fault remains visible.
7. Focused tests, full gated crate tests, Clippy, formatting, `uor-harness check`, the REPORT
   check and the RTX 4090 CUDA test pass at the same head.

## Stop

- No training run, capability claim or throughput claim.
- No artifact-format change.
- No loader-symmetry, shared-operator, generation or memory work in this unit; those are roadmap
  steps 2 through 6 and receive their own orders.
- STOP and report after step 1 is green or the cancel probe fires.
