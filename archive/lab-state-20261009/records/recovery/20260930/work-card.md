## Owner-authorized durable lab system and workspace recovery

The owner explicitly requested implementation of the full durable GitHub-led lab plan in Codex on September 29. Four labs are available: Codex, Claude, Antigravity, OpenCode–DeepSeek. No permanent director is required. This card owns the infrastructure implementation; model research remains in the existing Track A/B issues.

**Deliverable:** recover the restored workspace safely; deliver the existing Rust runner with crash-safe identity/accounting, resource enforcement, GitHub task generations and handoffs, independent delivery checks, client-neutral lab prompts and reconciled governance. Preserve all unique artifacts and the intentional owner checkout.

**Observed blockers:** APFS sparsebundle is present but detached; 28 model links and 10 registered worktrees depend on it. The launchd runner was repeatedly failing against missing paths and a smoke ledger. Owner target cache alias is separately broken. Prototype runner is local commit `4d0defa0`; live main is `3bc296ca`.

**Source:** isolated full worktree `codex/durable-lab-system-20260929`; prototype preserved by cherry-pick. Codex root owns recovery/deployment/coordination. Independent agents own runner/process/storage checks, ledger/delivery/ingestion, and shared policy/prompts. One Cargo process only; no model fits in this infrastructure task.

**Projection:** up to 6 hours preparation/implementation/review/recovery; up to 45 minutes cumulative focused builds/tests/smokes, at most 2 build threads and 3 GiB build/test RAM; at most 8 GiB new internal space (admit only above 40 GiB reserve after projection), at most 400 GiB new external space including a verified ~185 GiB recovery image copy and verification receipts. No paid/external compute. Preserve 128 MiB stop margin plus declared checkpoint headroom. Check actual storage before each material allocation. Preparation/model/build/I/O costs are separate; actual charges will be appended to the recovered ledger, never to smoke-ledger. While the programme ledger is inaccessible, write immutable local recovery receipts and hold model compute.

**Recovery:** disable the failed daemon first; preserve configuration and Git worktree metadata; copy the unattached sparsebundle exclusively and verify all files; read-only inspect APFS and recorded artifact identities before canonical writable mounting. Never prune missing worktrees or delete unique data. Same-SSD backup is recovery rollback, not an independent disaster backup.

**Checks:** conflicts and ambiguous pushes, stale task generations, singleton and PID reuse, launch/finalization crash windows, exactly-once charges, storage identity/RSS/budget limits, unavailable outcomes, stale delivery reviews, GitHub outbox recovery, live job surviving client exit and stewardship transfer, restored artifact/worktree integrity. Source-only and independent result checks are reported separately from actual execution.

**Decisions:** verified recovery permits bounded execution; missing/mismatched input remains unavailable; unsafe runner stays held. Source delivery uses protected PR + independent review + existing merge queue. Current GitHub account cannot make new checks mandatory; coordinator enforces procedurally while an exact administrator configuration is prepared.

**Cleanup:** only verified disposable caches/clean merged worktrees after exact two-hour notice and recheck; no cleanup is necessary to start this recovery. No unique deletion is authorized.
