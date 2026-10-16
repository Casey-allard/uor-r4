# Prospective finite-control execution packet

**TEST FITNESS: PENDING. Execution: NOT_RELEASED. Results: NOT_RUN.**
This packet is a proposed clarification of the failure/overflow/stale-epoch
coverage named in the existing decomposition pre-registration. An independent
FIT must accept or reject this precise scope before any model execution.
Source construction checks do not release finite controls or the diagnostic.

## One question and fixed conditions

Does the separately clocked D counterfactual kernel preserve U's admitted
integer loss/status/proposal behavior on the declared eligibility, tie and
virtual-endpoint fixtures, while model/batch/profiler admission fails closed?
Use the same compiled ordinary U and generated D pair, Route read, seed1 only,
epoch0, unchanged fixed grid1/block16, both admitted objective policies, and
zero apply. No corpus, training loop, seed search, warmup or timing repeat.
Keep all observations, including unexpected refusals; do not substitute a
different fixture after seeing a result.

## Fixed CUDA proposal inventory

For the first four rows below, execute both policies × Route(0)/VectorBlock(0)
× U/D, eight attempts per row. Each of the last two rows uses its one declared
parameter only, four attempts per row.
Roots and candidate-image literals come from the retained
`admission-rank-fixture-runner-2026-10-15/instrument/src/fixtures.rs` at base
`c461350058e6ea1b188a60944da964982f272b60`; no host model evaluates them.

| ID | Admitted literal fixture | Prospective criterion |
|---|---|---|
| all_ineligible | vocab2, blocks2, context1, K1; token0=[root0,root0], token1=[root15,root15]; input token0, target0 | U/D both status8, sixteen MAX losses, no proposal; D sixteen terminal fault8 lanes, visited mask1, eligible mask0 |
| eligible_reference | same literal snapshot/input, target1 | U/D success with identical sixteen losses and five proposal words; D baseline eligible mask1 |
| mixed_eligibility | same snapshot, two token0 windows, targets[0,1] | U/D parity; losses/proposal match eligible_reference; D visited mask3, eligible mask2; first row contributes nothing |
| identity_and_tie | vocab2, blocks2, context1, K2; token0=[root0,root0], token1=[root3,root0]; input token1,target0 | Route0 all identity ties, winner0. VectorBlock0 predicts improving candidates3/root4 and12/root13, tied best, lower resulting port selects3 |
| route_endpoint | vocab2, blocks2, context2, K2, zero routes; token0=[root3,root0],token1=[root2,root0]; window[0,1],target0; Route(0) only | full U/D parity; independently materialized endpoint equality; at least one nonidentity candidate differs from identity, else BLOCKED for disposition |
| virtual_ineligible | vocab2, blocks2, context1, K1; both tokens=[root0,root0]; two identical token0 windows,targets[0,0]; VectorBlock(0) only | status0 with all nonidentity losses MAX; identity visits both rows, nonidentity lanes break after row0; endpoint GPU rival count explains sentinel |

Total: **40 proposal attempts**, comprising eight expected refusals and 32
successful proposals. These controls have no timed/warmup classification and
are not samples for H2/H3. The diagnostic retains its original, separate
32-proposal inventory, 24 timed and eight warmup, and pair order.

For the tie fixture, the manual algebra prediction is: candidate3/root4 and
candidate12/root13 are each distance4 from queryroot3; identityroot0 is
distance8. Predicted nearest-address before/after448/352 and rank-hinge192/160
are declared expectations, **not GPU measurements**. A disagreement blocks
the fixture/implementation and must be investigated without rewriting a bar
on-lease. The context2 Route(0) fixture has a two-leaf frontier; candidate7 is
prospectively sensitive (TA64→256, TR64→128). These are manual algebra
expectations too. Context3 was rejected in preliminary review because its
one-parent frontier makes Route(0) inert; no compute was used to discover or
repair that fixture design error.

## Independent immutable endpoints

For eligible_reference VectorBlock0, identity_and_tie VectorBlock0, and
route_endpoint Route0 and virtual_ineligible VectorBlock0, load sixteen
candidate endpoint snapshots per policy: **128 one-row CUDA evaluations**.
Materialization uses the existing literal
candidate-image table or fixed frame action identifiers; no host carrier,
writer, loss computation, candidate ranking or apply is introduced.

Each endpoint is a new admitted snapshot. Recarry inputs and evaluate entirely
on CUDA; require the observed token-rival count to be nonzero before comparing
its GPU objective to the virtual candidate loss in the first three endpoint
fixtures. An unexpected ineligible endpoint there is BLOCKED and retained;
never equate its zero absent-comparison objective with
the proposal MAX sentinel, silently drop it or replace it. Endpoint0 also
checks the identity cost. Retain the complete endpoint input/snapshot hashes,
GPU metrics and equality disposition for each candidate/policy.

The virtual_ineligible fixture deliberately tests the other branch. Identity
retains an observed rival; every nonidentity endpoint must have observed
token_rivals0 and the corresponding U/D proposal loss MAX. Do not compare its
absent-comparison evaluation objective numerically with MAX. Its two-row D
proposal trace must show visited/eligible masks3 for identity and1 for each
nonidentity lane, status/fault0, proving early termination after the first
eligible-but-virtually-ineligible row. These expectations are fixed before
execution and must not be weakened after observation.

## Admission refusals and fault reachability

The fixed admission inventory is eight attempts per policy, sixteen total:
invalid Route index U, invalid VectorBlock index U, Gate-in-Route U, wrong
batch/model U and D, wrong profiler U and D, and wrong probe D. The command
enumerates each attempted case in its pinned plan and reports actual
counterfactual submission state and error. These are real admission tests;
they are not evidence of a CUDA fault code. No stale device trace may be
attached to a refused, unsubmitted call. The exact list/count in the executable
plan is a reviewable execution pin, not an open-ended fuzz campaign.

| Requirement | Source-bound disposition proposed for FIT |
|---|---|
| CUDA2 invalid read/plan | UNREACHABLE_BY_ADMITTED_API: opaque BinderPlan construction, admitted nonempty windows and production loader validation exclude corrupt plans/routes. No CUDA2 observation claimed |
| CUDA4 invalid Gate index | Host parameter inventory rejects invalid indices; D permits Route/VectorBlock only. Execute actual admission refusals, not a fabricated CUDA4 result |
| CUDA9 read/proposal-kind mismatch | Public parameter admission rejects Gate-in-Route and Route-in-Flock/Relation; decomposition handle is Route-only. Report actual admission refusals and no CUDA9 coverage |
| CUDA8 no eligible rows | Execute all_ineligible above, compare real U/D status/loss storage and all sixteen D terminal records |
| CUDA12 checked overflow | Source-bound unreachable within admitted Route dimensions and root closure; retain host receipt overflow/malformed-trace checks as instrument metadata tests, not GPU arithmetic observations |
| Wrong model/profiler/probe | Execute real public admission refusals with separate valid seed1 model instances and actual model/stream bindings; require no counterfactual submission |
| Stale batch epoch | NOT_EXECUTED under zeroapply: epoch metadata is opaque, and no epoch-changing operation is permitted. Loading another epoch creates a new model and tests wrong binding first; it does not isolate stale epoch |
| Stale proposal/status3 | Explicitly outside scope: belongs to apply, which must never be called |

Overflow bound: admitted width≤128, each exact root coordinate in[-2,2],
token L1 distance≤4×width, address distance≤64, token hinge≤5×width,
address hinge≤72, occupied-cell rank hinge≤255. Thus row lossTA≤14,336 and
lossTR≤37,760; at most64 rows give sum≤2,416,640, far below u64::MAX. This
argument depends on unchanged validated roots, Route closure, writer and row
bounds; any change to them requires a new disposition. It is not a dynamic
overflow-branch test or blanket proof of other kernels.

Sources: `Device::admit_batch`, `Device::parameter`, `carry_windows`,
`Device::from_snapshot`, `DecompositionProfiler` binding checks, and the
`discrete_counterfactual`/`writer` implementations. If review requires actual
CUDA2/4/9/12 injection or isolated stale epoch, the current command remains
BLOCKED until a separately designed kernel-ABI fault campaign is reviewed.
Do not bypass public guards or weaken zeroapply to manufacture coverage.

## Exact pins, reporting and admission

Before finite execution, bind merged commit/tree, complete standalone source
inventory/hash, compiled executable SHA256, composite source, ordinary U PTX,
D PTX, geometry, fixture/plan hash, CUDA/Rust versions and runtime/AOT resource
receipts. A prospective source/PTX pin is not an executable checksum. Refusing
templates and a host plan are NOT_RUN; run admission must reject placeholders,
identity mismatch, seed2 or training scope before model execution.

The refusing source-template schema is `m1-counterfactual-controls-pin/1`.
Execution requires stage `execution_pinned`, exact source commit, standalone
source SHA256, running executable SHA256, composite source/U/D PTX/geometry,
fixture-inventory hash, seed1/evaluation-only/non-training scope, a new output
directory, and independent FIT plus separate execution-release references.
References name reviewed external authority; a syntactically valid URL does
not mint that authority. The release reviewer checks the actual comments.

Host-only source/metadata commands, with an isolated build target:

```sh
manifest=docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml
export CARGO_TARGET_DIR="$PWD/local/controls-host-target"
cargo +1.97.1 test --locked --manifest-path "$manifest"
cargo +1.97.1 run --locked --manifest-path "$manifest" -- \
  decompose-controls-template --out-dir new-controls-records > controls-pin-template.json
cargo +1.97.1 run --locked --manifest-path "$manifest" -- \
  decompose-controls-plan --pin controls-pin-template.json
```

Those commands create no CUDA context or model result. The identity command
`decompose-controls-identity` returns the embedded standalone inventory,
running-binary digest, fixture inventory hash and, only in a CUDA build,
compiled crate/U/D/geometry identities without creating a context. Use the
CUDA-built executable's identity, not the host binary's digest, for execution.
The standalone source hash frames Cargo.toml, Cargo.lock and all four source
modules; the release also verifies the protected Git commit/tree and complete
build dependency/toolchain receipt.

Prospective on-pod commands, **only after all gates and separate release**:

```sh
./k16-timing decompose-controls-identity > controls-identity.json
sha256sum ./k16-timing controls-pin.json
./k16-timing decompose-controls-plan --pin controls-pin.json
./k16-timing decompose-controls --pin controls-pin.json
```

The future release supplies the actual executable/path, pin bytes, authority
references, pod identity and archived manifest. This source packet is not a
filled execution pin. Compile-only construction gates use the admitted remote
recipe at the exact protected PR head: CUDA check and strict clippy/all-targets
for this standalone manifest; they execute no finite fixture or diagnostic.

Retain expected and observed fields separately, before deciding the outcome:
fixture/input/snapshot identities; policy/parameter/mode; actual submission,
CUDA status, all losses, optional five proposal words; raw D16×40 words;
decoded lanes and readback errors; before/after state/epoch and apply_calls0.
Do not discard terminal fault records or turn failed trace readback into an
apparently absent optional field. Successful receipts use existing full
parity validation. Expected status8 receipts use a separate fault-aware
validator; success-only ProposalReceipt must remain strict.

Runtime resources must satisfy the accepted prospective rule: zero AOT
spills; both runtime register/AOT register and runtime local/AOT stack match
with zero deltas; identical shared bytes; D≤U+1024 local/stack;
max_threads_per_block≥16 is launchability disclosure. Retain both U/D reports
before controls. Failure stops all further dose and is UNRESOLVED, never
mechanism evidence. No candidate compiler tuning on-lease.

The later diagnostic still requires original checkpoint0/config transfer
digests and the evaluation-only in-memory source bridge. Preserve calibration
provenance: zero-spill layouts126 registers/22,096 stack and194/21,536 were
observed under the superseded register ceiling; both satisfy the amended
static envelope, with no runtime/timing comparison selecting among them.
The56/56-spill capped layout and U+16 ceiling remain superseded. Carry the
limitation: no profiling-off control for the already-merged enclosing
host/event instrument — E/S bounds only the incremental internal clocks.

## Resource and outcome decisions

Only after source/gate review, protected merge, fresh-main verification,
independent FIT of this inventory and separate bounded execution release may
one owned Vast5090 lease execute it. Twenty minutes at the previous$0.908/h
quote is a prospective$0.303 allowance; obtain the fresh quote, include setup,
build/control/archive and retries in cumulative cost, and respect ≤4 pods /
≤$8/h / no new spending class. Never cancel healthy progress because a
self-imposed planning estimate expires; renew within owner caps.

Enforce ≥30GiB free before admission, admitted uor-pod only, owned lease and
EXIT teardown trap. Preserve SHA256 manifest and verify cloud MD5 roundtrip
before teardown. A source-only PR has zero GPU execution cost; construction
leases and actual controls are separate recorded costs. Storage/transfer USD
remain UNMEASURED unless observed.

Any loss/status/proposal disagreement, wrong-binding acceptance, missing
terminal/readback evidence, resource failure, unexpected apply or changed
snapshot/epoch yields BLOCKED / INSTRUMENT_DEFECT. Passing yields only
FINITE_CONTROL_PASS for the exact binary/fixtures/U-D pair. It does not release
the separate diagnostic, H1 magnitude, optimization or training. Training/M2/
Stage B HELD; Q4 NOT_EXECUTABLE; no seed2.
