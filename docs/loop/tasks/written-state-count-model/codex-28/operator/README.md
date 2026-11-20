# [codex] Original whole-word value starts

The extended ordered arm reads the same signed integer bank records as the frozen ordered arm. It preserves the frozen eligible-and-word-start intersection at connector depths one through three. After that window it visits every original word start before the assertion's first copy stop. The incumbent writer requires a real stop after one through seven copied tokens. A start at eight or more pieces remains in the trace as inaccessible. Continuation pieces and positions at or after the stop cannot become candidates.

The source, first word end and stop are original BOS-inclusive addresses. Each record retains the exact connector and copy distances. The geometric descriptor uses the last three adjacent pieces before a later source and its complete first word. Shorter connector depths use the original descriptor. Ordered root words and their START and continuation classes keep their tape order. No skip, rewrite or truncation changes a source address or the writer's copy extent.

Every reachable rival retains the original veto. Missing positive or negative class support and a positive distance outside the radius withhold the read. A nonpositive best margin or an equal best positive margin also withholds it. The signed margin is the difference of nearest negative and positive L1 distances divided by the number of represented pieces. Comparison uses integer cross-products.

## Device differential

Build the scorer ahead of time on the authorized CUDA host and compare the raw device output to the Rust packet score.

```sh
cargo run --locked --manifest-path "$D/Cargo.toml" -- "$Q/finite"
nvcc -O3 -std=c++17 -gencode arch=compute_89,code=sm_89 "$D/score.cu" -o "$Q/d28-score"
"$Q/d28-score" "$Q/finite/canary.d28i" "$Q/finite/ordered.raw" "$Q/finite/canary.device.d28o"
cmp "$Q/finite/canary.oracle.d28o" "$Q/finite/canary.device.d28o"
"$Q/d28-score" "$Q/held.d28i" "$Q/ordered.raw" "$Q/held.device.d28o"
cmp "$Q/held.oracle.d28o" "$Q/held.device.d28o"
```

Here `D` is this directory and `Q` is the fresh run directory. The bank is the original ordered raw bank format, without the sealed envelope. Rust builds each row with `all_starts::original_packet` and scores it with `Bank::score_original_row`. The device executable performs the same score and abstention independently on those packet descriptors. Packet staging, address enumeration, tokenizer, role selection, assertion selection and the exact writer remain on the ordinary client path.

The finite packet plants an early winner despite a later rival, a later winner behind a long connector, equal positive margins, unsupported rival geometry on both sides of a supported candidate, an inaccessible early source and a missing stop. The host generator checks known exact source addresses and individual candidate stages before writing the differential inputs.

All fields use little-endian u32 words. The input header is `[0x49383244, row_count, 1926, 0]`. Every row begins with `[tape_length, last_fact_piece, query_start, stop, old_mask, count]`. Each of the 192 slots is `[source, first_word_end, true_connector_pieces, copy_pieces, signature, packed_roots × 5]`. Inaccessible entries omit descriptor words. Unused slots are zero.

The output header is `[0x4f383244, row_count, 1922, 0]`. Every row starts with `[status, chosen_source]`. Its 192 trace slots are `[source, first_word_end, true_connector_pieces, copy_pieces, stage, positive_distance, negative_distance, positive_radius, signed_margin_numerator_bits, margin_denominator]`. Stage 1 is inaccessible, stage 2 has geometry without admission, and stage 3 has an admitted margin. Missing output words use `u32::MAX`. Final statuses match the integer onset policy: 0 selects, 1 has no candidate, 2 withholds for missing support or failed radius, 3 withholds for a positive tie and 4 withholds for a nonpositive best margin.

## Independent fit fixture

`train_canary.rs` writes eight planted TRAIN records, a complete expected raw bank and two rejected packets. The first signature has cross-family positive minimum distances of 8, 4 and 4. Its radius is 8. Counting same-family neighbors would produce 4. A positive signature with only one family and a negative-only signature both retain 65,535. The inverse-label collision and eight-piece negative must each exit 4 and write no bank.

```sh
rustc --edition 2021 "$D/train_canary.rs" -o "$Q/train-canary-host"
"$Q/train-canary-host" "$Q/train-canary"
"$Q/d28-onset-fit-device" "$Q/train-canary/canary.fit" "$Q/canary.fit.device.raw"
cmp "$Q/train-canary/canary.oracle.raw" "$Q/canary.fit.device.raw"
```
