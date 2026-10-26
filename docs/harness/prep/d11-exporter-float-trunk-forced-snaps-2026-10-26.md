# Prep sheet: the forced-snap control (d11-exporter-float-trunk, claude-135), 2026-10-26

- Item: [d11-exporter-float-trunk](../../roadmap/items/d11-exporter-float-trunk.md) (#2820). ORDER: [claude-135](../../loop/tasks/d11-exporter-float-trunk/claude-135/ORDER.md).
- This is the next control set by the decision review after [claude-134](../../loop/tasks/d11-exporter-float-trunk/claude-134/REPORT.md) (#2820 owner log, comment 6424294556).
- Run: one job on one leased RTX 5090:
  - check that the evaluator is the qualified one;
  - trace claude-132's six artifacts over the full DEV panel;
  - run each saved model's D11 forward with its snaps forced to the artifact's choices.

No training.

## 1. Question

**When every icosian snap of the D11 forward (D) is forced to the artifact's (A's) traced choice, do D and A still differ, and would D have chosen otherwise at those snaps?**
- **Where the line stands:**
  - claude-132 left four of six models indeterminate, with D and A differing on 3.4% to 4.9% of the 14,287 DEV rows.
  - claude-134 found that every first flip between them is a near-tie snap, but could not show that the flips cause the residual: every window flips early.
- **The intervention:** forcing the snaps removes every discrete difference between D and A. What remains is the continuous arithmetic: the final norm, the Q12 query and the writer.
- **The would-flips:** at every forced site, D's own choice, given the forced history, is recorded with its margin. A site where D would choose another root is a flip on its own, not carried, because the states are held together.
- **The two readings together:**
  - forced-D ≈ A says the snaps carry the residual;
  - near-tie would-flips at the faithful models' rate say A's pre-snap arithmetic is D's to within rounding.

  Forced-D ≈ A alone does not clear the exporter, because forcing hides A's pre-snap arithmetic.

## 2. Arms and readings

There is one model per (configuration, seed), as in claude-132: plain-c126, d11-c130 and dual-c131, two seeds each, over **all 428 DEV windows** (82,176 positions, 14,287 scored rows).
- **A's snap choices** come from `d11-fbs-eval trace` over the full `windows.json`.
  - The layout is claude-134's: per position and layer, the transition and output gate per lane, then the feed-forward gate per 4 MLP channels.
  - 255 marks a zero raw, which applies root 0.
- **Unforced D:**
  - the D11 panel with its snap choices captured;
  - its rows against A are claude-132's, a check (`I-Unforced`).
- **Self-forced D:** D forced to its own captured choices, which must reproduce D (`I-Self-Force`).
- **Forced-D:** D with every snap forced to A's traced choice (`with_snap_override`), in the panel's call order (chunks of 16 windows, per layer, the three sites).
  - A capture runs alongside it and records D's own choice and margin at every forced site.
  - **The would-flips** are the sites, inert transitions at position 0 apart, where D's own choice applies another root than A's. They are the root disagreements and the one-sided zero blocks.
  - **A zero block on both sides is never a would-flip.** D and A compute the same arithmetic there and part only in the zero convention: the identity against −1. It is reported as a reading (`would_flip_count`).
- **A wrong-root gate:** forced-D with layer 0's output gate, and separately the last layer's feed-forward gate, moved to other roots must change the tokens (`I-Wrong-Root`).
- **The readings per model:**
  - **R_F:** the rows where forced-D and A differ;
  - **R_D:** the same for unforced D;
  - **the would-flip rate:** would-flips over the non-inert sites;
  - **the would-flip margins:** **m**, D's upper median margin at the root would-flips off an exact tie, and the exact ties;
  - the would-flips by class, layer and site.
- **The envelope** is the larger would-flip rate of the two models that claude-132 found faithful, d11-c130 s1 and dual-c131 s1, with a floor of one would-flip per 10⁷ sites. The floor keeps a zero envelope from being a bar no model can meet. `--mode d11-forced-summary` computes the envelope and applies the bar to every model.

## 3. Bar and decision table

The bar is per model, over its 14,287 rows. Every instrument check of section 4 must be green first, or the model is unclassified. Each model takes **the first outcome that applies, in this order**:

| # | Outcome | Applies when | It means | It decides |
|---|---|---|---|---|
| 1 | **Continuous residual** | R_F ≥ 2% (286 rows) | Forcing every snap leaves most of the residual. | The continuous arithmetic carries it. The next control compares A's and D's Q12 queries at the R_F rows, layer by layer from the final norm. It is never a mechanism result. |
| 2 | **Partly snap-carried** | 0.5% < R_F < 2% | The snaps carry part, the continuous arithmetic the rest. | The same continuous localisation, on the R_F rows only. |
| 3 | **Tie-break** | R_F ≤ 0.5%, and exact ties ≥ half the root would-flips (and > 0) | D's raw scores two roots exactly alike where A's choice differs. | An instrument repair: D's argmax tie rule is aligned with the integer path's lowest index, and the residual check is re-run. |
| 4 | **Localise upstream** | R_F ≤ 0.5%, and m ≥ 1e-2 or a would-flip rate above twice the envelope | The snaps carry the residual, but D would choose decisively otherwise, or far more often than on the faithful models. | An instrument localisation of A's pre-snap raws against D's, layer by layer. It is not a ruling. A departure is called only when it finds the defect. |
| 5 | **Faithful** | R_F ≤ 0.5%, m < 1e-3 (or no would-flips), and a rate at most twice the envelope | The snaps carry the residual, and D's would-flips are near-ties at the faithful models' rate. | The residual is D's own sensitivity at near-tie snaps. A computes the D11 forward. |
| 6 | **Indeterminate** | anything else | — | The readings are reported. The next step is a decision review, and the item does not park. |

- **The item's reading:**
  - If all four indeterminate models are **Faithful**, they settle as faithful, and the residual check reads "faithful on all six".
  - The exporter is then cleared on this panel at this bar: the snaps carry the residual, and D's would-flips are near-ties at the faithful models' rate. It says nothing further about the continuous arithmetic on the rows where forcing already matches.
  - The three FAILs (claude-129 to -131) stand as configuration results, never a mechanism result (§4.1).
  - The pivot note's decisive run goes next, under Stage 4b's unchanged bar unless the owner reads §3.7 otherwise (#2028). Nothing waits on the owner.
  - If the four split, each model's outcome is reported, and the next step is a decision review.
- **The two faithful models** are classified the same way. Their outcomes are readings, and their rates set the envelope, so they pass the rate check by construction. For them, only R_F and m test anything. The summary refuses to run without both, or if either fails its instruments.
- **Readings, with no bar:**
  - R_D, R_F, and the rows they share;
  - the would-flips by class, layer and site, the zero blocks on both sides included, and their margins below 1e-6, 1e-4, 1e-3 and 1e-2;
  - unforced D's disagreements by class, layer and site;
  - the wrong-root gate's rows.

## 4. Instrument checks (each must be green before the bar counts)

1. **`I-GPU-Tests`.** The float-train CUDA library passes, including `gpu_forcing_its_own_snaps_reproduces_d_and_a_wrong_root_does_not` and claude-134's capture test. The instrument's host parity tests pass.
2. **`I-Kernel`.** `d11-fbs-eval qualify` passes, and its PTX and cubin sha256s equal claude-134's qualified ones: PTX `5a947f09285a359dfb0ddfdf4bdf31746b959ef39f6056807eead9b4672f87ef`, cubin `2a386d8c563516aeef10d040200867330cafb54d5525cdd7580b7969554a4c86`. The evaluator is unchanged since claude-134, whose `run --verify` gave G = H = R at all 82,176 positions.
3. **`I-Trace`.** For each artifact, the traced run's outputs and trace equal the host twin's (G = H), and the trace fits the trunk's shape over all 428 windows.
4. **`I-Old-Run`.** For each artifact, the traced tokens equal the stored, qualified run's at every position.
5. **`I-Inputs`.** Each artifact's, the windows' and each model's sha256 equal their pins (section 5).
6. **`I-Unforced`.** Unforced D's rows against A equal claude-132's: 580, 484, 367, 697, 289 and 497.
7. **`I-Self-Force`.** D forced to its own captured choices gives D's tokens on every row.
8. **`I-Forced-Layout`.** D's own choices, laid out as an artifact trace (`trace_layout`), map back through `forced_calls` to the panel's own calls byte for byte. So A's trace is forced at exactly the sites it was read from. An offset or order error there would otherwise pass `I-Self-Force`, and read as a high would-flip rate.
9. **`I-Wrong-Root`.** Layer 0's output gate, and the last layer's feed-forward gate, each forced to other roots, change at least one token.
10. **`I-Layout`.** Each (layer, site) of unforced D agrees with A's trace on at least 50% of its snaps.
11. **`I-Finite`.** Every would-flip margin is finite, every step exits 0, every fetched file checks against its folder's `SHA256SUMS`, and the log is written before `SHA256SUMS`.
    - The forced calls must also be used up exactly.
    - If the panel skipped a chunk with no scored rows, the calls would fall out of step, and the run would end on the override's "never made" or "beyond the forced calls" error: an instrument failure with a nonzero exit, never a silent misforcing.

## 5. Base artifact and data split

- **The six models, their artifacts, their pins and their stored runs** are claude-132's and claude-134's, from the [snap-trace sheet](d11-exporter-float-trunk-snap-trace-2026-10-26.md) §5. With them come `I-Unforced`'s expected rows (§4).
- **The windows:** all of `windows.json` (sha256 `083c1be628f871ff1502e7d05f84cc3f55dda2ac461614a62df9167bcd08893e`).
- **DEV data:** `w2-inputs/2026-10-25/driver-data/ft/dev/`, pinned as in claude-132. No training data is read.

## 6. Steps and minimum test power

1. `uor-pod up --count 1 --gpu 5090 --ref <pushed head>`.
2. Build `spiralcore-fbs-sheet` (float-train) and `d11-fbs-eval` (the instrument) with nvcc, and copy both with their sha256s.
3. `I-GPU-Tests`.
4. Fetch and check the inputs (`I-Fetch`, `I-Inputs`).
5. `I-Kernel`: `d11-fbs-eval qualify`, with its pins compared.
6. For each artifact, run `d11-fbs-eval trace` over the full `windows.json`, then `spiralcore-fbs-sheet --mode d11-forced-snaps` with:
   - `--windows-count 428`;
   - `--stored-run`, its stored run;
   - `--expect-unforced`, its claude-132 count.
7. `spiralcore-fbs-sheet --mode d11-forced-summary` over the six receipts, with `--faithful d11-c130:1,dual-c131:1`.
8. Save the run folder to the store at `results/claude/d11-fbs-forced-snaps-2026-10-26/`, with a `SHA256SUMS`, from the laptop, confirmed before `down`.

**Minimum test power:**
- 14,287 rows per model. The bars sit at 0.5% (71 rows) and 2% (286 rows).
- Unforced, the indeterminate models differ on 484 to 697 rows. So the bars separate "forcing removes nearly all of the residual" (≤ 71 left) from "most of it remains" (≥ 286). With these counts, each outcome is unmistakable at the bar's coarseness.
- The would-flip rate rests on about 110 million sites per model: 82,176 positions × 6 layers × 224 sites, the inert transitions apart. Its sampling error is negligible next to the factor of 2 against the envelope.
- The sites are correlated, though: shared prefixes and recurring lanes. The REPORT gives each model's would-flips by layer and site, so a rate carried by one lane can be read.

## 7. Cost and stop rule

**Cost:**
- one RTX 5090, about $1.1/h, under a 2 h lease;
- builds about 3 minutes;
- the qualify check about 1 minute;
- per model, a 14 s trace and five D11 panels over 428 windows: unforced, self-forced, forced, and the two wrong-root gates. claude-133 ran six such panels per model in about a minute.
- In all, about 30 minutes, about $0.60.

**Stop rule:**
- Stop and save on a failed `I-GPU-Tests` or `I-Kernel`, or any nonzero exit.
- A failed per-model check leaves that model unclassified.
- Classify every failure as instrument or mechanism first.
- Never cancel a healthy run because the lease ran short; renew it.
- Save everything off the pod before `down`.

## 8. Non-author TEST FITNESS review

Two rounds, each by a fresh-context reviewer that read only the sheet, its sources and the code (claude-135, 26 Oct):
- **Round 1: NOT FIT.** The would-flip rate counted zero blocks on both sides, which under forcing are the same arithmetic in D and A, apart only in convention. So convention counts could fill the envelope and pass a model with ten times the faithful root rate. Its advisories asked for:
  - a gate on how `forced_calls` maps A's layout onto the panel's calls;
  - a second wrong-root site;
  - an envelope floor;
  - three wording fixes.
- **Round 2: FIT.** All fixed: `would_flip_count`, with a host test and a negative gate run; `I-Forced-Layout` with its round-trip test; two wrong-root sites; the 10⁻⁷ floor; the wording.
  - Its advisory: m reads root would-flips only, so a model whose would-flips are all one-sided zero blocks has no m and reads Indeterminate. That is the conservative side, and the REPORT says so.
  - Its second advisory: the tie-break comes before the rate check. It was accepted as written, because its decision is a repair and a re-run.

TEST FITNESS: FIT
