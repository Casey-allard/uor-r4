# Reproduce the finite observations

Use the pinned tokenizers and paired D11 files in the D15 ordered-role-transfer assets and the original D12 role files. Their per-seed hashes are in ../evidence/fixtures.json. Save computations to a new output directory. The archive listed in ../evidence/storage.json contains both seed packets and original device outputs.

Build the three Rust tools with `cargo build --release --offline --locked --manifest-path Cargo.toml` from this directory. Set BIN to this standalone package’s release binary directory, ASSETS to the parent/tokenizer directory, BANKS to the role-state directory, and OUT to a fresh directory. Recreate the approved token tapes and remove machine-local paths for the public receipt.

```sh
"$BIN/author-clause-fixtures" "$ASSETS" "$BANKS" | jq -f flatten.jq > "$OUT/fixtures-local.json"
jq 'del(.assets[] | .tokenizer,.paired_parent,.role_state)' "$OUT/fixtures-local.json" > "$OUT/fixtures.json"
cmp "$OUT/fixtures.json" ../evidence/fixtures.json
"$BIN/project-ordered-clause" "$ASSETS" "$BANKS" > "$OUT/positional.json"
for s in 1 2; do
    "$BIN/clause-coordinate-finite" "$s" "$ASSETS/seed$s-paired.d11" "$BANKS/seed$s.role" \
      "$ASSETS/tokenizer-seed$s.json" "$OUT/fixtures.json" "$OUT/seed$s"
    cmp "$OUT/seed$s/native.json" "../evidence/native-seed$s.json"
done
```

Build `device.cu` from the repository source tree with nvcc and SM89 machine code. The finite CUDA probe calls the D19 device score routine on each specified original span and its own clause. It emits six 32-bit key words followed by two signed score words, serialized little endian. Run twice in separate processes on the approved GPU for each original packet. Compare both output files byte-for-byte with the corresponding native.bin and the device-output checksum receipt. The positional projector is a host finite check on fixed token sequences. Its output consists of geometric feature coordinates before ranking or source lookup.
