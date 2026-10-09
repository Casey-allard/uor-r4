# Independent exact-head review: PR #1528

Reviewer: `/root/repair_execution_council`, non-author; same Codex provider/root launcher dependence disclosed. Reviewed head **09da29ab3311be03e2c14e11d2e573e451beea54**, base **61bbd85a0a49e1bb72c71b4e02752d575e3bee4b**. Live GitHub head matched at review. Read both prior review comments and Antigravity's 05:16:23Z response, the complete new selector/tests, other seven-file diff, reciprocal helper and actual references. No checkout/source edits, Cargo, model, GPU, compiled opcode audit, or allocation-instrumented execution performed.

**Decision: CHANGES REQUIRED.** The original private-field, tie-boundary, stable-sort and raw division defects are substantively corrected. Two small public-API correctness defects and unsupported source/PR claims remain. The reported author test passes are retained as author-reported results; no exact-head executable/log receipts were supplied in the reviewed comment. This source review neither reran nor independently verified them.

## Required fixes

### P2 — `ensure_capacity` still does not ensure the requested capacity

[flock.rs:139-150](https://github.com/UOR-Foundation/uor-r4/blob/09da29ab3311be03e2c14e11d2e573e451beea54/crates/uor-r4-integer/src/stack/flock.rs#L139)

`Vec::reserve(additional)` reserves relative to **length**, not current capacity. Both `rest.reserve(bound - rest.capacity())` and `entries.reserve(bound - entries.capacity())` therefore can be no-ops when growth is required. Concrete legal example: a new scratch with capacity129, empty vectors, then `ensure_capacity(200)`. The added amount71 fits the existing129 allocation, leaving both capacities below200. The slots grow, so later selection materialization can allocate despite the caller having explicitly prepared200 positions.

Use the additional amount relative to each vector's current length (with the existing capacity guard), and add the focused growth regression on initially empty and previously used scratch. Assert all three capacities are at least the requested bound, then verify the prepared valid selection does not grow them. The current new test only calls `ensure_capacity(64)` on capacity256 and cannot exercise this defect. [Rust Vec reserve contract](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.reserve).

The previous subtraction-underflow finding is fixed, but the method's capacity postcondition is not.

### P2 — maximum-context guards themselves overflow on `usize::MAX`

[flock.rs:184](https://github.com/UOR-Foundation/uor-r4/blob/09da29ab3311be03e2c14e11d2e573e451beea54/crates/uor-r4-integer/src/stack/flock.rs#L184) and [flock.rs:300](https://github.com/UOR-Foundation/uor-r4/blob/09da29ab3311be03e2c14e11d2e573e451beea54/crates/uor-r4-integer/src/stack/flock.rs#L300)

Both `Result` APIs evaluate `query + 1` before validating the public input. `query=usize::MAX` can panic in overflow-checked builds instead of returning the promised invalid-input error. Use `query >= MAX_FLOCK_CONTEXT` (equivalent for valid queries) or checked addition; then later additions are safe. Extend the existing bound test to `usize::MAX` in both entrypoints. No large score buffer is needed for that case.

### P2 — bound claims to implemented source and actual measured scope

Required corrections can be documentation/PR wording; they do not require a model run.

1. **The module does selection and rank-weight conversion, not score formation or softmax.** Its header at lines18-19 advertises integer Minkowski/dot scoring and integer softmax. Actual inputs are already formed `&[i64]` scores; no score/softmax API appears in this file. State those as external consumer responsibilities. Complete-repository exact-head reference search found the selectors and rank helpers only in this module/re-exports/tests; no `StackSession` or other production consumer invokes them. Describe a standalone selector implementation awaiting integration, not a delivered attention-serving path.
2. **Compiled numerical compliance is not established by eliminating `/` from source.** New source consistently uses table reads or `stack_div_u128` for rank divisions, and that helper is shift/subtract long division. This resolves the specific direct-source division finding. It does not establish the PR's absolute zero-hardware-opcode claim for every reached optimized root, standard-library selection/sort or future consumer. Retain the source-level fact; mark exact-artifact opcode qualification NOT_RUN/UNVERIFIED until a bound receipt exists. D5 selected-parameter access is not established by selecting attention positions.
3. **Total selector complexity includes ordering the final support.** `flock_select_integer` sorts all `scratch.entries` at lines268 onward. With support size `s`, a source-appropriate bound is `O(n + k log k + s log s)` (or a simplified bound that states fixed/bounded window assumptions). The public window is not fixed64: `window=n,k=1` retains and sorts all n. Remove the unconditional whole-selector `O(n+k log k)` interpretation and the PR's measured-sounding “over90% comparison” reduction unless backed by an actual comparison-count receipt. `candidates_scanned` counts candidate positions, not comparator calls.
4. **Blocked GEMV traffic savings are unmeasured.** `kernels.rs:705-709` says four-row blocking cuts pair-table cache traffic75%. Source does four lookups with potentially different indices into each1KiB table; cache-line reuse depends on weights/cache state and is not proven by sharing a Rust reference. State this as a locality hypothesis. The function remains `#[allow(dead_code)]` and has only a test caller, so it has not accelerated serving. This was already an out-of-selector scope concern in the prior review.

## Disposition of the seven original selector findings

| Original finding | Exact successor source disposition |
|---|---|
| Private scratch fields accessed by integration test | Fixed: public read-only accessors and test uses them. Compilation not rerun here. |
| Direct `/` in rank calculations | Fixed in source: constants plus restoring division. Compiled-opcode gate remains unverified. |
| Tie flag reads unordered kth element | Fixed in both selectors: chosen prefix ordered before comparing kth and next pivot. |
| Stable sort may allocate | Fixed: all selector sorting uses `sort_unstable_by`; total comparator includes position tie break. |
| MAX_FLOCK_CONTEXT unenforced | Ordinary out-of-range inputs rejected; extreme-input overflow remains above. |
| Capacity subtraction underflow | Underflow removed; reserve uses wrong reference quantity, as above. |
| entries reserved only129 | `new` now preallocates all three buffers to bounded requested capacity. Explicit growth helper remains defective. |

The two literal reciprocal arrays each contain129 entries. A small read-only integer arithmetic audit of those source literals found every Q32 value equal to floor(2^32/(i+1)) and every Q16 value equal to min(65535,floor(2^16/(i+1))). This is table-content evidence only, not a Rust execution or opcode audit. The first raw Q16 weight is saturated65535, not an exact representation of1; preserve that fixed-point distinction when comparing with the floating canonical arm.

## Canonical API and scope

Compared source semantics with OpenCode's pending canonical selector at PR1532 head `bd14a2791f78c5f775eb4bb3db34bebbf79a31bd`: inclusive causal prefix, sink-over-window precedence, deduplicated support, descending rank/lowest-position tie ordering, cutoff-tie semantics, normalized rank versus raw unnormalized hybrid scale agree in their source intent. The integer pointer-only entrypoint is an additional API. Integer quantization can change score ties/order, so this is not a general floating-to-integer model parity claim. The tests compare with a local integer reference, not the actual canonical float module. A later shared bridge test should use exactly representable score rows and separately report quantization effects; do not create a circular dependency just to run it.

The rebase repaired the historical huge merge-base issue: current PR has seven changed files relative to current base. It still includes a166-line four-row GEMV and unrelated benchmark-path edits. Preserve them, but either split them into their own tracked PR or explicitly retain a separate scoped validation obligation. The new GEMV test is one16×32 case: it does not exercise multiple quantization groups or the newly implemented remainder-row branch. If retained for delivery, add focused parity cases for a width64+ matrix and a row count not divisible by4, with heterogeneous scales. No broad model or corpus run is justified.

## Minimal successor acceptance

Fix the two API cases and claims; retain the valid selector changes. Compile and execute the affected existing package tests under a legitimate shared reservation, adding only the capacity-growth and extreme-query cases and, if GEMV remains, its two missing branch cases. Supply source/compiler/executable/log identities for reported checks. An allocation instrumentation or compiled-opcode result is required only for the corresponding strengthened runtime claim; otherwise scope delivery honestly as source/functional selector support awaiting that qualification and consumer integration. No language result, energy improvement, selected-weight access, or full serving compliance follows from this PR alone.
