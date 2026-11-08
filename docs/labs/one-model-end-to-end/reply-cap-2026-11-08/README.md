# reply-cap (claude-321)

Writes a copy of one SFT split in which each EOS-ending reply sequence keeps its response mask for
its first `k` occurrences and is masked to 0 after that. The tokens are not touched. Integer
arithmetic only, no floating-point types. Any inconsistency is a refusal: a nonzero exit with the
reason on stderr.

Build and test from this folder:

```sh
CARGO_TARGET_DIR=<target> ~/.cargo/bin/cargo test --offline --locked
CARGO_TARGET_DIR=<target> ~/.cargo/bin/cargo build --release --offline --locked
```

`Cargo.lock` is reply-census's lock (itself scale-probe's) with the root package renamed, so
`--locked` works offline.

## Usage

```sh
reply-cap split=DIR out=DIR k=K
```

`K` is a positive integer. An unknown, missing or repeated key is a refusal, and so is an `out=`
that already exists. The tool creates `out=` with three files and prints the `reply_cap` object
(below) as one JSON line to stdout.

## Input (the `split=` directory)

- `manifest.json`: a JSON object. `eos_id` (u16) is required. If it declares `tokens_sha256` or
  `mask_sha256`, the file's sha256 must match. A manifest that already has `reply_cap` is
  refused, so a capped split is never capped again over its own provenance.
- `tokens.u16`: the repository corpus format (64-byte header, u16 ids), read with
  `MmapCorpusReader`.
- `response_mask.u8`: raw bytes, no header, one per token. Its length must equal the token count,
  and every byte must be 0 or 1.

## The reply rule

A **reply** is a maximal run of consecutive positions with mask 1. It **ends on EOS** when its last
token is `eos_id`. Replies are walked in file order. For each EOS-ending reply, the tool counts how
many times its exact token sequence (the whole run) has occurred so far. The j-th occurrence
(1-based) keeps its mask bytes if `j <= k`. Otherwise **every byte of the run** is set to 0.
Replies that do not end on EOS, and every mask-0 byte (an EOS with mask 0 included), are left
unchanged.

## Output (the `out=` directory)

- `tokens.u16`: copied byte for byte from the input, header included.
- `response_mask.u8`: the capped mask, the same length as the input.
- `manifest.json`: the input manifest object with every field kept, plus:

| Field | Value |
|---|---|
| `source`, `revision` | the input string with ` (reply-cap k=K)` appended (only when present and a string) |
| `tokens_sha256` | sha256 of the output `tokens.u16` (equal to the input's), lowercase hex |
| `mask_sha256` | sha256 of the output `response_mask.u8`, lowercase hex |
| `reply_cap` | the object below |

`reply_cap`:

| Field | Meaning |
|---|---|
| `k` | the cap |
| `input_tokens_sha256`, `input_mask_sha256` | sha256 of the input data files |
| `replies` | all replies (mask-1 runs) |
| `eos_replies` | replies ending on EOS |
| `eos_replies_kept` | EOS-ending replies with `j <= k` |
| `eos_replies_masked` | EOS-ending replies with `j > k`, masked whole |
| `distinct_eos_replies` | distinct EOS-ending sequences |
| `scored_before`, `scored_after` | mask-1 counts of the input and output masks |

## Loading the output in scale-probe

The output is a `scale-probe mix` root: its manifest carries `bos_id` 0, `source`, `revision`,
`licence` and `attribution` from the input, and the two shas of the files. Root specs are
`NAME:DIR:TOKSHA:MASKSHA`:

```sh
scale-probe mix out=<new dir> fraction=1/1 \
  root=<name>:<out dir>:<tokens_sha256>:<mask_sha256>
```

Gate run (claude-321): mw-balp train (`local/claude/claude-314/mwbalp-root`) at `k=2` gave
`replies` 212968, `eos_replies` 212968, `distinct_eos_replies` 61616, `eos_replies_masked`
148561, `scored_before` 2425925 and `scored_after` 929120. `scale-probe mix fraction=1/1`
accepted it as a root and reported scored=929120.

## Tests

`tests/cap.rs` writes a 47-token fixture split under `CARGO_TARGET_TMPDIR`. It has ten replies:
one EOS-ending sequence three times, one twice, one once, a run without EOS twice, and a run
followed by a mask-0 EOS twice. The tests assert the exact output mask bytes and every count at
`k` = 1, 2 and 3. They also check that the tokens file is byte-identical, that the manifest shas
equal the files' sha256, that every input field is kept, and that the non-EOS runs and mask-0
bytes are untouched. Refusals are pinned for an existing `out=`, a bad `k` (0, non-integer,
negative, empty), a short mask, a manifest without `eos_id`, a mask byte of 2, a wrong declared
`tokens_sha256`, an already capped split, and bad arguments.

Two mutations were checked to fail the tests. Comparing `j < k` fails the k = 1, 2 and 3 tests.
Masking only the run's last byte fails the k = 1 and 2 tests.

## window-count: the trainer's usable windows

```sh
window-count split=DIR context=N
```

It prints one JSON line: `documents`, `windows`, `usable_windows`, `context`, `min_scored` and the split's two shas. The split loads as for `reply-cap`, so its manifest needs `eos_id`; `bos_id`, if declared, must be 0, and the first token must be BOS.

The count restates the trainer's rule:
- `build_document_windows` in `crates/uor-r4-float-train/src/hamilton_sheet.rs` cuts each BOS-delimited document's targets into chunks of at most `context`;
- `train_windows` in `crates/uor-r4-float-train/src/in_space_long.rs` keeps a chunk with at least 4 scored targets.

A cap can empty a chunk of scored targets, so it changes this count.

**Checked against the trainer's own logs (context 192), 8 Nov:**
- claude-318's `d3mw3` mix reads 357,907, the trainer's "usable TRAIN windows" (claude-318's sheet);
- `mw-balp/train` reads 68,218 (claude-313's trainer log).

`tests/windows.rs` pins hand-worked counts on a three-document fixture. Two mutations fail it: `n > min_scored` for `n >= min_scored`, and a chunk one target longer than `context`.
