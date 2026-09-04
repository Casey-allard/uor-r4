# Deterministic source-only agent execution

This is the repository-wide default for automated agents. Its machine-readable
contract is [agent-execution-policy.json](agent-execution-policy.json), and the
binding instructions are embedded in [AGENTS.md](../../AGENTS.md).

## The failure this prevents

An agent must not respond to a build-environment failure by constructing a
partial workspace, adding diagnostic probes, wrapping commands in supervisors,
or retrying variants. A partial copy can omit an explicitly declared Cargo
target while still looking like a plausible source checkout. Repeated attempts
then spend time and tokens without increasing evidence about the product
decision.

Every automated change now starts from refreshed `origin/main` in a complete
Git worktree. Agent work is limited to source inspection, declared edits,
reviewable Git operations, and the static policy guard. Sparse, pruned, or
hand-copied workspace capsules are prohibited.

## Execution boundary

Agents do not run or dispatch Cargo, Rust toolchain commands, builds, tests,
linters, benchmarks, fuzzers, model execution, fitting, evaluation, browser or
service probes, operating-system probes, or custom retry/supervisor wrappers.
Pull-request and merge-group automation performs this static policy guard and
the five ruleset transport acknowledgements only. Product and release QA stays
available solely through the owner-operated `workflow_dispatch` path already
defined in `.github/workflows/ci.yml`.

An explicit owner instruction plus a protected pull request is required to
change this policy. A task prompt, issue template, historical checklist, or
agent judgment cannot silently activate an exception.

## Failure and reporting budget

When owner-run remote QA reports a concrete source failure, an agent may make
one source correction based on that evidence. If the next owner-run result
still fails, the work is parked with the exact source blocker. The agent does
not probe the environment, invent another harness, or begin a retry campaign.
Environment-probe and automatic-retry budgets are both zero.

Agent reports contain the delivered result, the limits of source review, the
closure state, and one concrete next action. Individual build, test, probe, and
unchanged-poll narration is suppressed.

## What this establishes

The policy makes the automated development workflow bounded and predictable.
It prevents the specific runaway validation pattern described above and makes
policy drift visible in the protected path. It does not prove that research
algorithms, third-party tools, operating systems, or model outputs are
deterministic. Those remain separate product or release questions that only the
owner may choose to evaluate.

For this policy change, project builds, tests, probes, model runs, and product
evaluations are `NOT_RUN_BY_POLICY`. The only executable validation is the
standard-library static policy guard; it reads tracked text and does not import
or execute project code.
