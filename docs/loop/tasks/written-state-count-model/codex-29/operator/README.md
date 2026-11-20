# [codex] Independently fitted negative-rival admission

The opt-in carrier wraps the complete D28 fitted whole-word D11 and retains its table bytes, root words, positive radii, original source addresses, role and assertion selection, and exact writer. Its envelope binds SHA-256 of the parent, the exact D28 TRAIN packet and the calibration. The loader requires the wrapper TRAIN digest to match the fitted parent's TRAIN binding and verifies the shape, signature, capacity and padding of the calibration before attaching it. A calibration can clear a veto but cannot admit a value, change an admitted candidate's margin, break a positive tie, move the fact head or alter a copy step.

For each exact shape and signature, the CUDA fitter takes every actual D28 negative and finds its nearest negative in a different TRAIN family by the original integer D11 L1 distance. The radius is the maximum of these nearest distances. A negative-free signature or a signature in which any negative has no independent-family support stays unavailable. The D28 TRAIN record's signed class, roots, signature, shape and order must match the original fitted bank byte for byte. The family ordinal is checked against the frozen TRAIN roster.

Only an unresolved D28 rival can clear. Its nearest negative must be inside the calibrated negative radius, inclusive, and strictly nearer than its nearest positive if a positive exists. Both missing classes, equal class distances, absent calibration and out-of-radius negative evidence retain the veto. Already admitted competitors use their unchanged D28 signed margin and original address.

The shuffle control rotates the finite radius values by one position across sorted exact shape and signature keys. It preserves the same eligible signatures, signed exemplars and missing radii. The host command reports the number of signatures whose radius actually changes. When that number is zero the control is non-informative.

## Packet and carrier interfaces

The input and trace headers and sizes are unchanged from D28. Trace stage 4 marks a cleared original rival. For stages 2 and 4, candidate word 8 holds the independently fitted negative radius, with 65,535 marking unavailable geometry. Stage 3 retains the incumbent signed numerator and denominator. An admitted contender's negative radius can be recovered by joining its input shape and signature with the bank and calibration, whether or not the control ever consults it.

The 1,040-byte calibration contains little-endian u32 words. Its header is `[0x39324e52, used, 256, 0]`. The next 256 words give a radius for each original bank record. Every used record of the same shape and signature gives the same radius. An unsupported used slot is 65,535 and unused slots are zero. The outer carrier schema is `uor-r4.lut-discrete-rival-admission/1`. Its JSON envelope is followed by the raw calibration and complete original D28 carrier without a byte conversion.

```sh
cargo build --locked --offline --manifest-path "$D/Cargo.toml"
nvcc -O3 -std=c++17 -gencode arch=compute_89,code=sm_89 "$D/device.cu" -o "$Q/d29-rival"

"$HOST" finite "$Q/finite"
"$Q/d29-rival" fit "$Q/finite/train.packet" "$Q/finite/ordered.raw" "$Q/finite/actual-calibration.raw"
cmp "$Q/finite/actual-calibration.raw" "$Q/finite/expected-calibration.raw"
"$Q/d29-rival" score "$Q/finite/score.d28i" "$Q/finite/ordered.raw" "$Q/finite/actual-calibration.raw" "$Q/finite/actual-score.d28o"
cmp "$Q/finite/actual-score.d28o" "$Q/finite/expected-score.d28o"
```

The nine finite cases plant true early and late sources, later-rival clearance that preserves an early winner, two admitted candidates with equal margins, strict class ties, unavailable independent-family evidence, out-of-radius and missing-both vetoes, and negative-only clearance without a source. The independent finite fit oracle derives radii 4 and 16 from explicit root coordinates. The malformed fit packet and source packet must exit 4. The inconsistent calibration must exit 3. None should write an output.

Once the registered TRAIN and device reviews are complete, the same interface reads the unchanged D28 inputs.

```sh
"$Q/d29-rival" fit "$TRAIN" "$RAW_BANK" "$Q/fitted-negatives.raw"
"$HOST" shuffle "$SEALED_BANK" "$Q/fitted-negatives.raw" "$Q/shuffled-negatives.raw"
"$HOST" pack "$D28_FITTED_WHOLE" "$TRAIN" "$Q/fitted-negatives.raw" "$Q/signed.d11"
"$HOST" score "$INPUT_PACKET" "$SEALED_BANK" "$Q/fitted-negatives.raw" "$Q/host.d28o"
"$Q/d29-rival" score "$INPUT_PACKET" "$RAW_BANK" "$Q/fitted-negatives.raw" "$Q/device.d28o"
cmp "$Q/host.d28o" "$Q/device.d28o"
```

Here `D` is the operator directory, `Q` is the run directory, and `HOST` is the built `d29-rival-host` executable. Only the device fitter creates actual TRAIN-derived radii. Host scoring evaluates the shared integer packet operator for exact device replay. The ordinary client exercises the wrapped carrier, original role and assertion binding, generated IDs and terminal EOS.
