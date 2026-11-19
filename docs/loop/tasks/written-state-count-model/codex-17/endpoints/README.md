# #4106 independent requested-name seals

The source is the ordinary unmarked prompt and two separately provided person reads for each group. The held group uses the v5 independent readers. For base FIT, v4 and v5's reader-facing raw exports were verified byte-for-byte, so the two v4 person reads transfer unchanged. The supplement uses its two independent readers. Only the index and requested person were projected from those judgments. Gold item, gold source and outcome data were never supplied to this generator or its validator.

Fit ordinals 0–41 are unchanged v5 base FIT and 42–93 are the 52 supplemental supported TRAIN prompts in sealed order. Held preserves all 42 v5 rows without reordering. Each split has two unspecified requests at indices 20 and 41, encoded as null. Explicit names remain bound even when no answer is supported, including absent people and conflicting entries.

The author resolves a single whole-word occurrence within the raw question. Each fixture splits its written fact sentences and question at the last period followed by a space. In the registered two-sentence conflicting-record question, the preceding sentence belongs to the question as well, and the split moves one sentence back after verifying its visible conflict preface. This excludes earlier mentions of the same name within facts. The ordinal audit records source batch, original index and exact half-open question byte bounds. The independent person judgments must agree exactly before the author computes any name span.

The raw schema is `uor-r4.sealed-requested-names/1`. It binds the full raw JSON digest and includes `index`, `text_sha256`, and either `requested:{name,bytes:[start,end]}` or null. The released `prepare` subcommand re-encoded the raw literal protocol-2 prefix with each pinned tokenizer and validated each requested byte span. Its resolved endpoint rows contain only index, original prefix SHA-256, and a half-open requested-token span or null. Both seed token span arrays matched exactly. The held original blind files match byte-for-byte and the combined fit blind files match the base FIT plus supplement original records in order.

## Reproduction

Use the sealed tokenizer artifacts and the reviewed rank-capture tool's `prepare` entry point. The command reads no D11 model and runs no selector.

```sh
"$root/seal" "$root/inputs" "$root/replayed-raw"
digest() { shasum -a 256 "$1" | awk '{print $1}'; }
for split in fit held; do
  for seed in 1 2; do
    tok="$assets/tokenizer-seed$seed.json"
    raw="$root/replayed-raw/$split-raw.json"
    names="$root/replayed-raw/$split-names.json"
    "$rank_capture" prepare "$tok" "$(digest "$tok")" "$raw" "$(digest "$raw")" \
      "$names" "$(digest "$names")" "$root/replayed-$split-seed$seed"
  done
done
```

When rebuilding `seal.rs` independently, link serde_json and sha2 from the previously checked Rust dependency set. The executable has no model dependencies. The original compiler version, upstream input hashes, actual prepare executable hash and the original blind artifact hashes are in the seal.
