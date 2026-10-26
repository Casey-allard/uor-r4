# REPORT: r-quaternion-transport-order, kimi-38

Lab: kimi. Order: [ORDER.md](ORDER.md).
Closes #2882.

## What did not work, first

- The older positive quaternion-versus-identity result was confounded by softmax attention, a sigmoid gate, dense MLPs and 332,928 fewer parameters in the identity arm. Repeating it or starting another GPU run would not answer the item's lawful-model question.
- The unconstrained S³ configuration still has a negative transport result; this PR does not present it as a win or make the whole mechanism complete.

## Cancel probe

The merged #2890 work (PR #3245) already provides the required matched on/off comparison in a lawful geometric stack. Its sheet defines `Rec_Geometric_S3_Active` against `Rec_Geometric_IdentityTransport_Control` and `Rec_Geometric_U1_AbelianControl` on both seeds. Its lab record and `docs/research/notebook/float-training.md` record the outcome as `G-Rec-Transport = FAIL`: identity and U(1) outperform non-abelian S³ on mean and the both-seeds criterion at 1,024 steps. No new run adds information.

## What changed

- Set `r-quaternion-transport-order` to `done` because its required controlled comparison and notebook row already exist.
- The PR closes #2882 only. No mechanism, bar, hold, experiment, source file or item dependency changed. Size: docs 3 file(s) +72/-1 (item status 1/1, ORDER 26, REPORT 45); generated 1 file(s) +3/-3 (roadmap README).

## Measurements

- Existing `G-Rec-Transport = FAIL`: identity transport has the highest mean D11 integer DEV top-1, followed by U(1); unconstrained S³ does not beat both controls on both seeds at 1,024 steps.
- `docs/research/notebook/float-training.md` already carries the matched comparison and negative result. This is a negative on that configuration, not a mechanism-wide rejection.
- No GPU work, training, scoring or panel execution ran in this unit.

## Checks run at the head

- `uor-harness loop report --check`
- `uor-harness loop pr-check --local`
- `uor-harness loop gates --full`
- `cargo run -q --locked -p xtask -- roadmap render --check`
- `git diff --check origin/main...HEAD`

## Skills used

- `uor-research-architect` for artifact-bound evidence and exact negative scope.
- `working-discipline` for avoiding a redundant measurement and checking live records first.

## Still open

- In-unit fresh review, CI, queue, fresh-main verification and cleanup. The recurrence mechanism's other configurations remain separate items; this completion does not claim that S³ helps order.

## Plain summary

The controlled quaternion on/off experiment already ran and answered the item: identity and U(1) beat S³ on mean and the both-seeds criterion at this horizon. This PR closes only the item that required that comparison and does not run it again.
