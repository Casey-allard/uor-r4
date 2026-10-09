# Status

Updated 9 October 2026. **Pre-alpha: no model holds a useful conversation yet.** Broad
reasoning, coding, frontier capability and lower complete-path energy are not established.

This page is navigation, not a results ledger. Results and artifact identities are in
[current state](docs/integration/current-state.md); the plan is the [ROADMAP](ROADMAP.md)
(milestones M1-M8) with long-form history in the [canonical plan](docs/integration/project-track.md).
Coordination and live status are on the tracker [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028).

## Two model lines

| Line | Best measured result |
|---|---|
| A. Geometric stack language model | 19.9M chat stack: 0.933 BPB served, multiplier-free (11.9 MB). 214M Plan A base: dev NLL 2.073. Code and arithmetic answers are still wrong. The ~96M run's result is recorded in the #820 history, not repeated here. |
| B. Native geometric learner (compiler, exact store, emitter) | 8 of 512 replies complete; best conditional gate 9 of 15. [Pair-only correction excluded at one frozen error](docs/labs/pair-copy-bound-2026-10-09/README.md); [complete donor credit implemented](docs/labs/occurrence-joint-credit-2026-10-09/README.md), model benefit unmeasured. |

Both lines join at M4, one model served under D11.

## Serving

The D11 engine is bit-exact with the float path. `uor-chat --stack <ARTIFACT.lut>` serves the
geometric stack (PR #2050, merged 9 October 2026; greedy decoding only, and a measured bundle
negative is recorded there). The Pages Studio still runs the older R4G1 router, not the native
or stack model. Energy savings are unmeasured.

## Compute and labs

- Pods only through `scripts/pod/uor-pod`: ladder 5090, then 4090, then PRO 6000; at most 4
  pods and $8/h across all labs. See [compute](docs/labs/compute.md). Leases and the event log
  are on the compute board [#2037](https://github.com/UOR-Foundation/uor-r4/issues/2037).
- Labs: Claude, Codex, DeepSeek. Work is claimed on the milestone issues
  ([#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029) to [#2036](https://github.com/UOR-Foundation/uor-r4/issues/2036)); see the [lab protocol](docs/labs/protocol.md).
- `origin/main` is the single source of truth; branches are temporary delivery steps.
- Protected PRs need recorded exact-head review and checks run at that head. The queue
  acknowledgement jobs are not compile or test evidence.
- Open bugs: [#1542](https://github.com/UOR-Foundation/uor-r4/issues/1542), [#1476](https://github.com/UOR-Foundation/uor-r4/issues/1476), [#1718](https://github.com/UOR-Foundation/uor-r4/issues/1718), [#1738](https://github.com/UOR-Foundation/uor-r4/issues/1738).
