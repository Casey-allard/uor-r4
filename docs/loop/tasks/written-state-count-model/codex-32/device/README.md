# Original tape and device decision words

Build the standalone Rust packet tool on Wick using the locked manifest. The tool reads the original D26 model, its bound tokenizer and the two-column D30 user-text TSV. It emits each complete original prefix with its native content bounds, the sparse role bank and both vocab site classes. Mode 4 is historical/inherited, 5 full/inherited, 6 historical/repeated and 7 full/repeated. The four headers differ only in mode. The role state and row bytes are identical.

On the leased device, compile the [D19 operator](../../codex-19/operator/device.cu) with nvcc for the card's native SM and an explicit cubin target. Check the loaded image, then pipe each mode's packet to the executable on stdin and save stdout. Each row contains eight little-endian u32 words: requested start and end, excluded start and end, requested and excluded best score, requested and excluded runner-up. Missing spans are UINT32_MAX. Scores use the two's-complement representation of a signed i32. Compare the raw result to independent ordinary client choices and margins before interpreting any arm.

The following commands illustrate the input stages. Each output directory must be new.

```sh
cargo build --release --locked --manifest-path "$device/Cargo.toml"
role-endpoint-packet finite "$out/finite"
role-endpoint-packet pack "$d26" "$tokenizer" "$blind_tsv" "$out/real"
```

Use only the finite packets before the registered GO. The planted outputs are specified independently of ranker execution. The final row tests a continuation token immediately outside the framed content. The real panel's semantic key is read only by the independent grader.
