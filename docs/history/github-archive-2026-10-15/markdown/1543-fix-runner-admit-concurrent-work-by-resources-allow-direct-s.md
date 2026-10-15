# #1543 — fix(runner): admit concurrent work by resources; allow direct small checks

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-30T13:24:07Z
- **Updated:** 2026-10-09T13:17:55Z
- **Closed:** 2026-09-30T14:23:47Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1543
- **Merged:** 2026-09-30T14:23:47Z
- **Merge commit:** c2de9525d886a5d4e181d3219881668a0d857aab
- **Base / Head:** main / UOR-Foundation:codex/runner-small-validation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The old host-wide job lock made a long model test block unrelated small checks. Small development checks now explicitly run directly without the shared runner. For supervised jobs, replace one-job and lane-pair restrictions with aggregate CPU/RAM/storage admission; concurrent Cargo uses disjoint canonical caches. Retain immutable reservations, exact process ownership and genuinely exclusive measurements.

Additional-job input verification runs in an owned child so it cannot suspend monitoring other jobs. A completed peer awaiting shared reservation finalization keeps the candidate gated without holding the host. FIFO readiness precedes success publication. Exact reviewed small specs can run at warning memory pressure; critical pressure and physical reserves remain enforced.

Validation at `04981eb211a39cab23958996556dabf232b22a4c`: **84 library tests passed and offline runner build passed**, 29.54s, sampled peak411.4MiB. Diff/format and independent exact-head reviews are included in the delivery receipt. Earlier concurrent bootstrap runs passed while G1a continued. Attempt6 at this same source had83passes and one intermittent process-identity observation failure during peer termination; unchanged-head attempt7 after G1a stopped passed84. The failed record is preserved. This does not claim that inherited process-observation availability limitation is fixed; UNKNOWN remains conservative. Direct development checks are not gated by runner availability.

Deployment and actual installed-process verification are recorded separately. No model capability, speed or energy result is claimed. References #1536, #1520, #1510. Owner direct-check correction: https://github.com/UOR-Foundation/uor-r4/issues/820#issuecomment-5912984112


## Timeline (5 entries; chronological: conversation comments, reviews, review comments)

### comment 5912225363 — 2026-09-30T13:25:06Z — Casey-allard

# Independent exact-head review: bounded warning-pressure validation

**APPROVE at source-review scope. Council vote: YES to the prospective narrow host-policy change.**

- Reviewed head: `604eae9333a4e29efe5a902d241d25bc40d556c1`.
- Base: `d563b02f3270a2c931a7905d6624ba0beeb71d2a`.
- Worktree: `/Users/casey.allard/uor-r4-worktrees/runner-small-validation`; clean at inspection.
- Scope: the complete two-file diff in `tools/lab-runner/src/host.rs` and `tools/lab-runner/README.md`, plus canonical reservation digest, JobSpec fields/validation, and daemon admission/monitor call sites.
- Reviewer: independent_systems_review; non-author, same provider/root launcher. No source edits, Cargo/model execution, jobs/signals, GitHub or production changes.

No concrete blocking source defect found.

## Correctness and preserved boundaries

1. Host-policy warning_validation_specs defaults empty. Existing normal-only policies therefore retain their prior admission behavior. Policy loading rejects malformed, non-lowercase or duplicate digest entries. The only explicit HostPolicy Rust struct literal was updated.
2. The allowlist computes SHA256 over serde_json::to_vec(JobSpec), exactly the canonical form used by coordinator reservations. Struct fields and BTreeMap environment ordering are deterministic. Source/executable/input identities, command, environment, task generation, attempt identity, storage, and limits are all bound. Hashing arbitrary pretty/raw spec JSON would not be equivalent; deployment should use the actual reservation digest.
3. At warning pressure2 under the intended normal max1 policy, an approved digest is additionally constrained to 1–2 declared threads, finite positive RSS at most2GiB, wall_s1–600, and gpu=false. Changing the attempt or workload requires renewed digest approval.
4. Approval changes only pressure eligibility. Disabled admission, existing holds in the daemon, one-job/Cargo limits, provenance verification, current claim generation, exact reservation binding, ledger limits, every configured volume identity and storage floor remain applicable. Late admission checks in spawn_job call the same updated check_admission.
5. Admission rejects critical and unknown pressure values including0 and3. The existing deliberately broad admission_pressure_max2 setting remains backward compatible; the new production profile must keep max1. Critical runtime stop/hold, sampled per-job RSS and wall enforcement are unchanged.
6. The boundary fixtures cover legacy defaults, exact digest mismatch, each principal resource bound, and critical/unknown values. The disabled-admission fixture may also reject on live pressure, as its comment acknowledges; it is not proof of whole-daemon execution. `git diff --check` passed. Compilation and tests were NOT_RUN by this reviewer.

## Deployment and scope

Proceed through the protected PR after applicable executed checks are attached. Preserve old binary/policy and switch the singleton only after running jobs drain; this review does not authorize interrupting them. Keep max1 and add only actual canonical reservation digests supported by work cards and independent review, recording policy before/after identities. The daemon reloads policy each cycle after the new binary is deployed, so subsequent digest approvals need no restart. Adding the field to an old runner will not enable the new path: old serde ignores it and retains normal-only admission.

The reviewer approves this code and prospective policy, not a specific uninspected job, successful build, deployment, or pressure observation. Classification as validation rather than model training/quality evaluation remains an explicit review responsibility; a digest proves identity, not intent. Declared RSS is a monitored ceiling, not guaranteed free capacity. Existing stop-grace reservations remain part of complete cost even though the workload wall_s cap is600. Removing obsolete digests affects future admission and does not cancel an already admitted bounded job.

The separately identified blocking-filesystem-probe timeout risk is unchanged and is not reopened as a prerequisite for this focused pressure-policy correction.


### comment 5912241453 — 2026-09-30T13:26:02Z — Casey-allard

# Prospective council review: bounded CPU validation at warning pressure

**Decision: APPROVE. Required source fixes: 0.** This is a prospective D14 council vote for the exact proposal below, plus source-review approval of its bounded admission behavior. It does not claim executed tests, installation, a completed live pilot, or successful workload results.

- Reviewed head: `604eae9333a4e29efe5a902d241d25bc40d556c1`.
- Reviewed base: `d563b02f3270a2c931a7905d6624ba0beeb71d2a`.
- Worktree: `/Users/casey.allard/uor-r4-worktrees/runner-small-validation`; HEAD matches the proposal and the working tree is clean.
- Scope: `tools/lab-runner/src/host.rs` and `tools/lab-runner/README.md` only.
- Reviewer: `/root/second_council_review`; launcher/integrator: `/root` (Codex). I did not author or edit this memory-admission delta. I previously authored a different boot-identity/lifecycle repair in the ancestry, so independence here applies to this proposal, not to re-review of my own earlier work. Same provider/account; procedural independence, not provider diversity.
- Authority: the owner's direct instruction to fix memory-pressure admission, conveyed in the current parent assignment and recorded in `memory-admission-work-card.md` / `memory-admission-coordination.md`. D14 permits prospective changes to working resource constraints; mission, scientific acceptance, evidence integrity, preservation and external spending boundaries are unchanged.

## Reasons

1. **The default remains normal-pressure admission.** `warning_validation_specs` defaults to an empty list. With `admission_pressure_max: 1`, pressure 2 still rejects every unapproved job. Host-policy loading rejects malformed or duplicate digests. The README expressly keeps the production normal-only default.
2. **The new exception binds the full reserved job.** `check_pressure` computes SHA-256 over the same `serde_json::to_vec(JobSpec)` serialization used by coordination reservations. `JobSpec` is a typed structure with ordered environment maps and includes identity, command, source/executable/input provenance, limits and coordination. Changing one of those fields changes the approved digest; the claim/spec/reservation checks still execute afterward. This is not a command-name or lab-wide bypass.
3. **The exception has explicit ceilings.** Only warning pressure 2 is eligible, and only with 1–2 threads, finite positive RSS no greater than 2 GiB, payload wall time 1–600 seconds, and `gpu: false`. The existing stop grace/enforcement margin remain separately reserved; this does not promise an operating-system real-time 600-second total bound. The work-card/review process must confirm that an approved command is actually CPU validation, since an argv digest and declared GPU flag do not semantically classify arbitrary code.
4. **The physical and ownership checks remain effective.** Admission-enabled state, one-job/Cargo limits, host thread/RSS ceilings, immutable provenance, current claim/generation, exact reservation, storage identities and capacity checks remain in the call chain. Daemon admission and pre-spawn revalidation still check the cumulative ledger and provenance; the global admissions-held marker still blocks queue admission. The patch changes only pressure eligibility, not those checks.
5. **Critical and unknown admission values fail closed.** Pressure 4 and every value other than 1 or 2 reject, even when an exact digest is approved or the pre-existing global warning override is selected. Failure to observe pressure still propagates an error. Existing runtime critical-pressure stop/hold behavior is unchanged. I do not infer a new guarantee that every numerically unknown runtime observation stops a previously admitted job; this proposal specifically makes admission strict.
6. **The tests target the changed boundary.** Source fixtures cover empty-list legacy defaults, exact-spec and changed-attempt rejection, approved jobs outside each small-job ceiling, critical/unknown rejection under both normal/global-warning policy, and disabled admission. I inspected these tests but did not execute them. Focused execution and applicable compilation/format checks remain delivery evidence to be recorded by the integrator.
7. **Deployment can be useful without disturbing current work.** Approve protected delivery and subsequent deployment after active jobs drain. Preserve the previous binary and policy, verify the delivered source/executable binding, and keep the source of truth on GitHub. Adding/removing a reviewed exact digest is an atomic host-policy update; future approvals do not require restarting the daemon. The initial binary switch must not interrupt the running job.

## Prospective operating decision and limits

Approve the deployment steward's use of this exact-spec list for independently reviewed CPU validation within the stated ceilings, recording the work card, reason, digest and before/after policy identities. Do not raise the production global `admission_pressure_max` to 2 as part of this proposal. Its broader branch is retained for compatibility, not newly authorized by this vote. Reviewers must not put model training or quality-evaluation jobs on this validation list. Obsolete entries are removed after finalization; immutable attempt identity/tombstones continue to prevent replay.

At rollout, retain all normal resource reservations and checks, run the bounded admitted validation specified in its work card, and record the actual outcome and cost. A failed or blocked attempt remains failed or blocked. If pressure becomes critical, the existing runtime guard applies. Removing the list is an admission rollback; it does not retroactively cancel a bounded job already admitted. No threshold, result, source binding or completed job is rewritten.

**Minority objections:** none. **Material limitations:** declared RSS/GPU intent is not proof of spare system memory or of a command's semantic purpose; existing monitoring and independent exact-spec review remain necessary. Linux's existing memory-pressure approximation is unchanged. No broad runner framework expansion, scientific policy relaxation, or model-compute authorization follows from this approval.

Read-only review only: no build, test, provider call, job signal, production policy mutation, source edit or GitHub write occurred. This file is the sole output mutation.


### comment 5912540629 — 2026-09-30T13:43:53Z — Casey-allard

# Source review — #1543 at `604eae93` (Kimi coordinator, non-author)

Reviewed the exact-head diff (`host.rs` +127, README +29) against the admission policy I executed and debugged end-to-end this session (hold-reconciliation, verify_volume wedge, pressure gating). Source-only; no execution; the runner is operational.

**Decision: APPROVE by source review.** The change is conservative, correctly bounded, and leaves the default behavior bit-identical. It directly addresses the pressure-flap friction observed all session (small validation checks blocked at pressure 2 alongside heavy work) without weakening host protection.

**Design verification:**
- **Default unchanged:** `warning_validation_specs` defaults to `[]`; at pressure 2 with the stock policy every spec still defers with the original "memory pressure elevated" semantics. Only an explicitly registered spec digest is eligible, and only at warning (2), never at critical/unknown (0/3/4/…).
- **Exact-spec binding:** eligibility requires `coord::digest(serde_json::to_vec(spec))` to be present in the policy's `warning_validation_specs` — the **same canonical serialization as reservations** (coord.rs spec_digest), so an approved check is bound to its exact frozen spec; any edit (argv, id, any field) invalidates eligibility. Verified against the reservation digest mechanics.
- **Small-check bounds:** threads ∈ [1,2], rss_gib finite >0 ≤ 2.0, wall_s ∈ [1,600], `!gpu` — all enforced, and the tests cover each boundary violation (3 threads, 2.001 GiB, 601 s, gpu, 0 wall, 0 rss).
- **No bypass of other gates:** the new `check_pressure` replaces only the old blanket `pressure > max → Err`; coordination, storage, volumes, budget, job-lock and policy-enabled checks all still run afterward. `pressure_approval_does_not_skip_disabled_admission` confirms admission_enabled is still honored.
- **Tests:** exact-spec requirement (argv/id edits fail), bounds enforcement, critical/unknown never admits even with the override, legacy policy (`admission_pressure_max = 1`) still means 1 for unapproved specs. Four focused tests, all sound by inspection.

**Notes (non-blocking):** (1) Operationalizing a warning-pressure check requires adding its exact spec digest to the production `host-policy.json` — a deliberate per-check registration, which is the right governance; the README documents it. (2) This is orthogonal to issue #1542 (verify_volume timeout); both are worth landing.

Merge-eligible under the owner's merge rule once the exact-head compile/test evidence lands; dispatching the `fast-gate` (product_path) run on `604eae93` now.


### comment 5913034856 — 2026-09-30T14:12:45Z — Casey-allard

_edited 2026-09-30T14:12:54Z_

**Owner decision, 2026-09-30 14:11 UTC: the laptop runner's rules.** Relayed by the Claude lab. The owner chose "Simple spec to Codex" in a menu. Codex implements exactly these five rules; **nothing more is added without the owner's OK.**

1. **One FIFO queue** on the laptop.
2. **Each job declares its threads, RAM and wall time.** A job starts when the running totals plus its request fit within **8 threads and 11 GiB**. There is no separate job class with its own caps.
3. **Memory pressure** blocks new starts **only at critical** (macOS level 4). Normal and warning pressure do not. A running job is stopped only at its declared wall time.
4. **No per-job coordination reservations, receipts or finish events are required to run a job.** The runner keeps its own per-job record (argv, source, exit, elapsed, peak RSS) for the ledger.
5. **PR compile/test checks run on GitHub's runners, not the laptop.** The repo is public, so this is free. The laptop runner is for model training, evaluation and Metal work.

Existing disk-space floors are unchanged by this decision.

**Why:** the owner found the admission rules arbitrary and constantly changing. One serialized lane blocked every lab's checks behind any model job. PR validation has moved to GitHub under the owner's merge rule (#820 issuecomment-5912382942).


### comment 5913188368 — 2026-09-30T14:21:10Z — Casey-allard

# Independent final resource-concurrency source and council review — v7

Reviewer: Codex subagent `/root/second_council_review`, launched by `/root`.
Date: 2026-09-30.
Exact reviewed source head: `04981eb211a39cab23958996556dabf232b22a4c`.
Full-change base: `d563b02f3270a2c931a7905d6624ba0beeb71d2a`.
Worktree: `/Users/casey.allard/uor-r4-worktrees/runner-small-validation`; HEAD matched and tree was clean at final inspection.

## Decisions

**Source delivery: APPROVE. Required source fixes for this delivery: 0.**
**Prospective Class C council decision: APPROVE** the latest direct-owner correction: small development checks may execute directly without mandatory runner admission; supervised concurrency follows actual aggregate resource reservations rather than an artificial ordinary/validation job quota. This vote supersedes this reviewer's earlier narrow two-lane policy vote prospectively. It does not alter earlier outcomes, bypass protected delivery, authorize paid/external compute, or weaken scientific acceptance.

**Known unresolved limitation:** the unchanged process-ownership observation path produced one intermittent fail-closed monitor error in attempt 6. It remains documented below. This approval is for source delivery and bounded operational rollout, not a claim of uninterrupted production concurrency or complete autonomous-runner reliability. No minority objection to the revised policy; the operational qualification limitation is explicit.

## Final source assessment

Reviewed the complete seven-file change incrementally against the stated base, retaining prior exact-head reviews and rereading actual callers for successive corrections. The final changes affect README, admission fixtures, CLI verification dispatch, coordinator, daemon, host admission and JobSpec. `process.rs` has no diff against the base.

- **Resource-based reservations:** coordinator examines every unresolved reservation on the host, validates exact stored spec/digest/phase/root, and applies aggregate thread/RSS/GPU/exclusive constraints to their complete union. First reservations are bounded too. Per-issue, immutable attempt, generation and missing-running-reservation fences remain. Host policy permits positive max_jobs up to max_threads (at most eight under the current ceiling); lane labels do not ration slots. Every job declares at least one worker.
- **Host admission:** aggregate <=8 workers and <=11 GiB remain, including GPU unified memory, with one GPU and genuine exclusive execution preserved. Concurrent Cargo target directories are checked pairwise after canonicalization, including nested/symlink overlap. Existing storage identity, physical floors, complete reservation and ledger checks remain.
- **Warning memory:** the exact-spec digest approval remains a narrow eligibility rule for <=2-worker, <=2-GiB, <=600-second CPU validation under warning pressure, with global normal-pressure admission retained. Critical/unknown pressure rejects admission. It is not a blanket model-compute allowance.
- **Direct small checks:** README now records the owner's explicit authorization for bounded formatting/static/warm-cache build/focused test work in owned worktrees without the runner/heavy reservation requirement. Honest bounds, observed pressure/storage, separate caches, source-bound receipts and once-only costs remain; hidden model campaigns and unsupported exclusive timing claims do not follow. Coordinate genuine exclusive measurements so direct development does not invalidate them.
- **Stable specification compatibility:** optional validation_lane remains omitted when false, preserving legacy ordinary canonical digests; it retains its declared small-job bounds without allocating a mandatory separate lane.
- **Late checks:** actual running peers pass through the final host check, and complete wall/grace/enforcement reservations are summed with checked arithmetic before the late ledger check. No reservation omission is authorized.
- **Owned input verification:** additional-job provenance hashing runs under the candidate's owned supervisor while daemon monitoring continues. Exact canonical digest and source/input provenance must verify before payload launch. The parent refreshes identity, current spec, holds, policy, ownership, resources, storage and budget before consuming the attempt and opening the payload gate.
- **FIFO readiness fix:** success is now published only after the shell opens fd4 on the payload FIFO. Thus an observed verification success has an open endpoint ready to retain the parent's digest line. The once-only payload gate and digest check remain. This resolves the v4 ordering blocker.
- **Completed-peer wait:** only an exact originally observed peer with matching local DONE identity/spec/attempt/receipt and fresh positive stopped evidence may cause a bounded wait for shared finalization. The actual reservation validator still must pass before launch. UNKNOWN, changed, foreign or unobserved peers remain fenced. The candidate wall/RSS monitor continues; no reservation is silently dropped.
- **Regression correction:** test fixture time now uses nondecreasing observed server time; production clock-regression validation is unchanged. The obsolete blanket single-job expectation was replaced with an exclusive-predecessor rejection preserving the unchanged-state assertion.

## Exact-head executed evidence independently read

Attempt 7 is a single bounded unchanged-head retry after the parent's reported G1a stop reduced the heavy workload. Its packet explicitly records that causal hypothesis. The source, assertions, and UNKNOWN checks were not changed between attempts 6 and 7. One failure/pass pair does **not** prove CPU load caused the failure or that the observation issue is repaired.

Read intent, both command receipts, the aggregate receipt and logs; recomputed both log SHA256 values against their receipts:

- `cargo test -p lab-runner --offline --lib -- --test-threads=1`: **84 passed, 0 failed**, exit 0, confirmed stopped. Log SHA256 `e3caff8d14fa142659be019c087a6f0940498e0f9c7a393352f56dd4ca3cdc61`. Observer command elapsed 17.005375292 s; harness 16.45 s.
- `cargo build -p lab-runner --offline`: exit 0, confirmed stopped. Log SHA256 `05c22e1b458e048160622c5dc8000733f86ce86bef05d2179fbc9f1bb16f3332`. Observer command elapsed 11.611562083 s.
- Aggregate observer elapsed 29.53851175 s; sampled peak aggregate RSS 431390720 bytes; sampled cache growth 1753088 bytes; free storage after 49761652736 bytes. These are retained observer measurements, not exclusive performance or independently sampled measurements by this reviewer.
- Source binding `04981eb211a39cab23958996556dabf232b22a4c`; packet SHA256 `aa23ecc292d202481fea7156ded0e0cb96bcb2130e52a797df46b846a796dcbb`.
- `memory-concurrency-checks-attempt7/receipt.json` SHA256 `d173769f2c11ea70762e95beadc1cd39e5debb70a034a10d0b7c3908166913a3`.

The passing suite includes actual FIFO/verifier fixtures, peer monitoring and bounded timeout, the production reservation validator's completed-peer case, all-peer resource/identity checks, multiple same-kind reservations, physical maximum rejection and exclusivity. This supports implementation delivery; it is not a live multi-model production qualification.

## Preserved failure and operational limitation

At the **same source head**, attempt 6 produced **83 passes / 1 failure**, Cargo exit 101, and did not run the build. Its pending-verifier/peer-stop test failed because monitor returned `live process-group member ownership is unavailable; stop state is unknown`. The failure log and receipt remain intact; log SHA256 `2b02eac905382e5dce4b16af093cfdb940985006da93f6131597df6e080aca02`; aggregate receipt SHA256 `2fd7447b5c333c175350e360ffa9e0eccda7354d7d496a4a20020c2bca688c75`.

Production responds to such observation errors by holding admission, retaining the ownership/UNKNOWN protections and requiring reconciliation. This can interrupt availability and cannot be presented as a successful clean handoff. The later pass establishes that the revised tests/implementation can complete without this observation failure, not that future runs are immune. No source fence or assertion was weakened to obtain the pass. Since the observation code is unchanged, behavior remains fail-closed, and directly executed small checks are now authorized independently, this known intermittent hold is a recorded reliability limitation rather than a demand to repair every inherited process-observation behavior before delivering this scoped change.

Attempt 5's four fixture failures also remain preserved and are not relabelled. Necessary retries and reviews must retain their separate costs; the observer does not itself perform the programme ledger debit.

## Rollout and claim boundaries

Verify the current production runner is drained before daemon replacement: current production recovery stops surviving workers and does not implement uninterrupted adoption. The parent reports G1a positively stopped; this review did not independently inspect its process or scientific outcome. Confirm the empty/positively stopped state from the actual runner before install, preserve previous binary/policy, then verify the installed binary and host policy identities. A completed source/build review does not itself prove this deployment occurred.

Continue to bind actual command behavior during work-card review. Disjoint CARGO_TARGET_DIR declarations do not prevent `--target-dir` or wrappers from redirecting Cargo; inspect those overrides and actual worker/GPU purpose. No general command sandbox is claimed. Keep true performance/energy measurement exclusivity, unique material, artifact provenance, independent scientific review and protected delivery intact.

No capability or energy result, successful model result, installed branch protection, unattended client adapter or project completion is inferred from this infrastructure change. Future monitoring errors need honest held/unknown status and a recoverable handoff, not a fabricated pass.

## Independence and actions

Non-author of the memory/concurrency, verifier and fixture deltas. Reviewer authored an earlier boot/lifecycle repair in the ancestry; this dependence is disclosed. Procedural same-provider/shared-account independence, not provider diversity. The root and separate systems author implemented these changes.

Performed source and retained-log/receipt inspection plus read-only hash verification. Ran no Cargo/build/test/model/provider command and made no source/GitHub/production mutation. This review is the only new file written in this review turn. Earlier reports remain immutable.



---
_Comment/review ids in this file: 5912225363, 5912241453, 5912540629, 5913034856, 5913188368_

