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