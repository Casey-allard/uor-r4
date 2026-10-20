# #1783 — docs(labs): DeepSeek session record 2026-10-05/06 — localisers under D20, plus operational incidents

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T04:17:45Z
- **Updated:** 2026-10-09T13:22:16Z
- **Closed:** 2026-10-06T04:18:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1783
- **Merged:** 2026-10-06T04:18:40Z
- **Merge commit:** d2cae5ba575da5b3f584b866007c6ef6de7045fc
- **Base / Head:** main / UOR-Foundation:deepseek/session-record-20261006
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Docs-only. Adds `docs/labs/deepseek-session-2026-10-06.md`.

Records this session's work **under D20 (#1747)** — as **localisers that name a missing component**. No mechanism is retired by this document, and no near-zero delta is treated as a verdict.

## Findings, each with its named missing component

- **The engine computes exact arithmetic 80/80 and emits it 0/80.** The value *is* wired byte-wise into the emission input (bits 792..800) but the 1024→9 LUT cascade erases it at frozen init — 2–4 distinct contexts on a 0..255 sweep, constant in 3/9. *Missing component: the cascade is not trained to route the field — not a wiring gap.*
- **A council's structural-emission recommendation was refuted before anything was built.** A Python-only replay of `Topology::seeded` at `0x973`, verified **1380/1380 gate pins** against an independent rebuild, measures **10/10 separable digits** — at every seed sampled. *Missing component: downstream, in the learned 257-way readout and/or the fit.*
- **The STE hypothesis was falsified, and the method was not an STE.** The relaxation moved the discrete surface **8.5× more** than the score-function fit (606 of 5520 LUT signs vs 71), so *"gradient cannot reach the tables"* is **false**; it still collapsed 18→1 contexts with the gradient decaying to 1e-43. An audit then found **no hard-forward/soft-backward pair existed**. *Missing component: the objective, plus a genuine gradient-preserving straight-through derivative.*
- **On `chat-100m2-C` the served head IS the tied embedding** (rank 1024/1024); a plain linear softmax recovers **98.8%** and a zero-parameter 1-NN **93.3%** of served accuracy; **1025 nonzero singular values, not 9** — the participation ratio's 9.05 is a coincidence of the statistic, and the direct rank-9 truncation control costs **6.10 bits and 52 points**. A negative result that **closes a work class**.
- **Two premise corrections** found by reading source before running: the 9-param `OUTPUT_MIX` sign walk is **not** the served head (*"No retained-model dispatch uses it"*), and the `P − A = 0.5423`-bit measurement was **not** reproduced.

## Operational incidents (recorded for #820)

Coordination **expired silently** (heartbeat ~Oct 1 against sequence 896; the lab adapter states *"no heartbeat automation"*, and `coord.rs:492` expires stale labs). `coord` **STORE is `coord.git`, not the runner dir** — its failure reads as a git-config error and is not one. `host-policy.json` declared **two dead volumes** (now removed). Admission is held on a **stale 25.5 GiB measurement against a 40.00 GiB reserve** while the machine now measures **46.51 GiB**. A "lost" artifact was **catalogued in iCloud**. Dataless iCloud placeholders **read as an empty mirror**. And **local policy drifts from repo policy with nothing checking it.**

## D20 §2 compliance, stated honestly

**NOT satisfied**, so the §3 suspension does **not** apply and no mechanism may be retired. Conditions 1 (matched ordinary control) and 2 (≥3 seeds) are unmet for the geometric read; condition 3 is **met** for addressed-attention (`Work::selected_operations` is a firing counter); condition 4 is partly met; condition 5 is unmet.

References #820, #1747, #973, #1552, #1746.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

