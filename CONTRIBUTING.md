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
   [#2028](https://github.com/UOR-Foundation/uor-r4/issues/2028), the newest entries of
   [current-state.md](docs/integration/current-state.md) and recent PRs.
2. **Claim.** On the milestone issue (#2029 to #2036): post a claim comment, add the label
   `lab:claude`, `lab:codex` or `lab:deepseek` (humans: a comment suffices), and set
   `status:in-progress`. Work ownership is a renewable claim, not a monopoly.
3. **Branch.** One short-lived branch (or worktree) per deliverable. There are no standing
   branches. Leases go to the compute board
   [#2037](https://github.com/UOR-Foundation/uor-r4/issues/2037) through `uor-pod`.
4. **Check.** Compile and exercise the changed Rust path with focused tests (arithmetic,
   causality, serialization, interfaces). Run actual generated behavior for a model change.
   Run `python3 scripts/check_claim_wording.py`. Do not treat absent fixtures, unrun tests or the
   CI acknowledgement jobs as a pass: the `required-transport` job executes no tests.
5. **Record and update docs in the same PR** (the delivery cadence below).
6. **Open a protected PR.** Stage named paths only. Fill in the PR checklist. Use
   `References #N`, not `Closes #N`, for partial work. Post the checks you ran at the exact head.
7. **Review.** An exact-head review (self-review is allowed) plus passing tests at that head.
8. **Merge** through the merge queue. Never direct-push `main`, bypass protection, fabricate
   checks or use admin merge.
9. **Verify** on fresh `origin/main` that the merge commit and the delivered source match.
   Completed, validated work must be merged into `main`; opening a PR is not completion.
10. **Report** one short comment on the milestone issue: the number, its scope, the merged PR.
    No heartbeats or work logs on the tracker or milestones.
11. **Clean up.** Delete the branch on GitHub and locally, remove the worktree, move used results
    to iCloud (`cloud-store put <lab> <dir>`), then run `scripts/storage/uor-hygiene --apply`.
    Do not start the next piece while this one is unmerged or uncleaned.

## Delivery cadence: every completed unit (owner, 9 October 2026)

Each PR that completes a unit of work (an experiment, a fix, a feature, a negative result)
carries its own record and its own documentation updates. Nothing is logged only in an issue,
a chat or a local file, and nothing is left for "a docs pass later".

**1. Canonical research record (always).**
- A dated record for the work: `docs/labs/<topic>-<YYYY-MM-DD>/README.md` (or `docs/evidence/…`
  for sealed evidence). State the question, the exact artifact/data/config, what was run, the
  numbers with their scope, the decision (KEEP / REJECT / NOT YET PROMOTED) and limitations.
- A new entry at the **top** of [docs/integration/current-state.md](docs/integration/current-state.md):
  `## <one-line result> — <Month D>`, two to five sentences, a link to the record, and
  `**Next:**` with the next action. Negative and inconclusive results are recorded the same way.

**2. Every document the change touches (always).** Before opening the PR, search for what the
change makes stale and fix it in the same PR:
`git grep -n -i '<feature, flag, command, crate or number you changed>' -- '*.md'`.
This includes crate READMEs, `docs/geometry.md`, `docs/labs/*`, `CONTRIBUTING.md`, `AGENTS.md`
and code comments that describe behaviour.

**3. STATUS, ROADMAP and the tracker (when state changes).** Update `STATUS.md` and `ROADMAP.md`
when a lab's measured position, a best artifact or a milestone's status changes. Edit the
milestone table in the body of #2028 at the same time.

**4. README (only when the project actually changed).** The README is the curated front page,
not a log. Update it only when something a reader relies on changed:
- a capability that works or stops working;
- a new or better headline result, a published model or dataset;
- a milestone status, a command, a crate, or a new mechanism.

Keep its existing structure and section order. Edit the matching section and table in place:
Where we are, Training, Serving, Results so far, Roadmap, Models and data or Repository map.
Never add a dated changelog. Keep claims scoped and measured, with negatives where they exist.

**5. Honest wording.** Every number names its artifact, data split and comparison. Unverified
claims are marked as unverified. A failed or retracted result stays on the record with its
correction; it is never silently replaced.

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
- Never put project data, builds or downloads in `/tmp` or `/private/tmp`; use gitignored paths
  inside your worktree (`/local/`, `target/`), so `uor-hygiene` and worktree removal reclaim them.
- GPU work (geometric-stack training, CUDA evaluation) goes through `uor-pod`. **CPU-only work runs on the owner's laptop** (Ryzen-class workstation when available): the native prose learner (`train-native-prose`), data preparation, grading and other CPU-bound jobs. Rent a pod for CPU work only when it truly exceeds the local machine (RAM, disk or wall time), and then choose the smallest pod (one GPU, or a CPU-only offering) (owner, 9 October 2026). GPU work (geometric-stack training, CUDA evaluation) still goes through `uor-pod` and never falls back to the laptop CPU.
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
