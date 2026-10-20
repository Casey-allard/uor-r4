# D8 rung 1: continuous recurrent memory learner

Status: full planned exposure and comparison completed September 25. Both arms
pass the frozen rung 1 engineering continuation gate at the [reported scope](joint-recurrent-result-2026-09-25.md).
The initial 64-window schedule remains superseded; its matched warmup is retained.
References #973 and #820.
This follows the completed [rung 0 comparison](reference-evaluator-v2.json).
The programme remains [project-track.md](project-track.md); changing outcomes
belong in [current-state.md](current-state.md).

## Question and complete graph

Can a compact recurrent model learn useful next-token prediction and causal
memory access jointly from ordinary language, before discretizing its operators?
This is offline floating-point Rust scaffolding for the native path. It has no
transformer backbone. Dense affine training operators here are not a qualified
integer serving export, a mutable Hamiltonian proof, or evidence of energy savings.

The observed token and normalized prior state produce a candidate, write gate and
four raw coordinates per R4 lane. A gated transported state produces the query.
That query reads earlier contextual keys/values, with a learned age term and a
NoRead option. The read then changes recurrent state; the resulting state writes
the next key/value. A tied vocabulary head and normalized pointer mixture emit
the next-token distribution. Targets enter only the loss, and its gradients reach
all of these operations through the complete unroll. Exact token and occurrence
identity remain separate from the learned compatibility score.

Two arms share parameter shapes, initialization, data windows and optimization:
left multiplication by a normalized unit quaternion, and an ordinary product of
two Householder reflections. Four raw coordinates are retained in both arms.
Identity-centered scales 0.1 and 0.1/sqrt(2) match local rotation magnitude; this
does not make the families globally equivalent. Both have a radial gauge, and
their composition families differ. Quaternion sign is retained. Finite floating
point gives approximate norm preservation here, not exact runtime arithmetic.

## Frozen decisions before learning

- Vocabulary4096, width256, read width64, context256; one exploratory paired seed
  240924. Width128 is a disclosed resource fallback, not a hidden comparison.
- AdamW learning rate0.001, decay0.01, global gradient norm clip1; all named
  variables must receive a finite gradient. Checkpoints retain both moments,
  per-variable update counts, configuration and the stateless data cursor.
- Use both retained #1014/#1017 training stores. Sample counter-seeded contiguous
  windows uniformly over valid starts; never cross their file boundary. Train on
  shifted language targets only. No new teacher or auxiliary selector labels.
- First profile at batch16/context256 for12 updates, reporting full forward,
  backward and optimizer time after two warmup steps. Freeze the full batch/dose
  and resource projection after this measurement and before full training.
  The planning anchor is30million target visits per arm. A resource interruption
  preserves resumable state and is not relabeled a completed learning campaign.
- Select among planned checkpoints using the exposed development prefix only.
  Report the complete976block development population with the rung0 tokenizer,
  reset and shift convention. These data are open development, not a final holdout.
- Working-language evidence requires improved retained training likelihood,
  loaded generation without collapse and comparison-tail NLL below the selected
  count/cache baseline2.391786178860742. This is an engineering continuation gate,
  not alpha or general reasoning qualification.
- Run Enabled and whole-prefix NoRead from fresh state. A comparison-tail loss
  increase of at least0.02nats when disabling reads is a prespecified indication
  of useful read-path contribution; output changes alone are insufficient.
  NoRead removes both recurrent value feedback and pointer-copy probability, so
  this intervention measures their combined contribution. It does not isolate
  geometric addressing, value feedback, or distant retrieval individually.
- Preserve five historical prompt continuations with seeds2014..2018 and the
  explicit probability-based Q32 sampler. Record all16 literal source-edit story
  pairs in each fit attempt before model loading. These are exploratory task
  transfer; exact noun-plus-period completion and first noun correctness are
  separate. They do not establish general memory or diagnose every read failure.
- One paired seed cannot promote a geometric advantage. At least three matched
  seeds and a fresh final population follow a successful selected design.

## Execution and independent review

The [budget receipt](../evidence/joint-recurrent-budget-2026-09-24.json) charges
engineering, builds, profile, training, evaluation, retries and delivery together.
One Cargo process and one Metal training process own the laptop at a time.
RDC launched two concurrent read-only DeepSeek sessions on the connected M1 Mac;
this provided independent mathematics and systems reviews, not extra hardware.

The systems review inspected pinned Candle0.9.2 source. Its material findings
were full-unroll graph/stack cost, differentiable softmax requirements, Metal
index-add layout, and measuring complete updates rather than forward throughput.
These informed the implementation. The mathematics review confirmed the local
scale relationship and normalized pointer mixture. Its suggestion that the
transport families have identical global capacity or a strict set inclusion was
not adopted: those assertions do not follow from a local norm calculation.

Relevant mechanisms are supported by the original
[Householder recurrent-network paper](https://proceedings.mlr.press/v70/mhammedi17a.html)
and [Pointer Sentinel Mixture Models](https://arxiv.org/abs/1609.07843).
Neither source establishes this combined architecture's language capability.

Only focused causal, arithmetic, gradient and checkpoint checks precede the
actual learning run. No expansion into unrelated test repairs is planned.
Integer discretization, prime/zeta address admission and final serving costs
remain subsequent rungs once joint language learning is demonstrated.

## Initial64-window full schedule: superseded after owner review

The corrected pinned CPU backend completes the B64/T64 profile at approximately
3,650target visits/second, with a3.00GB measured peak process footprint. The
full campaign uses **two concurrent CPU arms**, two threads per process, with an
8GiB combined-RSS stop and the unchanged physical storage reserve plus128MiB
margin. A supervisor requests a checkpoint between updates if a ceiling is
reached. The five-hour cycle projection and prospective local time/storage
extensions are in the budget receipt; the training-loop deadline is11500seconds
per process with180seconds separately reserved for closeout.

Each arm has7324updates ×64windows ×64targets = **29,999,104target visits**.
The quaternion arm resumes the corrected backend's12-update profile with its
optimizer and data cursor; those visits count within this total. The ordinary
arm starts from the matched initialization at step0. Earlier implementation and
backend profiles remain separate attempts, with their time and exposure charged.

These are **independent64-token windows**: state and event tape reset per window.
Only read ages1–63 receive language gradients. Ages64–255 remain initialized and
subject to weight decay. The model's256-step development and generation remain
unchanged; later positions therefore measure extension beyond the training
horizon. Passing the existing gates supports this artifact's observed behavior;
failure does not isolate a geometric cause. A128/256-window continuation remains
the next same-graph option before claiming learned256-step credit.
Report descriptive likelihood slices for positions0–63 and64–255 from the same
per-target records to distinguish the fitted horizon from its extension. These
slices do not replace or weaken the full comparison-tail gates.

Save the midpoint at3662updates and the final artifact. Select the checkpoint
with the lower full64-block tune-prefix NLL, with an exact tie choosing the
earlier step; do this separately but identically for each arm. Selection uses
the recorded `quick_loss` score (F32 batch reductions, aggregated as F64), before
opening comparison-tail results. Preserve both scores and selected identities in
the closeout receipt. Later per-token F64 scoring does not reselect a checkpoint.
If the arms select different steps, report the exposure difference as a confound
for their comparison. Then evaluate
both selected checkpoints with reads enabled and whole-prefix NoRead on all976
blocks and all frozen generation/source-edit probes. Preserve and report both
candidate scores. No criterion or comparator is weakened for the shorter fit.

Apple BLAS is explicitly enabled only in the offline tool. A pinned, licensed
Candle0.9.2 source copy carries four corrected operand-slice lengths, verified
against all103 upstream file hashes. See
[the patch explanation](../../third_party/candle-core-0.9.2/UOR-PATCH.md).

## Corrected continuation: full256-token language credit

At00:39UTC September25, the owner correctly challenged spending the remaining
campaign on64-token fits while evaluating256-token histories. That throughput
choice leaves most read ages and longer state trajectories without language
training. Disclosing the mismatch was insufficient reason to finish that dose.
The lead requested safe checkpoints before further full fitting.

The original quaternion run stopped at2048updates and the ordinary run at2104.
Both checkpoints saved, sealed and reloaded with zero retained-loss difference.
An executed56-update continuation of quaternion, using the original binary and
64-window sampler, aligned both arms at **2104updates /8,617,984target visits**.
This retained learning is warmup, not256-token evidence. All stopped attempts,
models, optimizer moments and negative/unfinished claims remain preserved.

Both arms now continue with **batch16, training context256, model/evaluation
context256**. Targets per update remain4096. An explicit
`training_window_transition` binds the actual parent checkpoint/campaign hashes,
old/new dimensions, global optimizer/data step and prior exposure. Model weights,
AdamW moments/configuration, tokenizer/data identities and global step survive.
Changing the window dimensions changes sampled starts and lane count; this is a
declared new sampler phase, not an ordinary identical-window resume. Undeclared
dimension changes remain rejected. The new retained-batch loss is a different
sample/horizon and must not be directly subtracted from the old retained loss.

Profile12 complete256-token updates per arm before freezing the full resource
projection. Retain those updates inside the planned total7324steps. The remaining
phase comprises5220updates /21,381,120target visits at256, giving29,999,104total
visits per arm including warmup. Planned selection candidates are global step
4714 (midpoint of the256 phase) and7324. Warmup/profile checkpoints are not
selection candidates. Use the same recorded F32-origin tune64 score and tie rule,
then the unchanged full-population likelihood, combined NoRead, actual generation
and exploratory source-edit checks. Both0–63 and64–255 slices are now inside the
trained horizon; report them descriptively without changing the overall gate.

The [budget](../evidence/joint-recurrent-budget-2026-09-24.json) records preparation
and storage prospectively. Only two obsolete, reproducible Cargo library/metadata
objects were additionally removed, reclaiming187,551,744physical bytes. No learned
model, executed binary, report, research, source or worktree was removed. Full
continuation launch requires the measured256-step throughput/RAM projection.

## Selected hardware and full256 continuation (September25UTC)

The owner chose the full planned exposure after the corrected horizon profile,
and asked for CPU/GPU parallelism. The [hardware receipt](../evidence/joint-recurrent-hardware-2026-09-25.json)
retains every measured branch. Two synchronous CPU sequence workers per arm,
with both arms running concurrently, were fastest: approximately1,419 and1,575
targets/second. Four workers per arm were about6percent slower. The tested Metal
quaternion arm achieved823targets/second. Increasing only backend thread settings
did not help. Short profiles are a launch projection, not sustained performance
or energy evidence; whole-attempt times also favored the selected configuration.

Each worker receives complete256-token sequences. Fixed-order weighted mean
gradients precede one global clip and AdamW update; batch16 and4096targets per
update stay fixed. Both arms use two shards, nested backend threads1 and the same
executable. F32 reduction order can change; bitwise trajectory equivalence is
not claimed. The named-gradient/loss equivalence test and checkpoint shard
bindings protect the intended objective and the matched comparison.

Continue the selected profile checkpoints at global2128 to7324. Remaining work
is5196updates /21,282,816target visits per arm. The final lineage contains
8,617,984warmup visits at64 and21,381,120visits at256. Only4714/7324 are selection
candidates, using their recorded tune64 scores before comparison-tail evaluation.
The slowest selected profile projects15,000seconds of remaining training;
17,400seconds permits measured variability, with180seconds closeout and30minutes
evaluation/delivery reserved within the existing complete-cycle allowance.
The observed combined RSS peak is6,180,077,568bytes against the8GiB stop.
Source/executable/parent hashes, alternative attempts and storage stops remain
bound in the receipt. No architecture, optimizer, data or evaluation gate changes
accompany this execution choice.
