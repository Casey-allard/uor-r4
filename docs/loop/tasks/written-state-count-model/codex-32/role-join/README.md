# D32 role parity and summary

Run the post-score join on Wick after the primary and independent cold
CUDA launches, the primary and cold ordinary captures, and ordinary
grading have finished.

```sh
cargo build --locked --offline --release --manifest-path "$join/Cargo.toml"
"$join_binary" "$packets" "$cuda_primary" "$cuda_cold" \
  "$ordinary_primary" "$ordinary_cold" "$grades" > "$joined"
```

The packet's 22 ordered index entries bind each prefix to the SHA-256 of
its original token IDs serialized as consecutive little-endian `u32`
words. The join checks all four packet shapes, identical states and tapes,
and original IDs and user-frame bounds against the ordinary client.
The two CUDA roots each contain `mode-4.out` through `mode-7.out`. The
join requires exactly eight little-endian words per row and compares
both complete outputs to the ordinary client, including all four score
words. The ordinary cold output must also match the primary bytes. Each
graded row must carry the same role words, selected source and output IDs
as its ordinary capture.

The result groups role presence, role-specific admission, assignment,
source correctness, first differing ID, through-EOS exactness and
unsupported copies by arm and stratum. Each row also lists every original
historical-to-arm output ID difference at its zero-based emission offset.
A null ID denotes an extra or missing token at that offset.
