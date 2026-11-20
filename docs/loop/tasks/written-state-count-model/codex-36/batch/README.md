# Resident CUDA compact batch reader

This instrument produces the exact count features needed by the next R-only
fit. It uses the previously admitted direct compact lookup kernel for every
batch and keeps one immutable upload alive throughout the run. The native
compact reader supplies the fixed finite CPU oracle. See [the finite FIT
sheet](SHEET.md) for the case, bounds and admission conditions.

Build on Wick from the exact reviewed source and the pinned sm_89 image.
Use the image containing the exported compact lookup symbol. The frozen
D36 CUDA executable contains a second image without that symbol.

```sh
cuobjdump -xelf all /path/to/admitted/d36-count-device
export D36_BATCH_CUBIN="$PWD/d36-count-device.2.sm_89.cubin"
export CARGO_TARGET_DIR="$private_batch_target"
cargo build --offline --locked --release --manifest-path /path/to/batch/Cargo.toml
cargo test --offline --locked --release --manifest-path /path/to/batch/Cargo.toml
```

The finite modes build the planted UORT and the native expected tape.
The oracle mode is limited to 256 source IDs and a 32 MiB compact file.

```sh
exe="$CARGO_TARGET_DIR/release/d36-count-batch"
"$exe" fixture "$private_out/seams.uort"
"$exe" oracle "$compact" "$train_sha" "$compact_sha" \
  "$private_out/seams.uort" "$uort_sha" "$private_out/native.features"
```

Under the registered GPU lock, FIT and explicit GO, launch the finite CUDA
case in a fresh process for each batch size. Pass the verified actual compact
and UORT digests. Each output path must be fresh.

```sh
CUDA_DISABLE_PTX_JIT=1 "$exe" cuda "$compact" "$train_sha" "$compact_sha" \
  "$private_out/seams.uort" "$uort_sha" "$private_out/cuda-batch-3.features" 3
cmp "$private_out/native.features" "$private_out/cuda-batch-3.features"
```

The process prints counts, timings, and the live device admission reading
when it completes. An interrupted run leaves its exclusive partial artifact
for diagnosis. The exact row layout is in the FIT sheet.
