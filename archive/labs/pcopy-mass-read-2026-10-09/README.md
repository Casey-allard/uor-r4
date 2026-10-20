# The copy mass, read on the public path: the gate validates 166/166, the frame carries the mass, and the digits get it only where the pointer already looks

Lab: deepseek. Issue: [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029). Scope: **CPU only,
no pod, no training, no knob, no model saved, no v5 re-run, $0.** Result: the reading needed **a caller,
not an accessor** — `StackModel::score_targets` already returns `PointerRowStats::copy_mass`, and it is
already called on the reply path. Its `hit`/`reachable` agree with the recorded per-step trace on
**166 of 166 steps across 13 rows**, so the path is the decoder's own mixture. On that path the copy mass
follows the attention exactly: **0.60–0.90 on the 3 rows whose argmax lands on a stored digit, ≤ 0.20 on
the other 10 rows against a frame baseline of 0.34–1.00.** It lands **between** the first two
pre-registered outcomes, and the split is exactly the already-measured selection line.

## What the public surface gave, before any new code

Read first, as this line has learned to do:

| surface | what it is |
|---|---|
| `geometric_stack.rs:15827` `PointerRowStats { gate, copy_mass, hit, reachable }` | `copy_mass` is documented as "`p_copy(target \| t)` before the gate"; `hit` = the most-attended source holds the target (lowest on a tie); `reachable` = any source with attention holds it |
| `geometric_stack.rs:15843` `TargetScores { nll, pointer }` | one `Option<PointerRowStats>` per scored position |
| `geometric_stack.rs:8309` `StackModel::score_targets` | already scores arbitrary targets per position |
| `stack_dialogue.rs` `development` | **already calls it on the reply path** |

**So the previous piece's Site-2 accessor was not needed for this reading.** What was missing is exactly
what the brief said: a caller. It is
[`crates/uor-r4-training/examples/pcopy-mass-read.rs`](../../../crates/uor-r4-training/examples/pcopy-mass-read.rs),
~200 lines, no model code:

```text
pcopy-mass-read model=DIR tokenizer=T.json panel=PANEL.json checks=CHECKS.tsv \
  replies=REPLIES.json out=OUT.json [trace=TRACE.json] [frame=ID,ID,...] [rows=all|numeric|word]
```

It rebuilds each row's sequence exactly as the graded reply ran it — `history_ids` ++ the sealed last
turn's reply ids, **both from the sealed acceptance record, so nothing is generated** — locates the
value's own tokens by text on the tokenizer's decode, and calls `score_targets` once per panel target,
overriding the target at the probe positions (the value's span, the distractor's span, every reply
step). The forward pass is identical in every call, so the whole (target × position) grid costs one
forward per target.

| identity | value |
|---|---|
| model | `~/uor-r4-local/reply-cap96-inputs/ladder/runs/chat-29m-B-lr5e-4/model/model.safetensors`, 115,847,468 B, sha256 `d8a3c971ef07a82ca0db0704b34821e24782c7734baaafb504fd754f82988068` — width 576, pattern `rrarrarrar`, `pointer {dim 32, init_seed 1}`, and the same sha as the sealed v5 record's `model_sha256` |
| sealed replies | `replies-29m.json` sha256 `fd64bdff…`, from `icloud:UOR-R4/results/deepseek/v5-declared-acceptance-2026-10-09.tar` |
| trace | `full-per-step-arm-a.json` sha256 `f1ecccad…`, from `icloud:UOR-R4/results/deepseek/pointer-positional-read-2026-10-09.tar` (**control arm**, default knobs) |
| panel / checks / tokenizer | `conversational-v5.json` `e766cfe9…`; `conversational-v5-checks.tsv` `737c4dfd…`; tokenizer `d36d3e87…` |

## The gate, pre-registered before the run, and it passes

> on the natural path (target = the sealed next token) `hit` must equal the recorded trace's
> `matches_source` at every traced step; any mismatch makes the run VOID and no panel number is reported

**13 rows, 166 traced steps, 0 mismatches.** The public scored path reproduces the decoder's own
per-step selection exactly, so `copy_mass` read here is the mixture that produced the v5 replies. The
program enforces this: on any mismatch it writes the mismatches, prints `VOID` and exits without a
panel number.

## The measurement, per row

**Numeric 13 rows.** `self` is `p_copy` of the id *sitting at* that position; `reply steps` aggregates
the row's own reply steps — the largest copy-mass share over the value's span ids, the largest over the
9 frame ids, and how many steps the value wins. Full table, both occurrences and every step:
[the evidence transcript](../../evidence/pcopy_mass_read_2026-10-09.txt).

| row | value | self mass at the value's occurrences (hit) | reply steps: max value | max frame | value wins | digits selected / emitted |
|---|---|---:|---:|---:|---:|---:|
| mem-005 | 41 | 0.142 (F), 0.057 (F) | 0.021 | 0.996 | 0/14 | 0 / 0 |
| mem-006 | 75 | 0.428 (F), 0.166 (F), 0.349 (F), 0.007 (F) | 0.167 | 0.431 | 6/15 | 0 / 0 |
| **mem-008** | 68 | **0.988 (T), 0.825 (T)**, 0.107 (F), 0.003 (F) | **0.897** | 0.983 | 2/11 | **2 / 0** |
| mem-010 | 85 | 0.994 (T), 0.780 (T) | 0.104 | 0.679 | 4/11 | 0 / 0 |
| mem-013 | 88 | 0.322 (F), 0.559 (T) | 0.201 | 0.676 | 5/12 | 0 / 0 |
| mem-021 | 71 | 0.014 (F), 0.035 (F) | 0.040 | 0.996 | 0/14 | 0 / 0 |
| mem-022 | 89 | 0.630 (T), 0.576 (T), 0.775 (T), 0.001 (F) | 0.188 | 0.493 | 6/15 | 0 / 0 |
| **mem-024** | 54 | **0.981 (T), 0.899 (T)**, 0.190 (F), 0.059 (F) | **0.717** | 0.982 | 2/11 | **2 / 0** |
| mem-026 | 99 | 0.999 (T), 1.000 (T) | 0.072 | 0.691 | 4/11 | 0 / 0 |
| mem-029 | 83 | 0.530 (T), 0.471 (T) | 0.095 | 0.630 | 9/12 | 0 / 0 |
| mem-037 | 98 | 0.316 (F), 0.414 (T) | 0.010 | 0.997 | 0/14 | 0 / 0 |
| mem-038 | 71 | 0.428 (F), 0.362 (F), 0.296 (F), 0.001 (F) | 0.112 | 0.374 | 5/14 | 0 / 0 |
| **mem-040** | 84 | **0.886 (T), 0.709 (T)**, 0.070 (F), 0.005 (F), 0.024 (F), 0.003 (F) | **0.601** | 0.732 | 2/12 | **1 / 2** |

Three facts, and they line up exactly:

1. **The copy mass follows the attention.** The three rows whose value share reaches 0.40 or more —
   `mem-008` 0.897, `mem-024` 0.717, `mem-040` 0.601 — are **exactly the three rows whose trace selects a
   stored digit** (2, 2 and 1 steps). Every other row is ≤ 0.20. The mixture is a faithful function of
   where the head looks.
2. **The frame wins on all 13 rows** (0/13 have the value above the frame at the row's best step), and on
   the 10 non-selecting rows the frame's share at reply steps is **0.34–1.00** against the value's
   **0.0098–0.20**.
3. **The high "self" mass is position-specific, not a property of the read.** At its *first* occurrence
   the value's digit gets 0.32–1.00 with `hit` true on 11 of 13 rows — but at its *last* occurrence the
   same rows collapse (`mem-008` 0.107/0.003, `mem-024` 0.190/0.059, `mem-040` 0.070/0.005, `mem-006`
   0.007) and at the reply steps it stays low except on the three selecting rows. `hit` true at the
   current position is consistent with attending **the token being read**; the instrument cannot separate
   that from another occurrence of the same id (limitation 2).

At the value's own slot the copy mass does not even separate the value from the distractor: on 4 rows the
distractor's span carries strictly more mass there (`mem-005` 0.854 vs 0.142, `mem-021` 0.983 vs 0.014,
`mem-013` 0.479 vs 0.322, `mem-037` 0.630 vs 0.316), and on 2 more they coincide because the value and
distractor share a digit (`mem-022`, `mem-029`).

**Word control (27 word-valued memory rows, same path and positions).** 21 of 27 put their value span
above the frame at its best step and 16 of 27 reach ≥ 0.40; 9 of 27 emitted the value. The contrast with
the numeric rows (0 of 13 above the frame) says the copy channel is not broken in general — it carries
content when the head looks at it. Caveat, stated because it matters: word spans are multi-token and a
subword that also occurs elsewhere inflates the share (only one word value, `Polly`, is a single token),
so the control is directional, not exact.

**Correction (2026-10-09), from the [per-position re-read](../pcopy-posread-2026-10-09/README.md), which
lifted that caveat.** The id-based share was inflated on 5 of these 27 rows — `Howard` 14.05× (0.0371 at
its own positions against 0.5219 by id), `Mitchell` 3.96×, `Iowa` 1.26×, `Lloyd` 1.25×, `Wichita`
1.07× — and on 4 of the numeric rows (`mem-021` 9.45×, `mem-022` 1.90×, `mem-005` 1.70×, `mem-029`
1.24×). **Corrected numbers: 21 of 27 word rows still put the value above the frame (unchanged, and 0 of
13 numeric), but "16 of 27 reach ≥ 0.40" becomes 13 of 27**, and no row's value-versus-frame verdict
flips between the two measures. Quote the **per-position** share from here on; the id share stays
published beside it as a labelled secondary with its inflation factor.

## The bound, beside the measurement

`p_copy(digit) ≤ attention at the argmax` on steps where the argmax is elsewhere:

- **It is not a valid bound.** 5 of 166 steps violate it (one step each on `mem-013`, `mem-022`,
  `mem-024`, `mem-038`, `mem-040`; e.g. `mem-024` reaches `p_copy = 0.368` where `a_max = 0.286`).
- **The correct ceiling is tight in form and loose in fact**: `p_copy(target) ≤ 1 − attention(argmax)`
  holds at **0 violations**, and the largest fraction of that ceiling used anywhere is **0.52**.
- Where the published bound does hold, it is tight on 7 rows (`p/a ≥ 0.5`) and very loose on 6
  (`p/a` down to 0.036).

So the trace's attention alone **cannot** be used as a ceiling for a digit's copy mass; the honest
ceiling is `1 − a_max`, and it does not bind.

## The three pre-registered outcomes

| outcome | verdict |
|---|---|
| **1. copy_mass HIGH at the digits → nothing to train; the emitter is the target** | **true for 3 of 13 rows**, and exactly the 3 rows whose argmax already lands on a digit. Two of them still emit nothing, which is the emitter-side reading in its smallest measured form |
| **2. LOW at the digits against a HIGHER BASELINE ELSEWHERE → the frame reading survives** | **true for the other 10 of 13**: ≤ 0.20 at every reply step against a frame share of 0.34–1.00 |
| **3. LOW EVERYWHERE → the gate is broadly weak** | **refuted**: the frame class carries 0.34–1.00 |

**It lands between 1 and 2, split exactly along the selection line**, and that is the result rather than
a picked one. Stated in one sentence: *the copy channel carries whatever the pointer attends, and on
these 13 rows the pointer attends the frame — so on 10 rows the read does not carry the value at all,
and on the 3 rows where it does, the decoder still fails to emit it twice out of three.*

## What this does to the plan's decision rule

`pointer-target-change-plan-2026-10-09` says: *"If `p_copy` is already high there, the training piece is
unnecessary and the emitter is the target. Only if it is low does the single-rung labelled training run
become the next step."*

The rule is satisfied **at the value's first occurrence** (high, `hit` true on 11 of 13 rows) and **not
satisfied at the position where the answer is produced** (low against the frame on 10 of 13). So the rule
needs the position named before it can decide anything, and with the position named — the reply step that
produces the answer — its own test says the read is still the constraint on 10 of 13 rows. The plan is
**not** shelved under "the read said nothing to train": that third reason is earned only on the 3
selection-positive rows, where it is the emitter that drops the digit. Both facts are needed, and neither
alone decides the run.

## Decision

**KEEP as the measurement the plan's step 1 asked for**: the copy mass is now readable on the public
scored path, per row and per position, with the trace gate passing 166/166. **No capability change, no
model trained or saved, no knob set, criterion 1 remains NOT MET on both halves, 43/232 is unchanged, v5
was not re-run, and STATUS/ROADMAP/#2028 are unchanged** because no capability, served model or milestone
moved.

## Limitations

1. **13 numeric rows, one artifact, one arm, one knob setting.** The control arm at default knobs; no
   floor sweep, no other panel, no other artifact.
2. **The copy distribution is over token ids, not positions**, so `p_copy` cannot separate the value's own
   occurrence from another occurrence of the same id. That is why the report gives *every* occurrence and
   says which reading is which; it cannot prove the "self" reading at the first occurrence.
3. **The word control is directional.** Multi-token spans include subwords that recur elsewhere in the
   window, which inflates the share; only `Polly` is a single-token value.
4. Only the 13 numeric rows have a recorded trace, so only those 13 have the per-step gate. The word rows
   ride the same code path (validated on the 13) but are not independently validated per step.
5. `share` sums `p_copy` over distinct ids: where the value and the distractor share a digit the two
   shares coincide by construction (`mem-022`, `mem-029`), and the "shares" are not a partition of the
   copy distribution (the remaining vocabulary is unmeasured).
6. **No v5 re-run, no judge, no `check_pass` re-measurement.** This is a mechanism read-out.

## Cost

Zero pod spend, zero external compute. One release build; the numeric run **7 s**, the all-rows run
**65 s** (71 s wall) on the laptop CPU, peak RSS under 2 GB. Scratch and outputs live in this worktree's
gitignored `local/`.

## Evidence

[`docs/evidence/pcopy_mass_read_2026-10-09.txt`](../../evidence/pcopy_mass_read_2026-10-09.txt) carries the
identities, the method, the gate, Table A (numeric, per position), Table B (word control), Table C (the
bound), the three outcomes against the numbers, and the verbatim program output.

The run outputs — the program's stdout, the full (target × position) grid per row, the analysis script and
its tables, and the record as it stood at upload — are bundled at
**`icloud:UOR-R4/results/deepseek/pcopy-mass-read-2026-10-09.tar`** (object `pcopy-mass-read-2026-10-09`,
**6,560,256 bytes, md5 `4c79f7ea3cd3facdd6fde44fb85f5383`**), fetched back and MD5-verified with
`out/all.json` re-hashing to `e3ea8b69…` on both sides. Restore with
`cloud-store fetch pcopy-mass-read-2026-10-09 <dest>`.
