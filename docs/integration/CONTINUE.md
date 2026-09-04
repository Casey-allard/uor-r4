# Continue one UOR-R4 task

Use the prompt below for one later repository task. Live GitHub and refreshed
`origin/main` decide eligibility.

The project is in `build_first_pre_alpha` mode. The active priority is #973:
produce prompt-dependent source-free text through the actual repository CLI or
local service. #1107 remains historically source-frozen and unbuilt; #1084
workbench qualification is not the current priority. #954 remains blocked by
#973.

The latest #973 implementation loads the already-trained ordinary artifact into
`R4PositionPreservingCausalKVBindingV1` and exposes its `execution="r4"` path
through artifact-only CLI generation. It keeps one chronological K/V slot per
observed token and transports key/value content through the validated H4
frames. With the same seed-9738 top-k-40 sampler and 16-token limit, both fixed
prompts produced the exact token trajectories already recorded for the ordinary
control: `A purple turtle found a clock in the garden` continued `, there was a
time, there was a little girl named It found a big`; `Albert Einstein was born
in` continued ` his friend, a time, there was a little girl named he put it
with`. The R4 executions made zero provider, teacher, future, or forbidden
reads and did not load the invalid historical position-K/V fit.

This is measured two-prompt trajectory preservation through the full
position-preserving cache. It does not establish exact logit equivalence,
general language or factual quality, geometric advantage, recurrent
compression, or transformerless serving. The outputs remain generic, and the
Einstein continuation remains factually and grammatically wrong.

Stop modifying the exact-cache adapter at this checkpoint. The next task is to
implement one versioned fixed-size recurrent R4 cache path, with the frozen
ordinary artifact, sampler, and exact-cache generator held as its reference.
Fold displaced chronological content into bounded H4-addressed recurrent state
and directly compare the same two prompt trajectories before considering any
fit, attention scalar, or broader evaluation.

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
