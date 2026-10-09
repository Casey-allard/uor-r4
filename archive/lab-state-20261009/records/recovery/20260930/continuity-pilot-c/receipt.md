# Continuity pilot C — completed live stewardship transfer

At 2026-09-30T03:37:51.491047+00:00, one admitted `/bin/sleep 90` payload was live under origin epoch5. Origin released; successor epoch6 acquired while the exact same host, boot, supervisor PID/PGID5539, start time, nonce and payload PID5552 remained live at 2026-09-30T03:37:59.319578+00:00. The original reservation and execution were adopted; no new attempt was submitted.

The worker completed at 2026-09-30T03:39:23Z with exit0 and confirmed stopped ownership. Measured charged duration was 95.226s (including runner preflight/finalization), peak observed group RSS3616KiB. Exactly one immutable execution charge matches the attempt. The successor finalized that original reservation, released the task, and both pilot sessions are unavailable. The old process group has no remaining members.

Final host state: normal admission enabled, ceilings40/60/120GiB and pressure1 unchanged. No queue/running jobs or Cargo. Current memory pressure is 2; new admission pressure eligibility is False. This is a fresh host observation and the existing guard's condition, not a new submitted workload. All three UUID/sentinel/mount checks still match; free storage exceeds normal floors. Old hold and disabled-policy snapshots are preserved here, with the old hold moved to retired-admissions-held.json. No low-floor profile was used.

Ledger after exactly-once execution charge: 809108131/1130000000ms. Root's separately recorded generic preparation/review/storage/delivery charges are retained. This pilot's card limits orchestration15min, wall120s+250ms stop grace, one thread0.25GiB and3MiB output/checkpoint storage.

The two coordination identities were operated by one reviewer through distinct CLI transitions. This demonstrates real live claim transfer and reservation continuity; it does not demonstrate independent-provider wakeup, actual token-exhaustion recovery, model checkpointing or language quality. The historical A/B outcomes remain unchanged.

Evidence: preflight.json; immutable work-card.md/spec.json; all event/result receipts; before-release.json; handoff.json; state-after-successor-claim.json; after-successor-claim.json; monitor-samples.json; exit.json; single-charge-proof.json; final-coord-state.json; final-host-state.json; summary.json. No public GitHub comments were posted by this operator.
