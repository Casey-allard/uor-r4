# Continue one UOR-R4 task

Use the prompt below for one later repository task. Live GitHub and refreshed
`origin/main` decide eligibility.

The project is in `build_first_pre_alpha` mode. The active priority is #973:
produce prompt-dependent source-free text through the actual repository CLI or
local service. #1107 remains historically source-frozen and unbuilt; #1084
workbench qualification is not the current priority. #954 remains blocked by
#973.

The latest #973 implementation exposes the already-trained ordinary
causal-softmax control through artifact-only CLI generation. With the same
seed-9738 top-k-40 sampler and 16-token limit, `A purple turtle found a clock in
the garden` continued `, there was a time, there was a little girl named It
found a big`; `Albert Einstein was born in` continued ` his friend, a time,
there was a little girl named he put it with`. The two outputs are
prompt-dependent and materially more grammatical than the identical broken
continuations from the contextual retained and address-aware variants. They are
still generic, and the Einstein continuation is factually and grammatically
wrong. This is measured two-prompt behavior, not a general language or factual
recall claim.

Stop modifying the group-retained reader at this checkpoint. The next task is
to load the same ordinary artifact into the existing
`R4PositionPreservingCausalKVBindingV1` and expose its `execution="r4"` path
through artifact-only generation. Keep one chronological K/V slot per token and
transport key/value content through the validated H4 frames; do not fit or add
another attention scalar. This directly tests whether the position-preserving
geometric execution retains the working ordinary prompt behavior before any
later recurrent compression or larger-capacity fit.

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
