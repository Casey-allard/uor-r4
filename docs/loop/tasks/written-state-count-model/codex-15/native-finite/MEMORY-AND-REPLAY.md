# [codex] Added memory and frozen #4101 replay

The first raw-client bridge serialized the predecessor's ordinary next-token ID into the modern cursor word layout. That comparison failed before parity was accepted. The corrected bridge uses the exact source ID, or the inactive sentinel, in the cursor's exact-ID word. The original output and failed comparison remain in a separate Wick directory.

## Requested allocator bytes on Wick

The standalone probe wraps Rust's system allocator and samples live requested bytes after parse, then peak and retained allocations separately around the typed role predictor and full paired read. It loads the same bare trunk and positive finite banks from the first receipt. The inputs are the existing hand-configured single-prompt banks. The already-built client, its source, and the earlier receipt roots were left intact. The memory runs repeated in separate output directories and reproduced both seed receipts byte-for-byte.

| Seed | Bare trunk live heap | Ordered model live heap, each arm | Added live heap, each arm | Ordered parse peak heap, each arm |
|---|---:|---:|---:|---:|
| 1 | 608,899,725 | 608,939,245 | 39,520 | 617,717,429 |
| 2 | 593,775,837 | 593,815,357 | 39,520 | 602,007,137 |

On both seeds, the inline `RoleBank` occupies 386 bytes in the common 1,824-byte model type. Both modes used zero peak and retained heap bytes inside the exact shared role predictor. The full read, including the unchanged assertion path, peaked at 26 requested heap bytes and retained zero on all four positive cases. Caller-owned file buffers, allocator metadata, kernel page residence, and stack frames are outside these allocator counts. The exact uninstrumented client binary independently had the shared prediction and extractor frame sizes reported in [the first finite receipt](WICK-RECEIPT.md). This bounds the observed arm-local memory independently of process RSS.

For both modes and seeds, full eligibility admitted the positive multipart request at original `[28,30)` and chose source 7. Clipping to the complete span `[28,30)` kept it eligible and masked all 24 surrounding positions to zero. Shifting the interval to `[0,28)` excluded that candidate entirely. The identity and positional modes may then rank different surviving candidates. The test requires the same positional admission rule and disappearance of the withheld candidate.

## Frozen role, source, and writer replay

Only two already exposed #4101 tapes were read. The probe verified the frozen panel, role and writer manifests before replay. For each seed and both tapes, it recomputed original, identity, true and adverse interval decisions on the exact original IDs. It required equality to the recorded eight role words, chosen source, every ordinary next ID and top eight, all original cursor words, copy flag, release reason, emitted ID and terminal EOS. The two intervals use the historical frozen role scorer's full-prefix feature reads.

- The exposed supported row `held-0-true-0-0-unique-ignoring-after-0` retained the original noncopy fallback in the full interval and historical shifted interval `[3,14)`. The historical true interval `[14,25)` recovered request `[16,18)`, fact offset 4, source 7, and exact continuation `[1481,1]` on both seeds. Its original tape SHA-256 is `cc3f799dcd53beaafd0d56b3dc9bf857b4f8050c5f072dfd2fce714532b5f9bc`.
- The exposed nuisance row `nuisance-held-0-false-0-0-unique-ordinary-after-0-trailing` reproduced the full interval's unsupported source 11 and `[1730,1]`. The true interval `[13,31)` and historical adverse interval `[0,13)` declined the copy. Ordinary fallback emitted `[1155,302,311,14,340,1919,1203,383,16,1]` on both seeds. The tape SHA-256 is `39824ca38ab9ac4ea837dc98a5ec2ae0f16243dae60c25f341f79c3f01e6016c`.

The unchanged public release client also replayed unmarked original text for both tapes and seeds. With reads enabled, it matched the historical original arm's exact prompt, role, source, emitted IDs and eight cursor words per transition. Explicit no-read suppressed role selection and matched the historical adverse arm's no-source emitted and cursor trajectory through EOS on the same tape. Entries marked no-copy retain the observed fallback without claiming relevance. [Previously published #4101 examples](../../codex-13/writer/EXAMPLES.json)

## Reproduction and seals

Build this directory with `cargo build --release --locked --bin ordered-role-native-memory`. On Wick run `bash run-memory-wick.sh PROBE UOR_CHAT STUDY_ROOT POSITIVE_ROOT NEW_OUTPUT_DIR` from the custody configuration used in the first receipt. `POSITIVE_ROOT` contains its finite positive artifacts and seed pack records. A full replay also requires the verified historical `20261118-query-region-pick` tree under the existing Wick study root.

- Allocator and historical replay probe source SHA-256 `9819ff1aced0fba2f690380ed11e3213d99d30f9ec17d5744981ab8adec9fb33`, executable SHA-256 `2d511bd24b8f7ce7f2f97194507ba121831a3f7056dba2a0d8771653511da97d`.
- Public client executable SHA-256 `b8d31a0b42f39525a2c7587b5a58ee00d3e8b07eb32c5ac4434bb29b7b07a937`.
- Readback at Wick `native-finite/memory-check/run-01` and `run-02` under `20261118-ordered-role-transfer`. Manifest SHA-256 `7ed0164490cbd2601847e6848e1ddaa523c0326b7d355105f42cd5b1f0ba1141` and `0bc32dc6a1f75e9b32bb4e0b8b35fe21e2a89af0c4653084ef4b21b6197f0dea`, respectively.
