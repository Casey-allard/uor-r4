# #66 — F-emissions ablation: measure ΔT contribution before keeping transition edges

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:13Z
- **Updated:** 2026-07-23T01:46:18Z
- **Closed:** 2026-07-23T01:46:18Z
- **Labels:** graph-compiler
- **Milestone:** Phase 4: Transitions and residual emission
- **Assignees:** auser, Copilot
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/66
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: #64 (chain-telescoped redesign must land first). The A-only ablation during the 0.3% incident suggested predicted-cloud ΔT emissions contribute little to scoring quality, but the measurement was taken under the broken formula. Re-run the ablation under the fixed scorer: Gate C with F emissions enabled vs zeroed, same everything else. Acceptance: a recorded decision (keep/bound/drop ΔT) with the 4-way table showing the delta; if kept, a documented per-edge weight calibration check (top transitions sampled and eyeball-verified against corpus counts).


---

## Definition of Done
- The ablation is run under the fixed (#64) scorer: Gate C with F emissions enabled vs zeroed, all else identical.
- A written decision (keep / bound / drop ΔT) is recorded in this thread with the supporting table, and the scorer config reflects it.

## Acceptance criteria
- [ ] The comparison includes top-1 agreement and bits/token for both variants on the fixture held-out set.
- [ ] The decision is justified by the numbers, not assertion (drop/bound requires the data to show ΔT contributes no meaningful gain).
- [ ] If kept: a calibration spot-check of the top transition edges against corpus counts is included.

## Comments (1)

### comment 5053419880 — 2026-07-23T01:46:17Z — auser

DECISION: DROP ΔT emissions from the scoring path (deployed default f_emissions = false), recorded with the ablation table (fixture corpus, 30,036 held-out, byte-identical artifact across runs).

| scorer | top-1 agree | bits/token |
|---|---|---|
| Rule 1 (chain, F on) | 1.2% | 71.52 |
| Rule 1 no-F (ablation) | 1.2% | 17.73 |
| Rule 1+2 (F on) | 31.7% | 9.86 |
| Rule 1+2 no-F (ablation) | 31.7% | 9.86 |
| TLA3 baseline | 31.7% | 11.88 |

Analysis:
1. The folded transition offset is a UNIFORM bias added to every candidate — argmax-neutral by construction, so ΔT-as-implemented can never change the selected token, only distort the distribution (Rule-1 bits/token worsens 71.52 vs 17.73 with F on, a 4x penalty for zero agreement value).
2. The deployed precedence path is F-invariant on this corpus (identical numbers either way: every position fires ExactContext status, which clears graph state including the offset).
3. Candidate recall is driven by active/chain regions and root prior, not F (Rule-1 top-1 candidate recall 73.4% with F active in measurement; dropping the offset does not narrow candidates).

Therefore: default f_emissions = false everywhere (deployed + Gate C named columns now opt IN with set_f_emissions(true); witness carries f_emissions so replay inherits the exact mode — Theorem 6 stays sound, 64/64 replays green). E_f edges remain in artifacts for structure/witness and any future per-token ΔT design (a different feature from the folded offset; would need its own measured justification).

Caveat recorded per DoD: this corpus is 100% ExactContext-status, so ΔT value on Graph-status-heavy distributions is unmeasured — the default flips back only if a per-token ΔT design shows measured gain there.


---
_Comment/review ids in this file: 5053419880_

