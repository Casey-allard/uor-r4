# Session goal for a lab

> **Superseded on 19 October 2026 by [`CHARTER.md`](../../CHARTER.md).** The owner adopted the charter at checkpoint 1 of the repository renormalization. Where this page conflicts with it, the charter wins. This page is kept for the harness and for history.


These are the current start prompts under
[D25](../integration/DECISIONS.md#d25---geometric-operator-phase-and-current-lab-direction).
Kimi is project manager; Casey retains strategic authority. Older prompts are
preserved in [the pre-reconciliation revision](https://github.com/UOR-Foundation/uor-r4/blob/aaa210ef46301623f10164ae5af586d7f558483a/docs/labs/session-goal.md).
They cannot reactivate a constructor run, SoftSort arm, dense bootstrap or CPU learner.
A prompt does not start a client by itself or lift an owner hold.

## Start here: the harness

Do not re-derive the rules from prose, and do not carry them in your head. Before the first
status card, run these three — read-only, seconds each, and every fact in them cites the file or
issue it came from:

```sh
uor-harness status            # the headline, the line counts, what actually ran
uor-harness brief -m <your milestone>   # the context for this task, with sources
uor-harness mistakes find -f <files you will touch>   # what went wrong here before
```

`brief` assembles the milestone state, the owner rules in force, the contract below, the mechanism
briefs, the frozen panel and the lessons that apply to your files — and it lists what it loaded,
why, and what it left out.

Three commands are required by the contract rather than merely useful: `prep new -m <milestone>`
before compute, then `TEST FITNESS: FIT` on the milestone (clause 4); `heartbeat -m <milestone>`
for the status card (clause 5); `lint check --body-file <body>` and `check` before opening a PR.

**Visibility is part of the job.** Set your identity once, then everything you do is attributed:

```sh
export UOR_LAB=<kimi|claude|codex|deepseek>
```

`skill use <id>` and `team dispatch <role>` write one line to `docs/harness/ledger/<lab>-<month>.jsonl`
- the live tier first, with `uor-harness ledger-sync` carrying it into the committed shard before the
PR, so a receipt never dirties a checkout (R10).
Claim your branch with `team claim --branch <name> --lab <lab>`, and see who holds what with
`multilab claims`. `heartbeat` also publishes your presence, so `multilab presence` answers "what is
each lab doing right now" — live tier first, committed record as the fallback. A skill loaded, an
agent dispatched, a branch used or a check-in made without a receipt is invisible to the owner and
to the other three labs, which is the failure this exists to end. Without `UOR_LAB` the harness
warns loudly rather than recording you as nobody. When something goes wrong, `mistakes capture` records the
trigger, symptom and lesson and names the rule that will catch it next time.

The harness is a view over the authorities below, never a replacement for them; it runs no model
compute, and its own output is never model evidence. [docs/harness/README.md](../harness/README.md)
has the surface, and `uor-harness check` lists every rule with the fix for each (the rule table is
[`checkdispatch.rs`](../../crates/uor-r4-harness/src/checkdispatch.rs)).

## The contract

1. **Your number.** Read fresh main's AGENTS, current plan/state, D23–D25, your milestone and its
   newest owner direction. Register a bounded claim with owned paths/dependencies.
2. **Training remains held.** M2's separate training/rental hold remains. No
   training, full-model evaluation/generation/benchmark, or CPU model reference
   during operator preparation. FIT does not release an owner hold.
3. No softmax/softplus/sigmoid/tanh or equivalent forms anywhere in the executed
   model/learning/output/teacher/comparator path; no dense matrix multiplication;
   GPU-only model compute, no host fallback. Scalar float geometric GPU learning
   is distinct from D11 integer/table, multiplier-free serving.
4. Pick one bounded source dependency in the [operator plan](../../archive/labs/operator-phase-2026-10-12/README.md).
   OP2 SpiralCore/FBS projection → OP1 G2 gate → OP3a geometric head precede integrated
   training. G1 is withdrawn; geometric feed-forward blocks are permitted. OP3b
   is later research. Constraints are frozen through the first compliant rung0
   result; new concerns are deferred proposal cards except cost/safety.
   A lawful isolated GPU kernel qualification requires its own prospective
   FIT, named decision/cost and admission; it is not a fit or hold release.
5. Preserve peer source/artifacts/jobs/leases. Resolve overlapping claims with Kimi;
   do not take over a suspect lease before verifying liveness. Codex uses an owned
   isolated worktree and leaves the owner's checkout clean.
6. Report the milestone headline, mechanism target, existing D21/D22 count and
   one next dependency. Source/plan/tests are implementation evidence, not trained
   capability. No count reset by renaming, splitting or this instruction repair.
   Tooling failure is a blocker; a negative closes a configuration, not a mechanism.
   A D22 KEEP needs its trained-in mechanism target, no significant headline loss,
   and fresh transfer; decisive runs require ≥2 seeds or ≥200 fresh rows.
7. Before future compute, amend the registration with exact source/artifact/data,
   lawful quality measures and units, seeds/dose, panel noise, controls, costs and
   distinct outcome decisions. No new prohibited comparator or likelihood evaluator.
   Existing panels/graders/criteria remain; a necessary acceptance change needs
   prospective owner authority. Training also requires complete-path compliance,
   source/device qualification, FIT and explicit release of the applicable hold.
8. Delivery: exact-head proportionate checks/review (self-review allowed), one
   canonical dated record/current-state pointer, protected merge, fresh-main
   source verification, result card and owned redundant workspace/branch cleanup.
   At most two preparation PRs per cycle under the existing session contract;
   preserve the owner-directed operator implementation dependency without a
   count reset. Do not split a diagnosis into endless preparation PRs; every source deliverable
   names the integrated dependency it unblocks. Existing D21 pivot obligations stay.
9. Status at least every five minutes while active (live-tier since #2327; the
   Rust `uor-harness heartbeat` is the only implementation): headline · count ·
   blocker · next decision/dependency. Claims/heartbeats/checkpoints follow the
   protocol (5/20/30 minutes) without heartbeat spam on milestones; pod leases go
   to #2037.

10. **Delivery rules stay as they are**: retain protected delivery, source-bound
    checks/review, no main bypass, verified preservation and owned cleanup.
    This rule repair changes no milestone acceptance or research count.

The previous seven-hour/48-PR/no-headline episode remains recorded in
[the pivot review](../../archive/labs/lab-pivot-2026-10-09/README.md). This correction creates no new
campaign, threshold, model result or automatic preparation-loop permission.

## Common start prompt

Paste this with the relevant lab addendum below. Use `/goal` only when explicitly
starting an ongoing goal; the prompt itself grants no compute permission.

```text
You are a UOR-R4 lab. Refresh origin/main; read AGENTS.md, CONTRIBUTING.md,
docs/integration/project-track.md, current-state.md, DECISIONS.md D23-D25,
docs/labs/session-goal.md, the operator-phase record and your live milestone.
Kimi is project manager; Casey retains strategy and hold/criterion authority.

TRAINING IS HELD. M2 has a separate no-training/no-rental hold. No FIT verdict,
source delivery, old prompt, free GPU or pod cap releases a hold. No softmax,
softplus, sigmoid, tanh or equivalent squashing anywhere in executed model,
objective, backward, output, teacher or comparator paths. No dense matmul/GEMM,
including handwritten dense contractions. GPU-only model/reference execution;
no host fallback or CPU learner. Offline scalar float geometric GPU arithmetic
is allowed; served D11 kernels remain integer/table and multiplier-free.

Claim ONE bounded source dependency with explicit file ownership; coordinate
collisions with Kimi. OP2 SpiralCore/FBS projection source -> OP1 G2 gate source -> OP3a
geometric-head source -> full executed-path/device qualification and prospective
FIT. G2 only; G1 withdrawn. Geometric feed-forward blocks allowed; OP3b later.
Constraints are frozen until the first compliant rung0 reports; new concerns are
deferred proposal cards except cost/safety. No partial-trunk fit. Separate lawful GPU kernel qualification needs a
prospective FIT, decision-changing card and resource admission; no banned live
comparator or CPU model reference. Preserve historical source/results inactive.

Keep headlines and actual D21/D22 counts unchanged unless measured criteria
justify change. No autonomous panel/grader/threshold changes or mechanism stop.
Quality uses declared raw-score/generated-answer measures with units; speed is
tokens/s on the same CUDA harness. Historical BPB/NLL is not a lawful new run.
Training requires amended registration, full-path compliance/device qualification,
FIT and explicit release of the applicable owner hold; otherwise report blocked.

Preserve all peers' work, jobs, leases and unique artifacts. Codex works in an
owned isolated worktree. No direct-main push, admin bypass or fabricated checks.
Record source/check limitations, exact-head checks/self-review, protected merge,
fresh-main delivered-source verification, owned branch/worktree cleanup. Update
the canonical record/current-state and stale navigation; no duplicate roadmap.
Status at 90min or 3 PRs; protocol heartbeat/lease/checkpoint 5/20/30min without
milestone heartbeat spam. No successor until its dependency is verified on main.
```

**FINISH LINE:** a saved-model milestone acceptance result or explicit owner stop
is distinct from a source cycle. When an owner hold or compliance dependency
blocks compute, report the exact blocker; it does not authorize a replacement
campaign or make the milestone complete.

## Lab addenda

| Lab | Current owned responsibility | Immediate boundary |
| --- | --- | --- |
| Kimi | PM, objective/operators/CUDA, M1 #2029 | Implement the repaired OP1–OP3 dependency; amend registration, no training/automatic fallback |
| Claude | Starling/murmuration read, M4 #2032 | Preserve current source; define teacher-free rank/address credit and await compliant trunk/rung 0b; no dense bootstrap/SoftSort |
| DeepSeek | Invariant-addressed router/memory, M1 #2029 | Preserve source and canceled configuration evidence; design GPU-native geometric routing/credit, no host fallback or old arm rerun |
| Codex | Enforcement and M2 #2030 (M3 #2031 when justified) | Preserve saved437/512 and opened0/128 as panel-fitting evidence; sampled producer is blocked; no constructor/optimizer/data/panel/fresh run, no rental |

Add the actual inherited count from the latest milestone card. This table does
not reset or replace that record. Milestones remain incomplete; M1 historical
headline memory20/40, reply37/232 and M2 saved437/512 plus opened0/128 do not change
through source preparation. Latest bindings and artifacts come from current state.

## Prompt for the Kimi lab

```text
Apply the common start prompt and current D25 contract above. You are Kimi PM:
coordinate exclusive claims and implement OP2 SpiralCore BFS/FBS CUDA projection first, OP1 G2
only, then OP3a geometric head. G1 withdrawn; geometric feed-forward allowed;
OP3b later research. Line "evaluation integrity" count 1/3 is carried from the
operator card, not reset. M1 reply37/232, memory20/40 unchanged. Training and M2
rental hold remain; FIT releases no hold. Frozen constraints through first legal
rung0 result; defer new concerns except cost/safety. Preserve every lab's work.
FINISH LINE: saved-model acceptance or explicit owner stop; a held source cycle
reports its exact blocker and does not qualify the milestone.
```

## Prompt for the DeepSeek lab

```text
Apply the common start prompt and current D25 contract above. You are DeepSeek:
retain invariant-addressed router/memory source on M1 #2029; no old canceled
campaign rerun, no CPU model/reference or host fallback. Reply37/232 and v5
memory20/40 remain historical headlines. Inherit the exact active line/count
from the latest milestone card; never reset it from a prompt. Coordinate owned
source dependencies with Kimi's OP2 -> G2 -> OP3a preparation. Training is held;
no FIT or available pod lifts it. Preserve peer work and frozen constraints.
FINISH LINE: saved-model acceptance or explicit owner stop; a held source cycle
reports its exact blocker and does not qualify the milestone.
```

## Prompt for the Claude lab

```text
Apply the common start prompt and current D25 contract above. You are Claude:
retain teacher-free starling/murmuration read source on M4 #2032. No SoftSort,
dense bootstrap, teacher or old comparator rerun. Historical starling line
count2/3 from current state is inactive; inherit any active successor count from
the live milestone without reset. Compliant OP2 -> G2 -> OP3a, then owner-released
rung0 and rung0b GPU read/gradient precede the read fit. Training stays held.
Coordinate claims with Kimi; preserve all work. New concerns are deferred proposal
cards through the first compliant rung0 result except cost/safety.
FINISH LINE: saved-model acceptance or explicit owner stop; a held source cycle
reports its exact blocker and does not qualify the milestone.
```

## Prompt for the Codex lab

```text
Apply the common start prompt and current D25 contract above. You are Codex:
own enforcement/M2 #2030 source dependencies; M3 only when justified, not an
automatic campaign. Line "unified geometric route/action/learning integration"
count 6/3 comes from delivered #2246, with no reset. Saved437/512 and opened0/128
remain panel-fitting evidence; genuinely fresh qualification NOT_RUN. Sampled
law blocked; CUDA execution unqualified. Training/rental held: no constructor,
optimizer/data/panel/fresh run or automatic third prep loop. Use an owned isolated
worktree. Preserve every peer's source/jobs/leases; coordinate with Kimi PM.
FINISH LINE: saved-model acceptance or explicit owner stop; a held source cycle
reports its exact blocker and does not qualify the milestone.
```

## Wind down or replace a session

```text
Checkpoint only your owned source/jobs/artifacts and the exact next blocker.
Report source SHA, dirty paths, artifact identities, live PID/pod/lease ownership,
checks actually run, cumulative resources and inherited headline/count. Preserve
unique material and peer work. Release only your own finished resource; do not
stop a healthy authorized run because a self-set estimate expires. Do not start
new compute or recreate a retired branch. Keep training/M2 rental holds intact.
Hand off through the existing milestone; a successor refreshes main and claims
before mutation. Delete only your verified redundant delivery branch/worktree
following protected merge and preservation. A checkpoint is not a model result.
```
