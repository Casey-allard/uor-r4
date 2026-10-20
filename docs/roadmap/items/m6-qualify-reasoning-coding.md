---
id: m6-qualify-reasoning-coding
title: Qualify reasoning and coding on the served model
milestone: M6
area: evaluation
status: ready
open_to: lab
needs_gpu: true
depends_on: [m4-export-uorlut01-cli, m6-reasoning-panel-freeze, m6-causal-ablation-instrument, m6-exact-arithmetic-mechanism]
---

## Why it matters

M6 closes reasoning and coding on the same model that holds the conversation and memory. This item is the qualification run: the served M4 model is scored on the frozen panel, and the ablation shows the answers depend on the model's own state transport and addressed memory. Nothing else in M6 counts until this passes. See [serving and export](../../model/serving-and-export.md) for the integer artifact that gets served.

## What is missing

m6-reasoning is 0/1 qualified and not started. The served M4 model has to pass the frozen panel and the causal ablation. Today M4 is held: the integer serving engine is built, but no model trained under the current rules is exported to it (`STATUS.md`). The panel, the ablation instrument and the arithmetic mechanism do not exist yet, and the numeric bar for the panel is not yet registered.

## Done when

M6 bar: 'Executable reasoning and coding on the same model'; the served model passes the frozen m6-reasoning panel with deterministic oracles, and ablation confirms dependence on state transport and addressed memory.

## How to start

1. Check the four dependencies. Confirm that `m4-export-uorlut01-cli`, `m6-reasoning-panel-freeze`, `m6-causal-ablation-instrument` and `m6-exact-arithmetic-mechanism` are done: `ls docs/roadmap/items/` and the item status lines, `docs/harness/panels/m6-reasoning.json` (`frozen` must be `true`), and `gh issue view 2034 --repo UOR-Foundation/uor-r4`. Do not start while any is open. Run `uor-harness status`; the harness briefs cover m1 to m4, so use the issue for M6 context.
2. Post a draft pre-registration sheet on #2034 as your first deliverable: one page, with the bar from the frozen panel (state it as "not yet registered" if it is still missing), the measured noise, the controls, at least 2 seeds, cost, the stop rules and what each outcome decides. Use a registered sheet such as `docs/labs/a8-dose-2026-10-19/README.md` as a layout reference. The ablation controls must name which parts are knocked out (state transport, addressed memory) and what drop counts as dependence.
3. Get a non-author TEST FITNESS: FIT review of the sheet (CHARTER §4.5). A fresh-context reviewer counts. A FIT releases nothing. Staging the run needs the owner: Casey types the owner release personally with `UOR_LAB` unset, and the lab stages the exact commands (CHARTER §5.1).
4. After the release, run on the GPU through `scripts/pod/uor-pod` (`uor-pod status` first; at most 4 pods and $8/h across all labs, and lease only the GPUs you use). Serve the exported model through the integer path, never a float stack. Score the frozen panel with its deterministic oracles, then run the ablation instrument. Outside contributors may use their own GPU and must attach the hardware specs; a non-5090 run is re-run by a maintainer under the same FIT.
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
- issue#2034
