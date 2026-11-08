# Non-stoppable remount and evidence repair plan

1. Add a remount regression for `canStop === false`, a post-dispose completion assertion, and a wrong-row marker mutation. Run them before the adapter guard change.
2. Restrict synthetic remount stop to `active.canStop !== false`, preserving live predictions and actual completion.
3. Distinguish generation from verification/read deletion failures. Link kimi-60/kimi-61 and record the new zero-error totals and mutation evidence.
4. Rerun full checks at one committed head, obtain fresh exact-head approval, then deliver through the protected queue and fresh-main verification.
