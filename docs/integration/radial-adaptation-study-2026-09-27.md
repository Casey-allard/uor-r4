# Proposed three-arm radial reader adaptation

September 27, 2026. Fourth lab; References #973 under #820.

**Status: proposed fixed adaptation screen; fitting NOT_RUN.** The proposed dose
is 1,024 updates per arm, with one descriptive checkpoint at 512. This document
does not supply the complete resource envelope or authorize a fit launch. Before
execution, the owning #973 work card must bind the exact campaigns, optimized
binary and whole-cycle projection against the live cumulative ledger. Necessary
local extensions follow the existing standing authorization and must be recorded
before use; no new owner permission is implied for that already authorized class
of extension. [Current state](current-state.md) owns changing status and next
action; this document owns this prospective comparison, not another state mirror.

## Question and integrated deliverable

The [learned-parameter transfer](radial-parameter-transfer-2026-09-27.md) preserves
the language parameters of the retained Dot model while explicitly replacing its
read law. Its completed zero-update observation finds finite connected gradients
and different read masses in Lorentz and LorentzAffine. These gradients establish
a connected training signal on the declared inputs, not useful credit, adequate
gradient magnitude or learnability at a particular dose. No observed numerical
failure justifies another startup or a calibration sweep.

The [Dot-reset witness](../evidence/dot-reset-validation-2026-09-27.json) also
preserves all 21 arrays and the parent's predictions under fresh zero optimizer
clocks. Its lower loss on the same four observed windows makes the initial
disturbance from changing the read law explicit. That observation is neither a
fitted ranking nor a speed comparison; the adaptation study must earn back that
disturbance as well as demonstrate any eventual practical benefit.

The deliverable is one retained three-arm continuous adaptation result with
natural-token likelihood, complete source-edit responses and actual prose. It
answers two questions: does the nonlinear Lorentz distance earn its additional
score computation over the same lifted information with an affine score; and
does either radial reader improve useful behavior over adapting the existing
Dot reader under the same reset and exposure? A lower training loss alone does
not answer either practical question. The
[radial design](radial-read-control-2026-09-26.md) defines the score equations and
their limitations; this study does not assert unique curvature advantage.

## Fixed parent, arms and learning conditions

The continuous F32 parent is
`/Users/casey.allard/uor-r4/.uor-models/investigations/language-continuation-20260925/fit-quaternion-6/checkpoint-final`.
It has model/Adam/data step 15,672 and 64,192,512 historical training target
visits. Its [continuation result](language-continuation-result-2026-09-26.md)
remains negative; it is not the accepted September 25 quantized parent. Preserve
both lineages. The [transfer receipt](../evidence/radial-parameter-transfer-validation-2026-09-27.json)
binds the exact parent, all shared arrays, tokenizer and evaluator:

| Identity | SHA-256 |
| --- | --- |
| Parent model | `6defec21fc2be395a9505b9f10301c03aeb3c23529c6e1be4e2e1639c7ce6d79` |
| Parent checkpoint metadata | `490922decefee3e6036331dd0bc112746f24662d4bddfbe540b59c1d00820571` |
| Parent campaign | `6b34a04d49fa965dc20335002a1948a8253ef74e70890ba1e9484f63d2677f14` |
| Canonical evaluator | `d2432fbba0e24ba51d7568700d6718c4e85d01ccc08e4fc3cc3fa2a77e928a62` |

All three children copy the same 21 learned arrays exactly: 1,678,466 shared
scalars, including query/key biases, NoRead, age, value, copy, transport and output
parameters. The copy is an explicit new adaptation lineage, not an ordinary
resume with a relabeled operator. Each actual transfer receipt must be retained.

| Condition | Dot-reset | Lorentz | LorentzAffine |
| --- | --- | --- | --- |
| Read law | Existing scaled Dot | Existing clamped acosh distance | Fixed tangent of that distance |
| Shared learned arrays | Same 21 parent arrays | Same 21 parent arrays | Same 21 parent arrays |
| Additional learned radial scalars | None; no dummy scalars | log-beta and offset | Same two scalars |
| Radial initialization | Absent | Explicit `unit_scale`: beta 1, offset 5.33172894 | Identical beta and offset |
| Optimizer and data clocks | Fresh zero clocks | Fresh zero clocks | Fresh zero clocks |
| Local endpoint | 1,024 updates | 1,024 updates | 1,024 updates |

For Affine the tangent reference remains fixed at the constructor reference
distance; training the offset does not move that reference. Lorentz and Affine
therefore match learned parameter count and radial information, but not every
initial score, gradient, NoRead mass or later recurrent trajectory. Dot-reset is
the practical baseline and has two fewer scalars; it is not a capacity-exact
isolation of the nonlinear radial law.

The remaining conditions are common and fixed:

- Quaternion transport; model seed 240924; vocabulary 4,096; state width 256;
  read width 64. These coordinate widths are distinct from sequence lengths.
- Training length 256, evaluation length 256 and generation session context 256.
  Full causal history access within that window; no recent64, orthant admission,
  routing-budget change or context shortening.
- Batch 16 and two whole-sequence CPU gradient shards, or 4,096 targets/update.
  Shard count is not a declaration of process/BLAS/Rayon thread limits.
- Fresh Adam moments and all parameter/global clocks at zero. Preserve the saved
  optimizer recipe: learning rate 0.001, weight decay 0.01, parameter absolute
  limit 1,000,000 and `allowed_missing_gradients=[]`. There is no configurable
  clipping field in that saved campaign; retain the implementation's unchanged
  optimizer numerics. No learning-rate pilot, new schedule or optimizer sweep.
- Fresh counter sampling with `data_seed=240927`, local steps 0 through 1,023,
  identical counter algorithm and sampled windows across arms. Use both training
  stores and their identities from [reference evaluator v2](reference-evaluator-v2.json),
  with the existing valid-window-union sampler and no cross-store windows. A new
  seed does not create unseen data or guarantee nonoverlapping windows.
- Ordinary unweighted next-token objective. No termination/end-token reweighting,
  quantization, projection, alpha fitting or training-window transition.

The proposal supplies 4,194,304 new training target visits per arm, approximately
6.53% of the parent's historical visits, and 12,582,912 visits across the three
arms. Report those new visits separately from inherited exposure; neither the
reset clocks nor a new issue erases historical work. This is a fixed adaptation
screen, not a convergence claim. The endpoint is selected prospectively to
observe a substantive response of the learned reader without automatically extending the earlier
exposure-only continuation. Its adequacy remains a study
limitation, not a reason for automatic extension.

Use a 64-block development observation every 512 updates. Save
checkpoint 512 for recovery and descriptive learning progress; the sole planned
candidate is checkpoint 1,024. Do not select whichever checkpoint scores best,
promote the midpoint, or attach a second full evaluation campaign to it. A genuine
integrity or resource stop preserves the actual checkpoint and consumed cost.

## Evaluation and generated behavior

Use the existing canonical Rust evaluator and its exact tokenizer, data, prompts,
reset rules and operator identities. Reuse verified parent outputs when their
identities and policies match. Record initial 64-block development loss and the
learning curve for each arm; the four-window startup NLLs are not full-development
baselines. At the fixed endpoint, evaluate each loaded final checkpoint in Read
and NoRead modes over all 976 complete blocks: 249,856 targets, partitioned into
the first 64 blocks/16,384 targets and the following 912 blocks/233,472 targets.
Report both partitions and the full mean, paired block/target differences and
the actual input/target identities. The six final passes total 1,499,136 scored
targets, separate from fitting and periodic development work.

The entire development population has been exposed during prior design and
checkpoint decisions. The 912-block partition is called the comparison tail for
continuity, not an untouched test set. One seed and this exposed population
support a development decision, not statistical generalization. A NoRead penalty
measures the combined removal of read feedback and copying; it does not isolate
geometry. Changed mass/entropy without better generated behavior is descriptive.

Run the unchanged 16 source-edit pairs, both original and edited sides (32 rows),
using the existing greedy policy and 32-token maximum. Preserve the exact
[story probe contract](../../crates/uor-r4-training/src/joint_evaluation.rs): stop
at the first generated ASCII period while retaining the complete last token, or
EOS, terminal short cycle, or the cap. Complete correctness requires decodable
UTF-8, a period/EOS stop, and exact membership in the typed accepted noun-plus-
period answers. Do not truncate an extra phrase, repair punctuation, or count a
correct first noun as a complete answer. A pair passes only when both sides are
complete and correct; report output differences separately. Report all 32 row
verdicts, all 16 pair verdicts and lost/gained rows versus the unchanged continuous
parent and the matched Dot-reset final. Keep comparisons with older accepted
parents explicitly labeled by their different numerical path. This task format
is exploratory transfer, not by itself general-language qualification.

Also generate all five frozen prose prompts under both unchanged policies, with
128 new-token cap and full256 session context: greedy argmax with the existing
tie order, and sampled temperature 0.8/top-k40/Q32/SplitMix with seeds 2014–2018.
Use the existing `joint-evaluate` sampled outputs. Add the five greedy
continuations through its existing generation function while the same checkpoint
is loaded, as an explicit opt-in supplement. This avoids repeating source panels
or sampled outputs and leaves `joint-selection-replay` unchanged. That small
implementation remains to be delivered. Preserve complete text, IDs and stop reasons. No policy is selected after
viewing results, and no sampler, seed, prompt or cap sweep is introduced.

Score each story on the frozen four binary criteria: resolvable entity/role
identity, intelligible event progression connected to the prompt, understandable
literal language, and a complete ending clause at a stable/resolved point. All
four must pass. Personification and explicit new characters are allowed; minor
grammar issues, unfamiliar names and a length-cap stop alone do not fail a story.
A cap that leaves an incomplete clause still fails the ending criterion. Record
each dimension and actual textual evidence; preserve reviewer disagreements.

Report same-policy story gains/losses versus the unchanged continuous parent
(sampled 0/5, greedy 2/5 under the existing strict rubric) and versus the matched
Dot-reset final. Do not pool the policies or import a new aggregate acceptance
threshold into this exploratory comparison. Historical continuation decisions
keep their frozen scope and are not re-adjudicated here. A radial arm that merely
follows the Dot reset's improvement has not established a reader benefit. The
[selection diagnostic](selection-policy-diagnostic-result-2026-09-27.md)
already rejects a sampling-only account of the source failures; this study does
not restart that policy investigation.

## Distinct decisions after the fixed endpoint

| Observed result | Programme action |
| --- | --- |
| Lorentz improves natural likelihood and useful generated behavior over both Affine and Dot-reset, with source losses disclosed | Retain a scoped nonlinear-score result and prioritize integration of that learned reader into the existing Lorentz export/session path. Numerical preservation and useful integer output remain later obligations; no curvature, hierarchy or efficiency claim follows. |
| Affine retains or exceeds Lorentz's practical benefit and improves over Dot-reset | Prefer the simpler score provisionally for subsequent geometric-reader engineering; decide whether its absent integer operator is worth implementing from the actual benefit and cost. Do not relabel the result as proof that all geometry is unnecessary. |
| Dot-reset matches or exceeds both radial arms | Retain the native Dot path and park these transferred radial configurations. Advance the independently justified language/emission or architecture work; no automatic dose, scale or seed sweep. |
| Likelihood improves but prose/source behavior does not, or gains trade against concrete regressions | Report the exact rows, policies and loss changes as a mixed result. Use that evidence to select the next ranking/emission integration change; do not promote by an aggregate NLL or count alone. |
| All three remain poor at this dose | Preserve a scoped negative adaptation screen. It does not prove capacity saturation, inadequate training signal or impossibility of a radial reader. Continue an independent architecture or interface dependency unless existing evidence identifies a new causal intervention. |
| Nonfinite computation, broken provenance or an incomplete resource-limited run prevents the comparison | Report UNVERIFIED or NOT_RUN at the affected scope, preserve artifacts and costs, and repair only the named integrity/execution blocker within a prospectively accounted continuation. Do not label missing evaluation a model-quality failure. |

A numerical tie or inconsistent small-panel outcome remains unresolved at this
scope; it is not converted into evidence of equivalence. The three-arm endpoint,
not repeated startup measurements or a preferred NoRead threshold, supplies the
decision. Any later reopening needs a concrete new cause and a decision it can
change, under the existing progress-control rules.

## Whole-cycle cost and launch boundary

No fit is launched by this proposal. Before execution, record one complete
projection in the existing work card and shared ledger: preparation/review,
necessary build, initial observations, all three fits, periodic development,
checkpoint writes and reload exercise, six final full evaluations, source/prose
generation, report verification, analysis/delivery, recovery allowance
and stop margin. Configure wall time, CPU and all thread limits, peak RAM,
temporary/new/retained storage and checkpoint limits. Recheck physical storage
and the live balance; retain the 128 MiB stop margin and all unique parents and
negative candidates. The trainer's update deadline is not a closeout deadline.

In particular, `joint-fit` performs initial retained-batch/development work,
final retained-batch/development work, saved-checkpoint reload and another
retained-batch pass, then a 48-token generation. Periodic development at the final
step and closeout development both execute; budget both rather than counting
only update timing. Each `joint-evaluate` performs five sampled continuations
and the 32 greedy source rows before full likelihood. Six Read/NoRead calls
therefore already include six such packets; preserve the NoRead packets as
diagnostics, and add only the planned greedy free-prose supplement. Recovery repeats
setup/closeout costs. Six 44-byte-per-target final row files alone occupy
65,961,984 bytes; three arms' midpoint/final model-and-Adam files are roughly
120 MB before other reports, caches and recovery copies. These components still
do not constitute a full storage or wall-time allowance.

The [historical continuation cost receipt](../evidence/language-continuation-cost-detail-2026-09-26.json)
contains 5,509.46 seconds for 1,672 quaternion updates in fit6 and 8,600.74 seconds
for 1,723 ordinary updates in fit6: about 3.30 and 4.99 seconds/update under their
recorded conditions. Applying that range mechanically gives roughly 56–85 minutes
per 1,024 updates, or 2.8–4.3 hours across three arms, **for update work only**.
The ordinary rate is context, not a measurement of either new radial arm. This
arithmetic is not the missing whole-cycle projection and does not bind a budget.

The 183.70-second learned-parent startup used four windows and a test-profile
binary under shared load; its roughly 847 MB maximum child RSS and elapsed time
do not project B16/two-shard training throughput or memory. Do not extrapolate it
into the fit dose. Use a source-bound optimized build and record the actual
executable hash, compiler/profile, CPU/accelerator feature set and thread settings.
Those conditions must match across arms and any measurement used to project
them. Do not compare an unoptimized debug/test execution with a historical
optimized/Accelerate rate or assume that a source change kept a cached binary
valid. Actual early update timing within the accounted study can revise the cost
estimate and trigger its configured resource stop; it does not create another
calibration campaign or silently raise the limit.

Each report/checkpoint root is claimed exclusively before loading models, then
sealed and complete-set verified. Resume a resource-stopped child with its own
saved parameters, moments, clocks and transfer receipt; do not reapply the Dot
parent transfer. Preserve unequal completed doses honestly until the planned
comparison is complete. Reuse resolved transfer/initializer/serialization checks;
no blanket test suite or new validation programme follows from this document.
The old `joint-compare` transport comparator intentionally rejects this reader
study. Reuse the bound loader/evaluator/generation paths and compare their
existing row records in a scoped result; do not loosen that comparator's contract.

## Companion ownership and architectural limits

The separately owned termination-objective track changes end-token weighting and
retains a different optimizer/data history. It is not this study's Dot control;
neither its objective nor its results are silently mixed into these campaigns.
Coordinate the shared machine/build slot before starting work.

Claude's draft [Cycle 4 stack](https://github.com/UOR-Foundation/uor-r4/pull/1414),
reviewed at `58f931174a07c9145dacda3591ae4885da770584`, explores a deeper
quaternion-recurrence/Lorentz-read layout against a similarly sized transformer
control. Its language results were pending at review. It changes capacity,
layout and data; it does not supply the same-parameter affine control or a reason
to expand this dose. Its associative recurrence currently scans time within
parallel windows, and its floating-point generator recomputes the context suffix.
No cached/integer serving, exact addressed-memory integration or M1 efficiency
result follows. Keep that Claude-owned experiment in parallel and reuse its
completed evidence when it can change the next architecture investment. This
offline study adopts neither a dense serving backbone nor D10, and does not
qualify general chat, coding/reasoning, terminal sparsity or energy savings.
