# REPORT: one-model-end-to-end, claude-316

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). Laptop only; no GPU, no spend.

## What did not work, first

- **Every speed figure the scorer printed before this change understated decoding by about 2.6×.** `tokens_per_second` timed one full artifact parse per row along with the decode. On claude-313's seed-1 artifact the old figure is 209–243 tokens/s, while the same rows decode at 565–646 tokens/s. claude-311, claude-313 and claude-314 quote the old figure. Their rows are unaffected, but their speed lines measured the parse.
- **My first version of the timer under-counted.** It summed each row's whole milliseconds, so every row dropped up to 1 ms: up to 256 ms over the panel, about 4% of the decode time, which flattered the speed. I caught it before review. The timer now sums exact durations and rounds once. The gap between wall time and decode time fell from 168–170 ms in round 1 to 45–48 ms in round 2, as expected.
- **`decode_milliseconds` is not per-token decode alone.** It is the time inside `generate`: priming the prompt plus the reply steps. Prompt tokens are not counted in `reply_tokens`, so `decode_tokens_per_second` is reply tokens over prompt-plus-reply time. That is a lower bound on the step rate. Splitting priming from stepping is not done here.
- **`cargo fmt --check` fails on this crate at the base**, in `corpus-eos-exposure.rs` and `main.rs`, files this unit does not touch. I did not reformat them (out of scope). The changed file passes `rustfmt --check` on its own.

## What changed

- **`served-reply-panel`** (`docs/labs/one-model-end-to-end/instrument/src/bin/served-reply-panel.rs`):
  - it builds the forward once, before the timed loop, and reads `context_length` from it;
  - each row decodes from a `clone()` of that untouched forward, which starts each row in the same state as a fresh parse;
  - a `Duration` sums the time inside `generate` only;
  - two new report fields, `decode_milliseconds` and `decode_tokens_per_second`, sit beside the unchanged `wall_milliseconds` and `tokens_per_second`.
- No model, artifact, row, bar or score changes.

## Measurements

claude-313's seed-1 artifact (`1c9ae43c…`) on `m2-fresh256`, 256 rows, 3,667 reply tokens. Three runs each, alternating old and new, on this laptop's CPU.

| Scorer | Run | Rows equal to claude-313's `score/s1.json` (`5175c29b…`) | wall ms | tokens/s (wall) | decode ms | tokens/s (decode) |
|---|---:|---|---:|---:|---:|---:|
| old `be3d4cb0…` | 1 | yes | 17,533 | 209 | | |
| new `45f6692a…` | 1 | yes | 6,057 | 605 | 6,012 | 609 |
| old | 2 | yes | 15,052 | 243 | | |
| new | 2 | yes | 6,528 | 561 | 6,480 | 565 |
| old | 3 | yes | 16,854 | 217 | | |
| new | 3 | yes | 5,716 | 641 | 5,668 | 646 |

- **The parse cost** is about 10 s of the old 15–17.5 s wall, about 40 ms per row.
- **Round 1**, with the truncating timer (`664dce22…`), also gave equal rows in all six runs. Its decode figures were 599–707 tokens/s. They are kept in `local/claude/claude-316/round1/` and superseded.

## Checks run at the head

- **Rows:** all 12 runs (both rounds, old and new) wrote rows that `jq -S '.rows'` makes byte-equal to claude-313's `score/s1.json`. The script is `local/claude/claude-316/compare-316.sh`, and it pins the artifact's and the reference's hashes before scoring.
- **The decode time is at most the wall time** in every new run.
- `cargo build --release --locked --offline --features cpu-parity --bin served-reply-panel`: ok.
- `cargo test --release --locked --offline --features cpu-parity` in the instrument crate: 46 passed, 0 failed.
- `rustfmt --check --edition 2021` on the changed file: clean.
- **Banned literals and float literals** in the added lines: none.
- `loop report --check` on this REPORT.

## Skills used

None beyond the loop's own steps.

## Still open

- **Re-quote speed with the new fields.** claude-315's reading will be the first to report `decode_tokens_per_second`. It still scores with `be3d4cb0` for comparability with claude-313's rows. The speed line will come from this scorer on the same artifacts.
- **Split prompt priming from reply steps**, so the figure is a pure step rate. That needs a second timer inside `generate`, or prompt token counts in the report.
- **The crate's `cargo fmt` drift** in two files this unit did not touch.

## Plain summary

The scorer's speed number was mostly measuring how long it took to load the model file, once for every question. It now loads the model once and times only the answering. The answers are byte-for-byte the same. The served model's real speed on this laptop is about 565–646 tokens per second, not the 209–243 reported before.
