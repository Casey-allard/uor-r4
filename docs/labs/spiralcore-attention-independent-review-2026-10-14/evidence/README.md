# Evidence: independent reproduction of the #2480 byte/data-structure checks

Everything here is local byte/data-structure work. No tokenizer, model, reference, attention
simulation, CUDA, pod, training or active-run change ran. The adapters are the unmodified
retained scripts from `docs/labs/spiralcore-native-attention-review-2026-10-14/evidence/`
(SHA-256 as listed in that directory's `SHA256SUMS`), copied to an owned scratch directory and
pointed at the original attachments by absolute path. Results are compared byte for byte with
the retained outputs.

## Inputs

| Attachment | Path used | SHA-256 |
|---|---|---|
| A6 saved HTML | `/Users/casey.allard/Downloads/view-source_https___spiralcore.netlify.app.html` | `770142b4f343842cd0693d362d49f95272a5ded029d9682211a5215d64810fd8` |
| A2 Showcase (extracted) | `/Users/casey.allard/Downloads/SpiralCore_Showcase/` (`core.js` `7ca3dcfb…`, `transport.js` `1d254a75…`, both re-pinned by the adapter) | ZIP `6399240e51ad5f5347390690ce45f460eda390bbe1d9c5e86dee0e8fd5c26754` |

## Commands (from the scratch directory `local/repro/` inside the review worktree)

```sh
/usr/bin/time -l node scripts/check-source-codecs.cjs "$HTML"      > network/results.json
/usr/bin/time -l node scripts/check-current-golden-codec.cjs "$HTML" > physics/golden-results.json
/usr/bin/time -l node scripts/make-fixtures.cjs "$SHOWCASE" "$PWD/cs-fixtures"
/usr/bin/time -l rustc --edition=2021 -O scripts/range_reader.rs -o range-reader
/usr/bin/time -l ./range-reader cs-fixtures                          > cs-fixtures/results.txt
```

Node `v26.4.0`, `rustc 1.97.1 (8bab26f4f 2026-07-14)`, macOS (Darwin 27.0.0), 14 October 2026.

## Results

| Directory | File | Retained comparison | Exit | Wall | Max RSS |
|---|---|---|---:|---:|---:|
| `network/` | `results.json` | byte-identical to retained `network/results.json` | 0 | 0.61 s | 70,451,200 B |
| `physics/` | `golden-results.json` | byte-identical to retained `physics/golden-results.json` | 0 | 0.09 s | 65,568,768 B |
| `cs/` | `construction.json` | identical except `constructionNanoseconds` (timings are not expected to match) | 0 | 0.11 s | 62,439,424 B |
| `cs/` | (build) | `range_reader.rs` compiles with the edition flag | 0 | 1.23 s | 144,621,568 B |
| `cs/` | `results.txt` | byte-identical to retained `cs/results.txt` (813 ranges, 2,374 checks, PASS) | 0 | 0.15 s | 3,276,800 B |

The `*-resource.txt` files are the complete `/usr/bin/time -l` outputs. Wall times are single
local observations, not benchmarks. Generated fixtures (`*.raw`, `*.records`, `*.manifest`) and
the compiled binary are reproducible from the commands above and are not retained.

## Tools retained here

- `tools/pdftext.swift` — the PDFKit text extraction used to read A1 and A3 on this machine
  (`swift tools/pdftext.swift <file.pdf>`; prints `pages=N` on stderr). The poppler tools were
  not installed and were not installed for this review.
- `tools/induction_stat.rs` — a standalone, dependency-free Rust data statistic (no model):
  for each eligible target under the corpus window policy (response-masked targets,
  within-document predecessor, sliding prefix of at most `context` tokens, documents start at
  BOS id 0, `corpus.rs:246-290`), it counts targets recoverable by a copy-successor read (the
  query token recurred earlier in the window and the token after its most recent earlier
  occurrence equals the target), with distance buckets, the any-occurrence variant and the
  copy ceiling (target present anywhere in the window). Build:
  `rustc --edition=2021 -O tools/induction_stat.rs -o induction_stat`; run:
  `./induction_stat tokens.u16 response_mask.u8 192`.

**Result on the pinned DEV corpus** (`induction-stat-dev.txt`; the files were verified against
the `seed1-config.json` pins `tokens.u16` `0919e90f…` and `response_mask.u8` `a838bb1a…`
before use, copied from a read-only location into this review's own scratch directory; 0.17 s,
2,457,600 B max RSS):

| Quantity | Count | Of 14,287 |
|---|---:|---:|
| eligible targets (matches the registered DEV panel exactly) | 14,287 | 100% |
| query token recurs earlier in the window | 8,648 | 60.5% |
| **most recent earlier occurrence's successor equals the target** | **3,847** | **26.9%** |
| … of which at distance 1-8 / 9-32 / 33-96 / 97-192 from the query (disjoint buckets) | 50 / 2,872 / 900 / 25 | — |
| any earlier occurrence's successor equals the target | 5,182 | 36.3% |
| target token present anywhere in the window (copy ceiling) | 8,248 | 57.7% |

This is a property of the data and the window policy, not of any model. For scale: the seed-1
model at Q1 has 1,090 top-8 hits and the masked-TRAIN count reference has 4,060 on the same
panel.

## Scope

These runs establish that the retained adapters and the Rust reader behave identically on a
second machine session against the pinned attachments. They do not test waveform recovery,
browser execution, physical networking, the tokenizer, any model read or writer, learning, or
GPU parity.
