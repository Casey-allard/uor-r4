# R4 learned candidate-leaf associative-readout prompt-capacity contract and result (#973)

- **Issue:** [#973](https://github.com/UOR-Foundation/uor-r4/issues/973)
- **Authoritative public freeze:**
  [issue comment 5498390609](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5498390609)
- **Authoritative public outcome:**
  [issue comment 5499617619](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5499617619)
- **Live base:** `92aadcf01623792a03b9d7bb03c4c0df96afbda4`
- **Branch:** `issue-973-learned-associative-readout`
- **Mechanism policy:** `R4LearnedCandidateLeafAssociativeReadoutV1`
- **Campaign policy:** `R4LearnedAssociativeReadoutPromptCapacityV1`
- **Status:** `COMPLETED` / `LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY` /
  `INDEPENDENT_VERIFICATION_PASS`
- **Qualified predecessor:** `R4RetainedLanguagePathV1`, artifact CID
  `blake3:d1417b325e7a545057cd38e9f1a723933a3682801877433d20e98774a5e9172d`
- **Outcome:** result
  `blake3:cedba37738ee249457bb589f716ee75afb16a0c4937c2a22ae9f917dd3eb97c1`;
  independent verification
  `blake3:443d711ce9a228e26e2eb2eebb55c582848424e2677c3473d41deaf8afd69ec7`

> **Record structure:** the architecture and decision contract through
> **Nonclaims** is the preserved pre-run freeze. The
> [verified outcome amendment](#verified-outcome-amendment--2026-09-01) is the
> current empirical status and binding next decision.

## Decision and boundary (historical pre-run freeze)

The parameter-free retained-readout ladder had ended at
`LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL`. This record
froze the one authorized learned successor before implementation, V4
population creation, optimization, or outcome access. It added candidate-relative
learned query capacity over the qualified retained value field while preserving
the accepted V1 recurrence and language path.

This pre-run portion is the public architecture, data-boundary, and decision
freeze. It intentionally contains no V4 population contents, trained candidate
artifact, reveal, score, result, or verification claim. The outcome amendment
below records those later artifacts without rewriting this frozen contract.

## Exact frozen mechanism

**Definition.** For layer `l`, token step `t`, and exact-H4 relative address
`g`, `V[l,t,g]` is V1's strict-prior, post-transport, post-decay value field
immediately before the current write. `lambda(c)` is candidate token `c`'s
existing canonical exact-H4 leaf, `h[t]` is the unchanged final V1 hidden
state, `E` is the unchanged tied embedding/head, `N` is the existing final
RMSNorm, and `R` is a parameter-free RMS normalization that maps an empty zero
field exactly to zero.

**Definition.** The geometric arm learns one independent candidate-query table
`qG[l,c] in R^48 = (R^4)^12` and scores

```text
zG[t,c] = <E[c], N(h[t])>
         + (1 / (2*sqrt(48))) * sum_l <qG[l,c], R(V[l,t,lambda(c)])>
```

V1's separate learned key/value projections, exact-H4 recentering, four decay
timescales, delta writes, occupied-slot softmax, read-before-write order,
residual/MLP path, tied base head, and persistent state remain unchanged. The
new readout adds no direct key-energy term and is not defined as
QK-softmax-value attention.

**Definition.** The matched address-blind arm learns its own unshared query
table `qP[l,c]`, with byte-identical shape and initialization, and uses the same
score form. It replaces candidate-leaf selection with the occupied-address mean
of V1's same strict-prior transported value field before `R`. This is the
equal-parameter learned non-geometric-readout control over the same geometric
backbone.

Both query tables have shape `[2,4096,12,4]`, initialize to exact zero, and are
optimized independently. Each effective arm therefore reproduces qualified V1
logits before learning while retaining live first-step gradients. Each arm adds
exactly `393,216` learned f32 values: `645,376` total parameters per effective
arm, of which only the `393,216` new values are trainable. The predecessor's
`252,160` parameters are immutable. Recurrent state remains exactly `23,040`
f32 values / `92,160` bytes plus `240` validity bits.

A shared-feature implementation may train both heads in one process, but their
parameters, losses, optimizer slots, checkpoints, and artifacts remain
disjoint. Exact-leaf gather and occupied pooling both execute in every training
and evaluation call so the compared head work is fixed. A head-off intervention
must be byte-identical to qualified V1. Full state-off must zero both V1's
retained contribution and the added associative score.

## Geometry-destroying intervention

**Definition.** The fixed-leaf control cyclically deranges the sorted set of
actually used token leaves and applies the same derangement to every candidate
read. It does not map candidates into structurally unused slots. It reuses the
trained geometric query table and otherwise executes identical scoring work.
This isolates candidate-to-exact-H4-address binding from the mere presence of a
learned vocabulary table.

A row of twelve R4 vectors plus exact-H4 candidate-address selection is a
geometric representation under the repository's qualified convention. This
freeze does not define intrinsic Spin(4) transport, a learned orthogonal 4x4
action, or H4 superiority.

## Frozen fit and reachability

Both learned heads receive the exact predecessor training slice and BLAKE3
order: `43,680` nonoverlapping 121-token windows, `5,241,600` causal decisions,
batch `16`, seed `9738`, and exactly `2,730` AdamW steps. AdamW remains betas
`0.9/0.95`, epsilon `1e-8`, weight decay `0.1`, gradient clip `1.0`, a 100-step
linear warmup to `3e-4`, then cosine decay to `3e-5`. The two independent
cross-entropies use the same frozen-base features.

There is one trajectory. No sweep, alternate seed, table/rank change,
continuation, or scientific retry is authorized.

The new seam is structurally reachable on `5,197,920/5,241,600` training
decisions and `245,854/247,920` fresh-language decisions: 119 of 120 positions
in every window. All `8,192/8,192` prompt target tokens occur after a complete
48-token prompt and are reachable. These counts establish reachability only,
not predictive value or attention quality.

## Independently frozen data boundary

`R4RetainedPromptSwapContrastV4` preserves V3's selection law of `256` pairs,
`512` directions, and `8,192` target tokens. Selection must begin strictly after
revealed V3 source ordinal `324,230` and exclude the exact 1,536-story V1+V2+V3
union. That exclusion witness is
`blake3:e8d02abcf9ab326545afa80c5191285ec37110cf73f0d389cd6a2f75fcd5c121`;
the V3 population CID is
`blake3:165be397b73041afd39aa65ae796400ea539399f8586729ad19a168c4daa9e93`.

The fresh-language slice is fixed by source coordinates before materialization:

- token range `[156,032,138, 156,282,124)`;
- `249,986` tokens, `2,066` windows, and `247,920` decisions;
- slice CID
  `blake3:77dfa0744c140e5affe9be233244e616c940dbff469f786deadeb87768e3c752`;
- capacity stories `764,050..765,247`;
- source stories `848,493..849,802`;
- 1,198-story witness
  `blake3:c112790145657c771cf72d63d8e1f055b3b2d772f1cd3485c7cacb74dbb1e4a0`;
  and
- canonical index CID
  `blake3:0032889e32b38801476223c5bed7e401d77b61afbbd6cf9afddaceee18e2136e`.

V4 prompts and fresh heldout must be created and sealed together in one
create-once mode-`000` evaluation directory. A mechanics/performance probe may
read training batches only. Qualified V1 and both final learned-head artifact
CIDs must be fixed before one immutable reveal marker opens either evaluation
population. After reveal, optimization is permanently closed. The only allowed
recovery is result finalization from an already-completed identical checkpoint;
it may not construct an optimizer, fit, or change an artifact.

At this freeze, the V4 population, population commitment, preparation, probe,
run contract, learned artifacts, reveal, result, and independent verification
are all `NOT_CREATED` or `NOT_RUN` as applicable.

## Binding empirical criteria

**Empirical Criterion.** Each learned arm earns `ASSOCIATIVE_CAPACITY_PASS` only
if every condition below passes on the one V4 reveal:

- prompt gain `G >= ln(2)/16 = 0.04332169878499658`;
- arm-minus-V1 prompt gain
  `>= ln(1.5)/16 = 0.025341569256760274`;
- at least `308/512` own-prompt directional wins;
- own-prompt NLL no worse than V1;
- fresh-language NLL no more than `0.05` behind V1 and top-1 no more than
  `1.0` percentage point behind V1;
- full state-off costs at least `0.10` NLL and `2,480` correct fresh-language
  decisions, while its prompt contrast collapses within `1e-7`; and
- head-off/V1 identity, strict causality, stationary/direct parity, finite
  active gradients, artifact replay, fixed work, and zero forbidden reads all
  pass.

**Empirical Criterion.** `GEOMETRY_ATTRIBUTED` is a separate, stronger verdict.
It additionally requires geometric-minus-pooled prompt gain and
geometric-minus-fixed-leaf-deranged gain each to be at least
`ln(1.5)/16 = 0.025341569256760274`, at least `308/512` paired directional
improvements against each, and geometric own-prompt NLL no worse than either
control. A generic learned-head gain without both separations is not geometric
advantage.

Both criteria are **Unproven** and `NOT_RUN`.

## Predeclared outcome actions

- If the geometric arm passes capacity and geometry attribution, preserve it
  and authorize one separately frozen, disjoint autonomous
  subject/scene-retention smoke.
- If one or both learned arms pass capacity without geometric separation,
  retain the learned-readout result, report geometry attribution unestablished,
  and authorize one smoke only from the passing arm with the lowest frozen
  fresh-language NLL.
- If prompt capacity passes but fresh-language nonregression fails, stop before
  generation and freeze a joint language/capacity objective next.
- If neither head passes capacity, reject this exact learned query-to-value
  readout without table, rank, scalar tuning, or retry, then revisit the
  retained value representation/binding law.
- If mechanics are invalid, repair mechanics only and make no model claim.
- If compute is unavailable, preserve or resume only the identical pre-reveal
  trajectory, with no architecture or threshold change.

## Compute contract

A training-only probe will compare Apple Accelerate CPU with four threads, CPU
with eight threads, eligible two-worker CPU execution, and sequential MPS using
one warmup plus five measured steps. Eligibility requires numerical agreement
with CPU4 within `5e-5`, memory below `80%`, and a 1.25x wall projection. The
measured-fastest eligible plan is binding. Final scoring and fresh-process
verification remain canonical Apple CPU. CUDA and external GPU execution are
forbidden.

Runs projected beyond 15 minutes must emit durable progress, ETA, per-head
checkpoints, and exact resume instructions. The whole-process hard ceiling is
two hours.

## Nonclaims

A future positive result could establish only bounded learned associative
prompt capacity over qualified V1. Geometry attribution would require the
separate frozen control criteria above. No result from this rung, by itself,
establishes coherent generation, reasoning, correctness, intrinsic Spin/H4
superiority, exact/table/`no_std` lowering, browser readiness, release
readiness, #973 closure, or #954 unblocking.

A future negative would falsify only this exact frozen query-table/readout law.
At the present freeze, every empirical outcome remains `NOT_RUN`.

## Verified outcome amendment — 2026-09-01

This append-only amendment records the one completed run without rewriting the
pre-implementation freeze above. The authoritative outcome is
[issue comment 5499617619](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5499617619).
The terminal is:

`LEARNED_ASSOCIATIVE_READOUT_NO_CAPACITY`

**Empirical Criterion.** The completed campaign rejects this exact learned
query-to-frozen-value-field readout law. It does not reject the qualified
retained decoder, the ordinary causal-softmax reference, or the broader
geometric-attention programme.

### Immutable lifecycle ledger

The create-once population seal, execution decision, fitted heads, reveal,
score, result, and independent verification are bound by these identities:

| Record | CID |
|---|---|
| preparation | `blake3:25d0a2bdb9caf93b97c1244c19e5646bee40d724ddd77431ac97de99cb82f7c3` |
| joint population commitment | `blake3:bc490bc0c4354ae08b00978dc6657200afb1638409f191c714693f0886981f58` |
| V4 prompt population | `blake3:cc9a1c40fe753e269ea31edd804c32b2a0c208ef20fceb1167636d6f28d7da11` |
| fresh-heldout continuation | `blake3:77dfa0744c140e5affe9be233244e616c940dbff469f786deadeb87768e3c752` |
| execution probe | `blake3:deb8e179f41d4f56dd7ec92148ff930837b6456bf76aed5e91297d0a2b060f02` |
| selected CPU4 plan | `blake3:6b7b3cdc9ea178c6fb63f4b7acd095f6c4a73e7e149cf35854f8610a49699a06` |
| started envelope | `blake3:379dc4d888098da084c4bc330058c68f784452d44223c13ca23578eeadbbdc3f` |
| run contract | `blake3:cb0c191bb3f6fba22a777cb16741f49699961bb4ef4e708a52311f9d358a3f47` |
| geometric arm result | `blake3:3983416d7936c3fc02bab19f711cfab69adaf3607077df7f3407515a8057eb60` |
| geometric head artifact | `blake3:85a33965a7cd9ee952948ed6e6c5a925585edb9496377baa56a22ffaca40175f` |
| pooled arm result | `blake3:ca6f713a22a67b6a5749c9ebef374abeeb9d22a232d0ab4f77043ae09c69a08f` |
| pooled head artifact | `blake3:4eeba8bb99d200e77558d89529a1e9f33d7c1ea6f4439ec3cae64c79d0b0f0d1` |
| reveal | `blake3:0fcbeffa06ed2ef7496a5ead77ff9a81320c44a4e4aec2d29082f86c0b8634a9` |
| canonical scoring evidence | `blake3:3912a87c4da17c4a50aeb096b21168b991bc5f9908a268d090a6a4d0977c6153` |
| terminal result | `blake3:cedba37738ee249457bb589f716ee75afb16a0c4937c2a22ae9f917dd3eb97c1` |
| independent verification | `blake3:443d711ce9a228e26e2eb2eebb55c582848424e2677c3473d41deaf8afd69ec7` |

The joint seal contained `256` V4 pairs, `512` directions, and `8,192`
continuation decisions plus the separately fixed `249,986`-token / `2,066`-
window / `247,920`-decision fresh continuation. It excluded all `1,536` prior
V1+V2+V3 story CIDs and had zero V4/fresh story overlap.

The training-only probe selected `cpu-accelerate-4t-sequential`: Apple
Accelerate, four CPU threads, one shared feature pass, and two disjoint heads,
losses, optimizer states, checkpoints, and final artifacts. The run performed
the frozen `2,730` steps once. CUDA, a sweep, threshold change, outcome-driven
retry, and post-reveal optimization were not used.

### Prompt-capacity and attribution outcome

| Arm | Mean own-minus-crossed gain (nats/token) | Wins / 512 | Own NLL | Decision |
|---|---:|---:|---:|---|
| frozen V1 | `0.00642365` | `308` | `3.71279883` | comparator |
| geometric exact leaf | `0.00637679` | `299` | `3.71038302` | `PROMPT_CONDITIONING_CAPACITY_FAIL` |
| pooled equal-budget | `0.01026323` | `324` | `3.68289051` | `PROMPT_CONDITIONING_PARTIAL` |
| fixed-leaf deranged | `0.00666565` | `306` | `3.71127426` | attribution control |

The frozen capacity floors were absolute gain `>= 0.04332169878499658`, gain
over V1 `>= 0.025341569256760274`, at least `308/512` wins, and own NLL no
worse than V1. The geometric arm passed only own-NLL nonregression. The pooled
arm passed wins, own-NLL nonregression, and any-gain-over-V1, but missed both
capacity-effect floors. Therefore neither arm earned
`ASSOCIATIVE_CAPACITY_PASS`.

The separate terminal is `GEOMETRY_ATTRIBUTION_FAIL`:

- geometric-minus-pooled gain was `-0.00388645`, with `209/512` paired
  directional improvements;
- geometric-minus-deranged gain was `-0.00028887`, with `251/512` paired
  directional improvements; and
- each comparison required gain `>= 0.025341569256760274`, at least `308/512`
  improvements, and geometric own NLL no worse than the control.

### Fresh-language and state-load-bearing outcome

Both learned arms passed every frozen fresh-language and state-load-bearing
gate on `247,920` decisions:

| Arm | NLL | Delta from V1 | Top-1 | Delta from V1 |
|---|---:|---:|---:|---:|
| frozen V1 | `3.90363602` | — | `29.6285%` | — |
| geometric exact leaf | `3.90141233` | `-0.00222368` | `29.6342%` | `+0.00565 pp` |
| pooled equal-budget | `3.87375622` | `-0.02987979` | `30.0428%` | `+0.41425 pp` |

State-off NLL was `4.23919176`. Relative to state-off, the geometric arm had a
`0.33777943`-nat advantage and `16,795` additional correct decisions; the
pooled arm had a `0.36543554`-nat advantage and `17,808` additional correct
decisions. These exceed the frozen `0.10`-nat and `2,480`-decision floors.
Thus the retained state is load-bearing, and the pooled readout preserves a
real ordinary next-token control signal. That result is not geometric
attribution or prompt-specific associative capacity.

### Mechanics, timing, and verification

All ten final mechanics gates passed: selected-probe mechanics, exact head
artifact replay, zero arm/prompt/fresh forbidden reads, exact frozen-base
binding, exact prompt replay, reveal/artifact binding, population binding, and
zero post-reveal optimizer steps. Artifact replay maximum logit delta was zero.

The durable timing ledger was `1,105.58889358` seconds for fitting plus
`281.65394608` seconds for canonical scoring, or `1,387.24283967` seconds
total (`23.12` minutes), below the `7,200`-second hard wall. Independent fresh-
process verification reproduced evidence, mechanics, and the terminal decision
exactly; writer PID `22421` differed from verifier PID `23048`, and the verifier
created no optimizer, took zero optimizer steps, and scored zero training
batches.

### Binding action and nonclaims

The predeclared negative branch is binding. Do not tune or retry this query
table, add another candidate readout over the same frozen V1 value field, or run
generation from either head. Preserve the pooled fresh-language result as the
matched non-geometric control. The next #973 architecture must change the
retained **value write/binding law** so prompt-specific key-value information is
present before readout, then compare it with this pooled result and explicit
geometry-destroying controls under a new, separately frozen contract.

Generation, reasoning, exact/table/`no_std` lowering, geometry-native lowering,
coherent text generation, correctness, intrinsic Spin/H4 superiority, a
transformerless general model, browser readiness, release readiness, #973
closure, and #954 unblocking remain `NOT_RUN`, `NOT_ESTABLISHED`, or otherwise
unauthorized at this rung.
