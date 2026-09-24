# Compiled relative-query work plan, 2026-09-24

Base: `48106e7c0dd112c94f44bd535141002286ca92a1` (PR #1373). The five preceding workstreams already have implementations and scoped results in the causal-continuation handoff. Preserve those outcomes and do not repeat their experiments as new work.

## Objective

Use the learned finite quaternion action as a computation, not a decorative feature: reduce a relation path to a signed permutation, move the query into the inverse frame, and scan unmodified candidate keys. For signed permutation R, `||q - R k||_1 = ||R^-1 q - k||_1`. This replaces N repetitions of an L-step transport with O(L + N) work without changing selection, payload identities, tie-breaking, or existing Q8L1 model bytes. A caller may compile once and reuse `CompiledRelativePath`; the retained `RelativeActionModel::select` recompiles per call (still O(L+N)), so "compile once" describes the reusable plan, not the selector's internal path.

## Safety and numerical contract

A net signed permutation alone is insufficient. A path can negate `i32::MIN` at an intermediate step and later cancel that sign. Preserve a four-bit mask of original coordinates negated by any prefix. Reject precisely those keys the original checked path rejects. Transform the query in i64 so a valid i32 minimum query never introduces a new overflow. Preserve the original selector's candidate-count checks and first-candidate error precedence for invalid paths. Compile no learned parameters and add no model-format version.

## Implementation and verification

1. Add a standalone Rust test target importing the actual Hamilton and relative-action source modules, plus a deliberately unimplemented compiled-path interface; observe failing behavioral tests.
2. Implement constant-size compiled path, exact checked action, inverse-frame selection, and use it in the existing selector after preserving its error order.
3. Exhaustively compare paths through length four, boundary vectors and invalid cases against the unchanged sequential `act` oracle. Test composition, cancellation, ties, serialization, learned mappings and heap-free successful execution. Measure, but do not gate correctness on, CPU timing.
4. Inspect the final diff, run the scoped checks, record genuine execution evidence and limitations, then deliver code and documentation through the normal protected PR process.

## Execution boundary

Remote Desktop Commander reported no connected devices in this session. No local filesystem, model, storage, or cumulative-ledger update is claimed. A narrowly scoped standard GitHub-hosted job in this public repository is used only for standalone Rust correctness and a small CPU microbenchmark: one five-minute-limited job, read-only repository token, no model downloads, no secrets, no paid model compute. This does not measure the owner's M1, energy, language quality, training, or end-to-end attention. New local experiments and disk cleanup require the desktop connection to return.
