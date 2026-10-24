# ORDER: infra-loop-harness, kimi-37 (record remaining answered premise probes)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8 (notes/roadmap migration). Base: `f1f55c0c1`.

## Goal

Close only the normalization, exact-memory and reasoning-instrument finite-control premise rows whose probes have run once. Add their decision records with exact non-claims and pending model experiments, so the loop stops repeating no-new-info probes. No capability, milestone, bar or hold change.

## Cancel probe (run first)

At `f1f55c0c1`, all three PLAN rows remain open despite completed evidence: normalization 13/13 at `5844d911c` (#2842 `6381825466`), exact-memory arithmetic 4/4 at `9aaaf13bb` (#2833 `6382217138`), reasoning-ablation instrument 3/3 at `9aaaf13bb` (#2866 `6382231845`). Repeating these commands adds no information; the status records stand.

## Files in scope

- `PLAN.md`
- `docs/integration/DECISIONS.md`
- `docs/loop/tasks/infra-loop-harness/kimi-37/**`

## Done when

- The normalization, exact-memory and reasoning premise rows are `closed`, dated 2026-10-24, with their probe answers and pending registered/panel experiments named exactly.
- D62–D64 record evidence, non-claims, alternatives and reopening cost; no other premise, bar, hold, milestone or roadmap-item status changes.
- The ORDER includes an `## Owner approval` section naming `docs/integration/DECISIONS.md` with Casey's exact approval and ISO-UTC time from this chat.
- Report/local PR checks, full gates and diff-check pass; a fresh-context doc-only review covers the exact head.

## Stop and ask if

A result would be interpreted as model/trunk/reasoning capability, a bar or hold changes, the owner does not approve the owner-path decision text, or any file outside scope is needed.

## Owner approval

Owner path changed: `docs/integration/DECISIONS.md` (rows D62–D64).

Added by Claude, finishing this PR under the owner's direction while Claude runs solo. Casey, in Claude's chat, received by 2026-10-24T22:03:50Z. Asked how to close out the four other-lab PRs, he selected: "Finish and merge all (Recommended)". The option read: "#3148/#3149: render fix, my review, merge (closes #2849, retires the busywork probe)." Under D68, these decision rows are also the lab's own, approved by a fresh-context decision review.
