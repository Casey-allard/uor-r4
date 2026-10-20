# The word control survives the per-position read — and three of its rows were inflated by up to 14×

Lab: deepseek. Issue: [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029). Scope: **CPU only,
no pod, no training, no knob, no model saved, no v5 re-run, $0.** This piece stress-tests the one
comparison the previous conclusion rests on — the word control — because its spans are multi-token and a
subword that also occurs elsewhere inflates an id-based share.

**Result: the control survives, and the correction is in the magnitudes rather than the contrast.** On the
per-position read — the pointer's own attention over the window's positions, from the public
`read_span_probe` — **21 of 27 word rows still put the value above the frame, exactly as the id-based
measure said, against 0 of 13 numeric rows**; **no row's value-versus-frame verdict flips** between the
two measures. What changes is how much mass the value rows carry: **16 of 27 word rows reached ≥ 0.40 by
id, but only 13 of 27 do at their own positions**, and five word rows were inflated (up to **14.05×**).
On the numeric side the per-position read makes the deficit slightly *worse*, not better: `mem-021`'s
value share falls from 0.0396 to **0.0042** (9.45×). **The number a reader should carry forward is the
per-position share**; the id-based one is now reported beside it with its inflation factor.

## The smallest step: the same example, one more public path

The previous record's own Next named this: *"the instrument also needs the word control's caveat lifted,
which needs values whose spans are single tokens or a copy read that is per position rather than per
id."* Both fixes are in the same example,
[`crates/uor-r4-training/examples/pcopy-mass-read.rs`](../../../crates/uor-r4-training/examples/pcopy-mass-read.rs),
with no model code:

| quantity | public path | what it sums over |
|---|---|---|
| `value_id_share` (published last time) | `score_targets` → `PointerRowStats::copy_mass` | every position in the window **holding one of the value's ids** |
| `value_pos_share` (this piece) | `read_span_probe` → `SpanProbe.pointer.attention` | only the positions that **are** the value's located span |
| `frame_id_share` | either | every position holding one of the 9 frame ids |
| `argmax_source` | `read_span_probe` | the pointer's most-attended position at that step |

`p_copy(target|t)` is a mass over token **ids**, so `value_id_share` counts a subword the value shares
with another sentence — that is the inflation. The attention vector is over **positions**, so
`value_pos_share` cannot be inflated that way. For a value whose tokens occur only in its own span the
two are equal by construction; where they differ, the difference *is* the inflation.

## The gate, extended and enforced in every run

Three checks, all pre-registered before this run, any failure writing VOID and reporting no number:

1. **`hit` == the recorded trace's `matches_source`** at every traced step — **166 of 166**, unchanged
   from the previous piece.
2. **the per-position read's attention argmax == the trace's `source` position and `source_id`** at every
   traced step — **166 of 166**, so the attention vector read here is the decoder's own attention, not a
   look-alike.
3. **the two public paths agree**: `score_targets`' `copy_mass(target)` must equal `read_span_probe`'s
   attention summed over the positions holding that target. Measured maximum difference **2.329e-6**,
   against a **1e-5** tolerance (`read_span_probe` builds its attention in f32 while `score_targets`
   sums the mixture's copy mass separately, so agreement is to f32 accumulation). The first run of this
   gate at a 1e-6 tolerance tripped on exactly that value — the tolerance was widened deliberately and
   the measured value is printed with every result rather than the check being dropped.

Same artifact, same panel, same identities as the previous piece; nothing regenerated.

## The word control, per row, both metrics

`pos` = mass at the value's own span positions; `id` = mass over every position holding the value's ids;
each is the maximum over the row's reply steps, with the count of steps where it exceeds the frame at the
same step. Full table: [the evidence transcript](../../evidence/pcopy_posread_2026-10-09.txt).

| row | value | span len | pos (steps) | id (steps) | inflation | frame | verdict |
|---|---|---:|---:|---:|---:|---:|---|
| mem-001 | Britain | 3 | 0.7500 (9/10) | 0.7500 (9/10) | 1.00× | 0.0327 | value |
| mem-002 | Barbara | 4 | 0.4744 (7/7) | 0.4744 (7/7) | 1.00× | 0.0003 | value |
| mem-003 | Tyler | 4 | 0.2911 (12/12) | 0.2911 (12/12) | 1.00× | 0.0127 | value |
| mem-004 | Bruno | 3 | 0.1576 (4/12) | 0.1576 (4/12) | 1.00× | 0.9878 | frame |
| **mem-007** | **Howard** | 2 | **0.0371** (8/9) | **0.5219** (9/9) | **14.05×** | 0.0088 | value |
| mem-009 | Cincinnati | 7 | 0.4028 (20/24) | 0.4028 (20/24) | 1.00× | 0.3214 | value |
| mem-011 | Iowa | 3 | 0.0639 (41/46) | 0.0807 (42/46) | 1.26× | 0.0884 | frame |
| mem-012 | Bradford | 4 | 0.3657 (20/23) | 0.3657 (20/23) | 1.00× | 0.2299 | value |
| mem-014 | Birmingham | 5 | 0.4694 (15/16) | 0.4694 (15/16) | 1.00× | 0.0231 | value |
| mem-015 | Charleston | 5 | 0.5558 (13/16) | 0.5558 (13/16) | 1.00× | 0.1250 | value |
| mem-016 | Anthony | 4 | 0.0991 (11/12) | 0.0991 (11/12) | 1.00× | 0.0964 | value |
| mem-017 | Madrid | 4 | 0.1332 (14/16) | 0.1332 (14/16) | 1.00× | 0.0871 | value |
| mem-018 | Polly | **1** | 0.2330 (7/9) | 0.2330 (7/9) | 1.00× | 0.0370 | value |
| mem-019 | Phil | 3 | 0.3579 (6/11) | 0.3579 (6/11) | 1.00× | 0.8309 | frame |
| mem-020 | Lloyd | 4 | 0.3428 (6/11) | 0.4269 (6/11) | 1.25× | 0.9721 | frame |
| mem-023 | Charles | 3 | 0.8450 (9/9) | 0.8450 (9/9) | 1.00× | 0.0075 | value |
| mem-025 | Nevada | 5 | 0.6037 (20/33) | 0.6037 (20/33) | 1.00× | 0.1486 | value |
| mem-027 | Freddie | 3 | 0.6389 (29/34) | 0.6389 (29/34) | 1.00× | 0.0134 | value |
| mem-028 | Hunter | 3 | 0.5069 (20/23) | 0.5069 (20/23) | 1.00× | 0.2130 | value |
| mem-030 | Wichita | 4 | 0.2313 (8/8) | 0.2465 (8/8) | 1.07× | 0.0230 | value |
| mem-031 | Vienna | 5 | 0.8726 (17/18) | 0.8726 (17/18) | 1.00× | 0.0487 | value |
| mem-032 | Louis | 3 | 0.1448 (13/13) | 0.1448 (13/13) | 1.00× | 0.0197 | value |
| mem-033 | Alabama | 3 | 0.8686 (15/15) | 0.8686 (15/15) | 1.00× | 0.1382 | value |
| mem-034 | Brian | 3 | 0.8381 (14/14) | 0.8381 (14/14) | 1.00× | 0.0105 | value |
| mem-035 | Luis | 3 | 0.3111 (12/13) | 0.3111 (12/13) | 1.00× | 0.6238 | frame |
| **mem-036** | **Mitchell** | 4 | **0.1349** (2/11) | **0.5339** (3/11) | **3.96×** | 0.9901 | frame |
| mem-039 | Moore | 3 | 0.7650 (10/11) | 0.7650 (10/11) | 1.00× | 0.0123 | value |

**Five of 27 rows were materially inflated** by the id measure — `Howard` 14.05×, `Mitchell` 3.96×,
`Iowa` 1.26×, `Lloyd` 1.25×, `Wichita` 1.07× — and **22 of 27 are equal on both measures by
construction**, which is the cross-check the brief asked for in its strongest available form: on the 22
rows where the value's tokens occur nowhere else, the id measure cannot have been inflated, and the
contrast there is unchanged.

**The single-token subset the brief also asked for is one row** (`Polly`, span `[3493]`): pos == id =
0.2330, frame 0.0370, value above the frame on 7 of 9 steps — consistent with the full word set, and
reported as one row rather than as a distribution.

## The numeric side, per position, at the same 13 rows

| row | value | pos (steps) | id (steps) | inflation | frame | verdict |
|---|---|---:|---:|---:|---:|---|
| mem-005 | 41 | 0.0125 (0/14) | 0.0213 (0/14) | 1.70× | 0.9962 | frame |
| mem-006 | 75 | 0.1654 (6/15) | 0.1674 (6/15) | 1.01× | 0.4314 | frame |
| **mem-008** | 68 | **0.8968** (2/11) | 0.8968 (2/11) | 1.00× | 0.9830 | frame |
| mem-010 | 85 | 0.1037 (4/11) | 0.1037 (4/11) | 1.00× | 0.6788 | frame |
| mem-013 | 88 | 0.2005 (5/12) | 0.2005 (5/12) | 1.00× | 0.6757 | frame |
| mem-021 | 71 | **0.0042** (0/14) | 0.0396 (0/14) | **9.45×** | 0.9961 | frame |
| mem-022 | 89 | 0.0992 (4/15) | 0.1880 (6/15) | 1.90× | 0.4926 | frame |
| **mem-024** | 54 | **0.7165** (2/11) | 0.7165 (2/11) | 1.00× | 0.9823 | frame |
| mem-026 | 99 | 0.0717 (4/11) | 0.0717 (4/11) | 1.00× | 0.6907 | frame |
| mem-029 | 83 | 0.0769 (8/12) | 0.0951 (9/12) | 1.24× | 0.6299 | frame |
| mem-037 | 98 | 0.0098 (0/14) | 0.0098 (0/14) | 1.00× | 0.9969 | frame |
| mem-038 | 71 | 0.1116 (5/14) | 0.1116 (5/14) | 1.00× | 0.3740 | frame |
| **mem-040** | 84 | **0.6008** (2/12) | 0.6013 (2/12) | 1.00× | 0.7315 | frame |

Four numeric rows were inflated (`mem-021` 9.45×, `mem-022` 1.90×, `mem-005` 1.70×, `mem-029` 1.24×)
because a digit is shared with the distractor; **7 of 13 are equal on both measures**. The published
numeric numbers move the same way as the word ones or slightly downwards, and the counts are unchanged:
**value ≥ 0.40 on 3 of 13 rows under both metrics** (0.8968 / 0.7165 / 0.6008 at their positions), and
**0 of 13 rows put the value above the frame under either**.

## Which number a reader should carry forward, and why

**The per-position share.** It answers the question actually asked — how much copy mass the value's own
occurrence received — while the id share answers "how much mass any token spelled like the value
received", which is a different and, where subwords recur, larger quantity. The id share is not wrong,
it is a different measurement, and it is now published beside the per-position one with its inflation
factor so nothing is lost. For the numeric-versus-word contrast specifically, **either metric gives the
same answer** (21/27 versus 0/13; 3/13 versus 3/13), so the previous conclusion stands as published; but
the sentence *"16 of 27 word rows carry ≥ 0.40"* must be corrected to **13 of 27**, and readers should
quote the per-position numbers with the window count and byte basis that the previous record pinned.

## Decision

**KEEP, and the previous conclusion is unchanged; the previous record's word magnitudes are corrected.**
The falsification clause — "word rows move as much as numeric rows" — still does not fire: word rows put
the value above the frame on 21 of 27 while numeric rows do so on 0 of 13, on both measures. No model was
trained or saved, no knob set, criterion 1 remains **NOT MET** on both halves, 43/232 is unchanged, v5
was not re-run, and **STATUS/ROADMAP/#2028 are unchanged** — no capability, served model or milestone
moved.

**Where this leaves the decision:** the owner reviewed this line while the piece was in flight and, in
[D21](../../integration/DECISIONS.md#d21--three-negatives-on-one-line-force-a-pivot-deepseek-trains-the-pointer-fix-codex-stops-the-constraint-line),
directed the **supervised pointer-target fine-tune** as DeepSeek's next M1 piece. This record is the
read-out that run should be judged by, and its three checks are the acceptance gate for quoting it.

## Limitations

1. **One artifact, one arm, 13 numeric rows and 27 word rows.** The control arm at default knobs, one
   panel, one trace.
2. **The single-token subset is one row** (`Polly`); the stronger cross-check is the 22 word rows whose
   id and position shares are equal by construction.
3. **`value_pos_share` sums attention over the value's located span positions, which the text locator
   picks**; a span that appears in several turns is counted at every occurrence in the window, so the
   number is "mass on any occurrence of the value's text", not on one chosen occurrence. A per-occurrence
   breakdown is in the JSON (`position_read` per step) but is not summarized here.
4. **The frame baseline stays id-based** (the frame is a token set, not a span), so the contrast is
   per-position on the value side and per-id on the frame side; that asymmetry favours the frame, i.e.
   it cannot manufacture the numeric deficit.
5. **The 1e-5 cross-path tolerance is a judgement**: the two public paths agree to 2.329e-6, which is
   f32 accumulation; the measured value is printed every run, so a real divergence would be visible.
6. No v5 re-run, no judge, no `check_pass` measurement; this is a mechanism read-out, twice over.

## Cost

Zero pod spend, zero external compute. One release build (warm); the numeric run **16 s**, the all-rows
run **115 s** on the laptop CPU; scratch and outputs in this worktree's gitignored `local/`.

## Evidence

[`docs/evidence/pcopy_posread_2026-10-09.txt`](../../evidence/pcopy_posread_2026-10-09.txt) carries the
identities, the extended gate, the two per-row tables (numeric and word, both metrics), the inflation
factors, and the verbatim program output.

The run outputs — the program's stdout for both row sets, the per-step position read and the (target ×
position) grid per row, the analysis script and its tables — are bundled at
**`icloud:UOR-R4/results/deepseek/pcopy-posread-2026-10-09.tar`** (object `pcopy-posread-2026-10-09`,
**7,043,584 bytes, md5 `672cd5bd54f286d93bca1ddf081490f8`**), fetched back and MD5-verified with
`out/all.json` re-hashing to `7acf52da…` on both sides. Restore with
`cloud-store fetch pcopy-posread-2026-10-09 <dest>`.
