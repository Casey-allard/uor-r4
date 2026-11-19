# [codex] Finite artifact and writer receipts on Wick

One handwritten prompt exercised the protocol-2 tokenizer, both loaded seed trunks, role abstention, read-disabled fallback, and exact-copy release through EOS. The 32-row timing repeated that fixture to measure load and dispatch costs. It supplies no evidence of transfer or evaluation adequacy. The two shapes used Wick's RTX 4090, NVIDIA driver 595.99.02, and Ryzen 9 9950X CPU. The original CUDA writer acquired the existing lock and passed the free-device-memory check. The ordinary public client used its CPU integer serving path.

The published predecessor CUDA/Rust writer emitted `[3566, 421, 86, 1]` for the forced source at offset 7 on both seed trunks. The action writer replay compared 4 original-position cursor transitions at 1 row and 224 at 32 rows per seed, including 16 exact copies and 16 ordinary fallbacks in each 32-row batch. Each matched the identifier, ordinary next ID, post-step cursor words, copy flag, and release reason from the reloaded ordered artifact. The fallback rows matched the raw-text `uor-chat` emissions, which reached EOS in 10 IDs. Read-enabled zero-state and explicit no-read emitted the same IDs and cursor words. Enabling opt-in trace changed only the trace field.

A separate hand-configured bank put a prototype at the fixture request's feature for each arm and seed. Public `--say` selected fact position 4, request position 28 and source position 7, returned the literal cobalt IDs followed by EOS, and matched the independent artifact-library decision and assertion witness. This was a finite wiring check with a prescribed prototype, without optimizer steps or a corpus pass.

The shipped predecessor-format carriers produced byte-identical JSON from the old and new clients for enabled and disabled reads. The old client rejected each of the four new paired-copy/2 artifacts. The new parser rejected a planted frozen class-byte corruption. The new release binary was compiled from the worktree's reader/client sources. The old release binary was built from `0414090e7`.

## Measured costs

Public client values include a fresh process and file load on every invocation. The 32-row numbers are the sum of those 32 invocations. The one-row values exclude the extra trace and no-read subprocesses. The table keeps separate the writer action batch's 16 copied and 16 fallback rows. All durations are seconds.

| Seed | Arm | Artifact bytes | Pack and verify | In-memory file parse | Public 1 row | Public 32 rows | Public peak RSS KiB at 32 |
|---|---|---:|---:|---:|---:|---:|---:|
| 1 | Identity | 86,552,276 | 1.102 | 0.364 | 0.580 | 18.810 | 774,200 |
| 1 | Positional | 86,552,276 | 0.974 | 0.361 | 0.578 | 18.542 | 774,188 |
| 2 | Identity | 84,959,472 | 1.054 | 0.353 | 0.569 | 18.653 | 755,268 |
| 2 | Positional | 84,959,472 | 0.903 | 0.354 | 0.654 | 18.178 | 755,416 |

| Seed | Action writer 1 row | Action writer 32 rows | Action writer peak RSS KiB at 32 |
|---|---:|---:|---:|
| 1 | 0.83 | 0.86 | 1,032,364 |
| 2 | 0.80 | 0.87 | 1,015,220 |

The 400-byte bank has 192 learned bytes. Both arms on a given seed carry the same class bytes, encoded framing, header span, padding, and trunk. The single public operator uses fixed-size feature, position and ranking scratch without a per-call heap allocation. RSS is whole-process peak and includes the shared trunk, parser and allocator. These readings do not isolate role scratch or establish identical measured RSS. On the exact Linux client binary, disassembly shows the shared prediction frame reserves 648 local bytes and saves 48 register bytes. The shared extractor adds 600 local and 48 saved-register bytes while it runs. The signed-axis gather is a zero-frame leaf on its validated path. These are direct frame observations, excluding call return addresses and the internal work of the shared system `memcpy`. Neither mode changes the stack-frame reservations. The largest individual public invocation in the batch measurements took 0.775 s.

## Identities

- New release `uor-chat` on Wick, `native-finite/source/target/release/uor-chat`, SHA-256 `b8d31a0b42f39525a2c7587b5a58ee00d3e8b07eb32c5ac4434bb29b7b07a937`.
- Old release `uor-chat`, `native-finite/legacy-source/target/release/uor-chat`, SHA-256 `644f8025e5cb41794889bc4b6a22e2981c00fc5521c7e3019fc69c0bb0c7cd57`.
- Frozen `action-writer`, SHA-256 `004cc78a1d0b616c15986c2032b6c07f661536851c4d24a627d6eb6d4258e592`. Writer and completion cubins SHA-256 `b41d72ce289ff5daf29055687e2586929321fed6ddb800d10cddfc650e2eeb28` and `8a73fffb5edfe3f5b36b891b37a96ae5683aacde49d08ae69caab92a7cab9b85`.
- The timing runner source and executable are preserved in Wick `native-finite/probe-v1/`, with SHA-256 `88ef166de79e45e310802c41d8f730703953c48122fbd5f4da24104e8d93b7c4` and `e9e96c6bdf2c4a26cea3a72921297147028e0f60e4b9a4db2215b8ebc4b86da7`. The checked-in probe adds the prescribed-prototype positive check, source SHA-256 `9ee86a7da08fbc0be3379e9ca54c0c1ed95b4d0b8335b088ae7ff260199e4094`, executed with `bd0e1604836eae4c4eab0211523310dd93f8c17e26ff7cd1013ae45689a28ff3`.
- Raw outputs in Wick `~/uor-codex/20261118-ordered-role-transfer/native-finite/`. The `run-01.sha256` manifest digest is `630d88a89bd7115dab700fc02c3c3b53d30033b98e532389ca3966eb9a37d65c`. The `legacy-compare.sha256` manifest digest is `6f0ac2234b6b7d7a232e2f26a8e2664e44dccfed47baf7a9b1a0749757d8749d`. The `native-scratch.disassembly` digest is `07b296570794bcc58f80e33de06ca84870025e48a99294c764c9da535099c4a4`.

Build this directory's Rust probe on Wick with `cargo build --release --locked` and the matching client in the checked-in integer crate with `cargo build --release --locked -p uor-r4-integer --features serving-wasm --bin uor-chat`. Run `bash run-wick.sh PROBE UOR_CHAT STUDY_ROOT NEW_OUTPUT_DIR` with the two frozen paired seeds and their own tokenizers present under `STUDY_ROOT/assets/`.
