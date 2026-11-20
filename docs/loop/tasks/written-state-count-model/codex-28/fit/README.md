# D28 signed native onset fit

`prepare` loads the exact D21 ordered carrier, binds the TRAIN tape and the TRAIN gold SHA-256, and independently validates original coordinates, the loaded lexical classes, same-word continuations, original token IDs and D11 roots. The generated private little-endian packet has a four-word header containing magic `0x38324644`, record count, width 13 and reserved zero. Records retain ordinal, absolute source, exclusive first-word end, exclusive assertion fence, lexical family, shape, signature, polarity (0 positive, 1 negative), and five words containing the ten packed u16 D11 roots. Shape encodes descriptor depths 1–3 and first-word width independently of the original source offset. No shifted coordinates or synthetic legacy slots are introduced.

D17's original packet has only three candidate slots. The new CUDA input front end fills the same 256-record bank ABI directly from the exact original descriptors. The existing D17 distance, leave-one-family-out nearest-positive search, radius and final bank validation are reused. No float state is involved. The fit uses only TRAIN. Its output is a normal ordered-mode raw bank. The host checks that the CUDA output kept every ordered descriptor and label, then writes the D17 8,416-byte seal with the original model, tokenizer and site bindings. The seal binds SHA-256 of both the fit packet and the receipt that in turn binds the original IDs and labels. The experiment driver must carry executable and source pins in its run receipt.

Run the following only after the unit has a non-author FIT and pre-run release. Prepare and seal are host format checks; the only learned radii are produced by CUDA. AOT build on the leased Wick host using a fresh output directory. Preserve the build log and standard error, and repeat in a fresh output root for byte equality.

```sh
CARGO_TARGET_DIR="$BUILD/host" cargo build --locked --offline --release --manifest-path "$FIT/Cargo.toml"
host="$BUILD/host/release/d28-onset-fit"
"$host" prepare "$TRAIN" "$PARENT" "$RUN/prepared"
nvcc -O3 -std=c++17 -gencode arch=compute_89,code=sm_89 "$FIT/device.cu" -o "$BUILD/d28-onset-fit-device"
"$BUILD/d28-onset-fit-device" "$RUN/prepared/train.packet" "$RUN/ordered.raw"
"$host" seal "$PARENT" "$RUN/prepared" "$RUN/ordered.raw" "$RUN/ordered.bank"
```

The frozen ordered bytes and the new ordered bytes are inputs to independent legacy and extended enumeration arms. The oracle tapes never participate in fitting or nominal selection. A failed writer or a fitted profile missing one class remains a result at that stage. The fitted bank must not be replaced by a holdout-informed radius.
