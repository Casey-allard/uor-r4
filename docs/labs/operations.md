# Host operations, storage and recovery

The [shared protocol](protocol.md) specifies authority and claims. The
[runner README](../../tools/lab-runner/README.md) specifies the implemented
commands and wire formats. Do not infer live installation or successful recovery
from the presence of code. Deployment and drills must have an execution receipt.

## Host admission and budgets

The present host is an eight-core M1 with 16 GB unified memory. Its initial
runner envelope is at most eight declared compute threads, 11 GiB aggregate
job RSS including GPU unified memory, one GPU job, and exclusive quiet-machine
timing. Headroom covers the OS and interactive clients; declared RSS alone does
not guarantee safety. Observe memory pressure, swap growth and physical free
space, and reduce concurrency when those readings invalidate the projection.
One Cargo process at a time avoids multiplied link peaks and cache contention.

Use `lab-runner submit`, `status`, `tail` and `cancel` from the shared admitted
runner. Every job declares source/worktree, argv, threads, RSS, GPU use, wall
bound, outputs, storage projection and a stop decision. In production the daemon
requires its host policy; `test-mode` is restricted to isolated fixtures and
never bypasses a real host's admission. While deployment is unverified, retain
the existing single-heavy-job reservation and manual receipts. An absent daemon
does not authorize launching unlimited direct workers.

Budget preparation, subagents, builds, training, controls, evaluation, retries,
checkpointing and delivery cumulatively. Append immutable charges/extensions;
`model-time.json` is a derived view rebuilt under a lock, never the accounting
authority. `ledger migrate <manifest>` establishes `ledger-baseline.json` using
schema `uor-r4.ledger-baseline/1`: `baseline_id`, `recorded_utc`, `cumulative_ms`,
`limit_ms`, `covered_records` with each `file` and `sha256`, `authority` and
`rationale`. Preserve originals; late legacy records need explicit import
mapping rather than a “latest anchor wins” fold. V2 charges are unique by
`(job_id, attempt_id)`. Ambiguous totals remain reconciliation items, never a
silent reset. A necessary local allowance extension is already owner-authorized:
record projection, reason, increment and new cumulative limit before use. No
paid/external compute follows from that permission.

## Storage cadence

Logical roles are independent of paths: canonical Git source; active worktrees;
rebuildable caches; pinned model/data inputs; immutable results; checkpoints;
runner/control state; and derived search indexes. Identify the actual volume
UUID and mount for each. A directory with the expected name is not proof the
external SSD is mounted; never let a fallback path on the internal drive absorb
an accidental large write.

- At join/resume and before any heavy admission, check volume identity, writable
  location, physical free bytes, projected temporary/output bytes, active jobs
  and memory pressure.
- The active host steward checks resource state with the five-minute heartbeat;
  inspect after large builds/runs, before cleanup, and after any disconnect or
  restore. Keep one concise incident/status update only when the state changes.
- Owner-selected watermarks are **target / warning / stop** free GiB:
  **internal 60 / 40 / 25**, **inner workspace volume 120 / 60 / 30**, and
  **outer SSD backing volume 240 / 120 / 60**. Observe both the inner filesystem
  and its backing filesystem; inner apparent free space does not prove the
  outer disk can grow a backing image. A warning triggers planned cleanup and
  constrained admissions; a projected stop breach prevents a new job and
  checkpoints/stops affected execution according to its host policy.
  Reserve **30 GiB** for Track B traces within these projections and retain the
  existing **128 MiB absolute stop margin** in addition to operational headroom.
  These are configured thresholds, not claims of today's measured space.
  Target/warning actions are the steward's policy cadence; the production host
  policy's `reserve_bytes` and `stop_margin_bytes` implement admission/stop
  checks. Do not claim automated target/warning alerts unless separately observed.
- Stream or shard large teacher traces, reuse verified parents and one compatible
  cache per active toolchain/lane, and estimate peak temporary space before
  decoding/compressing. A backup on the same physical SSD is not an independent
  disaster-recovery copy.

Cleanup operates by manifest: canonical path, volume identity, bytes/class,
owner, last use, live process handles, Git status/upstream/merge state, unique
artifacts, and reconstruction source. Post a **two-hour notice** on #820 and the
affected board before removing lab-owned regenerable caches or clean merged
worktrees. After the notice, revalidate inactivity and scope. Use normal
`git worktree remove`, without `--force`, only for proven clean/merged worktrees;
preserve ignored unique material first. Delete only the identified regenerable
class. Do not prune worktree metadata while recovery mapping is unresolved.

Never delete unique models, source, unpushed commits, dirty work, research,
negative results, sealed roots, credentials or session history under routine
cleanup. Deduplication/compression of unique payloads is an owner-reviewed
proposal with verified hashes and restorability, not implicit cleanup. Measure
physical reclaimed space using `df` before/after; `du` is inventory and APFS
snapshots/clones may make it differ. Disk capacity and RAM are distinct, although
low disk can prevent swap and checkpoints.

## Restored SSD procedure

1. Declare a recovery incident and pause new affected heavy admissions. Inventory
   running processes and their actual executable/cwd/open-file paths; preserve
   healthy workers and never terminate another lab from a stale PID alone.
2. Record mount/volume identities and old→restored path mapping. Inventory every
   registered worktree, its `.git` pointer, common Git directory, branch, HEAD,
   tracked/untracked/ignored material and upstream. Validate unique source and
   artifacts against manifests before any repair or copy.
3. Retain recoverable dirty/unpushed work and restore references with supported
   `git worktree repair` on explicitly verified paths. Do not blanket prune,
   reset, force-remove or rebuild all worktrees. Reconcile session paths in each
   client and record manual/unavailable adapters honestly.
4. Verify artifact inventories and checksums, ledger records and job output
   status. Mark post-restore gaps unavailable; a restored snapshot can be older
   than a completed process. Do not reseal modified contents as the old result.
5. Diagnose filesystem/permissions/quarantine/dylib-loading/toolchain failures
   with a narrow read/execute/compile probe. Reuse verified binaries where valid;
   rebuild only an affected disposable cache after its manifest/notice. Do not
   disable system security or silently install credentials to make a test pass.
6. Run a temporary admitted smoke, then a bounded source-bound required check;
   record actual source, executable, host-policy, volume, outcome and costs.
   Resume dependent work only when its relevant boundary is verified.

Current historical incident: the September 29 restored mount interrupted Track
B parity and later rejected cached `serde_derive` loading (#1518). This record
does not assert that every session or compiler problem is now fixed. The newest
recovery receipt and live #1510/#820 activity own that status.

## Control-plane and delivery rollout

The implemented control-plane commands are `lab-runner coord init STORE REMOTE OWNER/REPO POLICY_SHA`,
`coord status STORE`, `coord apply STORE EVENT_JSON`, `outbox send STORE EVENT_JSON`,
`outbox replay STORE`, and `host-id`. Delivery uses `delivery check RECEIPT_JSON`
and `delivery enqueue RECEIPT_JSON`; accounting uses `ledger migrate RECORD_JSON`,
`ledger extend RECORD_JSON`, `ledger import-legacy RECORD_JSON`, and `ledger rebuild`.
GitHub import uses `github-sync OWNER/REPO OUTPUT_ROOT`. These are subcommands of
`lab-runner`; consult the [runner README](../../tools/lab-runner/README.md) for
queue/ledger global options and supported schema versions. Do not
invent CLI options from this prose. GitHub snapshots include repository/revision
and retrieval time and remain derived views. Incremental JSONL synchronization
paginates submitted reviews and inline comments for changed PR candidates,
including closed PRs, with source/supersession identities and server-time
freshness bounds. It does not reconstruct pending/deleted or unobserved reviews,
GraphQL thread resolution, check logs or full Git history; periodic full
reconciliation and direct inspection remain necessary for decisions requiring
those sources. Local events pending synchronization
do not authorize stealing a remote claim during a network partition.

Delivery receipts use `uor-r4.delivery-receipt/1` and bind repository, PR,
`task_issue`, author session, `claim_session`, nonzero `claim_epoch`, `work_card`,
exact 40-character head/base SHAs, `change_kind`
(`docs`, `code`, `model`) and review class (`A`, `B`, `C`). Checks bind the same
head/base, argv, exit code, outcome and an absolute regular nonempty log with
SHA-256. Reviews identify the independent session, launcher, decision and zero
unresolved required fixes. B/model receipts additionally name result evidence
and an independent reader; C receipts include the three distinct council
sessions and votes. The types in
[`delivery.rs`](../../tools/lab-runner/src/delivery.rs) are the executable schema.
The current claimed session/epoch/work card and available, fresh lab heartbeat
must agree with the live coordination state, and native GitHub blockers must
be resolved. A stale lease or moved head cannot reuse an old approval.

Task completion is a separate verified action. Its `delivery_receipt` object
binds a local receipt path and SHA-256, the observed merge SHA, and an acceptance
file path/SHA-256. The `uor-r4.task-acceptance/1` record names the issue, work card,
head/merge identities, non-author reviewer, `APPROVE`, `full_scope: true`, and
explicit acceptance criteria. The coordinator re-reads delivery checks and
reviews, verifies the exact PR was merged, and verifies that merge remains in
current main's ancestry. A partial PR cannot complete the task, and this action
does not close the GitHub issue automatically. Shared-account evidence remains
procedural; recording `full_scope` cannot replace the review it attests to.

Initial coordination permits only one unresolved execution reservation per host,
across all tasks and queues. An unknown or unreachable worker retains that slot
until verified reconciliation; admission cannot create two reservations that
wait on each other. Execution reservations bind the full typed job spec, its SHA-256, the local
host and internal runner root. A changed resource projection/command requires a
new reservation. Native `blockedBy` is checked independently of caller-supplied
dependencies before claims and admission; missing/truncated API data holds work.
Unresolved same-host reservations absent from the daemon's running set hold new
admission until reconciled, rather than disappearing from capacity accounting.
This is conservative cooperative coordination, not a cross-host physical resource
manager. Reserve only the next admitted work; do not pre-reserve an entire queue.

Finalizing a shared attempt requires a typed host/job/attempt receipt, verified
SHA-256 at its bound internal `done/<id>/exit.json`, matching saved/reserved spec
and claim identity, and confirmed stopped-process evidence. A queued cancellation
may instead prove that no launch intent existed. A failed preflight may retain
a positive, immutable proof written before supervisor spawn; reconciliation
verifies its source/spec/attempt binding and rejects contradictory launch evidence.
Missing launch files alone never prove that a restored job did not run. Unknown,
unreachable and legacy-unbound cases retain their reservations until explicit
reconciliation; text saying “finished” is insufficient. Declaring a lab unavailable does not
extend its task leases or release its live worker.

The checker expects real `diff-check`/`claim-wording` evidence for docs;
`diff-check`/`format`/`compile`/`focused-tests` for code; and `loaded-behavior` in
addition for a model change. These are receipt names, not fabricated GitHub
statuses. Choose the actual focused command to match the declared risk; a tool
success is not evidence that an unrun model campaign passed.

Stage the rollout: validate temporary fixtures; initialize/reconcile existing
records; perform one owner-session-loss/detached-worker smoke; exercise duplicate
claim, lease expiry, concurrent ledger append, insufficient-space and exact-head
delivery rejection; run each client adapter smoke; only then declare the
corresponding component verified. Remaining manual client steps remain manual.

Repository-administrator gate configuration is a separate deployment action;
the exact procedure and trust boundaries are in
[administrative enforcement](admin-enforcement.md):

1. Install the reviewed workflow producing `mission-delivery-gate`, then run it on
   both a pull request and a merge-group event. Record its exact observed check
   name and GitHub App identity in the deployment receipt. This change's
   coordinator receipt checker alone does not enable a required server gate.
2. In the ruleset protecting `main`, require that exact check and the merge
   queue. Retain existing protection, disable direct pushes and avoid bypass
   exceptions for lab credentials. Do not mark the five compatibility status
   names as executed tests.
3. The check must validate a head/base-bound review/evidence receipt and current
   blockers, and fail when the receipt is absent, stale or malformed. Verify
   deliberately stale/changed-head fixtures and one eligible delivery.
4. Record the ruleset ID/revision and the successful/failing check URLs. Until
   this is observed, report **procedural coordinator gate; server enforcement
   unverified**. A shared GitHub account cannot prove independent authorship;
   identified review roles and evidence remain required even after automation.

This rollout does not create unattended client capability. A durable runner can
finish already submitted jobs while chats are offline; starting new scientific
work still needs an active authorized client or an explicitly installed and
tested client scheduler. Every adapter states that boundary.
