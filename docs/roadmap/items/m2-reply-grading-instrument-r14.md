---
id: m2-reply-grading-instrument-r14
title: Close the remaining R14 gaps in the deterministic reply grader
milestone: M2
area: evaluation
status: ready
open_to: anyone
needs_gpu: false
depends_on: []
---

## Why it matters

M2 and Stage 5 ask for a first simple conversation graded by fixed deterministic checks ([where we are going](../../start/where-were-going.md)). Rule R14 voids every reply target that rests on an instrument that fails on evidence, so the 116/232 reply bar is BLOCKED and is not a closing condition. Until the reply grader passes R14 on evidence, a conversation bar cannot be scored, and no reply number can bind anything.

## What is missing

**The status pages are stale on one point.** R14 (`docs/harness/pm-rules.md`, section R14), `docs/integration/NOW.md` (line 95), `docs/integration/project-track.md` (line 101) and `docs/harness/panels/v5-reply.json` all say there is no instrument or that the v2 check table is still to come. The dated lab record `archive/labs/m1-grader-v2-checks-2026-10-14/README.md` (14 Oct 2026, merged as #2412 and #2433) shows a v2 grader already built. It has `crates/uor-r4-training/fixtures/reply-panel-checks-v2.tsv` (97 lines: one comment line and 96 checked rows), 35 frozen must-pass positive controls (`reply-panel-positive-controls-v2.tsv`), 17 frozen must-fail negative controls (`reply-panel-negative-controls-v2.tsv`), and paired comparisons of the model against the best constant reply and two copy controls (`copy_first_stated`, `copy_last_stated`) in `crates/uor-r4-training/src/bin/chat-grade.rs`. The v1 table (`data/panels/reply-panel-checks.tsv`) is unchanged.

R14 names three things that must exist before any reply target: a v2 check table with frozen positive controls, per-category best-constant pairing, and abstention-as-discrimination scoring. The lab record covers the first two. It does not say that the third is done, and the PM has not ruled the v2 grader non-vacuous: the record leaves the M1 reply headline SUSPENDED with no target. What the record measures on the 232-row open panel:

- The v2 table checks 96 rows. The other 136 rows stay BLOCKED, with no deterministic check.
- The default constant reply ("I'm not sure. Can you tell me more about what you mean?") still passes 38 of 96 under v2 (40 of 88 under v1). The models re-scored (parent 29M, rp-D0, rp-D10, rp-D25) pass 10 to 15 of 96, and none beats the best constant in any category.
- On the 40-row multi-turn memory panel, the models tie `copy_last_stated` (20 of 40 each).

The qwen-graded 116/232 target is banned under D28 (neural grader). Any target is set from measured mechanism behaviour, never picked, and registering one is a decision card to the PM or owner.

## Done when

A scope note on `#2029` records, for each of the three R14 requirements, whether the v2 grader meets it and on what evidence, and says what is still missing. Each missing part is built and tested, or the note says plainly that it stays out of scope. For abstention scoring, the note shows a constant-reply control run and the result. A non-author reviews the note and the instrument. The PM, not this item, rules whether the instrument passes R14.

## How to start

1. Read R14 in `docs/harness/pm-rules.md` (and R12, which it cites), `docs/integration/project-track.md` (the BLOCKED paragraph) and `docs/harness/panels/v5-reply.json`. Then read the lab record `archive/labs/m1-grader-v2-checks-2026-10-14/README.md`: sections 1 and 2 (what v2 changed) and 4 (the open-panel re-score). Also check `docs/harness/panels/m2-reply512.json` for the M2 panel definition. `docs/integration/DECISIONS.md` D39 now only points into the charter; the owner's M1 closing bar is in CHARTER §4.8.
2. Open the existing instrument: `crates/uor-r4-training/src/bin/chat-grade.rs`, the three v2 fixtures in `crates/uor-r4-training/fixtures/` and `data/panels/README.md`. For each R14 requirement, write down what the lab record shows it already meets and what it leaves open. The table is checked by `checks=v2`, which you pass to `chat-grade grade-replies`; the default stays v1.
3. Post the first deliverable on `#2029`: a scope note with that list. It also names the positive, negative, constant and copy controls that will be run, and says how any new checks are frozen and hashed (the v1 panel files are listed with sha256 in `data/panels/MANIFEST.sha256`; add any new frozen file there and leave the existing lines unchanged).
4. Build the missing parts in Rust inside `chat-grade.rs` or a sibling module under `crates/uor-r4-training/src/`. Write finite tests first, each control as a test that must fail without the new rule. The lab record used `~/.cargo/bin/cargo test -p uor-r4-training --bin chat-grade --features native-rung0` (the `chat-grade` binary needs the `native-rung0` feature, which is on by default). Grading must stay string checks and exact counts: no neural grader (D28), no softmax, no matmul.
5. Run the instrument on the host against existing replies and controls and record the constant-reply and copy-control results. This is string-check grading of stored replies, with no model compute (D32, as the lab record notes), so it does not need a GPU. The replies the lab record re-scored are not in git: it names seven `replies.json` files from archive tarballs with their sha256 (section 3), and `data/panels/reply-panel-canned.txt` and `data/panels/reply-panel-controls.txt` hold the canned and control replies. Ask a maintainer for the reply files. Ask a non-author to review the scope note and the instrument before any bar is proposed.
6. Open a PR that says `References #2029` (it closes this item only if the owner accepts the instrument). Record the result as one row in the closest research notebook under `docs/research/notebook/` (check `docs/research/notebook/README.md`; `harness-governance.md` is the likely home for instrument rows).

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`docs/start/where-were-going.md:52`](../../../docs/start/where-were-going.md)
- issue#2029
- [`docs/harness/pm-rules.md`](../../../docs/harness/pm-rules.md) (R14)
- [`docs/harness/panels/v5-reply.json`](../../../docs/harness/panels/v5-reply.json)
- [`crates/uor-r4-training/src/bin/chat-grade.rs`](../../../crates/uor-r4-training/src/bin/chat-grade.rs)
- [`archive/labs/m1-grader-v2-checks-2026-10-14/README.md`](../../../archive/labs/m1-grader-v2-checks-2026-10-14/README.md)
- [`docs/integration/NOW.md:95`](../../../docs/integration/NOW.md)
