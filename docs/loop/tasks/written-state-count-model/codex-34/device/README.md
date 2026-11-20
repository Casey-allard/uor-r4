# D34 original-token role and assertion replay

The Rust packet tool loads the exact D26 recurrence artifact and bound tokenizer, and uses the D32 exposed user-text tape in protocol-2 framing. It exports the unchanged frozen role and assertion sites, sparse role counts, and the tokenizer's vocabulary facts as integer words. It sends no key, output target, predicted roles, or source selections to the device. The independent CUDA kernel replays the two ranks and records the detached endpoint and factual witness before the rank chooses a winner. The ordinary integer client measures source and writer behavior.

## Device interface

Each packet begins with four little-endian u32 words carrying magic `0x31443452`, mode, row count and cell count. It then contains the following arrays.

- 4,096 sites with five words per site. The words carry the role root, inherited role class, inherited assertion class, sentence delimiter flag and tokenizer lexical class. The lexical classes are 0 for other, 1 for one exact raw ASCII blank, and 2 for a nonempty raw ASCII alphabetic piece.
- The specified count of sparse cells with four words per cell. The words carry the offset/root key and the unchanged unspecified, requested and excluded counts.
- The specified count of rows, in `index.tsv` order. Each contains 192 original IDs padded with zeros, full prefix length and the two user-content offsets. The index binds each row ID to the SHA-256 of its unpadded original IDs in little-endian order.

Mode 7 replays D32 full-user repeated handling. Mode 8 adds detached endpoint discovery. It requires a singleton blank and at least one lexical piece in the frozen continuation class. The full continuation chain must fit in the framed user content. The earlier factual witness begins after a frozen sentence delimiter inside the user content and must match the complete original tape, including the blank. A single punctuation token can bound a complete span. A blank followed by punctuation cannot start one. An unsupported detached mention enters the candidate set and cannot enter the requested rank without a matching earlier head. The excluded rank retains its historical evidence comparison and floor.

In each output row, eight D32 decision words precede 192 position records of five words. The total is 968 words or 3,872 bytes per row. The decision words encode requested start/end, excluded start/end, requested/excluded best scores and requested/excluded runner-up scores. Scores preserve the signed i32 bit pattern. Each position record encodes detached candidate end, earlier witness start/end, detached factual head end and flags. The index of the record is the original candidate or factual head start. Flag bits zero and one reflect requested and excluded rank admission respectively. Missing endpoints use `UINT32_MAX`. All position records in mode 7 are empty. Old endpoints and pre-ranking details are available in the ordinary trace.

## Wick build and run

Build the standalone Rust tool and create fresh fixture and exposed packet directories.

```sh
cargo build --release --locked --manifest-path "$device/Cargo.toml"
detached-endpoint-packet finite "$work/finite"
detached-endpoint-packet pack "$d26" "$tokenizer" "$blind" "$work/exposed"
(cd "$work/finite" && sha256sum index.tsv mode-{7,8}.{bin,expected} > packets.sha256)
(cd "$work/exposed" && sha256sum index.tsv mode-{7,8}.bin > packets.sha256)
bash "$device/build-wick.sh" "$work/aot"
```

The Wick build emits native SM89 cubins, verifies that the payload has no PTX, checks the application entry point and inventories every disassembled instruction. Launches verify the authorization manifest and its exact external digest, bind the staged executable and selected packets transitively, acquire the standing Wick Codex GPU lock, check the 4090 UUID and current compute processes, and run with PTX JIT disabled. Use a distinct reviewed authorization manifest for each stage. Paths inside it are relative to the manifest's directory. The launch only reads `.bin` inputs and reads planted expectations after finite execution.

```sh
bash "$work/aot/run-wick.sh" finite "$work/finite" \
  "$work/finite-admission.sha256" "$finite_admission_sha" "$work/finite-output"
bash "$work/aot/run-wick.sh" exposed "$work/exposed" \
  "$work/exposed-admission.sha256" "$exposed_admission_sha" "$work/exposed-output"
```

The finite exercise has 16 planted rows, including exact recovery, different local role scores on identical mentions, earliest witness selection, a score tie, absent support, prefix and suffix negatives, punctuation, multi-space and repeated-blank negatives, and framing tests. Expected words are planted in the fixture source. The runner requires exact agreement with independent primary and cold CUDA processes, then verifies that changing a known-positive endpoint fails the comparator. The exposed exercise repeats the two device modes in independent processes and records full raw outputs. Its agreement against the ordinary client and D32 is checked by the separate panel join.
