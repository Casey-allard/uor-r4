# Contributing to UOR-R4

Welcome. UOR-R4 is an open research project (goal and status in the [README](README.md)).
Contributors are human researchers, AI labs (Claude, Codex, DeepSeek) and anyone who
follows the procedure below. This is the current, owner-approved workflow; [AGENTS.md](AGENTS.md)
is the full rule set and wins on any conflict.

## Principles

- `origin/main` is the single source of truth. A branch or PR is a temporary delivery step,
  not a research authority and not completion.
- Honest evidence over impressive claims. State what was measured, on what, with what budget.
  Keep negative results; they land as archive patches, not as standing branches.
- Preserve unique source, artifacts and other contributors' active work.

## For humans: start here

1. Read the [README](README.md), [STATUS.md](STATUS.md), then [docs/labs/README.md](docs/labs/README.md)
   and the [project map](docs/PROJECT_MAP.md). Do not repeat a broad historical audit.
2. Build and test (Rust 1.97.1 is pinned and selected by rustup):
   ```sh
   cargo build --release -p uor-r4-training --example geometric-stack
   cargo test -p uor-r4-training --lib geometric_stack
   cargo fmt --check
   ```
   Run focused tests for the path you change; there is no blanket full-suite requirement.
3. Good first areas: milestone M6 (exact arithmetic in the stack, issue #2034), M5 (a laptop
   energy and RAM measurement plan, #2033), M7 (API and WASM for the stack, #2035),
   documentation and claim-wording fixes, and reading a negative result in
   [docs/integration/current-state.md](docs/integration/current-state.md) to find a cause.
4. Ask on the milestone issue before starting; comment what you will do.

## The workflow (every lab, agent and human)

1. **Refresh.** `git fetch origin`; read the milestone issue, the tracker
   [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028) and recent PRs.
2. **Claim.** On the milestone issue (#2029 to #2036): post a claim comment, add the label
   `lab:claude`, `lab:codex` or `lab:deepseek` (humans: a comment suffices), and set
   `status:in-progress`. Work ownership is a renewable claim, not a monopoly.
3. **Report results** as short comments on the milestone issue: the numbers, the scope and the
   merged PR. The tracker carries only milestone-level decisions. No heartbeats or work logs on
   the tracker or milestones; leases go to the compute board
   [#2037](https://github.com/UOR-Foundation/uor-r4/issues/2037), coordination to `codex/lab-state`.
4. **Branch.** A short-lived branch (or worktree) per deliverable. No long-lived branches.
   The only standing branch is `codex/lab-state`.
5. **Check.** Compile and exercise the changed Rust path with focused tests (arithmetic,
   causality, serialization, interfaces). Run actual generated behavior for a model change.
   Run `python3 scripts/check_claim_wording.py` when editing capability claims. Do not treat
   absent fixtures, unrun tests or the CI acknowledgement jobs as a pass: the `required-transport`
   job executes no tests.
6. **Open a protected PR.** Stage named paths only. Use `References #N`, not `Closes #N`, for
   partial work. Post the checks you ran, at the exact head, on the PR.
7. **Review.** An exact-head review (self-review is allowed) plus passing tests at that head.
8. **Merge** through the merge queue. Never direct-push `main`, bypass protection, fabricate
   checks or use admin merge.
9. **Verify** on fresh `origin/main` that the merge commit and the delivered source match.
   Completed, validated work must be merged into `main`; opening a PR is not completion.
10. **Clean up.** Delete the branch and worktree, then run `scripts/storage/uor-hygiene --apply`.
    Do not start the dependent successor while delivery is unfinished.
11. **Record** outcome, retained artifacts, limitations and next action on the owning issue.
    Close an issue only when its complete acceptance is met.

## Data, artifacts and compute

- Results and large artifacts go to iCloud with `cloud-store put <lab> <dir>` (MD5 round-trip),
  then the local copy is trashed; `cloud-store fetch` restores. Local-only data lives in
  gitignored paths. Keep 30 to 70 GB of disk free. Never commit models or corpora.
- GPU work only through `scripts/pod/uor-pod`: run `uor-pod status` first, pass `--lab` and
  `--session`, lease the GPUs you use, renew rather than stop a healthy run, release when done.
  Caps for all labs together: at most 4 running pods and $8 per hour. Another session's GPU is
  never yours. Never fall back to the laptop CPU for GPU work. Leases are tracked on
  [#2037](https://github.com/UOR-Foundation/uor-r4/issues/2037); details in
  [docs/labs/compute.md](docs/labs/compute.md).
- Never read or print API keys. No new paid or external compute class without the owner.

## Claim wording and evidence

- Follow [docs/formal_vocabulary.md](docs/formal_vocabulary.md): distinguish proof, measured
  behavior and hypothesis. Write "measured X on Y", not "X works".
- No frontier, general-prose, general-reasoning or energy-savings claims. The model is pre-alpha.
- Keep open development evaluation separate from final held-out evaluation. Preserve prior
  results at their exact artifact, data, operator, control and budget.
- Reject a repeat experiment without new causal evidence and a decision it can change
  ([D9](docs/integration/DECISIONS.md#d9--prevent-experiment-loops-and-preserve-the-context-contract)).

## For AI agents

Read [AGENTS.md](AGENTS.md) first (the top section and "Resources, verification and delivery").
Register and claim before mutating; heartbeat per the lab protocol ([docs/labs/protocol.md](docs/labs/protocol.md));
kill only your own PIDs; never delete another session's files or leases; use Rust for all model
code (no Python model implementation or dependency); report complete cost, including preparation
and retries.

## Proposing a change to this workflow

Open an issue labelled `workflow` describing the change and why it helps the project goal.
The owner (or a council under D14) decides. Accepted changes land through a PR to AGENTS.md or
this file, like any other change.

## License

By contributing you agree your contribution is licensed under the [MIT license](LICENSE).
