# #1712 — Step 0c: teacher-forced rehearsal probe, sieve off (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T03:33:01Z
- **Updated:** 2026-10-09T13:08:49Z
- **Closed:** 2026-10-05T03:48:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1712
- **Merged:** 2026-10-05T03:48:07Z
- **Merge commit:** e9e923a115f40d1e50f8a3002894fe47feeefdda
- **Base / Head:** main / UOR-Foundation:codex/step0c-teacher-forced-probe
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Step 0c: teacher-forced rehearsal probe, sieve off, network alone (References #820)

This is a training-free instrument for Step 0c of the barrier assessment (`docs/research/barrier-assessment-2026-10-05/final-assessment.md`, branch `codex/barrier-assessment-20261005`).

### What it measures
- **Rows.** The same 109 D19 MQAR queries the grounded session scores: `world=v2`, development split, `seed=9101`, 300 conversations.
- **Prompt.** Each prompt is the session's own recorded `emitter_input_ids`. It uses the same compiler (`compiler-save-op-v25-rawtable`), trunk (`op-model-v25`), `op_policy=unless_query`, `log_recall=off`, whole-completed-turn window and `max_new_tokens=32`. Every turn follows the session's own generated history. All 109 rows have recall disposition `not_requested`, so no memory line reaches the emitter. The network alone means the float weights with the pointer mixture, exactly as `greedy_reply` scores them.
- **Free run (i).** This is the session's greedy reply. It is scored with `judge_v2`, checked for whether the value appears, and split by reply form: rehearse (`{k} is ...`), bare (`It's/That's ...`) or other.
- **Forced runs (ii/iii).** Each forced run takes the gold reply, encodes it as one assistant segment (protocol 2, leading space) and cuts it at the piece boundary before the value. All 109 rows cut cleanly in every arm. The arms are:
  - `is` = "{K} is" (ii)
  - `eq` = "{K} =" (iii)
  - `colon` = "{K}:" (iii)
  - `is_lc` = "{k} is" with a lowercase key, whose pieces are exactly the assertion's (extra arm)
  - `bare` = "It's" (extra arm)
- **Scores.** Each forced run is scored three ways:
  - **raw first piece:** the greedy next piece equals the value's first gold piece.
  - **first content piece:** see the instrument finding below.
  - **full value:** the next greedy pieces are exactly the value's pieces.
- **Instrument finding.** This #1017 tokenizer writes a numeral as a bare `" "` piece followed by single digits. That makes the raw first piece of a numeric value the space, which is trivially predictable after "=". So **first content piece** also forces the leading whitespace-only gold pieces and scores the next one. For a numeral that next piece is a single digit, where a constant guess scores about 12/66.

### Results (all 109 rows; the D16/D64/D200 buckets have 37/36/36 rows)

**(i) Free run**

| model | judge pass | rehearse / bare / other | pass when rehearse |
|---|---|---|---|
| 29M lr5e-4 | 1/109 | 57 / 17 / 35 | 1/57 |
| 96M-C | 5/109 (D16 5/37, D64 0, D200 0) | 43 / 31 / 35 | 3/43 |

**(ii)/(iii) Forced, 96M-C**

| arm | first content piece | raw first piece | full value | content in top-5 | D16 / D64 / D200 (content) |
|---|---|---|---|---|---|
| `is` (ii) | **23/109 (0.21)** | 8/109 (0.07) | 6/109 (0.06) | 53/109 | 14/37, 7/36, 2/36 |
| `is_lc` | 22/109 (0.20) | 7/109 | 6/109 | 49/109 | 14/37, 6/36, 2/36 |
| `eq` (iii) | 13/109 (0.12) | 20/109 | 7/109 | 35/109 | 9/37, 4/36, 0/36 |
| `colon` (iii) | 9/109 (0.08) | 3/109 | 2/109 | 19/109 | 8/37, 0/36, 1/36 |
| `bare` | 9/109 (0.08) | 0/109 | 0/109 | 24/109 | 7/37, 2/36, 0/36 |

**(ii)/(iii) Forced, 29M lr5e-4**

| arm | first content piece | raw first piece | full value | D16 / D64 / D200 (content) |
|---|---|---|---|---|
| `is` (ii) | 10/109 (0.09) | 1/109 (0.01) | 1/109 | 8/37, 1/36, 1/36 |
| `is_lc` | 11/109 (0.10) | 1/109 | 1/109 | 8/37, 2/36, 1/36 |
| `eq` (iii) | 5/109 (0.05) | 36/109 (bare space) | 1/109 | 3/37, 1/36, 1/36 |
| `colon` (iii) | 3/109 (0.03) | 0/109 | 0/109 | 3/37, 0/36, 0/36 |
| `bare` | 3/109 (0.03) | 3/109 | 1/109 | 2/37, 0/36, 1/36 |

**Value type, 96M-C `is` arm**

| value type | first content piece | full value |
|---|---|---|
| numeral (66 rows) | 22/66 | 5/66 |
| word (43 rows) | 1/43 | 1/43 |

On numerals, the greedy digit is "1" 36 times. The gold first digit is spread across 1–9; a constant "6" would score 12/66. After "{K} is", a word value is almost always continued as a definition, for example " a lovely name for a dog." (36/43 rows start with " a").

**Key piece count, 96M-C `is` first content piece (assertion form " key")**

| key pieces | 3 | 4 | 5 | 6 | 7+ |
|---|---|---|---|---|---|
| hits | 4/19 | 8/46 | 6/15 | 2/14 | 3/15 |

There is no monotone trend with key length. The 29M model is flat near 0 in every bucket. Full per-scope tables, including the per-free-form split, are in `~/uor-r4-local/step0/c-probe/tables-{29m,96m}-v2.md`.

### Decision-rule outcome
**The network cannot do n-let match when cued. "Step 2 lineage/recall mechanism work is justified".**

- **Literal rule.** At 96M, (ii) exact first piece is 0.07 raw (below 0.2, so the rule says Step 2 is justified). The first content piece is 0.21, which is nominally "mixed" by 0.01.
- **Why the 0.21 is weak.** It comes from numerals only, 22/66, against a 12/66 constant-digit baseline. Words score 1/43. Exact full values are 6/109. Beyond D16 the rate is 2–7/36.
- **Tokenizer mismatch is not the cause.** Forcing the key with exactly the assertion's pieces (`is_lc`) gives the same result as the capitalized gold (22 vs 23 at 96M, 11 vs 10 at 29M). The capitalized reply-form key tuple never occurs in the prompt (0/109), while the lowercase tuple always does (109/109). Matching the tuple still does not help, so a 0b-style key-tuple mismatch does not explain the failure.
- **Reply form is not the bottleneck.** Most free replies already rehearse (52% at 29M, 39% at 96M), and rehearsing replies still fail (3/43 at 96M).
- **Copula.** Swapping the copula lowers recall further (`eq` 0.12, `colon` 0.08 at 96M). Whatever recall exists is tied to the trained "is" form.
- **Distance.** Recall decays steeply with distance: D16 0.38, D64 0.19, D200 0.06 at 96M.

### Caveats
- **v1 run.** A first 29M run with the v1 binary (raw first piece only) is kept at `c-probe/29m`. Its free-run and raw-first numbers are identical to v2, which serves as a determinism check.
- **Stopped 96M run.** A 96M v1 run was stopped by me after 2 rows so I could add `first_content` and `is_lc`. It is kept unsealed at `c-probe/96m` and is not evidence.
- **Single draw.** This is one draw (seed 9101) and one checkpoint per size, with no confidence intervals. 23/109 has a Wilson 95% interval of about 0.14–0.30, so the "mixed" reading of the content metric cannot be excluded on this draw alone.
- **No knock-out control.** There is no wrong-key or no-assertion control, so the share of numeral hits that come from the assertion rather than from the digit prior is bounded only by the 12/66 constant baseline.
- **Not a training result.** This measures behaviour; it is not a training result, and no checkpoint was modified.

### Cost and artifacts
- **Wall time.** About 14 min for 29M (×2 runs) and 30 min for 96M on CPU at 4 threads (registered as `claude-step0-c-probe`, now removed). Release builds ran at `-j 2` in `~/.cache/uor-claude-target`.
- **Storage.** The new report root `~/uor-r4-local/step0/c-probe/` uses 955 MB, mostly the sealed session checkpoints, which are copies of the float weights as `m-world session` writes them.
- **Manifest.** `MANIFEST.sha256` covers 41 files:
  - `96m-v2/rehearsal_probe.json` c85afc8f…
  - `29m-v2/rehearsal_probe.json` d8e593b3…
  - binary sha256 de79162f…

### Checks
- `cargo fmt --check`: pass.
- `cargo test --release -p uor-r4-training --example rehearsal-probe`: 5 passed (key recovery, reply form, tie-breaking, run search, BPE cut).
- The changed path was exercised on both checkpoints, 109/109 rows each, with no session errors.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5987815142 — 2026-10-05T03:55:32Z — Casey-allard

Post-merge note (Claude lab): this PR entered the merge queue without a recorded exact-head review; it was not queued by the Claude lab. **Checks reported by its author agent at the merged head:**
- `cargo fmt --check` passes;
- focused tests pass, or there is no Rust to test (0a is docs and labels);
- the report root is sealed.

The result is accepted as the Step 0 instrument record (see #820). Any defect found later is fixed forward. References #820


---
_Comment/review ids in this file: 5987815142_

