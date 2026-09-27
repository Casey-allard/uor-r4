# Matched radial-reader adaptation result

September 27, 2026. References #973 under #820.

**Decision: retain the native Dot path and park these transferred Lorentz and
LorentzAffine configurations at the declared dose.** Neither radial arm improves
likelihood or complete source responses over the matched Dot reset. Lorentz has
one additional acceptable greedy story on this tiny panel, alongside substantial
source regressions and worse likelihood. That tradeoff does not support its
serving integration. No extra dose, initializer, seed or decoder sweep follows.
This is a scoped negative transfer result, not rejection of geometric readers.

The next fourth-lab implementation is the independently justified
[complete-prefix dialogue learner](dialogue-prefix-learning-plan-2026-09-27.md):
retained model initialization, paired response learning, saved offline reload
and actual replies. Claude's deeper geometric stack remains a complementary
research track. The owner's native, multiplier-free serving target is unchanged.

## Executed comparison and provenance

The [prospective plan](radial-adaptation-study-2026-09-27.md) fixes the sole
endpoint at 1,024 updates per arm. All three fits completed that endpoint, each
with 4,194,304 new target visits, from the same negative step15,672 parent's
21 shared arrays and fresh Adam/data clocks. Total new fitting exposure is
12,582,912 visits; the parent's 64,192,512 historical visits remain separate.
Training uses Quaternion transport, width256/read64, full256 access, batch16,
two CPU gradient shards, seed240927 and ordinary unweighted next-token loss.
No termination weighting, quantization or context shortening was introduced.

All fits, six full evaluations, greedy supplements and the native comparator
executed source `488e3976149a50dc65a9dba76735646f4ed6b8ef`, using frozen release
CPU-Accelerate executable SHA256
`83984834ec014f5ea8070f9fb60e3f9f57f83ca70e97d377607c4a7d6044c08f`.
Rayon was limited to two and BLAS/OMP/GEMM to one. All nine supervised processes
exited zero without soft/hard stops. Each fit reloaded with zero retained-batch
loss delta. The pipeline verified complete report/checkpoint seals and completed
at 07:43:02 UTC. It deliberately left quality adjudication to the reviewers.

The [source-bound result packet](../evidence/radial-adaptation-result-2026-09-27.json)
binds final weights, campaigns, receipts, complete compact outputs, independent
prose review and original files. The three final model SHA256 values are:

| Arm | Final model SHA256 |
|---|---|
| Dot | `0800fa27e912225322b448aa3cd474c1905c02c072e39362598224e2e62127ad` |
| Lorentz | `e98f048ad681f6d41b41580ffcf11eaca1fe4d979d15b02dd9fecf196030d3fd` |
| Affine | `c0a31e5102675b4d17aa659a86d15ab0903bbfbca322666b2903f1acc461d83c` |

## Likelihood and contextual use

Each saved checkpoint was evaluated in Read and NoRead modes on the same976
complete development blocks: 249,856 targets per pass, 1,499,136 across the six
new population passes. The first64blocks and following912blocks are both
previously exposed development data, not a new held-out test.

| Read NLL, nats/target; lower is better | First64 | Following912 | Full976 |
|---|---:|---:|---:|
| Retained parent | 2.095483384 | 1.996490474 | 2.002981812 |
| Dot reset | 2.097546066 | 1.992477038 | 1.999366810 |
| Lorentz | 2.144904835 | 2.038808595 | 2.045765725 |
| Affine | 2.111374305 | 2.005743077 | 2.012669715 |

Full-population candidate-minus-Dot deltas are **+0.046398915 Lorentz** and
**+0.013302905 Affine**. Lorentz is also +0.033096011 worse than Affine. All three
directions agree in both partitions. Independent recomputation from all seven
retained target files matches the 27 paired means/counts and976block means
within1e-12. These dependent-target counts do not establish statistical
generalization. Dot's small improvement over the parent is partition-dependent:
full -0.003615002, but first64 +0.002062683.

Removing feedback and copying raises full loss by +0.502718425 for Dot,
+0.455473906 for Lorentz and +0.482873759 for Affine. This confirms useful
contextual machinery in that combined intervention; it does not isolate
geometry or counterbalance the radial arms' worse enabled-read results.

## Complete source responses and actual prose

The unchanged16source-edit pairs use exact noun-plus-period answers and the
existing first-period/EOS/cycle/cap policy. Complete and first-noun judgments
remain separate. All128parent-plus-new Read responses stop at the caller's first
sentence boundary; none is model-EOS evidence.

| Arm | Complete answers | Correct first noun | Both sides complete |
|---|---:|---:|---:|
| Parent | 21/32 | 26/32 | 8/16 |
| Dot reset | 21/32 | 29/32 | 9/16 |
| Lorentz | 14/32 | 22/32 | 6/16 |
| Affine | 19/32 | 25/32 | 8/16 |

The independent source review inspected every new Read response. Relative to
Dot, Lorentz gains `shell.` but loses eight complete rows; Affine gains `shoe.`
and `shell.` but loses four. Lorentz gains no complete row over Affine and loses
five. Its eight Lily/garden-template answers all become `little table.` despite
changed source objects. Affine also collapses brush/comb to that answer, but its
`little doll.` / `little bear.` retain the changed object while failing the fixed
exact-answer oracle. Preserve that partial distinction instead of treating every
failed oracle row as lost source information.

Dot's unchanged total hides turnover: pear/doll/brush are repaired, while key,
shell and rope become `key to the kitchen.`, `shell safe.` and `rope to the
chair.`. It remains the better matched control here, not a broadly repaired
language model. All three NoRead panels have zero complete answers.

The principal and adversarial reviewer read all30new Read stories against the
four frozen binary criteria: identity, prompt-connected progression, literal
intelligibility and a complete ending at a stable/resolved point. Both reach the
same all-four counts; dimension-level differences and borderline readings remain
in their separate records.

| Arm | Sampled | Greedy | Passing greedy stories |
|---|---:|---:|---|
| Historical parent, retained verdicts | 0/5 | 2/5 | Lion; Tim |
| Dot reset | 0/5 | 1/5 | Parents |
| Lorentz | 0/5 | 2/5 | Tim; Lily/Dave |
| Affine | 0/5 | 1/5 | Parents |

Lorentz's Tim story explicitly introduces a helpful dog and resolves the ball
riding obstacle; fictional ball riding is not automatically a failure. Its
Lily/Dave story passes with awkward repeated hill ascent. Dot's train/race story
is a recorded progression borderline. These judgments do not rescue sampled
outputs with role collapse, malformed actions or unresolved endings. Conversely,
a cap alone does not fail a complete ending: Affine's greedy lion ends with the
complete clause “They became good friends and played,” but fails identity and
progression. No parent output was regenerated or re-adjudicated.

## Cost, preservation and next action

The three fit supervisors total **11,476.693 seconds**; included fit-report time
is11,473.471seconds and included update time11,279.771seconds. Six evaluation
supervisors total **150.550 seconds**, including128.587seconds population work,
0.928sampled prose,1.346source completion and0.386greedy prose. These components
overlap and must not be added again to the wall ledger. Maximum child RSS is
3,861,364,736bytes for fitting and174,997,504bytes for evaluation. Different host
load across the sequential fits prevents using their elapsed ratios as a
geometry speed or energy claim. The comparator has no separate child timing/RSS
receipt; its elapsed contribution remains inside the whole-cycle charge.

Ten retained fit/evaluation/comparison roots total **204,932,260 logical bytes**,
excluding binary, outer logs/receipts and build cache. This is not a physical
allocation measurement. Original roots, intermediate checkpoints and negative
models are preserved. The frozen build/check receipt separately records588.06
seconds, including its focused checks; those checks were not rerun for this
documentation-only result.

The analysis cursor at07:47:47UTC charges all overlapping own model, specialist
and principal elapsed once:26,083,413ms owned,674,238,993ms shared cumulative,
within the verified722,400,000ms allowance. The foreign reported744,000,000ms
limit remains unadopted. Physical free space then measured32,587,120,640bytes
internally and172,554,039,296bytes on the SSD. Delivery tail is recorded separately
after this cursor; the earlier overlap correction remains preserved.

No integer Affine implementation or Lorentz promotion is justified by this
screen. Keep accepted integer parents and existing interface work intact.
Implement the paired full-prefix/role-only dialogue path next, retaining complete
answers and real EOS in both arms and making the short-response population
explicit. Its cost, fixed dose and output decisions must be recorded before the
new model work. This follows a new observed learning mechanism; it does not
extend the failed reader configuration.
