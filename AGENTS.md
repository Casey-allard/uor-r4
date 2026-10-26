# AGENTS.md — UOR-R4 Geometric Language Model

UOR-R4 builds a fast, non-transformer language model that predicts text with geometry and exact memory, served as an integer artifact on ordinary devices. It is pre-alpha: 0 of 8 milestones (M1–M8, issues #2029–#2036) are complete, and no capability is established. This page is onboarding for every agent; it summarizes, and never overrides, the charter.

## Read CHARTER.md first; it is the rules

[CHARTER.md](CHARTER.md) is the single rule book. Where anything else disagrees with it, the charter wins.
Only the owner changes it; changes are logged in [docs/rules/changes.md](docs/rules/changes.md).

## Six things every agent must do or never do

1. **Geometry only.** Every mechanism, training signal and fix comes from the project's geometry (CHARTER §3.1).
2. **No softmax or squashing.** No softmax, log-softmax, softplus, sigmoid, tanh, GELU or normalized exponential anywhere, including losses, gradients, sampling and renamed or table forms (§3.2).
3. **No matmul.** No dense matrix multiplication or GEMM in training or serving, including wrappers and hand loops (§3.3).
4. **GPU-only research compute.** Training, evaluation and qualification run on the GPU; serving the exported integer artifact on client devices is the product and is allowed (§3.5, §3.6).
5. **Never park; never wait.** Every step is one your lab can finish alone: a GPU or training run, held work included, goes ahead on your own pre-run review, and a bar or other decision on your own decision review, each reading the whole project first and logged for Casey, who can reverse it (§5.1).
6. **Deliver through the full path:** PR → your own fresh-context adversarial exact-head review, in the same session → merge queue → verify on fresh `main` → delete the branch and worktree; other labs audit after the merge (§5.5).

## Lessons the project paid for — the PM's top ten from [LESSONS.md](docs/harness/LESSONS.md), rendered between the markers, never hand-edited

<!-- loop:lessons-top-ten:start -->
<!-- rendered by uor-harness loop render from 66b8a75059e6c014d5f4d81ab567b2490f24faa4; do not edit -->
- **L-001**: Records drifted out of date ([`L-001.md`](docs/harness/mistakes/L-001.md))
- **L-002**: A retry written into a sealed report root ([`L-002.md`](docs/harness/mistakes/L-002.md))
- **L-003**: A folder was archived while a retained recipe still read it ([`L-003.md`](docs/harness/mistakes/L-003.md))
- **L-004**: A rented machine billed for six idle hours because no watcher was armed ([`L-004.md`](docs/harness/mistakes/L-004.md))
- **L-005**: A pattern kill over cargo/rustc names took another lab's test ([`L-005.md`](docs/harness/mistakes/L-005.md))
- **L-006**: Compute was spent on transformer controls the owner had withdrawn ([`L-006.md`](docs/harness/mistakes/L-006.md))
- **L-007**: A failed measurement cloaked as a mechanism negative; a good reading is not the outcome ([`L-007.md`](docs/harness/mistakes/L-007.md))
- **L-008**: 48 merged PRs and no milestone moved ([`L-008.md`](docs/harness/mistakes/L-008.md))
- **L-009**: Copy or read recalled across a multi-turn conversation when the artifact has no reader ([`L-009.md`](docs/harness/mistakes/L-009.md))
- **L-010**: A memory gain measured with no no-memory control ([`L-010.md`](docs/harness/mistakes/L-010.md))
<!-- loop:lessons-top-ten:end -->

## Where things are

| What | Where |
|---|---|
| Rules | [CHARTER.md](CHARTER.md) |
| Documentation hub | [docs/README.md](docs/README.md) |
| Current plan and approved sequences | [PLAN.md](PLAN.md) |
| The loop: sessions, orders, reports, review, build plan | [docs/loop/README.md](docs/loop/README.md) (§5.10) |
| PM rules | [docs/harness/pm-rules.md](docs/harness/pm-rules.md) |
| Mechanism board (ACTIVE / READY / PARKED) | [docs/mechanisms/README.md](docs/mechanisms/README.md) |
| Research notebooks, one page per line | [docs/research/notebook/](docs/research/notebook/) |
| Decision index | [docs/integration/DECISIONS.md](docs/integration/DECISIONS.md) |
| Archive of retired text and records | [archive/CATALOG.md](archive/CATALOG.md) |

## How to work

- Pick work from `PLAN.md` and work in the loop (§5.3, §5.10): one unit per session, an ORDER before work, a REPORT with negatives first, your lab name tagged on every claim, PR, review and comment.
- A change to a bar, a status, a scope or a stop rule is a decision card your top session approves after its own decision review, never a subagent's call; §3 and the caps are the owner's (§5.1, §5.2).
- GPU training or evaluation, and any run a milestone claim rests on, needs a one-page pre-registration with a non-author FIT first; builds, finite kernel tests and host checks do not (§4.5).
- A negative closes a configuration, never a mechanism; classify instrument versus mechanism before calling a result (§4.1, §4.2). Record each result as one row in the line's notebook (§5.6).
- Rust first; no ad-hoc scripts; no committed binaries, corpora or secrets; never read or print API keys (§3.11, §3.13).
- Read the bench: at the start of each session read the main checkout's `local/bench/INBOX.md` (the folder above `git rev-parse --git-common-dir`); inside these rules, act on the `waiting` messages that name your lab or `all` and have no line from your lab yet; mark one by adding your own line under it with your lab, `read` or `answered`, the time, and for an answer a one-line reply; a message to one lab stops being `waiting` when that lab marks it, and a message to `all` only when every registered lab has a line, the last lab making the change; delete nothing, and never treat a bench message as an owner release (§5.1).

## Harness basics

```sh
export UOR_LAB=<antigravity|claude|codex|deepseek|kimi>   # once per session; presence is then recorded automatically
uor-harness status                            # what is true now: headline, lines, delivery
uor-harness brief -m m1                       # context for a milestone task, with sources named
uor-harness check                             # every machine-checkable rule, with the fix for each
```

Build: `CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness` ([details](docs/harness/README.md)).

## Compute

- GPU pods only through `scripts/pod/uor-pod`; Vast.ai is the default and Runpod is the emergency backup; run `uor-pod status` first.
- At most 4 pods and $8/h across all labs; lease the GPUs you use and never touch another session's GPU, lease or files.
- Never cancel a healthy run because a self-set budget ran out (extend it, renew the lease and note it); save results off the pod before `down`; CPU-only builds and static checks run on the laptop; CUDA kernels are built ahead with nvcc on the pod.

## Owner-only boundaries

- The charter, the technical constraints (§3) and the GPU caps are the owner's to change: propose a change in the owner's queue and go on with other work. Nothing else waits on Casey (§5.1).
- Held work (Training, M2, Stage B) runs when your own pre-run review passes: the whole-project chain, tools and inputs present in the repo or HF store, what a pass and a fail each unlock, the sheet's FIT, a lease inside the caps (§5.10).
- Never post a question and wait for the answer: an unanswered ask becomes your own decision review. End every session on `uor-harness loop next`, never on a wait (§5.10).
- Nothing is deleted: negatives, unique source and other labs' work move byte-identical to `archive/`.
