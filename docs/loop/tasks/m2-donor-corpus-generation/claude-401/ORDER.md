# ORDER: m2-donor-corpus-generation, claude-401 (record Casey's 4-pod corpus caps as D88)

Lab: claude. Item: [m2-donor-corpus-generation](../../../../roadmap/items/m2-donor-corpus-generation.md).
Base: `origin/main` at `2fc561511`.

## Why this unit

D87 capped the corpus run at 4 pods and $35/h. On 16 November no qualifying Vast host was listed under that rate, so
Casey approved the one-hour measured run piece by piece. Tuning on vast-60886842 (4×B200, $43.41/h) then read about
9.5k kept tokens/s, and Casey approved the full 1B corpus on 4 pods at that price. This unit records those answers the
same day (§5.1), so the production unit (claude-400) can cite them. It runs nothing.

## Owner approval

Casey, 16 Nov 2026, in claude's chat, between `2026-11-16T04:30Z` and `2026-11-16T06:50Z`, verbatim:
- "approve the $56.85/h host for the one-hour measurement"
- To "Is the disk charge covered by your approval?": "Yes, that's fine"
- To "Allow any verified datacenter 8×H200 at up to $58/h for the hour?": "Yes, any such host"
- To "Allow a Vast datacenter 4×B200 at up to $44/h for the hour?": "Yes, use 4×B200"
- "yes approved go please", then "go i said go"
- "approve 4 pods at $43.41/h for the full 1B - make sure you do everything possible to make these run as quickly as possible for as cheap as possible because 1270$ sped is ridiculous"

These answers approve two owner-path edits that record the decision as D88:
- the D88 row in `docs/integration/DECISIONS.md`;
- the 16 Nov D88 entry in `docs/rules/changes.md`.

No other owner path is touched, and CHARTER.md is not edited.

## Files in scope

- `docs/integration/DECISIONS.md` (one row)
- `docs/rules/changes.md` (one entry)
- `docs/loop/tasks/m2-donor-corpus-generation/claude-401/**`

## Done when

1. The D88 row and the change-log entry quote Casey's answers verbatim, and keep the lab's conditions apart from the owner's terms.
2. `loop pr-check --local` passes.
