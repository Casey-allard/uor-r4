---
id: detached-run-watcher
title: A rented machine billed for six idle hours because no watcher was armed
date: 2026-10-03
severity: warn
class: compute
trigger: [nohup, detached, background job, rented machine, pod, lease, release, idle, billing, long run]
paths: [scripts/pod/**, docs/labs/compute.md, docs/harness/skills/cost-and-compute-admission/SKILL.md]
milestones: [m1, m2, m3, m4]
mechanism: ""
guard: none: the harness cannot see a job launched outside it; the habit is the guard
evidence: [docs/labs/compute.md, docs/labs/operations.md, AGENTS.md]
status: active
---

## What happened

On 2026-10-03 a rented Hetzner server finished the 8M transformer control at 08:38 UTC and then
sat idle for about six hours while still billing. Two laptop runs also finished without anyone
being told. The owner had to ask whether the server was done.

## Symptom

A machine is still running after its job ended. Cost accrues with nothing running. The owner
learns a result is ready only by asking for it.

## Lesson

Every detached (`nohup`) or remote job gets a completion watcher in the same breath as the
launch: a background `until <done-marker or exit line>; do sleep; done` that reports the
outcome. Deleting the pod is the only thing that stops the bill, so the watcher's job is to
make the release immediate.

Never promise a check-in time without a watcher behind it.

## How to apply

1. Launch with a visible completion condition: a final log line, a written result file, a
   `DONE` marker.
2. In the same command block, arm the watcher and note its job id.
3. When the watcher fires: report the result, release the lease, then `uor-pod down` if no other
   session holds it. `python3 scripts/harness/cli.py budget scan` shows live pods and cost.
4. A self-set budget running out is never a reason to kill a healthy run (AGENTS.md); extend
   and renew instead, but if the job *finished*, release at once.
