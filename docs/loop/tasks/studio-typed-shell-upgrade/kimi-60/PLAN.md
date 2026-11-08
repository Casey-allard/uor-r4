# Agent/session fail-closed and fidelity repair plan

1. Add red-first tests for failed pre-read mutation, completion catalog refresh, `canStop === false`, and observable SessionList replacement. Add a delayed external-request mutation to verifier evidence.
2. Prevalidate duplicate/resume/rename before any legacy mutation. Listen for real generation completion, refresh catalog metadata, and remove that listener on dispose.
3. Refuse interruption for `canStop === false` without calling stop or changing identity; synchronize after actual completion. Add SessionList dialog/error/replacement assertions and consistent StudioApp doubles.
4. Settle network events after typed evaluation, prove a delayed external request fails while normal flow remains local/exact. Correct characterization/history/advisory/delete-result wording.
5. Run focused/full/legacy/gates at one committed head, obtain fresh exact-head approval, then deliver through the protected queue and verify fresh main.
