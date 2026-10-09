# Shared autonomous lab protocol

> **Owner direction, October 9 — main consolidation and workspace cleanup:**
> `origin/main` is the shared source of truth for every participant. Finish and
> verify the protected merge of each deliverable before starting its successor,
> then clear its redundant local workspace and delivery branches. Branches are
> temporary transport for protected delivery, not alternate research authorities.
> A separate branch or isolated worktree is not mandatory for development; use
> one only where concurrent ownership or protected delivery requires it. Preserve
> unique source, accepted and negative evidence, and other labs’ active work
> before cleanup. Reconcile older unmerged work into main with explicit retained,
> superseded, unfinished or validated status; do not promote it by merging it.

> **Current owner direction, October 1 — D19:** grounded conversation and durable
> memory lead the active canonical plan. Codex rejoins Claude and OpenCode–DeepSeek
> for the owner-authorized continuation; other historical lab assignments are
> not reactivated. The owner's newer exact-head review rule (self-review allowed),
> actual scoped compile/tests and protected merges supersede conflicting older
> council/coordinator delivery prerequisites below. The delivery-evidence check
> is advisory; use the documented manual fallback when deployment is unverified.
> Scientific integrity, D11/D5, unique-material preservation, live job ownership
> and resource admission remain. See [D19](../integration/DECISIONS.md#d19--grounded-conversation-and-durable-memory-first).

Authority: owner-adopted [D14](../integration/DECISIONS.md#d14--durable-autonomous-labs-and-correctable-governance).
This is the common operating contract for every client. Its required behavior is
distinct from whether a particular CLI, daemon or GitHub check has been deployed
and tested. See [operations](operations.md) for the rollout gate and manual
fallback. Historical provider-specific charters are superseded operationally.

## GitHub authority and coordination

Accepted source, decisions and research knowledge live on protected `main`.
Issues/PRs carry proposals and live activity; sealed evidence manifests bind
actual artifacts. The `codex/lab-state` branch managed by `lab-runner coord`
contains live lab/work claims and heartbeat state, separate from scientific
source. Updates use atomic Git ref comparison: a conflicting writer rereads and
reapplies its event; it never force-overwrites the newer state. Local indexes,
chat summaries and `STATUS.md` are navigation/views, not independent authorities.

Use tracker #2028 for programme-level changes and incidents; the milestone issues
#2029 to #2036 for claims and results; the compute board #2037 for pod leases; and an
existing issue for each concrete deliverable where its scope fits. A milestone issue owns a
short current pointer, not copied experiment history. A completion record links
its PR/result and names the next dependency. Edit README only when its overview,
qualified capability or entry links change.

At join/resume, identify lab/session/client, source SHA, milestone claim, worktree,
available tools/models, strengths, quota limitations and observed adapter
capability. Register before claiming work. A lab is not an individual provider:
it may use cheap specialist subagents or another available model without
changing the shared evidence contract or buying service.

For every claim record issue, dependency IDs, branch/worktree, owned paths,
artifact inputs, deliverable, next checkpoint, reviewer need and host reservation.
Only one active claim owns a given mutating task/overlapping path set. Resolve
collisions through the coordination record; independent readers need no write claim.
Start from refreshed main and respect exclusive file ownership. A separate
worktree is optional. Use a temporary branch when protected delivery requires
it; name it `<lab>/<topic>` (Codex defaults to `codex/`). All work, including
negative, superseded and unfinished retained source, lands in main with explicit
status. Verify the actual merge, preserve unique material, then delete redundant
branches and workspaces before starting the next task. The live `codex/lab-state`
coordination record is the standing operational exception.

## Work loop, leases and recovery

1. Read the current roadmap, your milestone issue, relevant peers' latest activity, open
   PRs, claims and jobs. Recover prior source/artifacts before starting anew.
2. Select the highest-value ready dependency. Prefer an integrated capability
   or a measurement that changes a decision; use existing tools and source.
3. Claim it and post one short work card: goal, observed blocker, hypothesis,
   causal change, controls, outcomes, fixed conditions, necessary checks and
   complete resource projection.
4. Investigate, implement and use a cheap smoke when it resolves a named risk.
   Submit heavy work through the admitted durable runner, then advance an
   independent task. Do not launch a duplicate worker from a resumed session.
5. Inspect actual results and generated behavior; seal and verify reports;
   preserve failed/inconclusive attempts; publish source/results and request
   non-author review. A merged negative is useful delivery without promotion.
6. Drive the approved head through protected delivery, verify the delivered
   patch, update the owning issue and changed current-state pointers, release
   the claim, remove redundant delivery branches and workspace after preservation,
   and only then take the next ready task.

While actively working, refresh the lab/work heartbeat every **5 minutes**.
Claims have a **20-minute** lease; publish a recoverable checkpoint at least
every **30 minutes** and immediately before compaction, quota exhaustion,
disconnect, migration or deliberate departure. These are coordination cadences,
not experimental time limits. A checkpoint names source SHA and dirty work,
artifact hashes/locations, live job IDs, exact command/config, ledger state,
blocked decisions, next action and how to verify recovery. Push meaningful
source checkpoints so they survive a local session, without representing drafts
as reviewed or promoted.

An expired lease makes a claim **suspect**, not automatically abandoned. Inspect
the runner, process identity, branch, unpushed state and artifact writes. Renew
or adopt a verified running job without restarting it. If the prior lab is
offline and no job is live, record takeover and acquire a new claim; preserve
its branch/outputs and work from that handoff. If liveness is uncertain, keep
the resource reserved and choose independent work. Never delete, kill or reuse
an output root because an agent stopped sending messages. Split a blocked task
at an agreed interface when possible rather than inventing a parallel engine.

## Socratic research and the council

Trace each consequential task through mission → current bottleneck → artifact
and causal inputs → mechanism/history → source callers → experiment → downstream
integration. Ask: what must be true; what information exists at the decision
point; what was tried before; what simpler account explains the result; what
would refute each explanation; what useful ability follows; what does it cost?
Stop context expansion when remaining unknowns cannot change this decision.

Use on-demand experts with explicit packets and owned files: mathematics and
information, learning and optimization, systems/cost, data/evidence, and domain
research when it supplies a testable mechanism. Read primary papers and actual
source; use Wolfram or other available exact tools for specific derivations and
record inputs/outputs. Symbolic checks do not qualify finite-precision runtime
or empirical language. Reuse prior audits rather than repeating a broad one.
Provider diversity may reveal assumptions but is not evidence by itself.

A consequential architecture, shared-policy, promotion or constraint-change
decision uses a **three-seat council**, with **at least two non-author seats**.
The third may be the proposer. A decision needs **two of three recorded votes**,
identified participants/launchers, exact proposal revision, evidence and explicit
disposition of objections. Distinct independent passes may fill seats when few
labs are online; no author may manufacture a non-author review by changing a
label. No permanent director or specific provider has a veto. Scientific claims
still require evidence: a vote cannot make a false metric true or cure a missing
control. An unresolved factual objection calls for a bounded discriminator or
reversible provisional choice with the limitation recorded.

Under D14 the council may prospectively change shared methods, schedules,
interfaces, working constraints, evaluation designs and ordinary research
priorities. Record old/new rule, reason, alternatives, affected work, budgets,
review, effective revision, rollback and next test in a protected decision PR.
Lab-local tool tactics within that contract need only a work-card rationale.
Never retroactively weaken an executed gate or suppress a negative. Preserve
the original outcome and run a materially new question under a new version.
Mission/D11 target, honest evidence, preservation of unique material, and
paid/external spending remain owner boundaries. Existing necessary local budget
extensions remain allowed with prospective accounting.

## Experiments and promotion

Keep representation, access/admission, ranking, update, copy/emission and serving
arithmetic distinct. Include exact identity/version/occurrence and orientation
where needed. Prime hashes are identifiers, not a learned semantic metric;
finite summaries and compressed codebooks have information limits. Count all
stored bytes and selected or scanned parameters. A smaller file is not proof
of retained knowledge, and a multiplier-free kernel is not proof of lower
energy. Physical efficiency claims require actual comparable useful work.

Use staged measured parity: a small source-bound smoke verifies the seam; a
development comparison estimates numerical and behavioral differences; a
candidate gate compares paired arms at the relevant cost/capacity; a separate
sealed evaluation qualifies the selected artifact. Freeze each stage's data,
units, seeds, tolerance and outcome branches before running it. The inherited
geometric retention defaults are **0.02 nats or 0.03 accuracy**, paired arms and
at least two seeds for a promotion comparison. A narrower existing one-seed
result keeps its scope; do not rewrite its past verdict. Any revised tolerance
is prospective and justified by measured effects and consumer requirements.

A miss prevents promotion at that scope; it does not refute a family. Preserve
toolbox mechanisms. New compute after a negative needs a causal change, an
observable prediction and a decision it can change; another seed/dose alone is
not that change. Inconclusive instrumentation is repaired only if it blocks a
useful deliverable. Stop checks when the declared risks are resolved. No generic
proof campaign, full test suite or framework rewrite is a default prerequisite.

Reports bind source, executable, tokenizer, data/splits, configuration, model,
geometry tables, seeds, controls and cost. Use `claim → seal → verify` for
exclusive report roots; retries receive fresh roots. Keep status explicit:
`IMPLEMENTED`, `NOT_RUN`, `UNAVAILABLE`, `EXECUTED`, `REVIEWED`, `MERGED`, and
`PROMOTED` describe different facts. Missing fixtures/keys fail the intended
check or become unavailable, never an automatic pass. Preserve raw outputs and
row-level regressions against the immediately preceding best artifact.

## Review and delivery

| Class | Required evidence before protected delivery |
|---|---|
| A: ordinary code/docs | Named focused checks for the changed risk; one non-author review of the exact head |
| B: measured result or promotion | A plus sealed evidence and independent reading of headline values/manifests; rerun the gating measurement when provenance, reproducibility, magnitude or the risk warrants it; promotion separately recorded by council |
| C: shared interface, architecture or policy | A, B when numbers are claimed, and the recorded council decision; owner direction for the immutable boundaries |

The old provider-specific rule requiring every Anti-Gravity result to be rerun
by Claude is superseded by risk-based independent review. This changes no old
measurement or outstanding concrete reproducibility concern. Reuse a valid
review when its exact inputs and scope still apply; do not accumulate ceremonial
review rounds. Cheap subagents may check mechanical changes, while consequential
learning/math claims need an appropriate independent expert.

The delivery coordinator checks the exact approved head, current base, required
evidence and open blockers, then uses the protected merge queue. No direct main
push, admin bypass or force-push of shared work. A changed head invalidates its
head-bound approval; refresh the review. Historical queue compatibility statuses
execute no model checks. The future required delivery gate becomes enforceable
only after the repository administrator enables it as specified in operations.
Until then this remains a procedural coordinator gate and must be labelled so.

After merge compare the delivered squash patch to the reviewed change, record
the merge SHA and outcome, notify its GitHub consumers, and close only the scope
whose full acceptance passed. Partial delivery references an issue. A ready PR
gets a named reviewer promptly; if it is blocked, record why and who can resolve
it. Timing targets are service expectations, not reasons to invent approvals.

## Versioned authority in the coordinator

A claim supplies `policy_sha` equal to the operational state's protected policy
revision and a `work_card` of `sha256:<64 lowercase hex digits>` over retained
immutable work-card bytes. Put the content location in the checkpoint/issue;
a mutable issue URL alone is not an identity. Equivalent ownership spellings
with empty, dot, parent or trailing path components are rejected.

New claims, job admission and delivery compare the adopted revision with main.
Changes to AGENTS, the decision register, execution policy, this protocol or
the host operations contract hold
new admission until reviewed adoption; incomplete comparisons also hold.
Previously admitted bounded work keeps its reservation and may finish.

To adopt a protected policy change, checkpoint and release live task leases,
reconcile every unresolved attempt, retain the exact Class C delivery receipt,
and apply an `adopt_policy` event with `previous_policy_sha`, the new merged
`policy_sha`, and `decision_receipt` (`path`, `sha256`). The coordinator verifies
the council receipt and exact merged PR against current main before a normal
fast-forward state transition. Old claims remain historical; a resumed client
must acquire a new generation under the new revision. Never edit an old event
or force-push operational history.
