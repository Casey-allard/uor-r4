# #73 — Larger-teacher quality ceiling evaluation

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-07-23T00:05:29Z
- **Updated:** 2026-07-23T14:46:29Z
- **Closed:** 2026-07-23T14:46:29Z
- **Labels:** graph-compiler
- **Milestone:** Phase 2: Overlapping multiresolution regions
- **Assignees:** auser, Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/73
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Depends on: D3 observation run (#3 in this chain). The teacher's ceiling bounds everything downstream; SmolLM2-135M is a weak teacher by design. Evaluate one larger pinned HF revision (e.g. SmolLM2-360M or Qwen-class llama-family with unsharded BF16 safetensors) through the full pipeline (download → observe → compile → cover → score → Gate C) and compare teacher floor bits/token and graph agreement against the 135M run. Acceptance: a recorded decision on the default teacher with both ceiling tables; if adopted, the pinned revision and HEAD teacher CID are updated in one commit.


---

## Definition of Done
- One larger pinned HF teacher (unsharded BF16 safetensors, llama-family) is run through the full pipeline (download → observe → compile → cover → score → Gate C).
- A default-teacher decision (stay on 135M vs adopt the larger pin) is recorded with both ceiling tables.

## Acceptance criteria
- [ ] Teacher-floor bits/token and graph top-1 agreement reported for both teachers on the same D3 partitions.
- [ ] Artifact sizes and per-token latency included in the comparison.
- [ ] If adopted: pinned revision + HEAD teacher CID updated in one commit with the table attached.

## Comments (1)

### comment 5059817821 — 2026-07-23T14:46:28Z — auser

DESCOPED by maintainer decision (2026-07-23): the +5pp stretch target is structurally unreachable by agreement-margin regardless of teacher — under EXCT precedence the graph is argmax-IDENTICAL to the baseline on every measured partition (fixture 31.7%, D3 natural 15.04%, continuity 14.76%, all with +0/-0 win/loss). A larger teacher raises the floor for every scorer equally, which changes the level but not the stretch margin. If the M.V.G. §4 stretch clause is reframed (bits-margin or residual-value target instead of agreement-over-baseline), this issue reopens with a clear role: raising the floor. Work stopped ~12 minutes in; no artifacts produced, nothing to clean up. Priority went to completing tracker #65.


---
_Comment/review ids in this file: 5059817821_

