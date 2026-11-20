# Exact reserved-TRAIN A1 capture

The registered run uses the source and scope in [SHEET.md](SHEET.md). Build for the leased RTX 5090 with an audited source manifest. Use a fresh build directory and fresh output directories. The production commands require non-author FIT, successful pre-run review, verified build and active lease. Run these from the leased host.

```sh
UOR_NVCC_ARCH=compute_120 CARGO_TARGET_DIR="$BUILD/target" \
  cargo build --offline --locked --release --features cuda --manifest-path "$FIT/Cargo.toml"
nvcc -O3 -std=c++17 -gencode arch=compute_120,code=sm_120 \
  "$FIT/device.cu" -o "$BUILD/d31-a1-device"
host="$BUILD/target/release/d31-a1-fit"
device="$BUILD/d31-a1-device"
mkdir "$FINITE"
"$host" finite "$FINITE"
"$device" "$FINITE/finite.rows" "$FINITE/finite.trace"
"$host" verify-finite "$FINITE" "$FINITE/finite.trace"

"$host" prepare "$CR" "$TRUNK" "$COMPACT" "$COMPACT_SHA256" "$RUN"
printf '%s  %s\n' "$(jq -er '.rows_sha256' "$RUN/prepare.json")" "$RUN/train.rows" | sha256sum -c
"$device" "$RUN/train.rows" "$RUN/fit.trace" "$RUN/probe-ordinals.u64" "$RUN/probe-masses.u64"
"$host" seal "$RUN"
```

The row-tape header is the ASCII magic `A1EM0001` and a little-endian u64 count. Each target-major row contains a little-endian u64 bucket followed by fourteen Q48 u64 mass words. The device trace header is `A1TR0001` and the same row count. Each of the 97 entries contains one u64 Q12 sum and 7,392 bucket-major u64 Q32 weights. Only the final weights are serialized in the existing lossless `UORS5W01` file. The private JSON probe packet preserves native ids and the separate device probe file stores one Q48 word per original ordinal.
