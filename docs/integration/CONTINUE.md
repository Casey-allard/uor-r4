# Continue one UOR-R4 task

Use the prompt below for one later repository task. Refresh live GitHub and
`origin/main` before deciding eligibility.

The project is in `build_first_architectural_alpha` mode. Its exact stage
order is in [project-track.md](project-track.md). The old artifact-only
pre-alpha target was a mechanical checkpoint and is already complete.

The ordered track is fixed recurrent memory → sparse geometric attention →
nonlinear geometric block → scale/data/instruction → retrieval/tools → product
alpha → Rust/table lowering → release proof/evidence/QA.

## Current checkpoint

[#973](https://github.com/UOR-Foundation/uor-r4/issues/973) now has four
artifact-only causal paths over the same accepted ordinary weights:

- `R4PositionPreservingCausalKVBindingV1` is the full 120-position comparator.
- `R4FixedRecurrentCausalKVBindingV1` keeps eight exact live K/V records and
  four chronological binary-age H4 summary banks. It reads before writing and
  keeps its f32 K/V ledger fixed at 2,304 values / 9,216 bytes, 90% below the
  comparator's 92,160 f32 bytes.
- `R4SparseGeometricCandidateSoftmaxKVBindingV1` ranks only those twelve
  persistent metadata slots by exact signed-S3 shell and full-H4-root maximin
  diversity, admits at most eight plus current, and gathers K/V only afterward.
- `R4H4FrameQuaternionCubeResidualV1` keeps that sparse reader and replaces
  each dense SwiGLU residual with twelve ordered R4 blocks evaluated through a
  finite bank addressed by 120 current-H4-frame indices. Antipodal frame pairs
  select the same odd cube map, so there are at most 60 distinct operators.

The focused causal check is exact through the decision that performs the first
post-read eviction. In the frozen full-prompt no-fit, seed-9738, top-k-40, 16-token
comparison:

| Prompt | Full-cache continuation | Recurrent continuation | Common generated prefix |
|---|---|---|---:|
| `A purple turtle found a clock in the garden` | `, there was a time, there was a little girl named It found a big` | `, there was a time, there was a little girl named but so she saw` | 12 tokens |
| `Albert Einstein was born in` | ` his friend, a time, there was a little girl named he put it with` | ` his friend, a time, there was a little girl named and and a time` | 12 tokens |

The sparse runs shared 12 and 3 generated tokens with the fixed recurrent arm.
They selected a different set from age-only on 33/35 sparse decisions, admitted
55 summary records, stayed within nine sources, and reduced aggregate score
materialization from 3,824 to 3,240. Both reported zero complete-prefix scans,
omitted-payload, teacher, provider, future, or forbidden reads. These are
measured mechanism results. They do not establish useful retrieval, language
quality, long-context retention, geometric advantage, architectural alpha, or
table-native execution. The trained RoPE ceiling remains 120.

The quaternion-cube arm completed the same two prompts with 1,272 R4 block
evaluations, no dense-MLP call, no new parameter or state, a largest f32
block-norm error of `7.152557373046875e-07`, and the same nine-source/causal
ceilings. Its forced-prompt sparse selections exactly matched the dense arm.
Both generated trajectories diverged from the fitted dense-SwiGLU comparator
at token zero and produced visibly degraded text. This completes the
nonlinear mechanical checkpoint while leaving trainability and useful language
unestablished.

## Next implementation action

Specify and execute the first bounded **development-data fit** of the assembled
sparse-plus-quaternion-cube architecture under #973. Keep the fitted
`R4SparseGeometricCandidateSoftmaxKVBindingV1` dense-SwiGLU path as comparator,
keep the nonlinear law and sparse reader fixed, and use open development data
with an explicit resource ceiling. Decide whether this architecture can learn
useful prompt-dependent text before increasing width, corpus dose, or
instruction scope. Keep final held-out evaluation separate.

SpiralCore's finite labelled E8 action graph may inform an operator-indexed
candidate selector, but it is not already attention. Keep its H4/R4 and E8/R8
spaces typed separately. HELM can supply causal/cache comparator semantics;
W33, NEMESIS, UOR, and H4/zeta sources remain on-demand donors. Inspect original
source for the exact mechanism used and do not transfer external proof or
capability claims.

Do not start a larger scale campaign, retrieval/tools, workbench integration,
lowering, release proof, or broad QA in the same task.

```text
$uor-project-workflow

Continue UOR-Foundation/uor-r4 from refreshed origin/main and the live GitHub
issue graph. Read AGENTS.md, docs/integration/current-state.md, and
docs/integration/project-track.md. Complete exactly one active implementation
task and stop.

Apply build_first_architectural_alpha. Use one isolated full Git worktree.
Specify and execute one bounded development-data fit of the exact
`R4H4FrameQuaternionCubeResidualV1` architecture under #973. Keep its accepted
sparse geometric reader, continuous quaternion-cube law, and dense-SwiGLU
comparator fixed. Use open development data, a hard resource ceiling, and the
smallest fit that can decide whether the assembled bounded path learns useful
prompt-dependent text. Keep final held-out evaluation and any larger scale or
instruction campaign separate.

Query the knowledge index or inspect SpiralCore/HELM/W33/NEMESIS/UOR/H4-zeta
sources only for the concrete selector/read decision. Treat them as donor
material, preserve their evidence status, and inspect original source before
using a mechanism.

Preserve unique artifacts, user material, and prior negative results. A
negative binds its exact tuple; UNAVAILABLE is not model evidence. Keep proof,
measured behavior, and hypotheses distinct without manufacturing an evidence
dossier. Deliver through a protected pull request. Report the working behavior,
actual command result, remaining limitation, closure state, and one next action.
```
