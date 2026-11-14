# Direct trace result: Route and active leaf

Head: codex-6 implementation. Parent contract:
[claude-371/PARITY.md](../claude-371/PARITY.md).

## Conditions

- GPU: NVIDIA GeForce RTX 4090, compute capability 8.9, driver 595.99.02.
- CUDA: 13.1, AOT target `compute_89`.
- Host: `wick`.
- Fixtures: Route, Flock and Relation `UORLUT01` bytes.
- Rows: all 39 prefixes of lengths 1 through 3 over three tokens.
- Build: `NVCC=/usr/local/cuda/bin/nvcc UOR_NVCC_ARCH=compute_89 UOR_REQUIRE_CUDA=1`.
- Feature gate: `qualification` is non-default and exists only for this finite parity contract.

The synthetic fixture provenance bytes are not production artifact hashes. The finite test uses
the qualification-only loaders recorded by codex-5; production loading remains strict.

## Trace words

| Area | Comparison |
|---|---|
| Route header | frontier count, output root, completed stages and terminal atom |
| Route stages | configured frame, parent root, selected port, goal root, composed action and next node |
| Active-leaf header | query frame, output root, residual terms and query payload root |
| Active-leaf levels | gate, eligible count, kept count, consensus root, kept nodes, transport actions and carried roots |
| Relation tail | per-node scores padded to the context's maximum binder-node count, CUDA vs CPU vs serving |
| Trace parity | every base and Relation-tail word for all 39 rows and all three fixtures |
| Untraced control | no trace or Relation node-count allocation, with identical carry, predictions, losses, rivals and admitted cells |

## Fault controls

| Faults | Output-only K1 | Direct trace |
|---|---:|---:|
| Route-frame alternatives: 4 words x 63 other ports | **108 seen, 144 missed** | **252 / 252 seen** |
| Flock gate alternatives: 4 words x other two trits | **3 seen, 5 missed** | **8 / 8 seen** |
| Relation level-gate alternatives: 6 words x other two trits | not separately controlled before this unit | **12 / 12 seen** |
| Active kept-node slots per Flock/Relation 39-prefix fixture | not exposed before this unit | **126 / 126 words** |
| Active transport-action slots per Flock/Relation 39-prefix fixture | not exposed before this unit | **126 / 126 words** |
| Table-root decision mutations | not exposed before this unit | **Flock 90 faults: 54 kept / 90 action changes; Relation 240: 12 kept / 90 action** |
| Relation compact / padded score words | not exposed before this unit | **258 / 1,014 exact** |
| One-word carry fault | caught | still caught |

Result: the direct trace closes C-1 and C-4 and makes C-2's five output-only misses visible.
The bucket writer's committed source hash remains unchanged because its generated source strips
the marked trace-only blocks from the one trace-capable production Route read and checks the
remaining qualified shape. A mutation of the production Route decision line reaches generated
bucket source and fails the retained source hash; CUDA compiles no second Route body.
