# UOR-R4 lab runner

The existing Rust runner now separates lab sessions, task ownership, job
execution, scientific acceptance and protected delivery. A submitting client may
run out of tokens while its bounded job continues under the local supervisor.
The runner does not supply provider tokens or infer scientific success from an
exit code. See [the lab contract](../../docs/labs/README.md).

## Internal control state

Default queue: `~/.local/share/uor-r4/runner`. Default ledger:
`~/.local/share/uor-r4/ledger`. `--root`, `UOR_RUNNER_ROOT`, and `--ledger-dir`
override these explicitly. Control state beneath `/Volumes` is rejected. Keep
binary, startup logs, journal, locks, attempt tombstones and outbox internally;
external volumes contain verified payloads only.

```
runner/
  host-policy.json
  queue/<id>/spec.json
  running/<id>/{spec,attempt,process,exit}.json
  done/<id>/{spec,attempt,process,exit}.json
  locks/
  admission-blocked.json
  daemon.out.log / daemon.err.log
```

A missing/invalid host policy holds admission. Production policy names verified
volume UUIDs and sentinels, capacity reserves, the coordination store, memory
pressure ceiling and thread/RSS bounds. Concurrent admission uses combined
resource reservations; input verification preserves monitoring of other jobs.
Missing, wrong or substituted storage fails before launch. No missing mount
point is created by this runner. See `host::HostPolicy` for the versioned schema.

## Job identity and resources

The extended `uor-r4.lab-runner-job/1` retains argv arrays and adds `cargo`,
`storage` reservations, `provenance`, bounded `stop_grace_ms` and `coordination` (issue, session, epoch, work-card
version and immutable attempt ID). Production requires a matching shared
reservation, host identity and spec digest. A global immutable start tombstone
prevents replay of an admitted attempt through another local queue.
Provenance binds a clean source commit, absolute executable path/SHA256, and
input file/manifest SHA256 identities. Nonignored untracked source is rejected;
ignored outputs may remain. Executable identity is not a reproducible-build
attestation. Live ownership/reservation and the exact runner root are checked
again after hashing and immediately before consuming the attempt and spawning.

The host starts with one heavy lane and one Cargo process, no more than eight
threads and eleven GiB declared memory including GPU unified memory. Actual
process-tree RSS and OS memory pressure are observed; GPU residency is not
independently measured. Disk checks account for each volume and reservations.
Job wall limits use a monotonic clock while the supervisor lives.

Launch intent is persisted before spawn. A gated process is identified by host,
boot, PID, process start, process group and nonce before payload execution.
The nonce-bearing supervisor remains alive after payload exit so background
descendants cannot lose their ownership anchor. Signals require revalidated
identity; verified descendants are stopped before that supervisor.
`stop_grace_ms` defaults to 250 and may be 0–30000, permitting an existing TERM
handler to save its own state before KILL. Admission reserves the payload wall
limit, this grace and a 2000 ms enforcement margin. Normal supervisor lifetime
charges use measured monotonic elapsed time; recovery charges identify their
wall-clock estimates explicitly. This is a scheduling bound, not an
operating-system real-time guarantee.
Receipts distinguish a recorded TERM request and confirmed process stop from
a saved checkpoint. `checkpoint_status` remains
`unavailable_no_verified_payload_protocol`: no versioned, attempt/spec-bound
checkpoint manifest and reload acknowledgement is qualified yet. Do not admit
a model job whose safe continuation requires that unsupported acknowledgement.
After a
supervisor crash, production conservatively stops verified surviving workers,
records interrupted/unknown status and holds admission for reconciliation; it
does not reset a wall limit. A lab's token exhaustion alone does not restart the
supervisor and therefore does not interrupt its job.

Execution receipt precedes an idempotent charge keyed by job and attempt; DONE
follows both. Unknown exit status remains unknown. Restore/replay never turns
an unknown result into success.
`reconcile-stopped` can produce a separate immutable proof after verifying that
the owned group is gone (including a verified prior boot) or stopping a live
nonce supervisor. It retains the original UNKNOWN result and charge. For
recovered execution, it records a conservative wall-clock estimate with a
reserved-cost floor and adds any unaccounted time as an idempotent, explicitly
estimated supplemental charge. It does not claim a monotonic measurement across
a supervisor restart.

A separate positive `preflight-failure.json` proof uses schema
`uor-r4.pre-spawn-failure/1`. The runner writes it under the launch lock only on
a control-flow path that returns before supervisor/payload spawn and before
the consumed-attempt tombstone. It binds host, canonical runner root, job and
attempt IDs, spec/attempt/nonce hashes, phase
`preflight_aborted_before_supervisor_spawn`, reason and measured preflight
elapsed time. Reconciliation validates those bindings and rejects contradictory
launch files, a tombstone or a live nonce supervisor. This proves
`confirmed_not_started`, retains measured preflight cost and adds no execution
cost; the scientific result remains unknown. Without this positive proof, a
missing process record requires an observed nonce supervisor. Missing launch
files alone never prove that a restored attempt did not run.
Shared reservation finalization and release of the admission hold remain
separate, evidence-checked actions.

## Commands

```text
lab-runner submit SPEC_JSON
lab-runner daemon [--poll-ms N] [--monitor-ms N] [--stop-file PATH]
lab-runner status [ID]
lab-runner tail ID
lab-runner cancel ID
lab-runner reconcile-stopped ID
lab-runner install-agent
lab-runner ledger rebuild
lab-runner ledger migrate BASELINE_JSON
lab-runner ledger extend EXTENSION_JSON
lab-runner ledger import-legacy IMPORT_JSON
lab-runner ledger charge-resource RECORD_JSON
lab-runner ledger observe-legacy-snapshot RECORD_JSON
lab-runner coord init STORE REMOTE OWNER/REPO POLICY_SHA
lab-runner coord status STORE
lab-runner coord apply STORE EVENT_JSON
lab-runner delivery check RECEIPT_JSON
lab-runner delivery enqueue RECEIPT_JSON
lab-runner github-sync OWNER/REPO OUTPUT_ROOT
lab-runner outbox send STORE EVENT_JSON
lab-runner outbox replay STORE
lab-runner outbox retire STORE EVENT_ID REASON
lab-runner maintenance check REQUEST_JSON COORD_STORE
```

`install-agent` writes a plist and prints the bootstrap command; it does not
silently launch jobs. Migrate the real programme ledger with an independently
reviewed `uor-r4.ledger-baseline/1`, listing every covered historical record and
its hash. A restored cumulative snapshot is not an automatic accounting anchor.
Uncovered legacy records require explicit mappings. Never replace the real
ledger with a smoke ledger.

A migration from a still-known legacy ledger should bind `legacy_source` and
`legacy_snapshot_sha256` in the baseline. Preserve the original counter bytes
internally as `legacy-snapshot-<sha256>.json` before migration. Positive-budget
admission checks compare the old source's complete `charge-`/`extension-` file
set and hashes, plus its `model-time.json` hash, against reviewed coverage.
New, changed, missing or unreadable legacy data blocks new admission. Internal
rebuilds, job finalization and zero-budget monitoring remain usable while that
volume is absent; this watch does not cancel admitted work.

Import new legacy receipts normally. Preserve a changed same-name receipt
under a new internal filename and use optional `LegacyImport.legacy_observed`
(`file`, `sha256`, `previous_sha256`) to advance the watched external identity
without altering originals. Counter-only drift needs explicit accounting
review followed by `lab-runner ledger observe-legacy-snapshot RECORD_JSON` with schema
`uor-r4.legacy-snapshot-observation/1`, prior/new hashes, the reviewed internal
`LedgerState`, authority and rationale. The API preserves the actual snapshot
and appends a watch-only record; it adds no charge or allowance. Ambiguous,
disconnected or repeated historical hash chains remain blocked for manual
reconciliation. Double reads detect concurrent drift during inspection; they
are not a cross-client lock, so retire old writers as part of migration.

New job charges include typed `accounting` metadata with category `job_execution`.
Normal supervisor-lifetime execution and positive pre-spawn failure costs are
`measured`; cross-restart recovered execution and its reconciliation supplements
are `estimated`. Their measurement status follows the retained execution
evidence, not whether a process is now stopped. Older immutable receipts may
lack this metadata and remain
readable without rewriting them. `ledger::record_resource_charge` accepts
immutable `uor-r4.resource-charge/1` records for preparation, review, storage
work, build and delivery elapsed time. Each has a unique event ID, measurement
status, authority and rationale. A correction is a signed `adjustment_ms` with
`accounting.correction_of` naming the exact prior charge filename and SHA-256;
it appends history and never replaces a receipt. Rebuild rejects changed
targets, cycles, event conflicts, category changes and negative corrected
original totals. Storage byte reservations remain a separate host-policy
quantity; a storage-work time charge does not claim freed or reserved bytes.

Coordination uses a dedicated bare Git store and normal fast-forward pushes to
`codex/lab-state`. Events are immutable and idempotent by ID. A rejected race is
recomputed against fresh state; conflicting claims are never merged. Expired
ownership leaves unknown job reservations intact. Live GitHub is required for
new admission; an outage does not cancel a previously admitted bounded job.

Delivery checks real command/log receipts, independent review, applicable
council evidence, current task generation, exact PR head/base, dependencies and
live required statuses. Queue acknowledgement statuses do not count as local
tests. The checker enqueues only the reviewed head. Shared GitHub credentials
make reviewer independence procedural. The new server workflow and repository
rules require the separate [administrator setup](../../docs/labs/admin-enforcement.md);
shipping this binary does not install that protection.

GitHub sync produces immutable, source-pinned Markdown/JSONL snapshots with
explicit coverage/freshness receipts. It paginates issue/PR bodies, conversation
comments, submitted reviews and inline review comments. Review retrieval covers
every changed/all-state PR candidate, including closed PRs, plus PRs discovered
by the updated inline-comment feed. Review records retain source/supersession
identities and content-bound revisions in their JSON body without changing the
knowledge index's ten-field record interface. Server start/finish times bound
collection freshness; a five-minute overlap protects incremental cursors, and
future/legacy cursors trigger full reconciliation. Coverage remains explicitly
incomplete: pending/deleted or unobserved revisions, GraphQL thread resolution,
timeline events, check logs, binary artifacts and full Git history need direct
inspection. Periodic full reconciliation remains necessary when discovery
timestamps do not reflect a review update. This is a rebuildable retrieval aid.

## Focused verification

`cargo test -p lab-runner --offline` uses exclusive temporary queues and ledgers
for failure/race tests. The explicit `daemon --test-mode` accepts only bounded
inert fixtures under temporary roots; it is not a production bypass. Production
smokes must have real task/attempt reservations and a verified host policy.
Test execution, daemon installation, client continuation handshakes and recovery
receipts are reported separately. No model qualification follows from them.

## Small CPU validation under warning memory pressure

Keep production `admission_pressure_max: 1`. A yellow/warning observation (2)
may admit an independently reviewed small CPU validation attempt by adding its
canonical reservation `spec_sha256` to host-policy `warning_validation_specs`.
This optional list defaults empty; legacy policies retain their normal-only
behavior. The exact spec must declare at most two threads, 2 GiB RSS and 600
seconds, with no GPU. Changing the command, inputs, source, limits or attempt ID
requires a new digest approval. Do not use this exception for model training or
quality evaluation; reviewers check that scope before adding the digest.

The deployment steward may approve qualifying checks from their work cards and
independent review without another owner prompt. Record the reason, digest and
policy before/after identities on the owning issue; atomically update the host
policy. The daemon reloads it each cycle, so future approvals need no restart.
Remove obsolete digests after finalization. The global warning override remains
backward compatible but is not needed for this small-check policy.

This exception skips only the warning-pressure rejection. Lane/Cargo-cache
ownership, reservations, source/input provenance, cumulative budget, volume
identities, physical storage floors and measured RSS/wall enforcement remain.
Critical (4) and unknown pressure values block admission even with an override;
critical pressure retains the existing runtime stop/hold behavior. Declared RSS
is a monitored ceiling, not a guarantee of spare host RAM. If a check needs more,
preserve the failed attempt and revise the projection rather than relabeling it.
Rollback removes the new list (normal-pressure policy); it does not cancel an
already admitted bounded job. Preserve the previous binary and policy when
first deploying this additive runner change, and switch only after jobs drain.

## Resource-based concurrency and direct small checks (owner correction, September 30)

Small development checks do not require this runner or a heavy-job reservation.
Formatting, static checks, warm-cache builds and focused unit tests may run
from an owned worktree directly with honest bounded resources, separate build
caches and source-bound results. Monitor actual pressure and storage; stop only
your own work on critical pressure or exhausted physical reserves. Record costs
once. This does not authorize hidden model campaigns or exclusive timing claims.

For supervised jobs, concurrency follows combined CPU, memory and storage
reservations. Set `max_jobs` up to the declared CPU capacity (eight on this host);
there is no ordinary/validation lane quota. Aggregate ceilings remain eight
workers and 11 GiB including GPU unified memory, with measured pressure/RSS
monitoring. The optional `validation_lane` classification retains its small-job
bounds and serialization compatibility; it does not allocate an exclusive lane.

Reserve and start the first attempt before reserving its partner in the same
canonical runner root. Unknown or queued reservations absent from the daemon's
running set remain a reconciliation fence; do not create two waiting reservations.
Legacy clients must use the delivered runner binary to submit the additive lane
field. Each attempt retains its own task, generation, provenance, receipt and
charge. One failed validation does not turn another job's result into a pass.

Concurrent Cargo jobs require explicit existing absolute `CARGO_TARGET_DIR`
paths resolving to disjoint directories, including symlink resolution. This
allows a long model test wrapped by Cargo to coexist with a short build in a
different cache. Cargo's registry locks may still serialize short dependency
operations. Source checkouts remain separately owned. Do not set `exclusive`
for ordinary functional checks merely to obtain the old single slot. Reserve
it for actual performance/energy measurements or a documented shared-resource
need; true exclusive jobs still block all concurrent admission.

Warning-memory admission remains the separately reviewed exact-spec policy
above; concurrency is not permission to ignore pressure or storage. Preserve
physical floors and new-storage projections, including all jobs. A validation
that exceeds its envelope stops with its own receipt. The scheduler transition
must preserve any already running job's identity and immutable specification.
The initial repair bootstrap is recorded on #1536. The later direct owner
correction permits the bounded direct development checks described above.

For an additional job, source/input hashing runs inside its owned supervisor before
`payload.go` is opened. The same job wall/RSS/identity monitor remains active
for all attempts during verification. Verification failures never execute the
payload; verification output has separate logs, and late ownership/resources/
budget are checked again before the payload gate opens. An uncertain restart
keeps the existing fail-closed reconciliation behavior; this change does not
claim live production-worker adoption across daemon replacement.
