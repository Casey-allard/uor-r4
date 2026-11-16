# REPORT: m2-student-corpus-ingest, claude-403 (donor shards into the native trainer's token stream)

Lab: claude. Order: [ORDER.md](ORDER.md). Summary of the host run: [summary-pod3.json](summary-pod3.json).

## What did not work, first

- **Our tokenizer needs 1.85 times as many tokens as the donor's for the same text** [claude].
  Pod 3's own counts give 60,846,703 kept DeepSeek tokens for these 31,910 records; the native
  4,096-id tokenizer spends 112,243,091 tokens on the same reasoning and answer text (about 2
  bytes per token against the donor's about 3.8). A "1B-token" corpus in D87's unit (donor
  tokens) is therefore about 1.85B response tokens in ours, plus about 0.25B prompt tokens and
  framing: about 2.1B tokens in the trainer's stream. Every training-time and storage estimate
  must use our count, not the donor's.
- **About one token in ten is a lone high byte** [claude]. Fallback tokens (an emitted id that
  decodes to one byte ≥ 0x80) are 8.0% of reasoning tokens, 12.7% of answer tokens and 5.8%
  of prompt tokens. The commonest runs are math and typography: the last byte of an em dash
  (`94`), `²`, `é`, `≤`, `√`, `≥`, `α`, `π`, `→`, `θ`, and Arabic text. The 4,096 vocabulary
  was built on English driver text; it carries these symbols byte by byte.
- **Not run on the D41 pilot corpus** [claude]. The instrument reads its format (tested on a
  fixture), but the sealed pilot corpus is not on this laptop; the run here covers pod 3's
  D87 shards only.
- **Pod 3's shards are finished, not sealed** [claude]. The production writer had not yet
  written a `receipt.json`; the instrument pinned each input file's sha256 at read time
  instead (in the summary). Shard 17 was left out because it may still have been syncing.
- **The first two builder agents produced no code** [claude]: each started near its context
  mark and handed back after exploring. The lab wrote the instrument in the main session from
  their findings.

## What changed

- `docs/labs/m2-student-corpus-ingest/instrument/`: the standalone package
  `m2-student-corpus-ingest` and its binary `student-ingest --tokenizer <tokenizer.json> --out
  <dir> <shard>...` [claude]. It reads `.jsonl` or `.jsonl.zst` shards, detects the D41 or D87
  record format by its exact key set (any other key set, or a malformed turn, refuses the run
  with the record's id), frames each record (the framing is in ORDER.md), and writes:
  - `tokens.u16`: the 64-byte UORT header of `crates/uor-r4-core/src/native_geometric/mmap_corpus.rs`
    (magic, version 1, token count, vocabulary 4,096), then little-endian u16 ids; the first
    token is BOS and every id is below 4,096, as `load_unpinned_uort` requires;
  - `response_mask.u8` and `segment.u8`, one byte per token, streamed (memory stays flat:
    38 MB resident for 128M tokens);
  - `summary.json`: the tokenizer's sha256, each input's sha256 and record count, records per
    format and kind, per segment the text bytes, tokens, dropped bytes, fallback tokens and
    literal special-token surfaces, the marker ids, and the top 20 runs of dropped-or-fallback
    bytes as hex. Ratios are exact integer pairs.
- Coverage is counted by aligning every encoded piece back onto its text bytes, not with
  `encode_lossy` (which always reports 0, `crates/uor-r4-tokenizer/src/lib.rs:303`). A byte the
  vocabulary cannot emit is dropped silently by the tokenizer (`lib.rs:597-611`); the
  instrument counts it and cross-checks the count against `text_len − Σ token bytes`.
- ORDER.md: the framing as built, with the separator and the closing EOS masked in.

## Measurements

Host run on this laptop, release build, at the head's instrument source [claude]:

```
student-ingest --tokenizer <claude-370>/in/driver-data/tokenizer.json \
  --out local/claude-403/pod3 <pod 3 out>/shard-00000.jsonl ... shard-00016.jsonl
```

- Tokenizer sha256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`, 4,096
  ids; the vocabulary can emit all 256 bytes, so **no byte is dropped** (0 of 260,444,097 text
  bytes).
- 31,910 records (29,784 single, 2,126 two-turn); 128,039,957 tokens; 26.7 s wall, 38 MB peak
  memory. `tokens.u16` is exactly 64 + 2 × 128,039,957 bytes.

| segment | text bytes | tokens | bytes/token | fallback tokens | fallback share |
|---|---:|---:|---:|---:|---:|
| prompt | 31,637,552 | 15,324,614 | 2.06 | 890,686 | 5.8% |
| reasoning | 171,146,254 | 82,620,552 | 2.07 | 6,634,033 | 8.0% |
| answer | 57,660,291 | 29,622,539 | 1.95 | 3,769,023 | 12.7% |
| frame | 646,684 | 472,252 | — | 0 | 0 |

- No literal `<|bos|>`/`<|eos|>`/`<|unk|>` surface appeared in any text (0 in every segment).
- Marker ids: user `[201, 55, 2728, 28]` (`\n` + the `User:` ids that `PAIR_USER_MARKER` in
  `crates/uor-r4-float-train/src/in_space_long.rs:858` holds as `[55, 2728, 28]`), assistant
  `[201, 35, 560, 652, 714, 28]` (equal to `PAIR_ASSISTANT_HEADER`), separator `[201, 201]`.
- Projection, from these ratios only: a 1B-donor-token corpus is about 2.1B tokens in our
  stream, and its three output files take 4 bytes per token, about 8.4 GB.

## Checks run at the head

- G1: `cargo build --release --manifest-path docs/labs/m2-student-corpus-ingest/instrument/Cargo.toml`
  (`CARGO_INCREMENTAL=0`, its own `CARGO_TARGET_DIR`): `Finished`.
- G2: `cargo test` on the instrument: 8 passed, 0 failed (D41 framing and masks, D87 two-turn
  order and separator mask, dropped and fallback counts with a byte removed from a test
  vocabulary, a literal special surface refused, refused key sets and turn counts, a `kind`
  that disagrees with its turn count refused, raw against zstd shards with their file hashes,
  a tokenizer with the wrong BOS refused). With the special-surface refusal and the kind check
  each switched off, their two tests fail (6 passed, 2 failed); restored, all 8 pass.
- Fresh-context adversarial review of e85f89794 [claude]: APPROVE, no blocker; eight minor
  findings and nits. Taken at this head: a record with a literal `<|bos|>`/`<|eos|>`/`<|unk|>`
  in its text is now refused (it would encode as a false record boundary inside a target; the
  pod 3 run had none); a `kind` must match its turn count; a stale `summary.json` is removed
  before a run; the dropped-byte cross-check uses a per-piece counter. The rerun on pod 3's 17
  shards gives a byte-identical summary.
- G3: the ORDER commit is the first on the branch.
- G4: `git diff --check origin/main...HEAD` is clean.
- G5: the summary has 31,910 records; the token file size matches its header.
- `uor-harness loop pr-check --local`: PASS. Advisories only: the new instrument and task paths
  are outside item `one-model-end-to-end`'s listed scope (a new package, D76 scope advisory),
  and the standing AGENTS.md-size and FILEMAP rows.

## Skills used

- `scope-guard`: one new instrument package and this task folder; no shared crate changed.

## Still open

1. **The framing is the lab's call** (ORDER.md): no new tokens, no reasoning marker, the
   separator and EOS masked in. A reviewer or the training unit may change it.
2. **Whether to widen the vocabulary** before training: the 1.85× token count and the 8–13%
   fallback share cost sequence length and steps. That is a decision for the training unit,
   with its own measurement; this unit only measures.
3. **The D41 pilot corpus run**, once its sealed copy is on this laptop or wick.
4. **Where the trainer's stream lives:** about 8.4 GB at the full corpus. The laptop has about
   22 GiB free; the stream can be written on wick's pool (`/mnt/plexpool/uor-r4-corpus`)
   instead, beside the shards.
5. Left from the review: a tokenizer whose merged token holds a byte with no one-byte token
   would make the run refuse valid text (it aborts loudly, never miscounts); the native
   tokenizer emits all 256 bytes, so it cannot happen with it. Also left: `--out` and input
   paths are compared as text, not canonicalized; the frame segment's token count includes
   BOS and EOS, so bytes per token is quoted per content segment only.
6. The next readiness units: a minimal whole-model native training step on CUDA over this
   stream, then a pre-registered timed run on wick that measures seconds per step.

## Plain summary

The donor corpus can now be turned into the files our own trainer reads. On 32,000 of pod 3's
DeepSeek records, nothing is lost: every byte survives tokenization. But our small 4,096-word
vocabulary needs almost twice as many tokens as DeepSeek's for the same text, and about one
token in ten is a single byte of a math symbol or a non-English letter. So the 1B-token corpus
becomes about 2.1B tokens for our model, which is what training time must be planned on.
