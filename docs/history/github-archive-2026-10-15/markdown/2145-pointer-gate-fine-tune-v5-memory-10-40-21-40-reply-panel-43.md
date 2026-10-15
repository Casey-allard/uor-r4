# #2145 — Pointer-gate fine-tune: v5 memory 10/40 -> 21/40, reply panel 43/232 -> 28/232 (guard missed by one cell), matched control 22/40 and 22/232 (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T06:50:12Z
- **Updated:** 2026-10-10T06:53:13Z
- **Closed:** 2026-10-10T06:51:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2145
- **Merged:** 2026-10-10T06:51:23Z
- **Merge commit:** 4c777c477edb81ff24431d879c350021a71938c6
- **Base / Head:** main / UOR-Foundation:deepseek/pointer-gate-20261010
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Result: v5 memory 10/40 → 21/40 on the pre-declared arm (matched control 22/40) and the open reply panel 43/232 → 28/232 (control 22/232) — the primary bar is MET, the guard is MISSED BY ONE CELL, and the matched control shows the gain *and* the cost are the training data, not `gate_supervised_loss`.** References [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029).

Line: pointer-target supervised fine-tune · count 0/3 · headline 43/232 reply `acceptable` + 10/40 v5 memory `check_pass` → 28/232 reply + 21/40 v5 memory (pre-declared arm; matched control 22/232 reply, 22/40 memory)

## What and why

## What this is

The D21 M1 result piece, [pre-registered on #2029](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6093396145) **before any pod existed** (bar, base checkpoint, stream, arms, cost) and executed exactly as pre-registered. One 2 × RTX 5090 pod, **22 minutes, ≈ $0.87 of the $3.60 ceiling**; every acceptance reading on the owner's laptop CPU.

- **Base:** `chat-29m-B-lr5e-4` `d8a3c971…` — the artifact behind both headline numbers — with its **own run `report.json`** as `init=` provenance (fetched from `icloud:UOR-R4/results/deepseek/ladder.tar`, md5 verified; tokenizer `d36d3e87…`, `merges: null`).
- **Stream:** the base's **own** fine-tune store (`ft-mixed-mworld-chatv0`, `tokens.u16` `8794ad27…`, verified equal to the store the base's report records — *not* the look-alike `ft-mixed-c` on the volume) **plus** `dialogue-recall-corpus generate generator=v2` (124,000 dialogues, **`leak pass`** against `data/panels` *and* the three open-reply-panel request files), concatenated by `mix-chat-corpus`: **109,092,553 tokens, 327,304 response episodes, recall share 55.06 %** (pre-registered band 40–60 %).
- **The one change:** `pointer_gate_supervision=W` (`StackModel::gate_supervised_loss`). Arms differ in nothing else: **S = 0.1 (pre-declared primary)**, **C = 0.0 (matched control)**, **H = 0.5**.

## Results

| reading | base (sealed / re-run) | S — primary | C — matched control | H — high dose |
|---|---:|---:|---:|---:|
| v5 memory `check_pass` (**bar ≥ 14/40**) | 10/40 | **21/40** ✅ | **22/40** | 19/40 |
| v5 unknowable `check_pass` | 1/24 | 16/24 | 19/24 | 16/24 |
| v5 panel `check_pass` / `acceptable` | 11 / 4 of 64 | 37 / 19 | 41 / 20 | 35 / 18 |
| **reply panel `fluent_and_relevant`** (**guard ≥ 29/232**, the sealed `acceptable` metric) | 43/232 | **28/232** ❌ by one cell | **22/232** ❌ | not run (pre-registration bounds this reading to S and C) |
| reply `heldout_first_turn` | 34/200 | 21/200 | 17/200 | — |
| reply `acceptable` with the 88 frozen row checks | 27/232 (re-graded) | 16/232 | 12/232 | — |

Paired exact McNemar on the 40 v5 memory rows: base → S **p = 0.0127**, base → C **p = 0.0018**, base → H **p = 0.0225**, **S vs C p = 1.00**. The base re-run on the same build **reproduces the sealed declared reading exactly** (10/40, 1/24, `acceptable` 4 and 0), and every derangement control is 0.

**Decision: primary bar MET, guard MISSED BY ONE CELL, artifact NOT KEPT, attribution REFUTED.** The pre-registration said *KEEP iff primary and guard hold*: 28 < 29, so it is not KEEP — stated with its width, because the paired test (base-only 27 / S-only 12, **p = 0.0237**; the control's 43 → 22, p = 0.0002) says the drop is real but the miss against the bar is one cell. The matched control is **at least as good on the memory half and no better on the reply half**, so the +11-row memory gain **and** the 15–21-cell reply cost are the **generated recall-dialogue mixture**, not `gate_supervised_loss` — the supervision is large and correct on the development stream (pointer hit rate 0.377 → 0.761 over the control) and buys nothing on either acceptance panel. **Criterion 1 remains NOT MET on both halves** (21/40 against ≥ 34/40; 28/232 against ≥ 116/232); the base artifact is unchanged and stays the base.

The read-level instrument was re-run with its three enforced checks and **reproduces its own published record on the base** (13 rows, 0 trace mismatches, cross-path delta 2.329e-06, 0 of 13 numeric rows put the value above the frame): after the fine-tune the copy channel's value share on the numeric rows roughly doubles (0.2361 → 0.50–0.59) and the **frame's share rises with it** (0.7430 → 0.90–0.98), so the pointer did **not** stop attending the sentence frame and the panel gain is not a read effect. The next M1 piece this result designs is a **mixture-dose experiment** with the reply panel as the pre-declared guard; `pointer_gate_supervision` is dropped from the recipe.

## Delivery

- Record: [`docs/labs/pointer-gate-finetune-2026-10-10/README.md`](docs/labs/pointer-gate-finetune-2026-10-10/README.md) — pre-registration, stream identities, all three arms, both panels per category, decision, limitations, cost, evidence.
- Newest-first [`docs/integration/current-state.md`](docs/integration/current-state.md) entry with the same numbers and the next step.
- [`STATUS.md`](STATUS.md): the completed experiment row and the M1 line-A cell.
- Evidence bundle on cloud-store: object `pointer-gate-finetune-2026-10-10`, **25,377,280 bytes, md5 `cb7da3a3286db49b0e48b7222970604a`**; source revision `5b241d89c…` ran on the pod, all acceptance scoring on the laptop.
- Checks: `python3 scripts/check_claim_wording.py` (docs-only change; no Rust path touched, so no compile or test is applicable beyond that).

## Checklist

- [x] Checks run at this exact head: `python3 scripts/check_claim_wording.py` (docs-only change; no Rust path touched, so no compile or test applies). The measurements' own checks — the v5 frozen row checks, the derangement controls (0), the corpus leak check against both frozen panels, the base's exact reproduction of both sealed readings (v5 10/40, 1/24, 4, 0; reply panel 43/232 with the same per-category split) and the instrument's three gates (0 mismatches, delta 2.329e-06) — are in the record.
- [x] Research record: `docs/labs/pointer-gate-finetune-2026-10-10/README.md`
- [x] New entry at the top of `docs/integration/current-state.md`, with **Next:**
- [x] Documents this change makes stale: `STATUS.md` (experiment row + the M1 line-A cell) and the #2028 milestone table (posted after the merge)
- [x] README: nothing a reader relies on changed (no capability, command or mechanism is added; the result is a measured negative on one mechanism and a measured positive on the data)
- [x] `python3 scripts/check_claim_wording.py` passes
- [ ] After merge: verify on fresh `origin/main`, delete the branch and worktree, `scripts/storage/uor-hygiene --apply`, comment the result on #2029

References #2029


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6094823187 — 2026-10-10T06:50:22Z — Casey-allard

**Validation at the exact head `4483b5e1d03d25cf92bd91634453beb5434a9263`** (docs-only change: one new record, one `current-state.md` entry, one `STATUS.md` cell; no Rust source, schema or CLI path is touched, so no compile or test applies).

| check | command | result |
|---|---|---|
| claim wording | `python3 scripts/check_claim_wording.py` | **pass** — "Claim-wording gate passed: no unlinked machine-verified/exact-equivalence wording." |
| base reproduction, v5 | `chat-grade grade-replies` on the base's own re-generated replies, frozen `conversational-v5-checks.tsv` | **10/40 memory, 1/24 unknowable, acceptable 4 and 0, panel 11/64** — the sealed declared reading, exactly |
| base reproduction, reply panel | `chat-grade grade-replies` on the sealed replies, same build | **43/232 fluent-and-relevant, per category 1 / 34 / 1 / 1 / 6** — the sealed ladder reading, exactly |
| instrument reproduction | `pcopy-mass-read` with its three gates on the base | 13 rows, **0 trace mismatches**, cross-path delta **2.329e-06**, 0/13 numeric above the frame, 21/27 word — the published record |
| panel structure | `chat-grade check … checks=data/panels/reply-panel-checks.tsv constants=data/panels/reply-panel-canned.txt` | `check_panel: pass`, 88 checked rows, **all 13 constants 0**, all adversarial abstentions 0, worst-case history 168/384 |
| corpus leak | `dialogue-recall-corpus generate` with `panels=` = `data/panels` + the three open-reply-panel request files | **`pass: true`** — 5,678 panel strings, 3,861 panel 6-grams, 0 matching examples in train or dev |
| derangement controls | every graded report | v5 **0** on all four models; reply panel 4 (base) / 11 (primary) / 3 (control) |

Evidence bundle (arm reports and curves, every acceptance report and reply file, the four instrument outputs, the row-by-row v5 verdict table, the corpus generator/leak reports and manifests, and the pod-side run scripts): cloud-store object `pointer-gate-finetune-2026-10-10`, **25,377,280 bytes, md5 `cb7da3a3286db49b0e48b7222970604a`**.



---
_Comment/review ids in this file: 6094823187_

