# ORDER: r-quaternion-transport-order, kimi-38

Lab: kimi. Item: [r-quaternion-transport-order](../../../../roadmap/items/r-quaternion-transport-order.md) (#2882). Base: `2a495e129`.

## Goal

Verify that the merged #2890 work (PR #3245) already meets the item's done-when with a matched on/off comparison in a lawful model and a notebook row. Set only this item to `done` and close #2882. Do not rerun GPU or claim a positive transport result.

## Cancel probe (run first)

The sheet and `docs/research/notebook/float-training.md` compare `Rec_Geometric_S3_Active` against `Rec_Geometric_IdentityTransport_Control` and `Rec_Geometric_U1_AbelianControl` on both seeds across two clean builds. Identity and U(1) beat S³ on mean and the both-seeds criterion, so the required controlled comparison and its negative result already exist. The older positive result was confounded; this new comparison uses the lawful geometric stack.

## Files in scope

- `docs/roadmap/items/r-quaternion-transport-order.md` (status only)
- `docs/loop/tasks/r-quaternion-transport-order/kimi-38/**`

## Done when

- The existing sheet, lab record and notebook row are verified to cover S³ versus identity/U(1) on both seeds, with `G-Rec-Transport = FAIL` recorded.
- Item status is `done`; PR says `Closes #2882` and changes no mechanism, bar, hold or experiment.
- `uor-harness loop report --check`, `uor-harness loop pr-check --local`, `uor-harness loop gates --full`, `cargo run -q --locked -p xtask -- roadmap render --check` and `git diff --check` pass; the in-unit fresh review approves the exact head; CI is green.

## Stop and decide, never wait

Any new experiment, GPU run, notebook rewrite, mechanism change or broader status change needs a new ORDER.
