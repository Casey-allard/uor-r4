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
