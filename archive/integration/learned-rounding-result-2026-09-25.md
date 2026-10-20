# D8 learned integer-code choices: paired retention gate passed

**Completed September25, 2026.** Both final hard candidates pass all five
unchanged [prospective criteria](learned-rounding-plan-2026-09-25.md#acceptance-and-decision).
Retain them for **rung3 engineering continuation**. This clears the numerical
retention barrier that rejected the two previous discretization recipes. It
does not qualify integer execution, useful general language or geometric advantage.

## What changed and why

The [fixed-parent precision comparison](precision-factorial-result-2026-09-25.md)
identified parameter quantization as the dominant immediate numerical loss.
Nearest rounding minimizes individual coordinate error; the model's prediction
loss depends on coupled parameters across the recurrent state, contextual reads
and output distribution. The implemented change learns those code choices
jointly from the full256-step language loss, with a declared binary relaxation
and hardening penalty. It follows the [AdaRound motivation and inspected source](learned-rounding-plan-2026-09-25.md#independent-review-and-principal-decisions),
using a different full-trajectory objective and explicit normalization.

Both projected step8,348 parents, scales, bit widths, architecture and interface
grids remain fixed. The new alpha optimizer receives **512 updates /2,097,152
fitted target visits per arm**, B16/T256, plus32,768 training-only normalization
visits and zero normalization updates. The identical eight-batch norm rule fixes
coefficients **8.4047511404 /10.3671279136** before learning. These are one recipe's
initialization values, not selected winners from a search. The continuous
references remain the original step8,348 retention anchors; the new candidates
receive extra code-learning work, so this is not an equal-additional-compute
comparison against new continuous training.

`joint_rounding.rs` owns the legal floor/ceil neighborhood, tied alpha decisions,
soft/hard tensors and coordinate-mean penalty. `joint_rounding_campaign.rs`
binds training-only normalization, alpha/Adam checkpoints, contiguous sampler
counters and final materialization. The shared recurrent graph differentiates
through the soft weights once, retaining the fixed interface quantizers.
Exact-grid coordinates stay fixed. The original parent weights and Adam state
are preserved. Final packed loading requires neither alpha nor shadow weights.

## Final hard results — original thresholds unchanged

Comparison tail:233,472 positions of the same previously exposed evaluator-v2
population; lower NLL is better. Every249,856 target record is verified, including
the16,384-position tune prefix. No final holdout or checkpoint selection occurs.

| NLL, nats/target | Quaternion | Householder pair |
|---|---:|---:|
| Retained continuous reference | 2.090518499 | 2.064403221 |
| Prior projected hard parent | 2.150319798 | 2.147928712 |
| **New learned hard codes** | **2.114226169** | **2.110879800** |
| Change from projected hard parent | **−0.036093629** | **−0.037048912** |
| New hard minus continuous | **+0.023707670** | **+0.046476579** |
| New whole-prefix NoRead | 2.609608832 | 2.607989949 |
| Combined NoRead penalty | +0.495382663 | +0.497110149 |

| Frozen gate | Quaternion | Householder pair |
|---|---|---|
| Hard minus continuous ≤0.05 and below count/cache2.391786179 | PASS | PASS |
| Combined whole-prefix NoRead penalty ≥0.02 | PASS | PASS |
| Five actual continuations nonconstant, no short-cycle collapse | PASS | PASS |
| At most2 rung1-correct first-noun rows lost | PASS:2 lost | PASS:1 lost |
| Packed reload parity, finite normalization and provenance | PASS | PASS |

The ordinary learner retains lower natural NLL. One paired seed supplies no
geometric-advantage claim. All four comparison position quarters also remain
within0.05 of their continuous references: quaternion gaps
0.027753/0.021854/0.019575/0.025649; ordinary
0.048590/0.046706/0.047827/0.042783. Those quarters are descriptive, not additional
independent trials or evidence beyond the256-token context.

## Actual output and retained regressions

New first-noun correctness is **29/32 and30/32**. Exact completion is **28/32
and24/32**, with **12/16 and10/16** complete source-edit pairs. Against the
projected parent, exact-completion losses/gains are2/7 quaternion and4/6 ordinary.
Against rung1, **both lose3 previously correct complete answers**, while gaining
3/4 respectively. These complete-answer regressions remain visible; the frozen
gate governs first nouns, and gains never cancel losses.

Quaternion's lost rung1 first nouns are edited story06 (`cap.`→`captain.`)
and edited story15 (`ribbon.`→`father.`). Ordinary loses original story14
(`pencil.`→`pencils.`). The [full output record](learned-rounding-outputs-2026-09-25.md)
contains every new response and its corresponding rung1, initial PTQ,
continuous, unprojected and projected control, plus all retained free generations.

All ten new read-enabled generations were inspected. They avoid constant output
and short token cycles, but still have entity/role confusion, malformed words,
repetition and implausible events. For example, quaternion's lion story changes
Max from a dog to a cat; ordinary's family story describes Mommy as a boy.
Passing this engineering gate is not useful prose, conversation, reasoning or
coding qualification. No source-panel answer is used for training or repair.

## Artifact and execution evidence

- Executed Rust source: `036eabcc86e453f581aa0563e2967cc1f7db96d5`.
- Executable SHA256: `7cc87d9c4c3c3060c8e51c1a763d177efbd734228a6ed172bc18d635707881a3`.
- Container: `/Users/casey.allard/uor-r4-investigations/learned-rounding-20260925`.
- Final candidates: `fit-quaternion-rounding-3/packed-model` and
  `fit-householder_pair-rounding-3/packed-model` within that container.
- Changed hard codes: **128,083 /126,324** of1,678,466 coordinates. Legal
  fixed-grid code inventories, scales and layouts are unchanged. Fixed
  coordinates number7,362/8,513. Final clamp occupancy is reported separately;
  no assumption that every relaxed coordinate reached an endpoint is required
  to execute the actual hard candidate.
- Full-context packed reload probability delta is **zero** in both arms;
  seeded token IDs and per-decision probability hashes agree.
- Principal arithmetic verifies **32 sealed roots and44 binding records**,
  exact target identities, finite/normalized distributions, all partition means,
  source-row gains/losses and the complete resume lineage.
- Actual normalization hashes agree across arms. The three fit segments
  **0→1→479→512** also have equal ordered input/target hashes across arms.
  Parent model clock8,348 and rounding clock512 remain distinct in the artifact.
- **42 focused optimized Rust tests pass in3.59seconds**, plus the optimized
  binary build and actual model execution. An initial test compilation error
  and three unprojected fixture setups were corrected before model execution;
  their failed attempts/logs remain retained. Queue acknowledgements execute
  no tests.

The [complete arithmetic, outputs, lineage and principal decision](../evidence/learned-rounding-result-2026-09-25.json),
[resolved recipe freeze](../evidence/learned-rounding-resolved-recipes-2026-09-25.json),
[budget](../evidence/learned-rounding-budget-2026-09-25.json) and
[resource closeout](../evidence/learned-rounding-closeout-2026-09-25.json) bind these statements.

## RDC, resources and preservation

RDC ran two concurrent **DeepSeek** sessions: source/mathematics review and
implementation. The principal integrated the controller and reviewed source,
primary literature and final evidence. The normalization and memory-limit
corrections, rejected reviewer inferences and narrower floor/ceil neighborhood
are recorded in the [prospective plan](learned-rounding-plan-2026-09-25.md).
No Kimi model was used.

RDC also ran the paired local Rust jobs on the same M1; it did not provide extra
training hardware. Each arm uses two whole-sequence CPU workers, with nested
backend threads limited to one. Three nonoverlapping fit supervision intervals
total **2,889.479seconds (48.16minutes)**. Normalization and four final evaluations
are separately recorded. The complete-cycle ledger also charges preparation,
failed builds, review, storage interruption, resumes, analysis and delivery.

The first update hit the original physical reserve and checkpointed safely.
A prospective2GiB reserve adjustment under standing authorization preceded
resume. A later configured2700second process boundary checkpointed at479;
both resumed the same recipe to512. No learning was discarded or repeated.
All unique artifacts, negative candidates and worktrees are preserved; no
cleanup occurred. RSS is sampled, compiler/temporary peaks and physical energy
are unmeasured. Storage inventory is not a measurement of exclusive APFS extents.

## Next direction: trained bounded access, then integer execution

**Advance to D8 rung3; end this rounding campaign.** Retain both successful hard
candidates. The next implementation must put the actual bounded serving
admission rule into the shared training and incremental-session graph, preserving
exact occurrence identity, causality, orientation and an explicit NoRead path.
Use the full-context computation and a matched ordinary/exact-cache access rule
as controls. Count all admission work: a full-context scan followed by top-k is
not evidence of bounded admission cost independent of context length.

Then integrate observable quantized group transport and measure accumulated
state/output error. The model should learn under those deployed constraints.
Only after that bridge is retained should the same computation be compiled to
integer/add/subtract/shift/lookup execution and assessed on useful complete
conversation and code outputs, parameter access, latency/RAM and physical energy.
Prime/zeta identity, R4/H4 transport and exact memory keep their distinct roles;
this result does not establish a Hamiltonian mechanism or semantic distance from
identity hashes. No per-tensor/scale sweep, extra rounding schedule or broader
architecture retirement follows this completed result.

#973 and programme #820 remain open: integrated useful language and final serving
acceptance are still unmet. The target and D0-b/D4–D6 remain unchanged.
