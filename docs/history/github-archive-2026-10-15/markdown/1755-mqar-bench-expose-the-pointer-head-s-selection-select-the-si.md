# #1755 — mqar-bench: expose the pointer head's selection (select=), the single-source pointer

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T20:13:45Z
- **Updated:** 2026-10-09T14:17:23Z
- **Closed:** 2026-10-06T22:52:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1755
- **Merged:** 2026-10-06T22:52:22Z
- **Merge commit:** 806899c53ad7f65f333b09acefa72efa57852b1b
- **Base / Head:** main / UOR-Foundation:pointer-select-arg
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## `mqar-bench`: expose the pointer head's SELECTION — and the gate result that matters more

**The bench set `select: None` unconditionally**, so the pointer's copy distribution was always a **soft average over every source**. The design doc calls **`TopK(1)` "the single-source pointer"** (`geometric_stack.rs:79-80`), and that configuration had **never been run on this instrument** — the same "implemented, never reached" pattern as the `pointer=` route in #1749.

### What this adds

`select=<spec>` (`none`, `top:<k>`, `flock:<window>:<k>`) via the existing `parse_pointer_select`. **Setting it enables the pointer head**, because a selection without a head is not a working configuration — the coupling `pointer=` already needed. Both are installed **only when asked for**, so existing arms are unchanged.

### First results, pre-registered

| arm | first_piece | full | full \| first | rule |
|---|---|---|---|---|
| control (no head) | 0.2941 | 0.0331 | 0.1125 | 0.623 |
| `select=top:1` | 0.2941 | 0.0322 | 0.1096 | 0.623 |
| `select=top:4` | 0.2698 | 0.0277 | 0.1028 | 0.623 |

**The single-source pointer moves nothing.** `first_piece` identical, `full|first` flat.

### And the check that matters more — read from saved weights

```
pointer.gate.bias  = -2.091391   ->  sigmoid = 0.1099
doc: b_g STARTS at -2            ->  sigmoid 0.119
```

**The copy gate begins 88% CLOSED and training did not open it — it closed slightly further.** So `p_copy` contributes ~11% of the mixture throughout, which is a plausible explanation for every null in this family.

**I tested opening it** (`POINTER_GATE_BIAS` −2.0 → 0.0, sigmoid 0.119 → 0.500; **reverted after measuring**):

```
gate OPEN (0.500)     first 0.2939   full 0.0339   full|first 0.1155
gate closed (0.110)   first 0.2935   full 0.0343   full|first 0.1168    <- IDENTICAL
```

**The gate is not the constraint either.** That is a **null I would not have been able to report** without this instrument change making the head reachable.

### Five changes, none moves the model

| changed | effect |
|---|---|
| key formation (`ngram:2/4`) | flat → worse |
| read-out selection (`top:1/4`) | **flat** |
| pointer head + route | **flat** |
| copy gate (0.11 → 0.50) | **flat** |
| seed / duration | **flat** (5 seeds 0.272–0.295; 600 steps 0.2952 vs 1200 steps 0.2949) |

**`first_piece ≈ 0.29` and `full|first ≈ 0.11` are invariant to all five, while a NON-LEARNED rule holds 0.610–0.644 — a 19× gap.** The model converges, does not improve with steps, and cannot be moved from either side of the read.

### Checks

```
cargo fmt -p uor-r4-training -- --check     clean
git diff --name-only origin/main..HEAD      crates/uor-r4-training/examples/mqar-bench.rs   (1 file)
select=top:1 smoke run                      accepted, runs
```

**Per [D20](docs/integration/DECISIONS.md) §4 these are localisations, not verdicts.** Nothing retired — the pointer, the `ngram` route and the gate all remain live candidates.

**Instrument only.** No model, no serving path, no existing arm behaviour changed. Two unrelated `cargo fmt` reformattings (a local rustfmt version difference) were **deliberately excluded** to keep the diff to one file.

Refs #820, #1552, #1716, #1749.


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6002171776 — 2026-10-05T20:15:39Z — Casey-allard

## Exact-head self-review — `d4556ae3a3b43ebe3253401fcb588baf118d18c3`

`local == remote` verified. Self-review permitted under the owner's current exact-head rule.

### Checks

```
1. git diff --name-only origin/main..HEAD   crates/uor-r4-training/examples/mqar-bench.rs   (1 file)
2. cargo fmt -p uor-r4-training -- --check   clean
3. cargo build --release --example mqar-bench OK
4. select=top:1 smoke run                    accepted, executes
5. fmt / clippy / tests / no_std / κ         PASS
6. local head == remote head                 d4556ae3…  OK
```

### Verified by reading the diff

- **Default is inert.** `common.select` is `None` unless `select=` is passed and the new block is `if common.select.is_some() || common.pointer.is_some()`, so **every existing arm takes exactly the old path**.
- **Head and selection are wired together, deliberately.** A selection with no head is not a working configuration — the same coupling `pointer=` needed in #1749, where a route without a head failed at runtime with `"the model has no pointer head to route"`. Both are installed only when asked for.
- **Parsing reuses `parse_pointer_select`**, so `none`/`top:<k>`/`flock:<window>:<k>` behave consistently with every other binary; no new grammar.
- **The gate test left no residue.** `POINTER_GATE_BIAS` was moved −2.0 → 0.0 for one measurement and **reverted**; confirmed back at −2.0 and `geometric_stack.rs` is unmodified.
- **Scope held to one file.** Two unrelated `cargo fmt` reformattings of files I never edited (`relation_compiler.rs`, `cuda_stack_ops_parity.rs` — a local rustfmt version difference) were excluded. **Third time today `git add -A`/fmt has tried to widen a diff; caught each time.**

### On the result

**The PR's value is that it made a configuration reachable, and the reachable configuration produced the session's most informative null.** Reading `pointer.gate.bias = -2.0914` (sigmoid **0.110**) from the saved weights of a run with the head, against a documented start of −2.0 (sigmoid 0.119), is what showed **the copy gate begins 88% closed and training never opens it**. Testing that — by moving the bias to 0.0 (sigmoid 0.500) — gave `full|first` 0.1155 against the closed gate's 0.1168: **identical**.

**So the gate is not the constraint, and that null is only obtainable because this PR exists.** It also means the null is *trustworthy*: it comes from a head that is genuinely installed, genuinely selected, and genuinely gated open — not from a mechanism that silently never ran.

### Scope

**Instrument only.** No model, no serving path, no existing arm behaviour changed. Per D20 §4 the results are localisations, not verdicts, and nothing is retired.

**Reviewing at `d4556ae3`. Approving and queueing.**


### comment 6026898916 — 2026-10-06T22:51:31Z — Casey-allard

Exact-head self-review at `24735794bc5d9ed351e8ef98188f155a725dc4e3` (owner rule: self-review allowed). Merged origin/main (no conflicts). Checks at this head: `cargo check -p uor-r4-training --example mqar-bench` 0 errors; release build + 2-step CPU smoke with `select=top:1` and `select=none` both complete, `select=top:1` recorded in config/report and changes the read (max |dlogit| 0.010118 vs 0.010112); rustfmt drift reported in this file is pre-existing main code (lines outside this PR's 4 hunks). APPROVE.


---
_Comment/review ids in this file: 6002171776, 6026898916_

