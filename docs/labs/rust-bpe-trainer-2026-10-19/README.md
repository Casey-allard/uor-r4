# Deterministic Rust BPE trainer — 19 October 2026

The shared tokenizer crate now supplies `train-bpe`, a host-only data-preparation command for the prospective V=16,384 tokenizer. Owner direction to advance independent work and Kimi PM assignment [6260043935](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6260043935) authorize this tooling. It does not change the active A8 source, token files, table, tokenizer or vocabulary, and produces no model result. M1 count/headline remains unchanged; line: tokenizer data-preparation tooling, no model negative count.

## Command

```sh
cargo run --locked --release -p uor-r4-tokenizer --bin train-bpe -- \
  --input TRAIN_TEXT.txt --output NEW_TOKENIZER.json --vocab-size 16384
```

Repeat `--input` for separate UTF-8 documents. The output must be a fresh path. Only regular input files are accepted. Do not replace input paths concurrently while the command runs. `--pre-tokenizer FILE.json` accepts a ByteLevel object or exactly a Sequence of Digits then ByteLevel; it does not accept a full tokenizer template. The default is ByteLevel with no prefix space. The trainer shares the existing parser, segmentation and byte encoder with runtime encoding. Special tokens are atomic; no merge crosses a special, pre-token or input-file boundary.

IDs0/1/2 are `<|bos|>`, `<|eos|>`, `<|unk|>`, followed by all256 byte symbols. The requested vocabulary is bounded to259..=65536, so every ID fits u16. Merges select descending weighted adjacent-pair count, then ascending left/right token strings in Unicode code-point order. Overlapping occurrences count toward frequency; replacements run left-to-right without overlap. Unique pre-tokens carry integer frequencies; a pair-to-word index updates only affected words. Lazy stale heap entries are periodically compacted. This is an implementation choice, not a measured speedup claim.

The command emits HF-shaped tokenizer.json with dense IDs and rank-ordered merges. File serialization is deterministic and its exact raw-byte blake3 identity is printed on stderr. Runtime parsing uses the same raw-byte identity. Special IDs are not automatically inserted into encoded text. An exhausted corpus produces fewer tokens than requested and reports the actual size; it never fabricates padding merges. Consumers must check attained size before registering a future corpus.

## Bounds and evidence

Defaults:64MiB aggregate UTF-8 input,1,000,000 unique pre-tokens,16KiB per pre-token; all configurable CLI flags are listed by `--help`. The policy JSON limit is64KiB. Counts use checked integer arithmetic. An ingestion error invalidates the trainer; no output is created for invalid input. Output creation uses create-new and never overwrites an existing file or link. These are input/index-size bounds, not a measured fixed RSS guarantee. Large real corpora need their own declared resource budget and tokenizer/data identity before use.

Focused checks cover deterministic ties and corpus order; weighted/overlapping pairs; special and Digits/prefix boundaries; unseen Unicode/byte roundtrip; vocabulary exhaustion/u16 limits; failed input/poisoning; invalid/repeated policies; CLI UTF-8/overwrite/FIFO refusal; and comparison of incremental merge ranks plus runtime segmentation against an independent exhaustive recount across40 deterministic corpora. A bounded synthetic8,192-document fixture attains exactly16,384 tokens and roundtrips unseen text. This is tokenizer tooling validation, not a language-model evaluation or corpus-training release.

Architecture review and Rust/input-security review are independent. The FIFO opening regression was reproduced with a timeout test, then fixed by screening path type before open and retaining descriptor validation. Original task evidence stays external under `uor-r4-local/bpe-trainer-20261019`; no pod/paid compute or model execution is part of this delivery.

## Explicit ByteLevel policy admission

The post-delivery peer review of #2766 found that explicit `trim_offsets:false` was accepted and then emitted as `true`. The trainer now refuses that unsupported value at ingestion, just as it refuses `use_regex:false`; omitted or explicit `true` remains accepted. This applies both to standalone ByteLevel and the ByteLevel step in a Digits/ByteLevel Sequence. A regression test was observed failing before the fix and passing after it. Default tokenizer behavior is unchanged; no existing tokenizer file is rewritten or migrated.
