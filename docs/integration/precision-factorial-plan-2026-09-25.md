# D8 numerical bridge — fixed-checkpoint precision comparison

**Frozen before evaluation, September 25, 2026.** Follow the
[projected-continuation recommendation](projected-recurrent-result-2026-09-25.md#recommended-next-milestone-isolate-the-numerical-bridge-once)
under #973 and programme #820. Both quantized continuation recipes remain
rejected. This milestone diagnoses their numerical gap without another fit.
The transformerless integer/table objective and D0-b remain unchanged.

## Fixed inputs and intervention

Use both final projected floating checkpoints at step **8,348**, and their
retained packed/shadow read-enabled evaluations. The
[input receipt](../evidence/precision-factorial-inputs-2026-09-25.json) binds the
parents, frozen quantizer specifications, evaluator and endpoint reports.
There are **zero optimizer updates, no recalibration, no checkpoint selection,
no new corpus and no fresh final holdout**.

The first letter selects parameter precision; the second selects all five
declared interface grids. F means the stored floating values pass through;
Q means the existing full-strength fixed quantizer is applied.

| Mode | Parameters | Declared interfaces | Role |
|---|---|---|---|
| FF | Stored shadows | Floating | Reproduce retained shadow endpoint |
| QF | Fixed quantized values | Floating | Parameter intervention |
| FQ | Stored shadows | Fixed grids | Interface intervention |
| QQ | Fixed quantized values | Fixed grids | Reproduce retained packed endpoint |

All four views originate from the same sealed floating parent and retain its
unchanged quantizer metadata. The new QQ view is re-quantization of that parent,
not itself a packed-loader execution. Its entire population and response panel
must equal the retained packed-loader results. A focused loaded-codec test also
covers the current loader; historical packed roots stay separately bound.
Neither mixed mode is a serving candidate. Even QQ remains an F32 emulator:
matmul, normalization, softmax, nonlinearities and probability operations are
not converted into integer serving by this experiment.

Carry the mode through prepared tensors and incremental sessions; reject
cross-mode session reuse, training, saving/export, clock changes and stripping
the mode. Preserve the legacy default behavior and checkpoint schema. Reject
packed parents, incomplete ramps and already prepared/diagnostic views. Record
both precision switches rather than a misleading mixed scalar strength.

## Matched measurement and validity gate

CPU, release build, **batch 16 / context 256**, one nested backend thread,
at most two independent model processes. Batch 16 matches the retained
endpoints; it corrects the initial planning draft's batch 8 before any run.
Evaluate all **249,856 targets**: tune prefix 16,384 and exposed comparison tail
233,472. Keep the same five seeded free generations and all 16 source-edit
pairs (32 responses) per mode. Read access stays enabled throughout each prefix.

Execute fresh FF and QQ in both arms first. Require bitwise equality of the
complete `targets.bin` and every field of `generations.json` and
`story-probes.json` except their explicit `elapsed_seconds` timings. Verify all
sealed roots, target identities, evaluator/parent/configuration bindings and
unchanged quantizer specifications. Any mismatch stops attribution. Do not
relax equality or change batches to make it pass. Then evaluate QF and FQ on
the same population and panels. Retain every attempt in an exclusive root.

Focused tests cover independent parameter/interface application, full-forward
versus incremental computation, future-input causality, endpoint equality,
mode identity and prohibited mutations. The actual loaded runs establish
whether each factor changes outputs on these fixed parents. If one factor is
numerically inert, report it; do not infer a general absence of importance.

## Analysis and decision rule

For each arm and partition report L_FF, L_QF, L_FQ and L_QQ, all in nats/target.
Compute these signed conditional costs explicitly:

- Parameter cost with floating interfaces: `L_QF - L_FF`.
- Parameter cost with quantized interfaces: `L_QQ - L_FQ`.
- Interface cost with floating parameters: `L_FQ - L_FF`.
- Interface cost with quantized parameters: `L_QQ - L_QF`.
- Interaction: `I = L_QQ - L_QF - L_FQ + L_FF`.

Thus `L_QQ - L_FF = (L_QF - L_FF) + (L_FQ - L_FF) + I`.
The interaction is symmetric; it cannot uniquely belong to either family.
Report four contiguous comparison quarters, per-target improved/equal/worse
counts, and source-response gains/losses separately. Inspect all decoded free
generations. Any block-level dispersion is descriptive on exposed development,
not a population generalization guarantee.

Each mode generates its own recurrent history. Effects include history-mediated
changes in state, keys, values, copy mass and output, within a 256-token block.
Parameter quantization includes tied input/output embeddings; interfaces bundle
five grids. This factorial does not identify one tensor, one grid, clipping
versus rounding, a training counterfactual, an attention-specific benefit or
geometric advantage. A shadow-to-hard gap also omits the shadow's retained
learning deficit against continuous training.

If one family's positive conditional costs dominate consistently across both
arms and the fixed partitions, use that evidence to select one bounded,
contract-compatible numerical bridge candidate. If attribution changes across
partitions/arms or interaction dominates, state that a single family is not
identified and choose a joint numerical design or stop that proposed repair.
No empirical dominance guarantees recovery after training. Do not raise the
four-bit weight contract or weaken the original +0.05-nat retention gate.
Future candidate acceptance must use both factors on, retained behavior and
matched ordinary controls; no new training recipe is authorized by a mixed
mode's quality alone.

## Resources and delivery

The [prospective budget](../evidence/precision-factorial-budget-2026-09-25.json)
reserves two hours including recovery, implementation, build, all evaluations,
review and delivery, within the existing cumulative allowance. No extension
is required initially. Reuse the declared warm target cache, one Cargo process,
4 GiB per model / 6 GiB aggregate RSS, 2 GiB total new allocation, and a physical
20 GiB reserve plus 128 MiB stop margin and 64 MiB closeout headroom. Recheck
physical space and retained allocation before build and each pair; enforce both
ceilings. Preserve all unique research and prior candidates; no cleanup or paid
external training compute is planned.

Record source and executable hashes, all local checks, paired runtime/RSS,
storage receipts and cumulative wall time. Independent RDC mathematics and
systems reviews inform the principal decision; correct their claims against
source and arithmetic. Deliver code, evidence, outcome and exactly one next
recommendation through a protected PR; synchronize the live plan, current
state, README, map and owning issues. Queue acknowledgements are not tests.
