# Completed full-context language continuation

September 26, 2026. References #973 under #820. **Exposure-only milestone
complete; useful-language target unmet; neither new candidate promoted.**

## Decision

Both arms retained the entire planned step8,348→15,672 continuation. All frozen
continuous, learned-code, integer, source and prose outputs completed at
23:04:43 UTC. Natural likelihood improved, but none of the four arm/path groups
meets the unchanged prose improvement rule. The ordinary arm also fails specific
numerical/source gates; the quaternion integer goose continuation short-cycles.
Preserve both new candidates as completed evidence and retain the September25
accepted learned-code/integer parents.

The [principal result receipt](../evidence/language-continuation-result-2026-09-26.json)
binds the actual artifacts, reviews and decision. The
[frozen plan](language-continuation-plan-2026-09-25.md) remains the acceptance
contract. This result closes one exposure-only milestone; it does not close the
wider language and programme issues, qualify useful conversation or coding, or
authorize another training tranche.

## What actually completed

| Fixed quantity | Executed scope |
|---|---|
| Continuous dose per arm | 7,324 retained updates; 29,999,104 target visits |
| Start / sole final candidate | 8,348 / 15,672 |
| Training / evaluation / session | 256 / 256 / 256 tokens |
| Context access | Every causal occurrence in the window; no recent64 admission |
| Batch / gradient shards | 16 / 2 whole-sequence shards |
| Read width / recurrent state width | 64 / 256 coordinates; separate from token context |
| Changed learning factor | Additional exposure with the existing objective, parameters, optimizer recipe, tokenizer and data population |
| Final discretization | Fresh deterministic scales; 512 alpha-only updates per arm |
| Output packet | Existing five story prompts, seeds2014–2018, cap128; all32 source variants and fixed numerical controls |

The surviving chains are quaternion
`8348→10480→12707→13939→14000→15672` and ordinary Householder pair
`8348→10488→12715→13949→15672`. Their curves contain each update8349–15672
exactly once. Final model, data, Adam and all21 parameter clocks agree at15672;
parent hashes, moments and sampler descriptors remain bound. The original F32
trainer does not emit a literal sampled-token hash, so checkpoint/schedule
continuity is established without inventing an independently checked token-hash
identity. Final full Rust report/checkpoint verification passed before the
frozen downstream execution. [Completion evidence](../evidence/language-continuation-postprocess-6-2026-09-26.json).

The first attached controller failed before a saved checkpoint and lost its
2,031/2,032 logged updates. That work stays charged as execution loss and is
excluded from retained dose. Later allocation/reserve stops preserved the exact
saved states. After the verified cache offload, fit6 completed quaternion at
18:45 UTC and ordinary at21:11 UTC. Models and downstream commands ran one at a
time; no already completed arm was fitted again. Dated interruption and recovery
records remain linked from the [plan](language-continuation-plan-2026-09-25.md).

## Frozen prose decision

Every story was scored on the same four binary criteria: resolvable entity/role
identity, intelligible event progression connected to the prompt, understandable
literal language, and a complete ending clause at a stable/resolved point. All
four must pass. Personification and explicit new characters are allowed; minor
grammar problems, an unfamiliar name, or a length-cap stop alone do not fail a
story. The rule remains **at least3/5 acceptable, at least2 above the same-path
parent, and no loss of a parent-acceptable story**, separately for each arm/path.

| Path | Quaternion parent → final | Ordinary parent → final | Frozen improvement rule |
|---|---:|---:|---|
| Continuous Read | 0/5 → 0/5 | 0/5 → 0/5 | Both fail |
| Standalone integer Read | 0/5 → 0/5 | 0/5 → 1/5 | Both fail |

Principal and independent review agree on every final fully acceptable/not
acceptable decision. The ordinary integer lion story is the one acceptable
continuation: its explorer proceeds through the scene and ends with “He quickly
looked around and saw a big tree by the river.” It does not reach the required
3/5 or two-story gain. NLL was not used to grade prose.

Dimension disagreements remain visible rather than being erased by agreement
on the conjunction. For quaternion integer seed2017, the independent reviewer
accepts event progression while the principal rejects its causal connection;
both reject literal-language quality. For ordinary integer seed2014, the
principal accepts the final arrival clause while the independent reviewer rejects
completion because of the unresolved quotation; both reject literal-language
quality. Continuous dimension disagreements are likewise retained.
[Continuous judgments](../evidence/language-continuation-continuous-quality-2026-09-26.json),
[integer judgments](../evidence/language-continuation-integer-quality-2026-09-26.json),
[frozen parents](../evidence/language-continuation-parent-quality-2026-09-25.json).

These are exposed development stories, not a statistical generalization claim.
Continuous sampling uses temperature0.8 Q32/SplitMix; standalone integer uses
temperature1 Q48/xorshift. Each final is compared with its own same-policy parent.
A difference between those sampled paths alone is not numerical-export error.

## Numerical and source result

Values below are comparison-tail NLL in nats per target. Each of the eight F32
Read/NoRead evaluations completed249,856 targets, of which233,472 belong to the
fixed comparison tail; their ordered input/target identities match. All recorded
target probabilities/NLLs are finite and the completed Rust evaluations meet
probability normalization tolerance.

| Comparison-tail measurement | Quaternion | Ordinary Householder pair |
|---|---:|---:|
| Step8,348 continuous Read parent | 2.090518499 | 2.064403221 |
| Step15,672 continuous Read | 1.996490474 | 1.974724189 |
| Continuous NoRead | 2.492064889 | 2.472758698 |
| New hard Read | 2.042205663 | 2.046887850 |
| New hard NoRead | 2.548328697 | 2.567085430 |
| Hard minus new continuous; allowance≤0.05 | **0.045715189 PASS** | **0.072163661 FAIL** |
| Hard NoRead penalty; required≥0.02 | **0.506123034 PASS** | **0.520197579 PASS** |

Both hard models remain below the fixed count/cache reference2.391786179. NoRead
removes both read value feedback and copying, so its penalty measures their
joint utility. It does not isolate attention geometry or either component.

| Additional declared measurement | Quaternion | Ordinary Householder pair |
|---|---:|---:|
| Original-rung1 correct first nouns lost; allowance≤2 | **0 PASS** | **3 FAIL** |
| Integer Read maximum state drift; allowance≤0.01 | **0.006347656 PASS** | **0.012695313 FAIL** |
| Integer NoRead maximum state drift | 0.005859375 | 0.007324219 |
| Integer Read maximum probability drift; allowance≤0.01 | 0.007270126 | 0.006939266 |
| Integer NoRead maximum probability drift | 0.005116490 | 0.005937014 |
| F32 hard fixed-five noncollapse | PASS | PASS |
| Standalone integer fixed-five noncollapse | **FAIL** | PASS |

Integer drift covers **four full256 calibration-prefix windows per mode**,
1,024 targets each; zero integer comparison-tail targets were requested.
The ordinary Read state failure is witnessed at inputoffset394, block1,
position138. All causal slot counts were checked through255 prior occurrences;
slot exposure does not imply positive influence at every position. Integer
probability drift remains below0.01 in both arms/modes.

The F32 noncollapse criterion checks nonconstant output and no final three-block
repeat at periods1–4 on the fixed five stories. Both hard F32 panels pass that
scoped criterion. In the separately sampled standalone quaternion goose story
(index2, seed2016), generation stops after18 tokens with a period1 short cycle.
Preserve this served-output failure without attributing it to the sampler or
conversion from trajectory differences alone.

The original gate is first-noun retention against the selected rung1 parent.
Complete-answer membership and comparison against the later accepted parent are
additional required reporting; they do not silently replace that gate:

| Complete source answers, out of32 | Quaternion | Ordinary Householder pair |
|---|---:|---:|
| Original rung1 parent | 28 | 23 |
| Accepted September25 integer parent | 28 | 24 |
| New continuous | 21 | 23 |
| New F32 hard / new standalone integer | 25 / 25 | 21 / 21 |
| New standalone NoRead | 0 | 0 |
| Complete correct rows lost / gained versus accepted parent | **5 / 2** | **7 / 4** |

Quaternion passes the original first-noun retention gate yet loses three net
complete answers against the accepted integer parent. Ordinary loses three
original-rung1-correct first nouns and gains one, so its net31→29 first-noun
count does not hide the three losses. New integer source rows and generated IDs
match their own new F32 hard parent, with no losses at that handoff. The older
accepted-parent regressions remain real. All historical negatives and admission
overrides are preserved as descriptive row comparisons, not new acceptance
baselines. [Numerical/source receipt](../evidence/language-continuation-numerical-source-2026-09-26.json),
[complete row comparisons](../evidence/language-continuation-source-comparisons-2026-09-26.json).

## Handoff integrity and exact evidence scope

The fresh calibrated-shadow adapter makes zero neural optimizer updates or data
advances and preserves the original optimizer bytes, legal fractional interiors
and nearest hard codes. It **does change outlying shadow coordinates** by the
explicit projection:23,538 quaternion and22,522 ordinary coordinates out
of1,678,466. This is not a claim that all model weights are unchanged.

Each arm derives its coefficient from the same8 training batches/32,768 targets,
without evaluation data or a scale/coefficient sweep. Alpha fitting then records
exactly512 updates/2,097,152 target visits over data steps15672–16183; all21 alpha
clocks equal512 and the neural parent remains at15672. Training sample hashes
match across arms for both coefficient initialization and alpha fitting.
Original weights, scales, parent Adam state and architecture are fixed during
alpha optimization.

Packed export/reload records zero full-context probability delta and equal saved
generated IDs/probability hashes in the48-token-cap loaded exercise. Those are
the executed comparisons, not corpus-wide saturation or cross-backend proofs.
The standalone bundle binds its parent, packed manifest, tokenizer, tables,
requests and source. Its exported evaluator differs in serialized bytes from
the canonical evaluator, but the parsed JSON is exactly equal; the receipt
retains both byte identities and the serialization explanation.

Continuous training reuses source`b5b5fe757b9513877911719f014e73165f704b13`;
the adapter/evaluation/integer executable uses
`209483169323563d541df781b8817a4bf6dc31e3`. The result receipt binds actual
executables and artifacts. The [direction memo](language-continuation-direction-2026-09-26.md)
keeps its original source-snapshot caveat and the principal's focused cross-check
of the actual retained trainer. No whole-tree identity is inferred.

## What the existing traces establish, and the next decision

The [completed trace review](../evidence/language-continuation-trace-coverage-2026-09-26.json)
provides a concrete priority. In quaternion continuous seed2014, decision39
selects “ called” with raw model mass0.0017458474 while the same-prefix greedy
token is “.”; the emitted clause becomes “a very big, grey man had an idea
called out in.” The record has NoRead mass0.97919846 and total effective copy
mass0.00206467. Ordinary seed2014 likewise retains selected and greedy
alternatives in “He wanted the most so exciting!” These are concrete selection
divergences at the actual sampled prefixes; no alternate rollout or component
attribution is established.

Those witnesses do not establish a coherent greedy alternate trajectory or a
single root cause. Selected probability is the raw model mass, not the
post-temperature/top-k draw probability. The quaternion total copy mass exceeds
the selected token's raw mass, so even this small total cannot rule out copying
that token. Existing greedy source-panel regressions also prevent a sampling-only
explanation of all failures. Continuous records already contain top-read
occurrence/token/mass, gates, selected probabilities and greedy tokens; only
full source-weight distributions, separate per-token vocabulary/copy components
and alternate state trajectories are missing for the relevant causal questions.

The [independent direction review](../evidence/language-continuation-direction-review-2026-09-26.json)
and principal therefore prioritize **one later same-checkpoint emission/selection
causal diagnostic**, using the witnessed failures and both arms. Its decision
must distinguish poorly ranked semantic predictions from stochastic deviations
and collect missing component evidence only where necessary. The
[current work card](../history/current-state-2026-09-25-to-2026-10-02.md#executed-same-checkpoint-emissionselection-diagnostic-september-27)
sets its scope; this result executes or authorizes no replay, new acceptance
panel, decoding-policy change or weight update. Existing-record localization is
already complete to this priority; the root cause remains UNRESOLVED.

Extra exposure, attention pruning or a blanket decoder fix is not justified by
this packet. Nor does it show that attention is absent, capacity has saturated,
or a new Hamiltonian is necessary. The current learned contextual read path and
quaternion state transport remain implemented mechanisms. General language,
geometry advantage, D5 parameter sparsity and complete-path energy remain open.
Repairing ordinary numerical retention alone cannot qualify the continuous
language behavior that also failed.

## Delivery and cost closeout

The [closeout receipt](../evidence/language-continuation-closeout-2026-09-26.json)
records the completed model result, verified backup and cost through
**September26 23:43:06 UTC**. The subsequent protected delivery is owned by
[live PR#1398](https://github.com/UOR-Foundation/uor-r4/pull/1398) and its linked
#973/#820 updates; a queued PR or compatibility acknowledgment is not a merge
or test result. Both wider issues remain open and neither candidate is promoted.

- **Final SSD snapshot: verified.**
  `/Volumes/UOR-Workspace/Backups/language-continuation-final-20260926-6`
  contains the completed milestone, source bundle at
  `ddc751e7d8b5dcc659b08a0dd13c78b5df1c5d89` and a timestamped ledger copy.
  All **67 copied Rust report/checkpoint roots passed** complete-set verification;
  SHA256, modes, types and membership also matched. Its **1,082,432,690 bytes**
  plus the first snapshot's885,274,668 bytes total **1,967,707,358 bytes**, within
  the original4GiB allowance including the metadata reserve. Manifest SHA256:
  `53881742c4ad8375d472c462483ea50d07e4199106676534e5b0b182158a2ede`.
  The two snapshots form the recovery set:240 immutable dependency files in the
  first snapshot were reverified. The new snapshot alone is not self-contained;
  later delivery receipts are not claimed to be in that earlier frozen copy.
  [Backup evidence](../evidence/language-continuation-final-backup-2026-09-26.json).
- **Internal storage:** the earlier verified Cargo-cache offload reclaimed
  18,462,396,416 internal bytes at cutover. Models, source, checkpoints and
  Docker.raw stayed internal. At final backup completion, physical internal free
  space was **51,746,230,272 bytes**. Preserve `cache-locks-6` and the SSD
  compatibility links; future builds require the mounted workspace.
- **Complete elapsed accounting:**97,326.083 seconds (**27.04 hours**) from
  cycle start through the timestamp above. This includes preparation, all
  failed/recovered work, paused intervals, storage work, evaluation and review;
  the remaining GitHub delivery tail is charged once through the final local
  cursor and reported in the owning issue. Historical cumulative charge at this
  receipt is **643,296,442ms**, below the milestone's effective657,600,000ms
  ceiling. The externally changed shared limit744,000,000ms remains preserved
  with attribution UNRESOLVED and is not adopted as extra authority.
- **Model and build time, separate from full wall:** all continuous supervised
  phases total **47,677.005s (13.24h)**, with paired workers counted once and
  lost fit1's8,949.286s included once. The downstream controller used6,464.883s
  (107.75min), including5,521.657s of alpha fitting; do not add those nested
  quantities together. Adapter build used258.758s. Summed saved continuous
  process time62,799.308s overlaps early paired work and is not elapsed wall.
  [Detailed scope and per-phase records](../evidence/language-continuation-cost-detail-2026-09-26.json).
- **Validation:** the adapter's2 focused release checks and both actual parent
  interface exercises were already executed. Final JSON parsing, changed local
  link targets, `git diff --check` and claim-wording checks pass. No blanket
  suite or repeat model run was added. GitHub's five historical status names
  are explicit queue compatibility acknowledgments; local receipts carry tests.

The tracked32GiB allocation retains the original baseline including the relocated
shared cache. The separate32GiB cache-offload and original4GiB model-backup
allowances are not internal free-space claims. No paid training, active model
relocation, further deletion, threshold change or automatic next dose occurred.
