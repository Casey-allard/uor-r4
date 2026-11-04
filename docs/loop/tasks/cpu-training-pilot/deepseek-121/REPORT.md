# REPORT: long-window CPU acceptance, deepseek-121

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `b6561fe6e`.

## What did not work, first

- **This is a finite fixture, not a trained long-window model.** It proves that
  the existing CPU oracle, full served-forward audit and integer serving path
  accept and agree at 2,048 positions. It does not establish long-context
  language quality.
- **The boundary moved while this unit was in progress.** Claude's PR #3646
  merged during the unit. This branch was merged forward from the new main and
  all checks were rerun, but the unit deliberately uses only synthetic data and
  does not edit `uor-r4-float-train`.
- **No tracked run artifact is produced.** The fixture and both audits live in
  an in-memory test, so the retained record is the test plus this report rather
  than a saved UORLUT model.

## What changed

The CPU served-forward test module now exports a deterministic width-64,
three-layer FBS fixture at context 2,048 with:

- Hamming copy and type reads using mixed copy keys `[2, 1]`;
- an opening-memory row and a reply-memory row;
- a point-key row derived through the CPU forward path;
- a deliberately constructed long-range copy at position 1,024 whose kept
  set includes early position 100.

One release test runs the complete audit over exactly 2,048 tokens. It checks
2,048 x 64 Q12 coordinates, 2,048 token decisions and 2,048 x 8 kept cells
against integer serving, requires all mismatch counters and the instrument
fault bit to be zero, and compares both decision hashes. It also parses the
exported records, exercises turn memory and point fallback, checks more than
100,000 Hamming candidate comparisons, inspects the retained long-range copy,
repeats the audit exactly, and requires token 2,049 to fail as context
exhaustion.

The change is 177 added and 2 removed source lines, plus documentation. The
ORDER did not set a line estimate.

## Measurements

The fixture used an AMD Ryzen 9 9950X at 100% of one process CPU. The focused
release command was:

```sh
/usr/bin/time -f 'elapsed=%e rss_kib=%M cpu_percent=%P' \
  cargo test -q -p uor-r4-cpu-train --release \
  long_window_full_path -- --nocapture
```

It passed one test in 1.53 s and completed in 1.61 s at 52,676 KiB peak RSS.
The complete audit reported:

| Reading | Value |
|---|---:|
| Positions | 2,048 |
| Q12 query coordinates | 131,072 |
| Token decisions | 2,048 |
| Kept cells | 16,384 |
| Query-coordinate mismatches | 0 |
| Token mismatches | 0 |
| Kept-cell mismatches | 0 |
| Equal CPU/serving decision hashes | yes |
| Hamming candidate comparisons | 16,752,648 |
| Turn-memory lookups | 2,046 |
| Point-table rows on opening miss | 1 |
| Long-range retained copy | position 100 at row 1,024 |
| Token 2,049 | refused as context exhaustion |
| Second audit | byte-identical non-timing result |

The test prints those exact mechanism counts when run with `--nocapture`.
They are asserted to be non-vacuous but are not a capability score.

## Checks run at the head

- Focused release long-window test: **1 passed** in 1.53 s.
- Full CPU crate release suite: **190 library tests passed, 4 ignored**, plus
  binary suites **6, 1, 5, 5 and 13 passed**; all other binary targets ran 0
  selected tests. Total: **220 passed, 0 failed, 4 ignored**.
- `cargo clippy -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`:
  passed; dependency crates emitted pre-existing dead-code warnings.
- `cargo fmt -p uor-r4-cpu-train -- --check` and `git diff --check`: passed.
- Report, roadmap, documentation catalog, prep and full loop gates: recorded
  in the exact-head PR review after they run.

## Skills used

No available Codex skill applied. The FBS artifact format, served-forward audit,
integer trace and existing CPU regression patterns were inspected directly.

## Still open

- Claude's first materialized 2,048-token trained export and its CPU audit.
- A public or corpus-trained artifact exercising Hamming, turn memory and point
  fallback together.
- Long-context language quality, complete replies and fresh-phrasing transfer.
- GPU qualification under the project's standing compute authority.

## Plain summary

The CPU stack had no full-path test at the 2,048-position boundary. This unit
creates a deterministic long-context model fixture with the project's sparse
copy, turn memory and point lookup all switched on, then checks that the CPU
training-side computation and integer serving produce exactly the same query,
token and retained-cell decisions at every position. It also proves that an
earlier token remains available to a much later copy read, that replay is
stable, and that an over-limit sequence is rejected. This prepares the audit
for Claude's first trained long-context export; it is not evidence that the
long-context model has learned language.
