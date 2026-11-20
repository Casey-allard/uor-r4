# REPORT: written-state-count-model, claude-440 (decision card: P1's S5 copy path)

Lab: claude. Order: [ORDER.md](ORDER.md). Card: [DECISION.md](DECISION.md). Review:
[DECISION-REVIEW.md](DECISION-REVIEW.md). Doc-only; nothing ran on a GPU and nothing was rented.

**Decision: P1's S5 copy path is A1, Pick over the D11 trunk codes.** C2 (exact pairs) loses to it by
4,318 to 5,232 micro-BPB on DEV-tune at 100% of C, out of sample. The card does not credit the D11
codes with that lead. C3, with permuted codes, comes within 24 to 232 micro-BPB of A1.

## What did not work, first

- **The card chooses on bits, not on geometry** [claude]. Most of A1's lead over C2 seems to come from
  firing at distance > 0 under any code map (C3), not from the D11 codes. That is an inference from C3;
  no arm measures it directly. The geometry label stays with claude-439's reading (`geometric_gain`,
  small) and with P2's bar.
- **Serving cost and footprint are not measured** [claude]. A1's read on the M1 and its per-session
  state are estimates. The reversal rule's speed condition and the P1(b) recording duty carry them.
- **The first draft overstated three things** [claude]: that codex-28's packer carries A1's column,
  that A1 adds no state, and a paraphrased quote of claude-409. The decision review caught each one,
  and each is fixed at the head.
- **The first reversal rule could not fire on C2's strongest case** [claude]. A band wholly above 0
  slipped past "includes 0". It is fixed.

## What changed

- `docs/loop/tasks/written-state-count-model/claude-440/`: ORDER, DECISION (the card),
  DECISION-REVIEW (the review and its disposition), and this REPORT.
- `PLAN.md`: one sentence in P1's row under "Approved sequences" names S5's copy path and links
  the card.

## Measurements

None new. Every number is claude-439's, checked against its REPORT and its 100% JSON
(`claude-439/run2/pick-p1000.json`):

| Arm | DEV-tune all | DEV-tune novel |
|---|---|---|
| A0 pure count | 1,432,620 | 1,747,432 |
| A1 Pick, D11 codes | 1,360,806 | 1,651,188 |
| C2 exact pair | 1,365,580 | 1,657,701 |
| C3 rotated codes | 1,360,932 | 1,651,364 |

Bands: A1 − C2 [−5,232, −4,318] overall and [−7,141, −5,889] on novel targets; A1 − C3
[−232, −24].

## What the numbers say

- A1 is the best measured copy path, and it beats exact pairs by a band that clears 0 widely.
- Its measured extra cost over C2 is offline, about 9 min of device reads per pass on wick, against about
  1,300 ms on the host for C2.
- C3 is never the cheaper choice: it costs the same device read as A1 and scores slightly worse.

## Checks run at the head

- `UOR_LAB=claude uor-harness check`: info lines only, no error. The branch check said "only the
  first 25 branches were queried".
- `UOR_LAB=claude uor-harness loop pr-check --local`: before this REPORT existed, the only FAIL was
  "REPORT missing"; at `cb37292da`, the first head with this REPORT, the verdict was PASS. It is
  re-run at the final head and its result is posted on the PR. The read-path and filemap WARNs are advisory and come from base (AGENTS.md is 83
  lines, the same as on base).
- The decision review: APPROVE WITH CHANGES; the eight findings are disposed of in DECISION-REVIEW.
- An exact-head review at `cb37292da`: APPROVE with six minor findings (a charter-read gap in the
  review record, "extra cost is offline" without "measured", the per-class weight wording, PLAN's
  controls list, this section's pr-check timing, and no memory condition in the reversal rule). All
  six are fixed at the next head, which gets its own exact-head review.
- The card's numbers were checked against `pick-p1000.json` by `jq` (DECISION-REVIEW, "Gaps closed").

## Skills used

None beyond the loop's own steps: ORDER, a decision card under §5.1 / D68, a fresh-context decision
review, and REPORT.

## Still open

- **The refit-and-pack unit** (codex-27 names it next) adopts A1's column, or reconciles its own,
  and records S5's deployed bytes and per-session state for A1 and C2.
- **The D85 Pick replay** on ≥ 64 prompts, plus claude-439's unrecorded gate negative, in the unit
  that packs A1.
- **P1(b)'s sheet** registers a C2 arm before its sealed DEV-test run, if the bits rule is to be read
  there.
- **Untested hypotheses** carried from claude-439: a cheap nearest-pair read under any injective map,
  and a per-distance weight split (1, 2, 3+).

## Plain summary

The project's counting model has a "copy" helper that predicts what came next the last time the
same short context appeared in a document. One version uses the project's geometric codes to find
near matches. A cheaper version only copies exact matches. The geometric version predicts text about
a third of a percent better. Its measured extra cost is minutes of offline compute; its cost on the
laptop is not measured yet. So we keep
it, along with the cheap version and a scrambled-code version as checks. If a bigger run shows the
cheap version is as good, or the geometric one is too slow on the laptop, we switch.
