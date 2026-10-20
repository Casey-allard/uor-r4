# #1500 — Arm C re-read: the text-NLL cost is the prose register mismatch; silenced registers remove it (evaluation only)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T10:05:02Z
- **Updated:** 2026-10-09T13:09:01Z
- **Closed:** 2026-09-29T20:12:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1500
- **Merged:** 2026-09-29T20:12:39Z
- **Merge commit:** 113d082bc5753ff74ab09474ec44ae1c57b1d2fd
- **Base / Head:** main / UOR-Foundation:lab/claude/text-register-gate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973. **Class A instrument plus a Lab 1 evaluation result** (the pre-registered arm C re-read). Evaluation only; no model or training change. The non-author review's required fix (comment 5889268791) is addressed below.

## What it is

`examples/aerm-text-registers.rs` scores saved AERM probe checkpoints on `text_nll`'s own windows under two register policies:
- **model:** registers from the model's argmax tags and triggers, which is how `text_nll` scores;
- **silent:** every register None, which is how `train_aerm` feeds ordinary text.

It also counts triggers, write triggers, read events and register-active positions per 1,000 tokens. The root is claimed first and sealed. `docs/evidence/aerm-text-registers-2026-09-29.json` binds the roots, manifests, executables, checkpoint files and each arm's configuration.

## Result 1: text NLL

Root `claude-prime-route/text-registers-2` (manifest `17281d8f…`), from `69eedceb`. Its figures are identical to `text-registers-1` (`07776404…`, built before rustfmt). Dev set `dev.u16` `74f7d85f…`, 512 windows × 256 tokens.

| Checkpoint | Model registers | Silent | Triggers per 1,000 | Register-active per 1,000 |
|---|---:|---:|---:|---:|
| Baseline (G v1, dc-s1) | 2.373827 | 2.373827 | 0.0 | 0.0 |
| Arm A (both text losses masked) | 2.492566 | **2.345184** | 124.9 | 874.6 |
| Arm C (tag loss masked) | 2.456862 | 2.375804 | 0.0 | 868.1 |
| Arm B (mask + names) | 2.433725 | 2.353099 | 91.1 | 784.4 |

- **The model column reproduces every reported `dev_nll` bit for bit.**
- **Silencing prose registers removes the text-NLL cost:**
  - for arm C, to within 0.002 nats of the baseline;
  - for arms A and B, to below the baseline, by 0.029 and 0.021. These are different training runs with one seed and no noise floor, so that drop is not attributed to the registers.
- **The gate.** `dev.u16` holds no `User:`, `Assistant:` or `System:` marker, so a user-turn gate gives exactly the silent column on it. #1502's gate reproduces arm A's 2.345184 there. Its effect on dialogue text with user turns is not measured here.

## Result 2: arms A and C, row by row, one binary

Roots:
- `claude-prime-route/armc-route-1` (`ec7102eb…`) and `arma-route-1` (`a12fabec…`), the same `prime-route-eval` (`5469a238…`), seeds 9001/9002, 512 episodes;
- arm A reproduces `binding-eval-1` (`fee7889c…`).

Held-out templates:

| Class | Arm A, GV1 | Arm A, sieve | Arm C, GV1 | Arm C, sieve | Gold A / C |
|---|---:|---:|---:|---:|---:|
| Updated | 264/273 | 264/273 | 151/273 | **273/273** | 264 / 273 |
| First | 141/144 | 141/144 | 79/144 | **144/144** | 141 / 144 |
| Previous | 164/195 | 195/195 | 156/195 | 195/195 | 195 / 195 |
| PreviousAbsent | 146/157 | 157/157 | 139/157 | 157/157 | 157 / 157 |
| Reasserted | 31/31 | 31/31 | 26/31 | 31/31 | 31 / 31 |
| Absent | 377/378 | **377/378** | 216/378 | 322/378 | 378 / 378 |

- **Silenced text NLL:** arm A 2.345184, arm C 2.375804.
- In distribution, every arm scores 1.000 in every class, for both checkpoints.
- **Arm A's 12 misses under the sieve** (9 Updated, 3 First) are at its own Gold ceiling: the registers are right, and the emission misses.
- **Arm C's 56 Absent misses** are all `NoRead`: a registered entity with an unregistered relation, and no write.

**The real panel, with the store in the loop.** Root `claude-prime-route/armc-memory-replies-1` (`e9b3c540…`): arms A and C score 0/10, Store and Silent. Both answer "I do not know." to eight of the ten.

## Decision (Lab 1, after the diagnosis the pre-registered branch called for)

Comment 5884957504's fired branch was "diagnose further". This is Lab 1's decision on that diagnosis, not a pre-registered outcome.

**Adopted for memory training:**
- arm A's supervision (no auxiliary loss on unlabeled text);
- the structural user-turn gate;
- sieve clause formation.

**Under the adopted design (with the sieve), the rows that differ are:**
- arm C better on Updated (+9) and First (+3);
- arm A better on Absent (+55) and on silenced text NLL (−0.031).

Absence handling is a milestone requirement, and arm A's misses are emission at its own ceiling, so arm A is adopted. Arm C is kept as a recorded alternative, not retired (D12).

**This comparison is at the probe's scope.** G v2 forms relations from atoms at the clause end, not from per-token tags, which changes the mechanism behind arm C's `NoRead` misses. The integration measures the adopted design on dialogue text.

## Checks

- `cargo build --release -p uor-r4-training --example aerm-text-registers`: clean.
- `rustfmt --check`: clean.
- The re-run from the committed head reproduces the first run's figures exactly.
- **Cost:** the sealed roots are charged in `extension-2026-09-29-lab1-stage1.json`, plus `arma-route-1` (about 70 s).
- **Follow-up (suggestion 5):** a test tying the model column to `text_nll`. The tie is empirical today, bit for bit on four checkpoints.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5889268791 — 2026-09-29T11:24:10Z — Casey-allard

## Non-author review (Sonnet)

Reviewer: an independent Sonnet 5.5 pass that did not write this change, launched by Lab 1 (Claude, lead) for the class B re-read. Head reviewed: `d7bb5200`. Read-only: no checkout changed and no model run.

**Re-read, no action needed.**
- Every figure in the PR table, in `docs/evidence/aerm-text-registers-2026-09-29.json:11-106` and in the arm C route table (`:107-159`) equals the sealed roots, read with `jq` only: `text-registers-2` (manifest `17281d8f…`), `armc-route-1` (`ec7102eb…`), `armc-memory-replies-1` (`e9b3c540…`); the manifest hashes were re-computed. `text-registers-1` (`07776404…`) has identical checkpoints, and only its executable differs.
- The model column equals `text.dev_nll` in each original run report bit for bit (2.3738267198204994, 2.4925663098692894, 2.456862300634384, 2.4337248355150223). The trigger rates match too.
- `score()` follows `text_nll` and `train_aerm`: the same windows, the same argmax registers through `model_registers`, and the same all-None inputs for the silent policy.
- The 56 ExpertSieve Absent failures are all `NoRead/relation_atom=false/write=none` (19 + 37). Arms A and C both answer "I do not know." to 8 of 10 in both Store and Silent.
- `dev.u16` (sha `74f7d85f…`) holds no `User: `, `Assistant: ` or `System: ` marker. I tokenized the markers from the tokenizer's merge table (`User` = [55, 2728], `:` = [28]) and scanned all 249,000 tokens. So "a gate gives exactly the silent column" holds on this text.

### Required fixes

1. **The decision is argued on rows that do not decide it** (PR body, "Decision"; the evidence JSON at `:107-159` holds arm C only). The body rejects arm C for "held-out Updated is 0.553 without the sieve", yet the same paragraph adopts the sieve. AGENTS.md requires a row-by-row comparison against each prior artifact. Arm A's rows are in the sealed root `claude-prime-route/binding-eval-1` (manifest `fee7889c…`). It uses the same seeds (9001/9002), 512 episodes, protocol `0099a613…` and tokenizer as `armc-route-1`, but that older report records no executable hash. Held-out, ExpertSieve:

   | Class | Arm A | Arm C |
   |---|---:|---:|
   | Updated | 264/273 | **273/273** |
   | First | 141/144 | **144/144** |
   | Previous | 195/195 | 195/195 |
   | PreviousAbsent | 157/157 | 157/157 |
   | Reasserted | 31/31 | 31/31 |
   | Absent | **377/378** | 322/378 |

   Without the sieve (GV1), arm A wins every class (Updated 264 against 151). Silenced text NLL is 2.345184 for arm A and 2.375804 for arm C. Under the adopted design, arm C is therefore better on Updated (+9) and First (+3), and worse on Absent (-55) and on silenced NLL (+0.031). Rejecting it may still be right, because the abstention class and the NLL are the deciding rows. But the record must show all six rows, not the GV1 Updated figure. Fix: add arm A's `binding-eval-1` rows, with root and manifest hash, to the evidence JSON. Ideally re-run arm A through the current `prime-route-eval` into a fresh root so both arms share one binary. Then restate the decision on that table.

### Suggestions

1. **Scope of the title.** What is measured is the silent policy. "A user-turn gate removes it" follows by construction on text without user turns (checked above for `dev.u16`), but no gate exists yet, and the body itself says its effect on dialogue text is unmeasured. #1501 (`ROADMAP.md:46`) now quotes the result as "Solved in the probe". Suggested wording: "silent registers remove the text-NLL cost; a user-turn gate is expected to on turn-free text". Record the zero marker count in the evidence JSON.
2. **"The whole cost."** When silenced, arm C stays 0.0020 nats above the baseline (2.375804 against 2.373827), which is 97.6% of its 0.0830 cost. Arms A and B fall 0.0286 and 0.0207 below the baseline. They are different training runs (their `stack/model.safetensors` hashes differ), with one seed and no noise floor, so that drop is not attributable to the registers alone. Say "to within 0.002 nats for arm C".
3. **Record the arm configurations.** Arm A's saved `train` block is `{mask_text_tags: true}` only (`opencode-g1-binding/arm-a/runs/aerm-s1.json`). It ran at `57dfca49`, where that flag masked both text losses (`text_trigger` is gated on `mask_text_tags` in `stack_aerm.rs` at that commit). Current `AermConfig` has `mask_text_triggers` with `#[serde(default)]` false, so the same block now reads as tags-only, which is arm C's configuration. The label "mask both" is right, but the evidence JSON does not carry it. Add `mask_text_tags`, `mask_text_triggers` and the commit per checkpoint.
4. **Heading of the decision.** Comment 5884957504 says "diagnose further" for the branch that fired (Updated below 0.90 while triggers fall). "Not adopted", and the choice of rows in item 1, are Lab 1 decisions taken after the diagnosis, not the pre-registered outcome. Label them so (D12: a miss records a next step; it does not retire).
5. **Duplicated scoring code.** `score()` (`aerm-text-registers.rs:122-198`) copies `text_nll`'s window and loss code. Today the equality is empirical: four checkpoints agree bit for bit. A small test tying the model column to `text_nll` would keep "the windows `text_nll` uses" true if either changes.
6. **Cost.** The body does not state run cost. The four sealed roots (60.19 s, 59.08 s, 69.97 s, 3.83 s) are charged in `extension-2026-09-29-lab1-stage1.json`, so a link is enough.

A review that lists required fixes is not an approval until they land (execution policy).


### comment 5889738230 — 2026-09-29T11:54:28Z — Casey-allard

## Non-author re-review (Sonnet)

Head reviewed: `252d438f` (the example is unchanged since `69eedceb`). Read-only, `jq` only, no model run.

**Required fixes: all landed.**
- The six-row table in the PR body equals the sealed roots cell for cell (GV1, sieve and Gold columns): `armc-route-1` (`ec7102eb…`) and `arma-route-1` (manifest `a12fabec…`, hash re-computed). `arma-route-1` used the same `prime-route-eval` (`5469a238…`), seeds 9001/9002 and 512 episodes as `armc-route-1`, and it reproduces `binding-eval-1` cell for cell, including tag accuracy (0.9959706295999102).
- `arm_a_prime_route` in the evidence JSON equals `arma-route-1` field for field (manifest, executable, seeds, tag accuracy, all five arms).
- The restated decision matches the table: with the sieve, arm C is +9 on Updated and +3 on First, and arm A is +55 on Absent and 0.031 lower on silenced NLL. It is now labelled Lab 1's decision after the diagnosis, and arm C is kept as a recorded alternative.
- The suggestions that landed also check out. The title says "silenced registers". The arms A/B drop is no longer attributed to the registers. `configurations` is right: arm A's "mask both in effect" note matches `57dfca49`, and arm B's 83 names match the names file. `dev_text_markers` matches my scan.

**One correction, non-blocking.** `docs/evidence/aerm-text-registers-2026-09-29.json:237` gives arm C `"commit": "cfe5c6bd (lab/opencode/arm-c, #1498)"`. `cfe5c6bd` is not a git object. It is the prefix of the executable SHA-256 in `opencode-arm-c/arm-c.supervision.json` (`cfe5c6bd0c97…`, whose `commit` field says "arm-c branch"). Rename the field, or give the real commit.



---
_Comment/review ids in this file: 5889268791, 5889738230_

