# Independent count-ceiling verification

Build this detached Rust instrument from the same tracked source bundle staged for collection and fitting. It uses no GPU and runs no count model, scoring, or EM. It streams packed rows into separate frame and KN digest accumulators, checks denominators and masks, and compares device receipts with the exact recorded controls.

```sh
cargo build --release --locked --manifest-path "$verify_dir/Cargo.toml"

kn-ceiling-verify "$frame_dir" "$source_sha256" "$collector" \
  --query "$inputs"

kn-ceiling-verify "$frame_dir" "$source_sha256" "$collector" \
  --query "$inputs" --fit "$fit_dir" "$fitter"
```

`--query` independently checks every row's BOS-derived parity, novel flag, and byte credit against the frozen UORT query, novelty and ID-byte files. It checks SHA-256 of each input it opens. The TRAIN digest and cut length are matched to the registered historical receipt. Full preflight independently hashes the actual TRAIN file. The verifier accepts only the four historical cut sizes for D87. `--fixture` admits a separately flagged synthetic frame and suppresses scientific classification.

Run the first command before starting a CUDA fit or reopening its frame. The second command compares the aggregate with separately written mode receipts, checks endpoint rules and exact Q32 starts and masks, and verifies the pure KN, uniform starts and held parity totals against the old runs. A fresh fit can use the original sealed frame and a new fit output directory. Compare objective and mode records word for word for a cold replay. Keep the independent verification JSON with each run.

At full size, the verification JSON records the pass-24 and selected-endpoint cutoff projections and their exact signed values. A lower-cut record contains only a same-cut reading. Use full-size results only after the cold pair and sizing checks, including all/novel KN containment, pass the registered checks. The optional certificate has its own review.

## Historical baselines

Each score cell is overall / novel Q12 bits. Both cuts use the whole-record prefix rule.

| TRAIN and query cut | Pure KN | Uniform U_count pass 24 | Uniform KN+U pass 24 | Parity U_split |
|---|---|---|---|---|
| 1‰ | 163,999,686 / 135,423,529 | 162,413,985 / 134,221,115 | 163,907,055 / 135,359,999 | 167,316,057 / 137,370,772 |
| 1% | 1,722,902,561 / 1,264,382,774 | 1,712,586,642 / 1,261,913,276 | 1,721,722,467 / 1,264,193,755 | 1,728,344,325 / 1,268,912,485 |
| 10% | 14,655,846,210 / 12,646,028,431 | 14,643,279,758 / 12,632,493,310 | 14,653,818,808 / 12,644,738,505 | 14,656,068,128 / 12,642,527,884 |
| 100% | 128,649,439,118 / 114,471,363,642 | 128,726,996,235 / 114,413,535,652 | 128,638,070,224 / 114,461,909,169 | 128,739,857,700 / 114,424,177,643 |

The verifier pins all 25 Q12 values in both historical uniform starts and both parity fit trajectories, all populated historical uniform U_count bucket weights and target counts, and the adjusted per-order triple and context digests. Historical receipts encode Q12 and weight values as decimal strings. The new fit receipt uses JSON integers. The old receipt has no independent digest for the full raw lookup tape. The 1‰ host raw-lookup oracle checks its semantics and the cold pair checks the new digest's replay identity. [Historical Wick slices](../../claude-433/run1/c433r/) · [Historical full ceiling](../../claude-433/run1/c433p/ceiling-p1000.json) · [Frozen plain DEV-tune KN](../../claude-422/run1/sweep-10.json)

## Final review

- Check source/archive and executable hashes against the live dispatch packet, plus count, lookup and KN CUDA image identities. A sealed manifest attests to the inputs it names, while the separate preflight proves which input bytes existed at dispatch.
- Check collection's exact UORT prefix rule and the first BOS toggle to odd. Confirm raw lookup counts and totals are stored as canonical little-endian u32/u64 words and that an unseen target in a live ML lane can have mass zero. Confirm incomplete collection never publishes the seal.
- Check the fitter's Q48 × Q32 arithmetic, remainder/tie order, absent and enabled-lane invariants, per-pass floor counts and disjoint folds. Confirm its finite witness compares every responsibility, update and score to the separate exact integer oracle. Confirm the deliberately broken raw lookup and Q32 renormalization fail their respective checks and the restored binaries pass.
- Check the complete registered trace through its first qualifying checkpoint and score that endpoint. Check pass-24 and endpoint projections against exact 85% and 90% comparisons. Verify all and novel containment for both near-KN modes on every collected cut before recording a full-data reading.
