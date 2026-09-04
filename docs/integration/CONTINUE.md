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

[#973](https://github.com/UOR-Foundation/uor-r4/issues/973) now has two
artifact-only causal paths over the same accepted ordinary weights:

- `R4PositionPreservingCausalKVBindingV1` is the full 120-position comparator.
- `R4FixedRecurrentCausalKVBindingV1` keeps eight exact live K/V records and
  four chronological binary-age H4 summary banks. It reads before writing and
  keeps its f32 K/V ledger fixed at 2,304 values / 9,216 bytes, 90% below the
  comparator's 92,160 f32 bytes.

The focused causal check is exact through the decision that performs the first
post-read eviction. In the frozen full-prompt no-fit, seed-9738, top-k-40, 16-token
comparison:

| Prompt | Full-cache continuation | Recurrent continuation | Common generated prefix |
|---|---|---|---:|
| `A purple turtle found a clock in the garden` | `, there was a time, there was a little girl named It found a big` | `, there was a time, there was a little girl named but so she saw` | 12 tokens |
| `Albert Einstein was born in` | ` his friend, a time, there was a little girl named he put it with` | ` his friend, a time, there was a little girl named and and a time` | 12 tokens |

The recurrent runs made 17/20 evictions, 66/80 summary-slot reads, and 15/18
summary merges respectively. Both stayed within 13 attention sources, made zero
teacher/provider/future/forbidden reads, and performed no fit. These are
measured mechanism results. They do not establish language quality,
long-context retention, geometric advantage, architectural alpha, or
table-native execution. The trained RoPE ceiling remains 120.

## Next implementation action

Implement the first versioned **sparse geometric attention** successor under
#973. Keep the full-cache path and fixed recurrent path as comparators. Map the
current query/state to a bounded set of geometric candidates, read and aggregate
only those candidates, and prevent any complete-prefix attention scan. Compare
the same two prompt trajectories before fitting; add a small open-data retention
probe only if those two executions leave a specific unresolved design choice.

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
Implement the first versioned sparse geometric-attention successor under #973.
Keep the accepted full-cache and fixed-recurrent paths as comparators. Select and
read a bounded geometric candidate set without retaining or scanning the full
prefix. Run the smallest direct no-fit comparison that resolves the mechanism;
use bounded open development data only if a specific design choice remains.

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
