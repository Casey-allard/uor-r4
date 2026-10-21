---
id: m8-independent-verification
title: Independently verify the alpha on consumer hardware
milestone: M8
area: evaluation
status: ready
open_to: anyone
needs_gpu: true
depends_on: [m8-release-packaging]
scope: [xtask/**, docs/products/**, docs/labs/m8-independent-verification/**]
---

## Why it matters

This confirms the alpha claims without the authors' own setup. A result that only the authors can reproduce is weak evidence, so M8 asks someone else to install the one `UORLUT01` file and re-check every earlier bar. It is the last step of the model's path to alpha. See [releases and artifacts](../../products/releases-and-artifacts.md).

## What is missing

The M8 criteria (`docs/integration/project-track.md`, M8) require independent verification on consumer hardware of the one `UORLUT01` artifact. Nothing exists to verify yet: no trained model has been exported, and no checksum-pinned release exists. The bar is not yet registered for what counts as consumer hardware (M5 uses "an M1-class laptop") or who counts as a third party.

One rule shapes the kit. Under CHARTER §3.5 and §3.6, panel scoring, qualification and milestone evaluation of an exported artifact are GPU research compute; only product serving, and measuring its tokens per second and energy, run on client devices. So a consumer machine can check the install, the checksum, bit-identical tokens across the library, command line and browser, and tokens per second, memory and energy. The bars that score a panel (for example M1 integer top-1 and M2 graded questions) are scored on a GPU, never a CPU. How that GPU re-scoring counts toward "independent verification on consumer hardware" is a bar question for the decision card in step 2.

## Done when

A third party reproduces the install and the M1 to M7 bars from the checksum-pinned artifact: the serving-side bars on consumer hardware, and the panel-scored bars on a GPU as CHARTER §3.6 requires. The bars are the rows of the M1 to M7 table in `ROADMAP.md`; M1's integer top-1 condition was set by the owner on 19 October 2026. The owner declares alpha; this item produces the evidence for that decision and does not make it.

## How to start

1. Read the M8 row and the M1 to M7 rows of `ROADMAP.md`, the M8 checklist in `docs/products/releases-and-artifacts.md` (from line 52), and `docs/roadmap/items/m8-release-packaging.md`. Note which bars each earlier milestone froze and what each one needs to run.
2. Post a design note on the issue: a proposed definition of consumer hardware and of a third party, listing the machine specs to record, and which bars run on the consumer machine and which on a GPU (CHARTER §3.6). Defining either is a bar change, so send it to the PM or owner as a decision card; do not decide it here.
3. Draft the verification kit as a plan, without a model: a one-page runbook that starts from the release notes and the artifact checksum and ends at each bar's command and expected number; a report template that records hardware, operating system, artifact hash, build commit and each bar's measured value; and a rule that a number counts only if it is copied from a command's output.
4. Decide where the checks live, in Rust (for example a subcommand in `xtask/` that verifies the checksum and runs the bar commands and prints a report). Write the first test as a failing one: it should reject an artifact whose checksum does not match. Run it with `~/.cargo/bin/cargo test -p xtask`. Add no Python.
5. Dry-run the host-side steps of the kit (checksum check, install, serving a fixed prompt) on a synthetic artifact. From the repository root, run `~/.cargo/bin/cargo run -p uor-r4-integer --features serving-wasm --bin mk-synthetic-stack -- 5 <output path outside the repository>`. The binary takes `[SEED] [OUT_PATH]` and, with no path, writes `synthetic-stack.lut` into the current directory; never commit that file. The stack has random weights, so the dry run cannot score a panel. Label the result a synthetic dry run, not a verification.
6. Wait for `m8-release-packaging` to publish the artifact and release notes. Then recruit the verifier (someone outside the authors' machines), have them run the runbook, and post their report on the issue unedited. The M8 claim rests on that run, so it needs a pre-registration sheet with a non-author FIT review first (CHARTER §4.5). If the verifier scores a panel on their own GPU, they attach its specs (GPU model and memory, driver and CUDA version, CPU and RAM), and a maintainer re-runs on an RTX 5090 when the hardware differs (CONTRIBUTING.md).
7. Open a PR that says `Closes #<this issue>` only after a third party's report is attached; use `References` for the kit alone. Add one row to `docs/research/notebook/m4-serving.md` (there is no M8 notebook yet; ask the PM if a new one is wanted).

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- issue#2036
- [`ROADMAP.md`](../../../ROADMAP.md)
- [`docs/products/releases-and-artifacts.md`](../../products/releases-and-artifacts.md)
- [`docs/integration/project-track.md`](../../integration/project-track.md) (M8)
- [`CHARTER.md`](../../../CHARTER.md) (§3.5, §3.6, §4.5)
