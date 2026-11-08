# Agent/session verification repair plan

1. Add red-first controller tests for no-op create, source-ID duplicate, unchanged delete, an 81-code-unit legacy rename, throwing active-ID reads, and synchronous stopped completion. Add reducer tests for absent active IDs and a failed-start session assertion.
2. Verify operation effects using before/after snapshots: new active creation, non-source duplication, and target removal with a valid active result. Preserve current-session resume. Normalize rename to the legacy 80-code-unit bound, fail closed on read errors, and allow interruption to use the just-stopped generation snapshot.
3. Restore command grouping and correct only the named doubles/labels. Bound rename input without changing short-title behavior. Add the failed-start `lastGeneration.sessionId` assertion and reject active IDs absent from the catalog.
4. Move browser request capture after typed evaluation, re-sample MCP, and require `17 + 25 = 42`, the R4 exact-arithmetic label, and the place-value trace. Run built-browser and all product/repository checks at one committed head.
5. Correct REPORT/NOT_RUN and per-session versus bulk-clear wording, link the repair chain, then obtain a fresh exact-head review. Push/ready/queue only after APPROVE and verify the squash on fresh main before guarded cleanup.
