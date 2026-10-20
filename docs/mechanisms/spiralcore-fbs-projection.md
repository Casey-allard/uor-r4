# SpiralCore BFS plus retained ternary FBS projection

**Milestones:** M1 #2029 and M2 #2030. **Record:** [native projection](../../archive/labs/spiralcore-fbs-projection-2026-10-12/README.md).

## Idea and implemented role

Replace an all-input learned projection sum with exact carrier selection and
geometric transport. Input channels are 8D atoms, kept in ordered ternary
binders whose complete children and remaining forest frontier are retained.
Learned query points choose roots in a fixed E8 anchor orbit; the actual
SpiralCore BFS edge history composes the signed action that transports the
selected carrier. Exact atom identity stays separate from geometric root/state.
There is no learned coefficient for every input/output pair.

This explicitly couples the supplied Atomic Ternary FBS **binder abstraction**
to SpiralCore's action graph. It does not implement the full marked FBS object,
derive semantic distance from atom length, or claim that the external source
provides this learning law. One anchor orbit contains 16 roots and 64 signed
operator states; it does not represent every E8 root or independent H4 partners.

## Source and learning boundary

`crates/uor-r4-training/src/spiralcore_fbs.rs` compiles the retained nodes and
actual BFS/composition tables. `spiralcore_fbs.cu` implements CUDA forward and
adjoints for F32/BF16 carriers with F32 query masters. Forward transports one
carrier per output block. The input adjoint is its exact inverse signed action.
Query credit is a declared geometric root-edge secant surrogate: compare each
lawful sibling's fully rerouted carrier, pair its displacement with device output
credit, and resolve that contrast along the root edge. It is not the true
derivative of discontinuous hard selection or an established descent guarantee.

The static compiler runs on the host without reading model tensors; activation
routing, distances, transport, credit and validation arithmetic run on CUDA.
Only a validation status word returns to the host. Artifact hashes bind compiler,
kernel, source geometry, graph tables, children, shapes and mode. Dense artifacts
remain inactive; CPU fallback and native dense head fallthrough are refused.

## Fair tests and unresolved questions

Finite kernel qualification uses independent GPU gather/sign operations, all
27 terminal children of a three-level binder, an exact selected-input adjoint,
a device-computed two-atom secant control, both storage dtypes and error refusals.
No CPU model reference, dense comparator or training is included. A tooling or
numerical failure is a blocker to repair, never a mechanism negative.

Language utility, learned routing stability, surrogate fitness and capacity/cost
remain unmeasured. Input selection by shared learned queries does not establish
input-conditioned attention. Total block count must be justified prospectively;
there is no MiniLM384 requirement. A future model run must train this mechanism
in, supply its own fair targets/ablations, keep fresh transfer and D22 noise
requirements, trace every executed operation and obtain the applicable owner
hold release. The common vector writer, geometric vocabulary head and D11
integer/table serving remain dependencies. Training stays held.
