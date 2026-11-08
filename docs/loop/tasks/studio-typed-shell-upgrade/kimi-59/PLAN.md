# Agent/session idempotence and evidence repair plan

1. Add red-first checks for all six `Navigation` command IDs, repeated interruption, throwing `getMode`, exact transcript equality, and verified deletion with no active result. Run them before production changes.
2. Restore the base command group, reject interruption when `activeId` is already current, guard `getMode`, and return `true` only for verified no-active deletion while retaining null failure semantics.
3. Replace the named shared session-double defaults with real create/select/delete/replacement behavior. Keep active-delete fixtures consistent with legacy `startNewChat()` replacement.
4. Require `Session: complete` before resuming and `restoredTranscript === firstTranscript`. Rerun the built browser and mutation checks, then all product/repository checks at one committed head.
5. Correct REPORT evidence/advisory wording and historical labels, link kimi-59, and obtain a fresh exact-head review before any ready/queue action.
