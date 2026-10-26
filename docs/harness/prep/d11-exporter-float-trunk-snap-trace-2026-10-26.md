# Prep sheet: snap-flip localisation (d11-exporter-float-trunk, claude-134), 2026-10-26

- Item: [d11-exporter-float-trunk](../../roadmap/items/d11-exporter-float-trunk.md) (#2820). ORDER: [claude-134](../../loop/tasks/d11-exporter-float-trunk/claude-134/ORDER.md).
- This is the next control named by [claude-133](../../loop/tasks/d11-exporter-float-trunk/claude-133/REPORT.md), whose investigation ended "other arithmetic, still indeterminate".
- Run: one job on one leased RTX 5090:
  - requalify the traced evaluator;
  - trace claude-132's six artifacts on 32 DEV windows;
  - compare with each saved model's D11 snap choices.

No training.

## 1. Question

**Where do the D11 forward (D) and the artifact (A) first part, and how does each parting start?**
- claude-132 left four of six models indeterminate: D and A differ on 3.4% to 4.9% of tokens.
- claude-133 found no gross signature beyond the faithful models' envelope.
- The untested hypothesis: a snap whose two best roots nearly tie flips between the float and integer forwards, and the recurrence carries the flip to later positions.
- Three conventions could also start a parting, and the comparison separates them:
  - **Zero blocks.** A zero block is a raw that is zero in all four channels. The float forward snaps it to the identity (root 1), the integer path to root 0, which is −1.
  - **Tie-breaks.** On an exact tie the integer path takes the lowest index; the float forward takes its argmax's choice.
  - **Continuous drift.** Drift in the final norm, the Q12 query or the writer can part the tokens with no snap disagreement at all.

## 2. Arms and readings

There is one model per (configuration, seed), as in claude-132: plain-c126, d11-c130 and dual-c131, two seeds each. All are compared on **the first 32 DEV windows**: 6,144 positions and about 1,060 scored rows.
- **D's tokens and snap choices** come from one forward: the D11 panel (`evaluate_dev_panel_d11`) run under `with_snap_capture`. Its chunks of 16 windows are joined per snap call. That gives, per position and layer, the root chosen at:
  - the transition, one per lane (64);
  - the output gate, one per lane (64);
  - the feed-forward gate, one per 4 MLP channels (96).

  With each choice comes its relative margin, (best − second) / best, and 255 marks a zero block.
- **A's tokens and snap choices** come from `d11-fbs-eval trace`, the traced kernel, at the same sites in the same layout, with 255 for a zero raw.
- **A disagreement** is a site where the two forwards apply different roots, a zero block counting as the root its side applies. Each has a class:
  - **root:** both blocks non-zero;
  - **zero both, zero float only, zero integer only:** the zero-block classes.
- **Inert sites.** A transition at position 0 rotates a state that is still zero on both sides: the float scan starts from h = 0, and the integer path zeroes its state per sequence. Its disagreements change nothing, so they are counted apart and enter no reading.
- **Two masks.** The root disagreements and the zero-block disagreements are placed separately. A zero block that multiplies nothing could otherwise sit everywhere and make every reading mechanical.
- **A window's first flip** is its first root disagreement in causal order: position, then layer, then the transition, the output gate and the feed-forward gate. That call's root disagreements are the window's **primaries**. Every later root disagreement in the window comes after them and may be carried. Each window's first zero block is recorded too.
- **The readings** (`--mode d11-snap-trace`, which also applies the bar):
  - **R:** the scored rows where D and A differ.
  - **Per mask (root, zero):**
    - **P:** the share of R with a disagreement of that kind in its window at or before its position;
    - **B:** the same share over the rows where D and A agree. This is P's base rate, since every row after a window's first such disagreement counts as preceded;
    - **L:** over the windows with a row in R, the share whose first such disagreement is at or before their first row in R.
  - Over those windows' primaries:
    - their count;
    - those at an exact tie: D's margin 0, where D's raw scores two roots exactly alike. Only D's side is read, since the trace holds A's choice but not its scores;
    - **m,** D's upper median margin at the others.

## 3. Bar and decision table

The bar is per model, over its 32 windows. Every instrument check of section 4 must be green first, or the model is unclassified. Each model takes **the first outcome that applies, in this order**:

| # | Outcome | Applies when | It means | It decides |
|---|---|---|---|---|
| 0 | **No rows differ** | R is empty | D and A agree on every scored row of the 32 windows. | Nothing to place. The readings are reported (the faithful models may land here). |
| 1 | **Zero convention** | zero mask: P ≥ 50%, L ≥ 50%, and P − B ≥ 20 points | Zero blocks sit before most partings, well above their base rate. There, D applies the identity and A applies −1. | D is not A's twin there: an instrument repair. D's zero convention is aligned with the integer path's, the twin is requalified, and the residual check is re-run. It is not a mechanism or trunk result. |
| 2 | **Not snap-carried** | root mask: P < 50% or L < 50% | Most partings start with no root disagreement before them, and zero blocks did not lead them enough for outcome 1. The REPORT gives the zero mask's P, B and L beside this outcome. | The residual is in the continuous arithmetic: the final norm, the Q12 query or the writer's kept cells. The next control compares A's and D's queries at those rows. It is never a mechanism result. |
| 3 | **Tie-break** | exact ties ≥ half the primaries (and > 0) | Partings start where D's raw scores two roots exactly alike, and D's tie rule parts from the integer path's lowest index. | The same kind of instrument repair: D's argmax tie rule is aligned with the integer path's lowest index. |
| 4 | **Localise upstream** | m ≥ 1e-2 | D chose decisively where A chose differently, so A's input to the snap differs from D's by more than rounding. | An instrument localisation of the pre-snap values on that model: the gate and transition raws, layer by layer. It needs no lift, because it decides a localisation, not a ruling. A departure is called only when that localisation finds the defect. |
| 5 | **Snap-flip near-tie** | root mask: P ≥ 90%, P − B ≥ 20 points, L ≥ 90%; and m < 1e-3 | Partings start at snaps whose two best roots nearly tie, and the recurrence carries them. | The residual is consistent with D's own sensitivity at its discrete choices. **It does not clear the exporter.** The next control compares A's pre-snap raws with D's at the primaries. Only if they agree within claude-133's measured drift (median 1 to 3 codes) does the next unit settle the four models as faithful, so that the residual check's "faithful on all six" follows. |
| 6 | **Indeterminate** | anything else | — | The readings are reported. The next step is a decision review, and the item does not park. |

- **The lift guards against structural sites.** Zero blocks that sit before every row, partings or not (a structurally zero lane, say), have P ≈ B, so outcome 1 cannot fire on them.
- Likewise, if the root mask's B ≥ 80%, its lift cannot reach 20 points. Outcome 5 then cannot apply: the flips are too common for this panel to place the rows, and the model reads as Indeterminate.
- **The zero convention comes first** because it is a repair to the instrument. Readings of the root mask are taken only once the twin computes the same rotations as A.
- **The item's reading:** if all four indeterminate models take one outcome, it is the item's. If they split, each model's outcome is reported, and the next step is a decision review.
- **Readings, with no bar:**
  - the same readings on the two faithful models, as the comparison;
  - every disagreement by class, layer and site, and the inert count;
  - each window's first flip, first zero block and first row in R;
  - the primaries over all 32 windows;
  - the root-disagreement margins below 1e-6, 1e-4, 1e-3 and 1e-2.

## 4. Instrument checks (each must be green before the bar counts)

1. **`I-GPU-Tests`.** The float-train CUDA library passes, including `gpu_the_snap_capture_records_each_site_and_changes_nothing`. The instrument's host parity tests pass, including the trace test.
2. **`I-Qualify`.** The rebuilt evaluator, with its two kernels, passes:
   - `d11-fbs-eval qualify`;
   - `d11-fbs-eval run --verify` on all 428 DEV windows of plain-c126 seed 1, with G = H = R at all 82,176 positions, as claude-128 qualified it.

   Its new PTX and cubin sha256s are recorded.
3. **`I-Trace`.** For each artifact, the traced run's outputs and trace equal the host twin's (G = H), and the trace fits the trunk's shape.
4. **`I-Old-Run`.** For each artifact, the traced build's tokens on the 32 windows equal the stored, qualified run's: claude-129's, claude-130's or claude-131's.
5. **`I-Inputs`.** Each artifact's, the windows' and each model's sha256 equal their pins (section 5).
6. **`I-Capture`.** The D panel's snap calls cover its two chunks, three per layer each. A miscount is refused, not read.
   - **`I-Layout`,** added after code review and before the run's readings were seen (the first job was then still qualifying): each (layer, site) must agree on at least 50% of its snaps.
   - D and A agree on nearly every token, so nearly every snap. A site read at a wrong offset would agree on about one root in 120.
   - This catches a layout permutation that still writes every byte, which the host tests and G = H cannot see, because they share one C core. A failure leaves the model unclassified.
7. **`I-Fetch` and `I-Finite`,** as in claude-132: every fetched file against its folder's `SHA256SUMS`, every step exiting 0, and the log written before `SHA256SUMS`. Every root-disagreement margin must be finite too (`i_finite_margins`), since a NaN would otherwise read as an exact tie.

## 5. Base artifact and data split

- **The six models, their pins and their stored runs** are claude-132's, from the [residual sheet](d11-exporter-float-trunk-residual-2026-10-26.md) §5.
- **The six artifacts:**
  - `results/claude/d11-fbs-export-gap-2026-10-26/d11gap/save/seed-{1,2}/artifact.uorlut` (plain-c126);
  - `results/claude/d11-fbs-d11-train-2026-10-26/d11t/artifact-seed-{1,2}.uorlut` (d11-c130);
  - `results/claude/d11-fbs-dual-train-2026-10-26/dual/artifact-seed-{1,2}.uorlut` (dual-c131).

  Their sha256s are the residual sheet's artifact pins.
- **The windows:** the first 32 sequences of `windows.json` (sha256 `083c1be628f871ff1502e7d05f84cc3f55dda2ac461614a62df9167bcd08893e`), as the job cuts them. Their file's sha256 is recorded, and it is the `--windows-sha256` of the comparison.
- **DEV data:** `w2-inputs/2026-10-25/driver-data/ft/dev/`, pinned as in claude-132. No training data is read.

## 6. Steps and minimum test power

1. `uor-pod up --count 1 --gpu 5090 --ref <pushed head>`.
2. Build `spiralcore-fbs-sheet` (float-train) and `d11-fbs-eval` (the instrument) with nvcc, and copy both with their sha256s.
3. `I-GPU-Tests`: the float-train CUDA library and the instrument's tests.
4. Fetch and check the inputs (`I-Fetch`, `I-Inputs`).
5. `I-Qualify`: `d11-fbs-eval qualify`, then `run --verify` on plain-c126 seed 1's artifact over all 428 windows.
6. Cut the 32 windows. For each artifact, run `d11-fbs-eval trace`, then `spiralcore-fbs-sheet --mode d11-snap-trace … --windows-count 32 --stored-run <its run-seed-s.json>`.
7. Save the run folder to the store at `results/claude/d11-fbs-snap-trace-2026-10-26/`, with a `SHA256SUMS`, from the laptop, confirmed before `down`.

**Minimum test power:**
- Each model has about 1,060 scored rows, about 35 to 50 of them in R on the indeterminate models (3.4% to 4.9%), in perhaps 15 to 25 of its 32 windows.
- P's 95% interval on 40 rows is about ±9 points near 90% and about ±16 near 50%. L's on 20 windows is about ±13 near 90% and about ±22 near 50%.
- So the bars are coarse by design. They separate "nearly all" from "most" from "few", not finer shades. A model near a threshold reads on whichever side its count falls, and the REPORT gives its counts.
- If R gathers in fewer windows, L's interval widens past ±22, so the REPORT gives each model's window count beside its L.
- The primaries number at least one per window with a flip, so m rests on roughly 15 to 25 or more values per model.

## 7. Cost and stop rule

**Cost:**
- one RTX 5090, about $1/h, under a 2 h lease;
- builds about 3 minutes;
- the qualification's `--verify` about 20 minutes (claude-131 and claude-132);
- six traces and comparisons a few minutes each;
- in all, about 45 minutes, about $1.

**Stop rule:**
- Stop and save on a failed `I-GPU-Tests` or `I-Qualify`, or any nonzero exit.
- A failed per-model check leaves that model unclassified.
- Classify every failure as instrument or mechanism first.
- Never cancel a healthy run because the lease ran short; renew it.
- Save everything off the pod before `down`.

## 8. Non-author TEST FITNESS review

Three rounds, each by a fresh-context reviewer that read only the sheet, its sources and the code (claude-134, 26 Oct):

- **Round 1: NOT FIT.** It found seven problems:
  - the margin mixed first flips with carried ones;
  - P had no base rate;
  - zero blocks were treated as benign;
  - "clears the exporter" did not follow;
  - there was no outcome precedence;
  - D's tokens came from a second forward;
  - no power was stated.
- **Round 2: NOT FIT.** All seven were fixed bar one. Zero blocks still drove the position mask, so a structurally zero lane would fire "zero convention" mechanically.
- **Round 3: FIT.** It confirmed:
  - root-only masks and first flips;
  - a separate zero mask with its own 20-point lift;
  - position-0 transitions as inert, checked against `fbs_sequence`, `fbs_recurrence` and `quaternion_scan`;
  - the code's precedence and thresholds against the table.

  Its advisories were folded in: outcome 2's wording, and finite margins as an instrument check.

TEST FITNESS: FIT
