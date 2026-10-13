---
name: cost-and-compute-admission
description: Admit and budget a run before execution — CPU on the owner's laptop, GPU only through uor-pod within the owner caps, the cumulative ledger, and the hard stops that are not negotiable.
version: 1
harness: 1
kind: research
applies-to: [docs/labs/compute.md, scripts/pod/uor-pod, docs/harness/milestones.json]
when: [before compute, cost, budget, GPU, pod, lease, storage, disk floor, admission]
requires: []
outputs: [a complete cost projection in the pre-registration, a pod lease under --lab and --session, a cumulative ledger charge, a release and cleanup record]
forbidden: [CPU model compute, GPU work that cannot change a decision, exceeding 4 running pods or 8 dollars per hour, a new spending class, cancelling a healthy run on a self-set budget, touching another session's lease or files]
milestones: [m1, m2, m3, m4]
---

## When to use

Use before any run that costs compute or storage, and again whenever a run needs
more than it projected. Authorities: AGENTS.md § "Resources, verification and
delivery" (GPU pods, owner rules of 5 October 2026), CONTRIBUTING.md § "Data,
artifacts and compute", and D24 in [DECISIONS.md](../../../integration/DECISIONS.md#d24---no-cpu-model-compute-and-no-gpu-on-non-decisive-work).

## Do this

1. **Classify the work before choosing a machine.**
   - CPU-bound work — data preparation, grading, the native prose learner, local
     fixtures — runs on the owner's laptop. Rent a pod for it only when it truly
     exceeds the laptop (RAM, disk or wall time), and then the smallest offering.
   - GPU work — geometric-stack training, CUDA evaluation — goes through
     `scripts/pod/uor-pod` and never falls back to the laptop CPU.
   - Model compute never runs on the CPU (D24). Training, evaluation and every
     part of the forward that scoring, normalization or the value mix touches are
     GPU work; a silent host-side fallback is a defect to fix with a kernel, not a
     cost to report and not mechanism evidence.
2. **Check the decision first (D24).** If the configuration is already excluded by
   D23/D24 — a softmax site, or a comparison whose baseline those rules forbid —
   or if neither outcome changes the next step, the campaign is cancelled, not
   extended. This check comes before any price estimate.
3. **Write the complete projection** into the pre-registration: preparation,
   build, fit, controls, evaluation, retries, checkpointing, delivery; wall time,
   CPU threads, peak RAM, new/temporary/retained storage, and the 128 MiB stop
   margin above the 30 GiB disk floor. Charge all of it, including retries and
   resume work, to the shared cumulative ledger — an issue or a session never
   resets it. Record reason, increment and updated cumulative limit before using
   any extension (standing owner authorization, 6 September 2026).
4. **For GPU work, read live state first:**
   `scripts/pod/uor-pod status` — read its "Free GPUs you can lease now" line and
   the caps before deciding anything.
5. **Lease, do not assume.** Identity is lab + session:
   `scripts/pod/uor-pod lease POD --lab L --session S --gpus 0,1 --purpose "<one line>" --hours H`
   with `--hours` covering the whole job plus margin. One job per GPU:
   `scripts/pod/uor-pod run POD --lab L --session S --gpu K -- CMD`; GPU evaluation
   uses `device=cuda`.
6. **Respect the caps.** All labs together: at most 4 running pods and $8 per hour,
   and no new spending class. A configuration these rules already exclude, or a
   comparison whose baseline they forbid, does not get a run. Caps reached and no
   free GPU: wait for an expiry, or ask the owner — never fall back to the laptop
   CPU for GPU work. Another session's GPU is never yours, even idle; never touch
   another session's lease, files or `KEEP_ALIVE`. Only an expired lease may be
   taken over, and `lease` checks that no live job holds it.
7. **If the run is healthy, extend the run.** A self-set `max_seconds`, step count,
   planned-token count or `--hours` is an estimate, not a stop rule. Renew
   (`scripts/pod/uor-pod renew POD --lab L --session S --hours H`), extend the
   budget, let it finish, and record the extension in one line on the owning
   issue. The only hard stops are the owner caps above, the disk floor, a failed
   or diverged run, and an owner halt.
8. **Durable output on the pod, evidence off it.** Write durable output only to
   `/workspace`; move results to iCloud with
   `~/.local/share/uor-r4/bin/cloud-store put <lab> <dir>` (MD5 round-trip) as soon
   as they are used, and keep local-only data under a gitignored path (`/local/`,
   `target/`). Never put project data, builds or downloads in `/tmp` or
   `/private/tmp`. Keep 30–70 GB free, never below the 30 GiB floor plus 128 MiB.
9. **Release, then take down only what is yours:**
   `scripts/pod/uor-pod release POD --lab L --session S`, then `down` when no other
   session holds a lease. Do not keep stopped pods as storage. Finish with
   `scripts/storage/uor-hygiene --apply`.
10. **Report complete cost**, including preparation, review and delivery elapsed
    time alongside model and build time. A host-fallback execution is not
    mechanism evidence and its arithmetic and step time are wrong.

## Output

- The cost projection and its ledger charge, inside the pre-registration sheet.
- A lease under `--lab L --session S` with a purpose and an `--hours` that covers
  the whole job, on the compute board [#2037](https://github.com/UOR-Foundation/uor-r4/issues/2037).
- A release line, the artifact location (`/workspace` or `cloud-store`), and the
  final cumulative charge including retries.

## Do not

- Do not run model compute on the CPU, and do not treat a host fallback as a
  measurement of the mechanism.
- Do not run GPU work that cannot change a decision.
- Do not exceed 4 running pods or $8/h, and do not open a new spending class
  without the owner.
- Do not cancel a healthy run because a self-set budget ran out, and do not treat
  an expiring lease as a reason to stop a live job.
- Do not touch another session's pod, lease, files, branches, jobs or `KEEP_ALIVE`.
- Do not keep a stopped pod as storage, hold a second local copy of a result, or
  pollute `/tmp`.
