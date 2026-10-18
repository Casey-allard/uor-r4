# A1 completion synced to canonical docs; the #2700 claim corrected — 2026-10-18

K16-A/seed1 completed its registered 16,404 batches / 407 accepted, exit 0 at 20:58:35Z; report raw
`fa863063…`, final channel `6936cf99…`, all four evaluation hashes matched. Status
`COMPLETE_PENDING_REGISTERED_TWO_SEED_INTERPRETATION`: no KEEP, paired, fresh or capability claim.

Corrects the stale canonical text — NOW step 5 + paragraph, ROADMAP line 112 + M1 row, three STATUS lines,
the README M1 row and the docs/labs README line — from training NOT_RUN/RUNNING to the completed-A1 state.
R-seed1 is host-unblocked at the restored floor but NOT_STARTED; A-seed2/R-seed2 remain serial.

Also corrects the earlier record's false claim: #2700 commit `1cf87485` contains only its record and the
current-state entry, not the canonical-page edits. No result claim is added. Checks: the prebuilt
claim-wording checker (crate unchanged between `df18b165f` and this head) passed; docs-only; no new build;
no compute, $0.
## Post-delivery correction (2026-10-18, deepseek)

R1 actually started at **21:26:32Z** (before this record was merged): same pod/GPU0, existing admission,
frozen source `21224b44`/ELF `3add3e8d`; PID 176300, source cursor 1,408 completed / 58 accepted / next
target 90,112 at 21:29:48Z. The NOT_STARTED/host-unblocked statements above are therefore superseded as
current status; they remain dated pre-R1 history. A-seed2/R-seed2 are still NOT_STARTED/serial.

Also: A1 partial DATA cloud custody completed for the 2109Z snapshot (three exact compressed shards;
332 regular members / 1,848,283,971 raw bytes; outer `89138176 B`/`93328920…`, controls `19,968 B`/
`2896c581…`, metadata `71,168 B`/`f1f654f3…`; root join `9576bc73…`). Four-role/final late-OP custody is
not complete; `safe_to_down=false`.

Optional context: A1's final one-arm DEV counts were 14,287 rows, 2,237 target-admitted, 65 top-1 and 547
top-8 (Codex `6255106277`). Registered paired interpretation is pending; no pair outcome, KEEP, fresh
result or mechanism verdict follows.