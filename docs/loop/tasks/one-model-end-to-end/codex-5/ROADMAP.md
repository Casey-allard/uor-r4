# Native CUDA kernel roadmap

The discrete path is the default native path because it already carries one integer
representation from training through serving. Work proceeds in dependency order; it does not
start a new mechanism while an existing transition lacks direct parity evidence.

## 0. Completed

- Inventory the native state and identify the F32 export gap.
- Compare CPU training to integer serving over two fixtures.
- Compare CUDA forward, evaluation, counterfactual losses, chooser, apply and post-apply state
  to the CPU and serving words on the RTX 4090.
- Expose and compare direct Route and active-leaf traces across CUDA, CPU and serving
  (`codex-6`).

## 1. Direct route and read trace (completed by `codex-6`)

Add one read-only trace contract for:

- the route port selected by each block and stage;
- the Flock/Relation kept-node list.

Order: `codex-6`.

Acceptance: CUDA, the CPU oracle and serving return the same raw trace words for all 39 fixture
prefixes. Every single route-frame or level-gate fault changes a compared trace word. Every
kept-node or transport-action mutation that occurs in a fixture changes a traced word, and the
control pins the count. Succeeded: all 252 route faults and all 8 gate faults change the trace;
the 144 route faults and five gate faults that escape output-only checks are directly visible.
Each Flock and Relation fixture contains 126 kept-node and 126 transport-action slots. Recomputed
table-root mutations changed 54 kept / 90 action decisions on Flock and 12 kept / 90 action
decisions on Relation. Relation's 258 compact and 1,014 padded score words match across backends.
CUDA has one trace-capable Route body; the bucket generator strips marked trace blocks from that
body, and a decision-line mutation invalidates its qualified source receipt.
Untraced calls allocate no trace and return unchanged public values.

## 2. Loader symmetry

Make serving refuse a Route artifact with non-zero gates, matching the CPU training loader.

Order: `codex-7`, after step 1.

Acceptance: one malformed artifact is refused by training and serving at load, before any row
executes.

## 3. Shared typed operators

Give CUDA, CPU and serving one source definition for the operators each calls:

- signed-axis action;
- composition;
- `root_cost` with pinned arithmetic bounds;
- `snap_n`;
- `checked_scaled_add`.

Order: `codex-8`, after step 2.

Acceptance: a planted change to one operator definition turns the same transition red on every
backend; no backend keeps a hand-copied rule.

## 4. Exact update kernel parity

Extend the current proposal/apply leg to Table, Route and Gate moves plus status refusals:

- stale epoch 3;
- bad index 4;
- non-improvement 5;
- mode mismatch 9.

Order: `codex-9`, after step 3.

Acceptance: CPU and CUDA return the same status and leave identical table, route, gate and
epoch words.

## 5. Generation and stop state

Add the missing decoder loop and exact stop/carry state in the same integer representation.

Order: `codex-10`, after step 4.

Acceptance: a finite prompt replay produces identical prompt carry, route decisions, emitted
tokens, stop reason and final state through training simulation and serving.

## 6. Memory and conversational carry

Bring exact copy and turn-memory into the discrete native path.

Order: `codex-11`, after step 5.

Acceptance: memory key, address, read choice, copied value and resulting token are compared as
raw words, with empty-table and matched-shuffle controls.

## Current smallest next dependency

Step 2: make serving refuse a Route artifact with non-zero gates, matching the CPU training
loader. No generation, training or performance claim is required to finish it.
