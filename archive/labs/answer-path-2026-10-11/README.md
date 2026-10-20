# Which path produces the answer: the read does, the copy head does not — a causal attribution in four serving-time conditions

References #2029 (M1 acceptance; pointer mechanism brief, D22 **order 3**). Lab: DeepSeek, session
`deepseek/attribution`. Runs 2026-10-11 01:19–03:41 UTC. **Serving-time only: no training, no pod, laptop CPU.**

> **One line.** Before building anything else on this line, the question that had never been asked: **which
> mechanism actually produces the answer token?** Four conditions on the same artifact, the same frozen v5 panel
> and the same harness answer it. Removing the facts from the prompt takes the memory panel to **0/40** (all 40
> rows fail with *no value at all*) — the answers come from the context through the **read**. Switching the
> **copy head off entirely** costs **one panel row** (23 → 22, inside the ±3-row noise) — the copy head does not
> produce them. Forcing the copy to carry half the mixture costs **sixteen rows** (23 → 7). The pointer/identity
> programme has been tuning a branch that carries ~6 % of the output and contributes ~1 row of the answers.

## The four conditions

One artifact throughout: the order-3 address arm **S2** (`09e9dd3d…`, `pointer_route=prime-ranked:1`,
`pointer_identity=none`, 2,000 steps from the chat base on the adopted `mix-10` store; memory 23/40, wrong-value
9, BPB 1.17109). Frozen v5 panel, frozen checks, cap 64, `chat-grade` reply + `grade-replies` with the frozen
grader. The two interventions are serving-time read-out changes recorded in each `replies.json`
(`pointer_gate_ceiling=`, and a rewritten request file) — **no parameter, no gradient, no training path**.

| # | condition | how | memory `check_pass` | wrong-value | no/other | `acceptable` (judge) |
|---|---|---|---:|---:|---:|---:|
| **A** | **normal** — the copy at its learned gate (~0.056–0.064) | — | **23/40** | 9 | 8 | **17** |
| **B** | **copy head off** | `pointer_gate_ceiling=0.0` | **22/40** | 7 | 11 | 9 |
| **C** | **facts removed** — only the question turn kept | `panel-qonly.json` | **0/40** | **0** | **40** | 2 |
| D | copy forced to half the mixture | `pointer_gate_floor=0.5` | 7/40 | 21 | 5 | 1 |
| E | copy forced to all of it | `pointer_gate_floor=1.0` | 9/40 | 18 | 4 | 4 |

## What each reading establishes

**1. The answers come from the context, through the read (C).** With the fact turns removed, **every one of the
40 memory rows fails and every failure is a missing value** (`no/other` 40, `wrong-value` 0). The model is not
answering from pretraining, and it is not producing a value it learned elsewhere: it produces no value at all
when the facts are absent. Grounding is real on this panel, and its carrier is the **softmax read over the
context**.

**2. The copy head is not the answer path (B).** Switching it off costs **one row** — 23 → 22, inside the panel's
±3-row noise — while removing 6 % of the mixture. The learned gate is therefore not suppressing a correct
answer; it is using the copy for something else. What it *is* using it for shows up in the judge column:
`acceptable` falls **17 → 9** with the copy off, so the copy contributes **fluency** (a locally copied token)
rather than the value.

**3. Forcing the copy on is destructive because it displaces the reader (D, E).** Half the mixture costs 16 of 40
rows and 16 of the 17 `acceptable` answers; all of it costs 14 rows. A branch that is worth ~1 row cannot be
given 50 % of the output without pushing out the branch that is worth 23.

**4. The pointer line's metrics and the panel were measuring different things.** The order-3 result
([pointer-address record](../labs/ptr-addr-2026-10-10/README.md)) raised the pointer's hit rate **0.377 → 0.54**
across three seeds at zero likelihood cost with the panel flat. Conditions A–C say why: a pointer that finds the
value contributes it to a branch holding ~6 % of the output, and the answer is produced by the read.

## What this changes

- **The next mechanism on this line must act on the read, or the output path must be rebuilt so that a copy
  *can* be the answer.** Enabling the existing copy head cannot do it: the oracle says so directly (D, E), and
  the ceiling says the branch is worth one row (B).
- **The read is a softmax search** over the context's positions (3 read layers, 16.6 % of the model's MACs at
  context 384 but **40.5 % at 4,096** — [geometry audit](../integration/geometry-audit-2026-10-11/README.md)).
  That is where an addressing mechanism has both a target (it *is* the answer path) and a compute argument (its
  cost grows with context while a lookup's does not).
- **Every pointer/identity result on this line is re-scoped by this measurement**, including the ones that
  looked like mechanism movement: they were read-outs of a 6 % branch. They are not retracted — the numbers are
  what they are — but they cannot be evidence about what answers the model gives.

## Limitations

- **One artifact, one seed, 40 memory rows** (±3 rows of noise); the judge column (`acceptable`) moved further
  than `check_pass` in condition B (17 → 9), which is a *judge-side* reading on the same replies and is reported
  as such rather than folded into the headline.
- **Condition C removes rather than masks**: the fact turns are gone, so it measures dependence on the prompt,
  not the read's internal weights. A span-mask ablation (the read cannot attend to the fact while it is present)
  would separate "the read finds it" from "the generator uses it", and is the next instrument if this line
  continues.
- **No training anywhere in this record**; nothing here bounds a model *trained* with an addressable output path
  or with the read replaced by an addressed lookup — those are the two candidate load-bearing mechanisms named
  in the audit, and this measurement is what tells us which of them starts first.

## Cost

| | |
|---|---|
| compute | laptop CPU only: five reply sweeps (64 rows, cap 64) and four gradings on the local judge |
| pod | none |
| external | none |

## Evidence

- Reply and grading reports: `~/uor-r4-local/attribution-20261011/{A=B-as-oracle-0.0,B,Bg,C,Cg}` and
  `~/uor-r4-local/oracle-20261011/{g0.0,g0.5,g1.0,gg0.0,gg0.5,gg1.0}`, bundled to cloud-store.
- The instruments: [#2187](https://github.com/UOR-Foundation/uor-r4/pull/2187) (the gate floor, merge
  `f97ccc3ad`) and [#2188](https://github.com/UOR-Foundation/uor-r4/pull/2188) (the gate ceiling).
- The question-only panel: `panel-qonly.json`, 40 memory rows with only their question turn, derived from the
  frozen panel and recorded with this bundle.
