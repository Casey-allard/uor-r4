# reply-census (claude-320)

Counts the scored replies of one SFT split, optionally against a second split. Counting only,
integer arithmetic only; every share is a count, its denominator and parts per million rounded
down. Any inconsistency is a refusal: a nonzero exit with the file and the reason on stderr.

Build and test from this folder:

```sh
CARGO_TARGET_DIR=<target> ~/.cargo/bin/cargo test --offline --locked
CARGO_TARGET_DIR=<target> ~/.cargo/bin/cargo build --release --offline --locked
```

`Cargo.lock` is scale-probe's lock with the root package renamed, so `--locked` works offline.

## Usage

```sh
reply-census split=DIR name=NAME [against=DIR2 against_name=NAME2] [top=10]
```

An unknown or repeated key is a refusal; `against=` and `against_name=` go together. One JSON
object goes to stdout.

## Inputs (each split directory)

- `manifest.json`: `eos_id` (u16) is required; `bos_id` (u16) is optional. If it declares
  `tokens_sha256` or `mask_sha256`, the file's sha256 must match.
- `tokens.u16`: the repository corpus format (64-byte header, u16 ids), read with
  `MmapCorpusReader`.
- `response_mask.u8`: raw bytes, no header, one per token; its length must equal the token count
  and every byte must be 0 or 1.

A **reply** is a maximal run of consecutive positions with mask 1. It **ends on EOS** when its last
token is `eos_id`.

## Output fields

| Field | Meaning |
|---|---|
| `schema` | `uor-r4-reply-census/v1` |
| `name`, `split` | the `name=` label and the `split=` directory |
| `sha256` | sha256 of `manifest.json`, `tokens.u16`, `response_mask.u8` |
| `eos_id`, `bos_id` | from the manifest (`bos_id` null when absent) |
| `documents` | count of `bos_id` tokens; null without `bos_id` |
| `tokens` | token count |
| `scored_targets` | positions with mask 1 |
| `eos_targets` | positions with mask 1 and token `eos_id` |
| `replies` | number of replies |
| `replies_ending_eos` | replies whose last token is `eos_id` |
| `eos_not_at_reply_end` | EOS targets that are not the last token of their reply |
| `distinct_replies` | distinct replies by exact token sequence |
| `reply_length` | `median_lower` (element `(n-1)/2` of the sorted lengths), `min`, `max`; null with no replies |
| `eos_in_repeated` | EOS-ending replies whose exact sequence occurs at least twice, of `replies_ending_eos` |
| `eos_in_top1` | occurrences of the most common EOS-ending reply, of `replies_ending_eos` |
| `reply_8grams` | `windows`: 8-token windows lying inside one reply; `distinct`: distinct windows (packed exactly, 16 bits per id, into a u128) |
| `top` | up to `top=` replies sorted by count descending, then sequence ascending: `count`, `length`, `first_ids` (first 16 ids) |
| `against` | null, or the comparison below |

Shares (`eos_in_repeated`, `eos_in_top1` and the `against` shares) are
`{"count", "of", "ppm"}`, with `ppm` null when `of` is 0.

`against` (the second split must have the same `eos_id`):

| Field | Meaning |
|---|---|
| `name`, `split`, `sha256` | the second split's label, directory and file hashes |
| `replies`, `distinct_replies`, `distinct_8grams` | the second split's own counts |
| `replies_in_against` | replies of `split=` whose exact sequence is a reply of `against=`, of `replies` |
| `distinct_replies_in_against` | distinct such sequences, of `distinct_replies` |
| `distinct_8grams_in_against` | distinct reply 8-grams of `split=` that are reply 8-grams of `against=`, of `reply_8grams.distinct` |

## Tests

`tests/census.rs` writes fixture splits under `CARGO_TARGET_TMPDIR` and asserts every count
exactly: split A (seven replies over six documents, including a one-token mask-0 gap between two
replies and a mask-0 EOS), a split B without `bos_id` for `against=`, a 21-token reply for the
16-id cut, and refusals for a short mask, a manifest without `eos_id`, a mask byte of 2, a wrong
declared `mask_sha256`, and bad arguments. Two mutations were checked to fail them: merging reply
runs across a single mask-0 gap, and counting EOS without the mask.
