# AGENTS.md — UOR-R4 Geometric Language Model

UOR-R4 builds a fast, non-transformer language model that predicts text with geometry and exact memory, served as an integer artifact on ordinary devices.
It is pre-alpha: 0 of 8 milestones (M1–M8, issues #2029–#2036) are complete, and no capability is established.
This page is onboarding for every agent; it summarizes, and never overrides, the charter.

## Read CHARTER.md first; it is the rules

[CHARTER.md](CHARTER.md) is the single rule book. Where anything else disagrees with it, the charter wins.
Only the owner changes it; changes are logged in [docs/rules/changes.md](docs/rules/changes.md).

## Six things every agent must do or never do

1. **Geometry only.** Every mechanism, training signal and fix comes from the project's geometry (CHARTER §3.1).
2. **No softmax or squashing.** No softmax, log-softmax, softplus, sigmoid, tanh, GELU or normalized exponential anywhere, including losses, gradients, sampling and renamed or table forms (§3.2).
3. **No matmul.** No dense matrix multiplication or GEMM in training or serving, including wrappers and hand loops (§3.3).
4. **GPU-only research compute.** Training, evaluation and qualification run on the GPU; serving the exported integer artifact on client devices is the product and is allowed (§3.5, §3.6).
5. **Never grant or assume an owner release yourself.** Ask Casey in your own chat; only his approval there counts, and you record his words verbatim with the time (§5.1).
6. **Deliver through the full path:** PR → exact-head review by another vendor's lab (doc-only or generated PRs: a fresh-context self-review) → merge queue → verify on fresh `main` → delete the branch and worktree (§5.5).

## Where things are

| What | Where |
|---|---|
| Project overview | [README.md](README.md) |
| Rules | [CHARTER.md](CHARTER.md) |
| Documentation hub | [docs/README.md](docs/README.md) |
| Current plan, roster and approved sequences | [docs/integration/NOW.md](docs/integration/NOW.md) |
| The loop: sessions, orders, reports, review, build plan | [docs/loop/README.md](docs/loop/README.md) (§5.10) |
| Mechanism board (ACTIVE / READY / PARKED) | [docs/mechanisms/README.md](docs/mechanisms/README.md) |
| Research notebooks, one page per line | [docs/research/notebook/](docs/research/notebook/) |
| Decision index (D0–D59) | [docs/integration/DECISIONS.md](docs/integration/DECISIONS.md) |
| Workspace, storage and hygiene | [docs/labs/workspace.md](docs/labs/workspace.md) |
| Archive of retired text and records | [archive/CATALOG.md](archive/CATALOG.md) |

## How to work

- Pick work from `NOW.md`, post a one-line intent note on the owning issue, claim the branch and continue unless redirected (§5.3).
- Work in the loop (§5.10): one unit per session, an ORDER before work, a REPORT with negatives first, and your lab name tagged on every claim, PR, review and comment.
- A change to a bar, a status, a constraint or spend is a decision card to the PM or owner, never a subagent's call (§5.2).
- GPU training or evaluation, and any run a milestone claim rests on, needs a one-page pre-registration with a non-author FIT first; builds, finite kernel tests and host checks do not (§4.5).
- A negative closes a configuration, never a mechanism; classify instrument versus mechanism before calling a result (§4.1, §4.2).
- Record each result as one row in the line's notebook; keep a dated evidence folder only when a run seals artifacts (§5.6).
- Rust first; no ad-hoc scripts; no committed binaries, corpora or secrets; never read or print API keys (§3.11, §3.13).

## Harness basics

```sh
export UOR_LAB=<antigravity|claude|codex|deepseek|kimi>   # once per session; presence is then recorded automatically
uor-harness status                            # what is true now: headline, lines, delivery
uor-harness brief -m m1                       # context for a milestone task, with sources named
uor-harness check                             # every machine-checkable rule, with the fix for each
```

Build it with `CARGO_TARGET_DIR=local/harness-target cargo build -p uor-r4-harness`. Details: [docs/harness/README.md](docs/harness/README.md).

## Compute

- GPU pods only through `scripts/pod/uor-pod`; Vast.ai is the default and Runpod is the emergency backup; run `uor-pod status` first.
- At most 4 pods and $8/h across all labs; lease the GPUs you use and never touch another session's GPU, lease or files.
- Never cancel a healthy run because a self-set budget ran out; extend it, renew the lease and note it on the owning issue.
- Save results off the pod before `down`; CPU-only builds and static checks run on the laptop; CUDA kernels are built ahead of time with nvcc on the pod.

## Owner-only boundaries

- The charter, the technical constraints, milestone bars, holds, spending caps and exceptions are the owner's to change.
- Training, M2 and Stage B stay HELD until the owner releases them; a FIT review releases nothing.
- One owner release, approved by Casey in your own chat, covers one pre-registered sheet, with all its arms, seeds and stop rules.
- A pending owner question means wait; do not act on your own reading of it.
- Nothing is deleted: negatives, unique source and other labs' work move byte-identical to `archive/`.
