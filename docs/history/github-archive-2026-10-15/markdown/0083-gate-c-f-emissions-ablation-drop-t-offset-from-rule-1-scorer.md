# #83 — Gate C F-emissions ablation: drop ΔT offset from Rule-1 scorer

- **Type:** Pull request
- **State:** closed
- **Author:** @Copilot
- **Created:** 2026-07-23T00:18:32Z
- **Updated:** 2026-10-09T14:17:23Z
- **Closed:** 2026-07-23T03:33:45Z
- **Assignees:** auser, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/83
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:copilot/uor-64-f-emissions-ablation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This PR reruns the F-emissions ablation under the fixed scorer by comparing Gate C with transition-edge scores enabled vs zeroed, with all other inputs held constant. The measured delta shows no top-1 gain from ΔT, so scorer policy is updated to drop ΔT offset application.

- **Ablation result + decision (recorded)**
  - Added the 4-way Gate C comparison to docs (enabled/zeroed × no-EXCT/with-EXCT).
  - Recorded decision: **drop ΔT offset**; keep `E_f` only for predicted-cloud candidate generation.
  - Included a transition-edge calibration spot-check sample (top edges by corpus count) for quick eyeballing.

- **Scorer semantics update**
  - Updated Rule-1 scoring to remove folded transition-edge weight from per-token score accumulation.
  - `transition_offset` remains in witness schema for compatibility, but is now always zero by policy.

- **Fixture ablation workload**
  - Added an ignored release-only fixture test that emits both variants (enabled vs zeroed), computes Gate C for both, and prints the 4-way table and deltas.

- **Tests adjusted for new policy**
  - Updated hand-computed and overlap-focused score expectations that previously included `+ ΔT-offset`.

```rust
// score_runtime.rs (Rule 1 scoring path)
let transition_offset = ScoreQ::ZERO;

for (token, (residual, mut contributions)) in candidates {
    let base = self.root_score(token);
    let score = base.saturating_add(residual); // ΔT offset no longer applied
    // ...
}
```

<!-- START COPILOT CODING AGENT SUFFIX -->

- Fixes #66

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5054006039 — 2026-07-23T03:33:45Z — auser

Superseded by 6ec5a95 (already in main): the F-ablation ran (Rule 1 bits 71.52→17.73 with ΔT off; Rule 1+2 invariant), and the decision is DROP — ΔT emissions are off by default with the witness carrying the mode flag for replay. Everything here duplicates merged work.


---
_Comment/review ids in this file: 5054006039_

