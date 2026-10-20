# Gate2 actual outcome and transitive generated-path audit

Verdict: **ACCEPT_OPERATIONAL_BLOCKER_RECEIPT; ALL_FOUR_CARGO_GATES_NOT_RUN**. The guard directly rejected an unanticipated operational file created by the unchanged setup-started reaper. This is an operator inventory/retention defect, not compiler, source, model or mechanism evidence. This result authorizes no paid retry; gate3 preparation and dispatch remain separate.

Exact declared and observed source: `c4907cd04c44073dd28813734c5dd2f1223c684b`.

Pod/session: `vast-56382752`, `codex/codex-attention-compile-gate2`.

Evidence root: `/Users/casey.allard/uor-r4-local/codex-attention-compile-gate2/attention-compile-gate2-archive`.

## Direct observations and integrity

Archived `attention-setup.exit` is 0 and `command.exit` is 1. The command/job transcripts print identical expected/actual source, then full status containing this session's eval/job records plus `?? pods/reaper.log`. `source-status-unexpected.txt` contains exactly:

```text
?? pods/reaper.log
```

The native job starts at 10:51:01Z and ends with exit 1 at 10:51:02Z. No Cargo command, gate-specific log/exit or AOT output follows. All four declared Cargo checks/tests are **NOT_RUN**. Unlike gate1's inferred failed predicate, gate2's unexpected-path evidence is directly recorded.

All **10 remote**, **20 operator** and **12 frozen packet** SHA entries verify. Manifest identities:

- Frozen packet: `3fe972314751367b2e5aa166df2e0ba828eaf843315ca44b4a7a83e118c61dc9`.
- Remote `SHA256SUMS`: `a5de45315fb001d10492f59dd1e62c8429fa6a6412c7448a99860f522cee39cb`.
- `OPERATOR-SHA256SUMS`: `c88eee22401929a2c8614920518c1d18cf9e482e319685a54f3c16120bd2b36e`.
- `cloud-readback.log`: `1d0e9d69458175debb7364c5667fb410fe2d3c50cc764e709db978eabcc4f8a6`.

The retained full-readback receipt records 353280 bytes, MD5 `b368f635e426cbb6675a327bc05980d5`. It precedes release and deletion in `lease.log`. The final status reports 0 pods and $0/h. No new cloud/provider readback was performed by this reviewer.

Ledger events: up 10:40:32Z; release 10:51:48Z; down 10:51:55Z. Provider uptime is 679 seconds at $0.96/h, estimate `$0.181066667`. The ledger wall interval is 683 seconds, separate estimate `$0.182133333`; neither is an invoice.

The supplied sealed archive omits the raw `pods/reaper.log` and `/workspace/reaper-start.log`. Their content cannot be claimed from this record. Presence of the former is retained by the full/unexpected status and job transcripts, sufficient to support the observed refusal. Existing checksums establish integrity of archived members, not completeness of this missing operational-log inventory.

## Complete path inventory for the declared composition

Paths below are derived from the exact supplied setup helper, native runner, reaper, gates and enabled host metadata tests. This is a finite inventory for this compile-only composition, not a claim covering unrelated bootstrap, model or provider modes.

| Writer / source citation | Generated path | Source guard and retention disposition |
| --- | --- | --- |
| Gate wrapper, `gates.sh:8–11,17–28,38–60` | Repository `evals/<exact-session>/**`: command/status/gate logs and exits, pending marker, copied AOT outputs | Permit only exact session subtree as untracked; global cached/worktree checks still include it. Existing archive root retains these files. |
| Native detached run, `runner/uor-pod:1662–1678` | Repository `jobs/codex/<timestamp>-<exact-session>-gpu0.log` and `.sh` | Permit only matching own-session names; retain matching raw job script/log. Peer and other job names remain refused. |
| Setup helper, `setup-helper.sh:25–27`, starts reaper; reaper `runner/uor-reaper.sh:22–25,81,113–117` | Repository **`pods/reaper.log`** | Add exactly this untracked file to the prospective allowlist, never `pods/**`. Global cached/worktree checks must still detect any staged/tracked change there. Copy raw bytes during terminal cleanup. This is the actual missed path. |
| Native diagnostic PROBE, `runner/uor-pod:465–514`; invoked by status and relevant lifecycle checks | Repository `jobs/probes/probe.XXXXXX/{gpu,gpu.err,app,app.err,runs,runs.err}`; successful probe removes its private directory, failed probe retains it | Do **not** broadly allow `jobs/probes/**`. A residual failed probe remains unexpected/blocking. Collect any residual receipts on terminal failure before teardown; use the existing unknown/probe-failure recovery policy, not a source-cleanliness exemption. |
| Host library metadata storage tests at exact source, `controller.rs:705–715`, `completion.rs:1023–1030`, `completion_baseline.rs:367–374`, `redirect_admission.rs:198–206` | Repository `local/controller-metadata-tests/**`, `local/uor-seed2-auth-*`, `local/redirect-admission-test-*` | Existing `/local/` ignore is already source-bound, with global tracked/index checks preserved. These are synthetic metadata files; the seed2-named storage test executes no seed2 model. No new ignore is required. Record metadata test transcript/result; no production state is emitted. |
| Setup/operator | Outside checkout: `/workspace/attention-setup-helper.sh`, `/workspace/attention-gates.sh`, `/workspace/attention-setup.exit{,.pending}`, `/workspace/cuda-check-setup.log`, `/workspace/reaper-start.log`, `/workspace/rustup-init.sh` | No repository allowlist needed. Preserve helper/gates/terminal/setup records as already done; add reaper launcher stdout/stderr and retain the small downloaded rustup installer as setup provenance if present. Missing files should be explicitly recorded rather than silently invented. |
| Setup and lease/reaper internals | Outside checkout: `/root/.cargo/**`, `/root/.rustup/**`, `/root/uor-reaper.sh`, `/root/leases/{.<holder>.tmp,<holder>.json}`, `/root/.uor-idle-since`, `/root/gpu0.lock` | Tool/runtime/ownership state, not source changes. Retain tool versions, installed reaper identity and sanitized own lifecycle metadata as appropriate; never copy credentials or wholesale private runtime state. No broad repo allowance needed. |
| Cargo/build.rs / static construction | Outside checkout: `/workspace/attention-compile-target/**`; Cargo registries under `/root/.cargo` | Target identity stays explicit; retain generated PTX, cubins, resource reports and target marker on terminal success/failure. Do not archive reproducible compiler caches wholesale. `--locked` forbids lockfile changes; tracked source/lock changes remain failures. |
| Git clone/status plumbing | Checkout `.git/**` administrative files | Normal Git metadata is outside Git status's tracked/untracked source inventory. Record exact HEAD and guard output; no source exemption or wholesale Git archive needed. |

Native full bootstrap's `pods/bootstrap-*`, model `runs/**`, qualification/evaluation directories and other labs' artifacts are **not** outputs of this declared no-bootstrap/compiler-only path. Do not whitelist them speculatively. The native wrapper can source an existing `/root/.uor-pod-env`; this fresh no-bootstrap composition does not create that file, and unrelated inherited mode changes are not permission to expand the inventory.

## Minimal prospective repair

1. Add only `:(exclude)pods/reaper.log` to the untracked-path filter; keep the separate whole-tree cached and working-tree diffs unchanged.
2. Add a shared-guard fixture in which the exact untracked reaper log is accepted, an unrelated file under `pods/` is refused, and staged/tracked changes to the exact allowed path are refused. Preserve the existing staged-add/missing-file regression.
3. Extend terminal collection for exact raw reaper log and `/workspace/reaper-start.log`, plus retained failed `jobs/probes` receipts. Collect the installed reaper/setup provenance without secrets; record absence/errors, and retain the lease on collection failure. Make no change to reaper behavior, native source, compiler commands, training barriers or paid-run policy.

This closes the observed path and accounts for normal plus failure-generated paths of the current transitive helpers. A prospective gate3 packet still needs its own exact manifest review and distinct PM disposition; no automatic paid retry follows from this outcome.

Reviewer actions were read-only source/evidence inspection and hash verification. No pod/provider call, build, new fixture, model/reference execution or project source mutation occurred. Only this external review artifact was written.
