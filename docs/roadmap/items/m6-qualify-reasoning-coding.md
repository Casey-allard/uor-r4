---
id: m6-qualify-reasoning-coding
title: Qualify reasoning and coding on the served model
milestone: M6
area: evaluation
status: ready
open_to: lab
needs_gpu: true
depends_on: [m4-export-uorlut01-cli, m6-reasoning-panel-freeze, m6-causal-ablation-instrument, m6-exact-arithmetic-mechanism]
notebook: docs/research/notebook/m6-reasoning-coding.md
scope: [crates/uor-r4-integer/**, crates/uor-r4-discrete/**, docs/harness/panels/**, docs/harness/prep/m6-qualify-reasoning-coding*, docs/harness/milestones.json, docs/labs/m6-qualify-reasoning-coding/**, docs/loop/tasks/m6-qualify-reasoning-coding/**, docs/research/notebook/m6-reasoning-coding.md]
---

## Why it matters

M6 closes reasoning and coding on the same model that holds the conversation and memory. This item is the qualification run: the served M4 model is scored on the frozen panel, and the ablation shows the answers depend on the model's own state transport and addressed memory. Nothing else in M6 counts until this passes. See [serving and export](../../model/serving-and-export.md) for the integer artifact that gets served.

## What is missing

**November 17, 2026.** The frozen CodeTasks grader reads four labeled integers (`V0` through `V3`) from each candidate answer. The independent `rustc --test` verifier compiled the panel’s 80 canonical reference programs and checked all 320 held-out vectors. The grader accepts no candidate program, so these measurements establish the reference verifier’s behavior. [Frozen panel](../../harness/panels/m6-reasoning.json) · [Grader implementation](../../../crates/uor-r4-harness/src/panel_m6_reasoning.rs) · [Verifier result](https://github.com/UOR-Foundation/uor-r4/pull/3821)

Before qualification, the pre-registration for #2865 must resolve the model-facing meaning of the frozen verifier clause. Specify what the model emits and what the grader can score, and use a D68 decision review to bind, drop or reword the clause. Any change to a panel row or oracle requires a fresh FIT. Qualification also needs an identified, accepted M4 artifact and the registered comparisons with state transport and addressed memory ablated. [Verifier registration and FIT conditions](m6-code-task-rustc-verifier.md) · [Qualification issue](https://github.com/UOR-Foundation/uor-r4/issues/2865)

## Done when

M6 bar: 'Executable reasoning and coding on the same model'; the served model passes the frozen m6-reasoning panel with deterministic oracles, and ablation confirms dependence on state transport and addressed memory.

## How to start

1. Check the four dependencies. Confirm that `m4-export-uorlut01-cli`, `m6-reasoning-panel-freeze`, `m6-causal-ablation-instrument` and `m6-exact-arithmetic-mechanism` are done: `ls docs/roadmap/items/` and the item status lines, `docs/harness/panels/m6-reasoning.json` (inspect the frozen M6 panel entry), and `gh issue view 2034 --repo UOR-Foundation/uor-r4`. Do not start while any is open. Run `uor-harness status`. For M6 context read the `m6` entry in `docs/harness/milestones.json`, the M6 section of `docs/integration/project-track.md` and the issue.
2. Post a draft pre-registration sheet on #2034 as your first deliverable: one page, with the bar from the frozen panel (state it as "not yet registered" if it is still missing), the measured noise, the controls, at least 2 seeds, cost, the stop rules and what each outcome decides. Use a registered sheet such as `docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md` as a layout reference (its record is `docs/labs/a8-dose-2026-10-19/README.md`). Resolve the model-facing CodeTasks clause under D68 in that registration. The ablation controls must name which parts are knocked out (state transport, addressed memory) and what drop counts as dependence.
3. Obtain a non-author FIT and complete the lab's D67 pre-run review, binding the selected model, controls, hardware and cap-compliant lease.
4. After the pre-run review passes, run on the GPU through `scripts/pod/uor-pod` (`uor-pod status` first; at most 4 pods and $8/h across all labs, and lease only the GPUs you use). Serve the exported model through the integer path, never a float stack. Score the frozen panel with its deterministic oracles, then run the ablation instrument. Outside contributors may use their own GPU and must attach the hardware specs; a non-5090 run is re-run by a maintainer under the same FIT.
5. Save the results off the pod before `uor-pod down`. Seal the checkpoint hash, panel hash and logs in a dated evidence folder under `docs/labs/` only if the run seals artifacts.
6. Open a PR that says `Closes #<this issue>` only if the bar is met, or `References` otherwise. Add one row to `docs/research/notebook/m6-reasoning-coding.md` (create it in the format of `docs/research/notebook/README.md` if it still does not exist). Use one of the five verdict words, and do not read a negative as closing the mechanism.

## Rules that apply

- **Geometry only** (CHARTER §3.1–§3.3): every mechanism, training signal and fix comes from the project's geometry. No softmax or squashing anywhere, including losses, gradients, sampling and renamed or table forms. No dense matrix multiplication or GEMM, including wrappers and hand loops.
- **GPU only** (§3.5): training, evaluation and qualification run on a GPU, never a CPU. Outside contributors may run on their own GPU and must attach the hardware specs to the PR. If the hardware is not equivalent to the project baseline (an RTX 5090), a maintainer re-runs it after the PR, under the same FIT (owner, 20 Oct 2026).
- **Pre-registration first** (§4.5): a GPU training or evaluation run, and any run a milestone claim rests on, needs a one-page sheet with a non-author TEST FITNESS: FIT review before it runs.
- **Rust first** (§3.11): new code is Rust, and no new Python is added.
- **Records** (§4.1, §5.6): each result is one row in its line's research notebook (`docs/research/notebook/<line>.md`). A negative result closes a configuration, never a mechanism.

## Sources

- [`ROADMAP.md:19`](../../../ROADMAP.md)
- [`STATUS.md:19`](../../../STATUS.md)
- issue#2034
