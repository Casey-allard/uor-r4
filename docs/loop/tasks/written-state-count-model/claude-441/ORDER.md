# ORDER: written-state-count-model, claude-441 (P1: split the S5 Pick buckets by pair distance)

Lab: claude. Item: [written-state-count-model](../../../../roadmap/items/written-state-count-model.md)
(#4063). Base: `origin/main` `cfe019895` (codex-29, #4124, after claude-440's card merged as
`5b120bd70`).

## Why this unit

claude-440's card made A1, Pick over the D11 trunk codes, P1's S5 copy path. It left one question
for a later unit [claude-440, "What this card does not decide"]:

> EM gives Pick about 10‰ of the mixture in the distance > 0 class, against 190‰ at distance 0
> […]. A split by distance (1, 2, 3+) may change that.

claude-439's REPORT lists the same split as untested ("Still open"). In claude-439, every fired
read at distance 1 or more shares one class of buckets. A near pair at distance 1 is likely to be
right far more often than one at distance 12. One shared weight averages the two, so the weight
for distance 1 is probably too low and the weight for distance 12 too high [claude, untested].

## The question

Does splitting the fired class by pair distance lower DEV-tune bits beyond claude-439's
three-class A1? And does the D11 codes' edge over a permuted code map (C3) change under the
split?

## What changes

1. **A second bucket scheme beside claude-439's.** Five Pick classes: did not fire, fired at
   distance 0, 1, 2, and 3 or more. Each class holds one copy of the 176 base buckets (11 orders
   × 16 total classes), so 5 × 176 = 880 buckets, inside S6's 4,096.
   - The distance is the XOR + popcount distance the read already returns in each row
     (`PickRow::distance`). The device read and the host read are not changed.
   - Distance 0 is claude-439's exact class unchanged, so only the near class is split.
2. **claude-439's three-class scheme stays as it is.** `PICK_CLASSES`, `from_lookups_with_pick`
   and `FrozenPickMixture` keep their meaning: codex's `s5-mixture` and the frozen form use them.
   The split is a new constructor and a new constant beside them.
3. **Two new arms in the `--pick-arms` run, and claude-439's five unchanged:**

| Arm | Pick read | Buckets | Role |
|---|---|---|---|
| A0, A1, C1, C2, C3 | as in claude-439 | 1 or 3 classes | the replication: must match claude-439's 100% JSON |
| A1s | A1's rows | 5 classes | the arm under test |
| C3s | C3's rows (permuted codes) | 5 classes | geometry under the split |

   C2 needs no split arm: exact pairs fire only at distance 0, so its split equals its
   three-class table.
4. **New bands**, paired over DEV-tune's 2,670 records as before: A1s − A1, A1s − C3s,
   A1s − C2 and C3s − C3.
5. **A split reading, fixed before any number is read:**
   - `no_split_gain`: the overall A1s − A1 band does not lie wholly below 0;
   - `split_gain_geometric`: A1s − A1 and A1s − C3s both lie wholly below 0;
   - `split_gain_no_geometry`: A1s − A1 lies wholly below 0, and A1s − C3s does not.

   claude-439's own reading map is computed from the same five arms and must come out
   `geometric_gain` again.

## Gates

- **G1, laptop:**
  - host tests of the split class rule (no distance → 0, distance 0 → 1, 1 → 2, 2 → 3, every
    distance ≥ 3 → 4);
  - the 880-bucket rule on fixtures, and a three-class table that is byte-identical to the one
    `from_lookups_with_pick` builds;
  - a refusal when a row's split class disagrees with its three-class class;
  - every existing `kn_ceiling` and instrument test still passes;
  - stub-nvcc `cargo check`s of the instrument are clean, and the D32 grep of the added lines is
    clean.
- **G2, negatives,** each fix set aside against the base, run, expected to fail, then restored:
  - mapping distance 3 to class 3 fails the class-rule test;
  - a split table built with three classes fails the 880-bucket test.
- **G3, wick's RTX 4090, after a prep sheet and a non-author FIT:**
  - one run at 100% of C on claude-439's split v1 (its hashes checked first), with
    `--check-host`, so B1 (device rows = host rows, and the trunk oracle) is run again;
  - **instrument check first:** A0, A1, C1, C2 and C3's DEV-tune `bpb_micro`, their bands and
    B2 must equal claude-439's `run2/pick-p1000.json`. If any differs, the split is not read and
    the difference is the finding (L-007);
  - B2 and every B3 must hold, as in claude-439;
  - a completion watcher is armed, and the result is copied off wick before the REPORT.
  - DEV-test is not read; it stays sealed for gate (b).

## Decisions recorded here

- **Distance cut at 3.** It needs no fitted boundary, and claude-440's card names it. A finer cut
  can follow if the split gains. [claude]
- **One 100% run, not four sizes.** claude-439 already measured the size trend. The question here
  is the bucket scheme at P1's size. The run reuses the device reads, so it costs about one hour of
  wick time, like one of claude-439's runs. [claude]
- **wick, in the same `uor-r4-claude` container as claude-439,** so the replication check reads
  the same inputs and toolchain. Bare metal is allowed, but it would add a toolchain difference to
  the instrument check. [claude]
- **The split does not change S5 by itself.** If A1s wins, the REPORT recommends it, and a
  decision card under §5.1 changes S5's bucket scheme. claude-440's reversal rule is untouched.
  [claude]

## Files in scope

- `crates/uor-r4-discrete/src/kn_ceiling.rs`: the split constant, the class rule and a constructor
  with a class count; `CeilingTable` carries that count.
- `docs/labs/kn-reference/instrument/src/pick_fit.rs`, `pick_run.rs`: the two arms, the bands and
  the split reading.
- `docs/harness/prep/`: the prep sheet.
- `docs/loop/tasks/written-state-count-model/claude-441/**`: ORDER, the run packet, REPORT.
- `docs/research/notebook/`: one row in the line's notebook.

## Stop list

- No change to the CUDA read, `discrete.cu`, `frozen_pick.rs`, `s5-mixture.rs` or the meaning of
  `PICK_CLASSES`.
- No change to the 13 count experts, `EmStart` or `EM_ITERATIONS`.
- DEV-test is never opened. No D87 text, stream, mask or count table leaves wick or is committed.
- codex's units and files are not edited.

## Done when

1. ORDER is the first commit on the branch and REPORT the last.
2. G1 and G2 pass on the laptop, with their output in the REPORT.
3. G3 ran once at 100%, the replication check is reported first, and the split is read only if it
   passed.
4. The notebook row is in, `UOR_LAB=claude uor-harness check` shows no new failure, and
   `uor-harness loop pr-check --local` passes or its output is reported.
