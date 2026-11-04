# REPORT: cpu-training-pilot, deepseek-106 (artifact-bound point vote)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `35bfa2708`.

## What did not work, first

- **The first public run exposed an instrument defect.** The CPU FBS oracle
  parsed the new `vote_k` field but its independent point-key copy ignored the
  field and still used the one-row rule. Both seeds therefore failed the
  extracted-candidate versus served-candidate check. The CPU port now executes
  the same exact ordering. This was an instrument fault, not a mechanism
  negative.
- **A full sort is unnecessary for the served path.** The first exact
  implementation sorted all rows. The session now selects the first `vote_k`
  rows with `select_nth_unstable`, sorts only those rows exactly by
  `(distance, row)`, then counts targets. The legacy `vote_k = 1` path remains
  a linear scan and does not sort the table.
- **The voted mechanism remains weak.** Seed 1's opening vote was `3/60`
  against nearest `2/60`; seed 2 was `3/60` against nearest `1/60`. Seed 2's
  shuffled-target vote also reached `3/60`, so this unit repairs serving and
  does not turn deepseek-105's subthreshold result into a capability gain.
- **The WASM test harness does not compile on `wasm32` on current `main`.** Its
  unit tests call native-only worker and stage-timing helpers. The WASM library
  itself checks successfully; browser test execution remains unverified.

## What changed

`PointKeyImage` and `StackPointKey` now carry optional `vote_k`. Its default is
`1`, and the field is omitted from JSON at that value, preserving every
existing `/8` artifact byte for byte. Values of zero, values past the row count
and malformed JSON are refused.

At `vote_k > 1`, the native integer session and the independent CPU FBS oracle
rank rows exactly by `(L1 distance, row index)`, count targets in the first
`vote_k`, and break equal counts toward the smaller token id. Session scratch
is still allocated once at the table's row count. No nested Rayon pool was
added: independent examples remain the parallel unit.

`fbs-export --point-key --point-vote-k N` can now build a voted artifact;
omitting the flag preserves the default. The CPU point-transfer driver writes
`point-vote.uorlut`, carries `voted_artifact_sha256` and requires its vote to
match the independent extraction. The served-forward manifest reports
`vote_k`.

The source change is approximately 330 lines across code, tests and docs.
The ORDER estimated a finite schema/kernel/serving unit, so this is within its
intended scope.

## Measurements

Both seeds used the fixed deepseek-105 inputs and split. They ran concurrently
as two 16-worker jobs on the Ryzen 9 9950X:

| Seed | point-off / nearest / vote / shuffled opening top-1 | Full teacher-forced point-off / nearest / vote / random | Vote = served | Wall |
|---|---|---:|---:|---:|
| 1 | `0 / 2 / 3 / 2` of 60 | `169 / 171 / 172 / 169` of 2,050 | 60/60 | 1.444 s |
| 2 | `0 / 1 / 3 / 3` of 60 | `163 / 164 / 166 / 166` of 2,050 | 60/60 | 1.433 s |

Every arm on each seed retained zero query-coordinate, token and kept-cell
mismatches over 2,110 positions, 540,160 Q12 coordinates, 2,110 decisions and
16,880 kept cells.

Default outputs are byte-identical to deepseek-105:

- seed 1 point `ecd2c616…7ec56`, random `85ed6785…40569`;
- seed 2 point `5f127f79…08ece`, random `48b4e621…b52ff`.

Voted artifacts are seed 1 `0dd5a835…8fac` and seed 2 `cd76559c…eee6`.
The complete comparison is [comparison.json](evidence/comparison.json).

## Checks run at the head

- Focused integer point tests: 14 passed.
- Focused CPU point-transfer driver: 8 passed.
- Full `uor-r4-integer` library suite with `serving-wasm`: 454 passed, 1
  ignored.
- Full `uor-r4-cpu-train` library plus binary suites: 163 library tests
  passed with 4 release benchmarks ignored, followed by the complete binary
  suites, including corpus `6/6`, counterfactual `1/1`, discrete `5/5` and
  point transfer `8/8`.
- WASM library check on `wasm32-unknown-unknown`: passed.
- Strict `uor-r4-cpu-train` Clippy with all targets and `-D warnings`: passed.
- Formatting, diff whitespace, evidence checksums and evidence JSON: passed.
- Strict all-target `uor-r4-integer` Clippy is **not green** under the current
  Rust 1.97 toolchain: current `main` already has 51 library and 70
  test-target warnings across unrelated code. None is in the new point-key
  implementation; the pre-existing warnings in `format.rs` and `session.rs`
  remain outside this unit.

Report/policy gates, exact-head review, merge and fresh-main verification are
run after this report head is frozen and recorded in the PR.

## Skills used

No available Codex skill applied.

## Still open

- The eight-row vote is served but remains a small, subthreshold mechanism
  result; it is not useful-chat or language capability evidence.
- Seed 2 has no margin over its shuffled-target vote.
- Browser unit tests need a WASM-compatible test split or cfg gate. Only the
  WASM library build is verified here.
- CPU remains development research under D84; GPU qualification remains open.

## Plain summary

The previous CPU experiment found that voting among eight nearby geometric
points could name a missed reply opening slightly more often than trusting one
point. This unit makes that rule part of the model file and proves the real
serving session, the independent CPU implementation and the full served-forward
audit all return the same token.

Existing one-point models keep exactly the same bytes and behavior. A new model
may record `vote_k = 8`; it ranks points by exact integer distance and row
order, counts the first eight targets and uses the smaller token id when counts
tie. On the two fixed public seeds the served rule reproduces the earlier
`3/60` opening result with zero parity mismatches, but that small gain does not
prove useful language ability.
