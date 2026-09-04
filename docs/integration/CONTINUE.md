# Continue one UOR-R4 task

Use the prompt below for one later repository task. Live GitHub and refreshed
`origin/main` decide eligibility.

The project is in `build_first_pre_alpha` mode. The active priority is #973:
produce prompt-dependent source-free text through the actual repository CLI or
local service. #1107 remains historically source-frozen and unbuilt; #1084
workbench qualification is not the current priority. #954 remains blocked by
#973.

The latest bounded #973 implementation makes the retained softmax consume its
current relative H4 address through a versioned per-layer/head/address logit
bias. Its first 128-update fit changed neither of the two direct continuations
and moved the last-16 mean training loss by less than `0.00005` nat, so it does
not advance to a longer fit. Artifact-resident persistent memory is not the
next step. The next task is to expose the already-trained ordinary
causal-softmax control through the same artifact-only generator and run the
same two prompts without fitting. If that control is also incoherent, stop
attention micro-edits and correct the compact language recipe or capacity; if
it is materially more prompt-responsive, continue at the retained read seam.

```text
$uor-project-workflow

Continue UOR-Foundation/uor-r4 from refreshed origin/main and the live GitHub
issue graph. Read AGENTS.md and docs/integration/current-state.md. Complete one
active implementation task and stop.

Apply build_first_pre_alpha. Use an isolated full Git worktree. Implement the
actual feature, compile it, and run the smallest command that directly exercises
its behavior. Agents may build, test, train, evaluate, and run repository CLIs,
models, services, and browser flows when the task needs them. Do not require a
test merely because code changed.

Do not create frozen experiment contracts, supervisor or watchdog programs,
receipt chains, independent-review tasks, replay packages, claim-ledger updates,
knowledge-index maintenance, source audits, duplicate roadmap updates, formal
proof work, publication work, NEMESIS/W33 mapping, or broad QA unless I
explicitly request it.

Work on one task. Automatic retries are zero. After a concrete failure, use the
existing output to make one direct source or input correction and rerun once
only when that change plausibly resolves the failure; otherwise report the
blocker. Any run expected to exceed 15 minutes, create more than 10 GiB, or
incur external cost needs explicit authorization, a hard limit, and a stop
condition.

Preserve user material, unique artifacts, and prior negative results. Keep
mathematical proof, measured behavior, and hypotheses distinct without building
an evidence dossier. Deliver through a protected pull request. Report the
working behavior, actual command result, remaining limitation, closure state,
and one next action.
```
