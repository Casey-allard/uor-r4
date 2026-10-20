# Learned geometric typed selection — #1139, 2026-09-06

## Mechanism and pre-run decision

The parent `2600b95b` generates three initial sums but fails all six subsequent
Copy/Add instructions. Their intermediate values and new operands are captured;
the prior result therefore calls for operator/operand selection learning.

This implementation adds an optional outer artifact component, preserving the
entire exact parent identity. Initial literal-only decisions retain the existing
numeric head. When a derived source exists, the new head scores NoOperation and
all existing Copy/Add operand proposals before executing the winner. NoOperation
does not fall back to the old numeric scorer; ordinary response mechanisms still
handle its output. Exact overflow reselects in the existing bounded score order.

The new head reuses `SourceRouting::encode`, signed relative-cosine rank scoring
and the discrete joint candidate/action learner. Its two independently learned
signed-H4 states fold ordered features by existing group-table products. Learned
action landmarks and integer biases choose the operation and operands together.
This is not a new paired-icosian/E8 construction or a whole-language decoder.

Represented metadata: derived flags for both operands, relative source ranks,
and ordered whole-word query identities plus position/identity features. At most
16 recent query words are matched against a construction-only prime dictionary;
numeric words are excluded. Literal numeric values never enter these geometric
scoring features. Query words outside the dictionary contribute no feature.
Feature selection retains structural entries first, then construction frequency,
up to 128 entries. The fold can collide in 120 x 120 states. It retains neither exact
payloads nor all raw query history; exact values, source IDs and derivations stay
in the existing typed records. Required fixed-zeta and paired-H4 state remains
inherited, without claiming a new predictive contribution from those fields.

The hot path uses integer comparisons, table reads and ordered group operations;
no runtime matrix multiplication, floating-point projection or response provider
is introduced. Startup validation retains its existing host floating-point scope.
The candidate scan remains bounded, not sublinear: at most 256 valid Copy/Add
proposals, each with bounded feature/dictionary/scoring work. Existing work counts
all proposal passes, exact executions and overflow fallback; a nested RoutingWork
counts the new source scans, comparisons, code/table reads and logical operand
bytes. Logical bytes are not measured DRAM traffic or an instruction census.

## Construction and evaluation

Rust prepares 42 authored follow-ups: six numeric worlds across three Copy,
three Add and one NoOperation wording. The initial response is actually generated
and observed; a supplied correct initial answer cannot populate training state.
Offline labels supervise the joint operator/operand choice. Initial generation
must match its expected complete answer before a frame is admitted. No expected
follow-up answer is supplied to serving. Before fit, effective feature signatures
must not have incompatible target sets; this checks only these actual frames.

Angular and exact-code arms use 128 feature slots, four passes, twelve sampled
code proposals, the same seed, and 30-second per-arm search caps. Their realized
search trajectories can differ. Fitting the hard joint choice is separate from
success of the assembled response generation. Complete text, EOS, operator and
intermediate provenance are required by evaluation.

Six previously exposed failures remain OPEN development. Six operand/wording
transfers are predeclared in the same source and opened after design selection;
they are small transfer checks, not broad or independently sealed language data.
Adoption requires 6/6 development, 6/6 transfers and previous response/write/session
preservation, plus focused arithmetic/counter/serialization/allocation checks.
Intermediate removal checks causal dependence. A failed transfer cannot be
silently reused as a held-out success after revision. No general reasoning,
prose, syntax or whole-model speed claim follows from this bounded step.

## Resource projection

Under the owner's standing necessary-allowance authorization, the recorded cycle
adds 240 seconds of cumulative local model allowance (1890 to 2130 seconds), with
160 seconds projected work plus 80 reserve. The build projects 600 seconds under
a 900-second ceiling. A recorded 128 MiB storage increment supports a 256 MiB peak
growth projection while preserving the existing stop margin. One model process,
4 GiB RSS target, no paid external compute and no deletion. Charge retries and
preserve all artifacts/results under the existing root with `typed-routing-`
prefix. The pre-run projection does not claim execution or capability success.

## Initial result and materially revised case handling

The first angular artifact `50eff8ad` fits 42/42 choices and generates 42/42
construction and 6/6 exposed answers; exact-code `9adf0cb5` fits 36/42 choices
and also generates 6/6 exposed answers. The angular intermediate-removal control
gets 0/6. Earlier 48/62/24/28 and five-session preservation all pass.

Both artifacts then get only 3/6 on the predeclared transfer set. All Add outputs
are correct, but `Please copy ...` selects Add. Construction dictionary metadata
contains `Copy` (prime 5) and has no lowercase `copy` entry. These failures remain
first-use negatives; neither initial artifact meets the 6/6 adoption criterion.

The revised artifact explicitly binds `fold_ascii_case: true`. It folds ASCII
case only for query metadata dictionary construction/lookup; exact typed values,
source identity and payload spelling remain unchanged. Older artifacts omit
this optional field and retain exact-case behavior and identity. The same 42
construction labels are refit with the same capacity and search schedule. This
is a new metadata normalization mechanism, not fitting on the failed transfer
answers or a serving grammar rule. Case distinctions in query scoring are lost;
case-sensitive identifier reasoning is not qualified by this numeric-word task.

The old six transfer cases become exposed repair checks. A new predeclared set
uses three new operand triples with `Please repeat ...` and `Please add ...`
wording. It stays unopened until revised construction/development selection.
Require 6/6 new transfers plus preservation before adopting the revised artifact.
Results, resources and final decision will be appended after actual execution.


## Executed result — 2026-09-06

**Retain case-folded angular `bb79456b` at bounded composition scope.** The
[evidence receipt](evidence/native_geometric_typed_routing_1139.json) binds the
full artifact IDs, source digests, command results and local report hashes.
Its parent remains exactly `2600b95b`; the optional component is 13, 865 serialized
bytes, with 109 learned features and 25 construction word identities. The same 42
construction records collapse to seven effective query classes with no detected
incompatible targets before fit. They are not 42 independent language patterns.

The revised angular arm fits 42/42 joint choices and generates 42/42 construction,
6/6 OPEN development, and 6/6 exposed V1 repair outputs. After that design selection,
it generates 6/6 new numeric/wording transfers with full text, EOS, correct operator
and actual intermediate provenance. The matched exact-code arm fits 24/42 and
gets 3/6 new transfers: Add works, while Copy abstains. This supports the selected
angular fit over this exact-code fit; one seed and six authored cases establish
neither universal angular superiority nor broad language generalization.

Actual new cases include 19+12 ->31, then repeat ->31 or add 5 ->36; 23+8 ->31,
then repeat ->31 or add 6 ->37; and-4+15 ->11, then repeat ->11 or add 3 ->14.
Every full-path follow-up consumes the actually generated intermediate. Removing
that record gives Unknown on 6/6, hence 0/6 exact. Removal also disables the derived-
source eligibility gate, so this is a whole-intermediate causal control, not an
isolated angular-score ablation. Initial-response generation is included in each
complete session's work; expected answers are only offline labels/comparisons.

Preservation passes 48/48 dependent, 62/62 earlier, 24/24 prior transfer, 28/28
exposed-name and 28/28 longer-context answers/writes, plus 5/5 persistent turns.
Four exact unchanged generated Rust bodies compile and pass 12 semantic assertions.
The CLI revision produces ` Zurich.\n`. Two metadata tests, four selected-execution
arithmetic/commit/overflow tests and one nested-counter test pass. The current
kernel source check passes, and actual two-turn typed routing/commit measures
zero allocations and zero allocated bytes. Loading each artifact validates its
identity and parent. The older ignored chain diagnostic is deliberately not rerun.
Release probe/CLI and final focused test binaries compile; formatting, architecture
policy and claim wording are checked locally. Broad release QA is NOT_RUN.

Across the six new two-response sessions there are 120 typed proposals, 12 selection
passes, 12 exact operator executions and 9 additions (six first sums and three new
adds). New routing counts 6 predictions, 420 source examinations, 27, 456 comparisons,
6, 120 code reads, 13, 362 table reads and 523, 692 logical bytes read. Full inherited
writer/readout/state/output counters remain in the retained reports. Routing is
bounded scanning, not demonstrated sublinear retrieval or a whole-model speedup.

The complete cycle charges 162.033 seconds of model/preparation/evaluation/check
work and 686.095 seconds of monitored engineering work. Model use is 2033.039/2130
seconds cumulative, leaving 96.961 seconds; the cycle ceilings were 240 and 900
seconds. Builds exceeded the 600-second point projection but stayed within 900.
The recorded 128 MiB storage increase is applied. Peak sampled known accounting is
6, 329, 102, 336 bytes below the effective 6, 506, 475, 520-byte ceiling; sampled model
child RSS is at most 853, 278, 720 bytes. These are periodic samples, not exact peaks.
No user material was deleted and no paid external compute was used.

**Next: role-sensitive choice among competing intermediate results.** This head
uses query identities, derived flags and relative source ranks. Its tiny numeric
transfer establishes a useful causal chain but cannot show that it selects a
named intermediate independently of recency. Start with two retained derived
results, change their order and the later operand, inspect captured query/role
features, and train the same joint geometric choice only where the comparison
exposes a failure. Reuse the current exact records/operators and existing Rust
checks. No additional cache, store, tokenizer or broad corpus campaign is needed.
Full #1139/#1140 acceptance and general syntax/prose/reasoning remain unmet.
