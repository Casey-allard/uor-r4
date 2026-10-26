# ORDER: d11-exporter-float-trunk, claude-133

Lab: claude. Item: [d11-exporter-float-trunk](../../../../roadmap/items/d11-exporter-float-trunk.md). Base: `ccd4e8fec`.

## Goal

The item's done-when reads:

> A float-trunk snapshot exports to D11 with the audits green and the export gap measured inside the band. The trunk is a SpiralCore/FBS recurrence trunk with the Hamilton gate and the native vector writer head, as Stage 3 trains it. The synthesis (Stage 4) adds tokens per second, reported for decode and for context separately.

**This unit is the instrument investigation that claude-132's indeterminate row opened** ([REPORT](../claude-132/REPORT.md)).
- **What claude-132 found.** Its pre-registered classification was 2 faithful, 4 indeterminate and 0 departures:
  - The D11 panel (D) and the artifact (A) differ on 2.0% to 4.9% of tokens.
  - The head is identical to A on every row.
  - D against A passes the paired test on all six models.
  - D's and A's Q12 queries never match exactly. The largest per-row drift is tens of thousands of codes.
- **The question.** Is the D-against-A residual the integer session's own arithmetic, or an exporter departure? The integer session keeps its residual stream at Q16 (`RESIDUAL_EXP` = −16), while D is a float32 forward.
- **Why not on the laptop.** claude-132 named laptop host checks against the f64 reference. But `fbs-compare` (claude-126) records that a host run of the reference is a host fallback, and CHARTER §3.5 says model and parity reference execution run on the GPU. So the whole investigation runs on the GPU.
- **The plan:**
  1. **A Q16 control, D16.** The D11 forward with its residual stream rounded to 2^-16 after the embedding and after each sublayer add, where the integer session holds it. This runs in evaluation only, under a hook, and the D11 forward with the hook off is unchanged.
  2. **The code review's drift readings,** for D against A and for D16 against A:
     - the column of each row's maximum;
     - the median absolute difference;
     - the mean signed difference per channel;
     - saturation counts on each side;
     - a per-row least-squares scale.
  3. **One short GPU run** over claude-132's six saved models and stored artifact runs, with a prep sheet, its FIT and a D67 review. The sheet pins how the readings classify the residual: as Q16 arithmetic, as other arithmetic, or as a departure.

## Cancel probe (run first)

Cancel if a GPU Q16 control or the extended drift readings exist or are in progress. To check:
- `git log ccd4e8fec..origin/main -- crates/uor-r4-float-train docs/loop/tasks/d11-exporter-float-trunk docs/harness/prep/`;
- the open PRs;
- a search of `origin/main` for a Q16 residual rounding in `crates/uor-r4-float-train`.

Result (26 Oct):
- No commit after `ccd4e8fec` touches those paths.
- The other open PRs are #3283 (kimi) and #3169 (codex). None touches float-train or this item.
- The float trunk's `hidden` has no residual rounding.

**Not cancelled (instrument: none needed).**

## Files in scope

- `crates/uor-r4-training/**`
- `crates/uor-r4-integer/**`
- `crates/uor-r4-float-train/**`
- `docs/harness/prep/d11-exporter-float-trunk*`
- `docs/labs/d11-exporter-float-trunk/**`
- `docs/loop/tasks/d11-exporter-float-trunk/**`
- `docs/research/notebook/m4-serving.md`

## Done when

1. **The Q16 control.** With the hook on, the float trunk rounds its residual stream to 2^-16 at the integer session's points. With it off, the forward is unchanged.
   - A GPU test pins both: values on the grid, and the hook off equal to the old forward.
   - A host test pins the hook's set and restore.
2. **The readings.** `d11-residual` reports D16's panel, its pairings with A and D, and the drift readings for D against A and for D16 against A. Host tests pin the new arithmetic.
3. **A prep sheet, its FIT, and a D67 pre-run review.** The sheet pins how the readings classify the residual and what each outcome decides.
4. **The run:** one pod job over claude-132's six models, with every receipt in the store.
5. **A REPORT and a notebook row.** This unit cannot close #2820 by itself, so the REPORT says `References #2820`.

## Stop and ask if

Stop and decide; never wait.
- **The control needs a host reference run:** no, that is a host fallback (§3.5). The control must be a GPU forward.
- **The Q16 hook changes the D11 forward with the hook off:** that is a bug. Fix it before any run.
- **The readings point to a departure:** report it as one and open a repair item. Never explain it away.
- **The bar or the comparator** (CHARTER §3.7): the owner's. It is already asked on #2028, so this unit changes neither.
