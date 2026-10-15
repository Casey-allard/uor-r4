# Contributing to UOR-R4

> **Current rules (12 October):** [D25](docs/integration/DECISIONS.md#d25---geometric-operator-phase-and-current-lab-direction). No softmax/softplus/sigmoid/tanh or equivalent squashing anywhere in the executed model path; no dense matrix multiplication in training or serving; GPU-only model compute, no host fallback. **Training remains held, including the separate M2 training/rental hold.** Source/build/document work may proceed. The owner separately authorized the current Codex SpiralCore/FBS claim for CUDA12.8+ Ubuntu24 pod development/build/finite kernel tests; this releases no training hold or unrelated campaign. Kimi coordinates the [operator phase](docs/labs/operator-phase-2026-10-12/README.md); source delivery and FIT do not release a hold. Preserve all peer work.
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

## The harness: run it, do not reconstruct it

Planning, the milestone headline, what a skill requires, what a run costs and what went wrong
before are all queryable. Do not rebuild them from prose each session, and do not carry them
in your head:

```
uor-harness status              # what is true now
uor-harness brief -m m1         # the context for this task, with sources
uor-harness prep new -m m1      # pre-register before any compute
uor-harness lint check --body-file <pr body>
uor-harness check               # every machine-checkable rule
```

Two habits keep it honest, and both are cheap:

- **Receipt the work.** `cli.py skill use <id>` and `cli.py team dispatch <role>` write one line
  to `docs/harness/ledger/`. A skill that was loaded, or an agent that was dispatched, without a
  receipt is invisible to the owner - which is the failure this exists to end. `uor-harness` writes
  the line to the **live tier** first and `uor-harness ledger-sync` carries it into the committed
  shard before the PR, so recording a skill never dirties a checkout (R10).
- **Make a lesson a guard.** When something goes wrong, `cli.py mistakes capture` records the
  trigger, symptom and lesson and names the rule that will catch it next time. A lesson with no
  guard is a note that will be forgotten again.

See [docs/harness/README.md](docs/harness/README.md) for the surface and
[docs/harness/CONTRACT.md](docs/harness/CONTRACT.md) for every rule and its rationale.
The harness is tooling: it runs no model compute, and its own output is never model evidence.

## One checkout, and no duplicated data (owner direction, 12 October 2026)

Labs work in `~/uor-r4` rather than each keeping a full worktree, so the same ~1 GB of
tracked material is not checked out five times. Before you start:

```sh
uor-harness team claim --branch <your-branch> --lab <lab>
```

The rules, the handover format and the cases where a worktree is still correct are in
[docs/labs/working-in-one-checkout.md](docs/labs/working-in-one-checkout.md). One branch
has one owner at a time; keep the tree clean between sessions; serialise `cargo` builds.
`scripts/storage/worktree-audit` reports what each workspace costs and what is safe to
remove, and `scripts/storage/uor-hygiene --apply` removes only worktrees that are clean,
free of ignored data, and already in `origin/main`.

Large binary material is **archived, not tracked**: `research/prime-analysis`,
`research/ai-research` and `research/archives` live in the MD5-verified iCloud store.
`scripts/storage/research-data status` shows what is where and
`scripts/storage/research-data restore <name> --run` brings one back in about a minute.

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
- Non-model CPU work (editing, compilation, static checks, Rust data preparation and grading stored outputs) runs locally. Every model forward, backward, evaluation and parity/reference execution is GPU-only under D24/D25. No CPU learner exception; no training or rental follows from this reconciliation.

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
