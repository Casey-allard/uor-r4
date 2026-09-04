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

[#973](https://github.com/UOR-Foundation/uor-r4/issues/973) now has three
artifact-only causal paths over the same accepted ordinary weights:

- `R4PositionPreservingCausalKVBindingV1` is the full 120-position comparator.
- `R4FixedRecurrentCausalKVBindingV1` keeps eight exact live K/V records and
  four chronological binary-age H4 summary banks. It reads before writing and
  keeps its f32 K/V ledger fixed at 2,304 values / 9,216 bytes, 90% below the
  comparator's 92,160 f32 bytes.
- `R4SparseGeometricCandidateSoftmaxKVBindingV1` ranks only those twelve
  persistent metadata slots by exact signed-S3 shell and full-H4-root maximin
  diversity, admits at most eight plus current, and gathers K/V only afterward.

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

## Next implementation action

Implement the first versioned **nonlinear geometric block** successor under
#973. Keep `R4SparseGeometricCandidateSoftmaxKVBindingV1` as the attention path
and the current dense SwiGLU block as the comparator. Define one finite R4
operator block with an explicit state map, nonlinearity, residual/readout, and
bounded cost. Keep E8/R8 separate unless a typed bridge is implemented. Run one
direct prompt execution before any scale or instruction campaign.

SpiralCore's finite labelled E8 action graph may inform an operator-indexed
candidate selector, but it is not already attention. Keep its H4/R4 and E8/R8
spaces typed separately. HELM can supply causal/cache comparator semantics;
W33, NEMESIS, UOR, and H4/zeta sources remain on-demand donors. Inspect original
source for the exact mechanism used and do not transfer external proof or
capability claims.

Do not start the nonlinear block, scale campaign, workbench, lowering, release
proof, or broad QA in the same task.

```text
$uor-project-workflow

Continue UOR-Foundation/uor-r4 from refreshed origin/main and the live GitHub
issue graph. Read AGENTS.md, docs/integration/current-state.md, and
docs/integration/project-track.md. Complete exactly one active implementation
task and stop.

Apply build_first_architectural_alpha. Use one isolated full Git worktree.
Implement the first versioned nonlinear geometric block successor under #973.
Keep the accepted sparse geometric reader and dense SwiGLU block as
comparators. Specify and execute one finite R4 state map, nonlinearity,
residual/readout, and bounded-cost path. Keep H4/R4 and E8/R8 typed separately
unless an explicit bridge is implemented. Run the smallest direct no-fit prompt
comparison that resolves the mechanism.

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
