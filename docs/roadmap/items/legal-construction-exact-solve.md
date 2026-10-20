---
id: legal-construction-exact-solve
title: Solve the saved legal-construction case in exact arithmetic
milestone: research
area: mechanism
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

It reopens a PARKED optimizer path that needs no gradients. Legal construction would replace the optimizer and the rounding step that follows it: each parameter change is picked straight from values the integer model can hold, so every accepted step is already servable. See [legal construction](../../mechanisms/legal-construction.md). It is a candidate learning rule for the discrete model, and today it has no score.

## What is missing

Legal construction picks each parameter change from values the integer model can hold, but it has never been scored: every attempt stopped on solver numerics. The saved case has 1,960 variables (and 440 constraints) and has not been solved in exact arithmetic. No GPU is assumed, but whether a host solve is allowed is open (see the classification bullet below).

From the mechanism page and the archived record:

- The program runs offline in a pure-Rust solver (`microlp`) with a 4,096-node branch-and-bound cap. It returned `InternalError("Singular matrix")` and no assignment.
- A scale-aware pivot threshold did not make the backend usable on the saved problem.
- The record's own next step is to instrument the exact failing LP basis and factorization context.
- The classification is open. The page says it is not yet assessed against D30, because the guard test forms a 380-by-1,960 Jacobian product and the solver does dense LU. The page adds that a rerun would have to be classified or run on the GPU, and CHARTER §3.5 makes research compute GPU-only, so a host solve is not yet cleared. It also says the float solver can return only as an offline float sheet under D37, with a fresh review, an owner release, a separately scanned module and a reported export gap; whether an exact-arithmetic solver falls under that is for the PM to say.
- The pooled-ranking loss behind the older M2 learner used banned softmax and softplus forms, so a rerun on the real panel needs a compliant objective. The solve itself minimises a saved gradient `g·δ`.
- No bar for what "scored" must beat is registered.

## Done when

The saved 1,960-variable case is solved in exact arithmetic and the result is recorded as an optimal assignment with its objective, or as an infeasibility proof. Scoring that result on the real panel is a separate step.

## How to start

1. Read `docs/mechanisms/legal-construction.md` and `archive/labs/direct-legal-construction-2026-10-09/README.md` (the attempts, the saved-case counts and the Next section). Locate the saved case, which the mechanism page calls `input245`: the record cites the authenticated #2101 objective and protected derivatives, and `archive/labs/direct-legal-construction-2026-10-09/storage.json` lists two verified archives, `codex-direct-legal-absolute-failure-20261009` (md5 `661527eb7d8c4636e66f147032a598fb`) and `codex-direct-legal-centered-20261009` (md5 `5a965dade6ed7b04b8238b4d1e020be3`). They are held in the owner's iCloud cold archive (see `docs/reference/storage.md`), so ask a maintainer on the issue for the case file before you start. Post where it lives and its hash.
2. Post a design note on the issue first. It must give: the exact-arithmetic method and number type; why the solve uses no gradients, softmax or squashing (the objective is a saved vector); and a classification against CHARTER §3.3 and D30 for the dense LU and the Jacobian product. If that classification is unclear, raise it as a decision card to the PM instead of choosing; instrument and mechanism are classified before any result is called.
3. Write finite CPU controls first. Use tiny instances solved by hand, plus a brute-force enumeration over the keep, lower or higher choices (3 per coefficient, so a few coefficients at most). Cover the legal endpoints `q = ±7` and the noop branch. The archive's exact-source checks (`exact-source-checks.json`, `centered-exact-checks.json`) show the earlier fixtures. The exact solver must agree with enumeration on every control before it touches the saved case.
4. Implement the solver on the host in Rust. Old code in `archive/` is read-only: copy Rust only, and do not edit it. Add no Python; the folder's `prepare.py`, `qualify.py`, `recover.py` and `run.py` are for reading, not copying. Run the saved case and record either the optimal assignment and objective, or an infeasibility proof. Re-check the answer independently: evaluate all 380 guard rows exactly on the returned change.
5. Builds, finite controls and host checks need no pre-registration (CHARTER §4.5), but running the saved-case solve waits on the classification in step 2, because CHARTER §3.5 makes research compute GPU-only. Scoring a result on the real panel is a separate step: it needs a one-page pre-registration and a FIT review by someone who did not write it, and it runs on the GPU. An outside contributor may use their own GPU and must attach its specs; a maintainer re-runs on an RTX 5090 when the hardware differs.

6. Open a PR that says `Closes #<this issue>`, or `References` if partial. No notebook page covers optimizers, so add one row on a new page `docs/research/notebook/<line>.md` in the format of `docs/research/notebook/README.md` and name the line in the design note.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/mechanisms/README.md:111`](../../../docs/mechanisms/README.md)
