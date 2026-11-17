# ORDER — written-state-count-model (claude-420)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-420, P0.5 (R\*), part 3 of 5: the sealed per-split token streams that R\* and every
later model are counted on and scored against. The R\* plan is re-cut from four parts to five:
- claude-418: the counts (merged);
- claude-419: the host scorer (merged);
- claude-420 (this unit): the streams;
- claude-421: the counts filtered to scored contexts, a GPU scorer equal to claude-419's host
  scorer, and the bits instrument;
- claude-422: the sheet, FIT, N tuned on DEV-tune, and the sealed run.

## What it builds

One new binary, `d87-streams`, in `docs/labs/d87-split-audit/instrument/`. It reuses claude-411's
`plan`, `tokenize`, gram sets and Bloom pass, and claude-403's UORT layout. So a record's stream is
the one the audit counted.

It reads D87 split v1 (claude-411's split directory, no drop list; claude-417 kept split v1). Like
`d87-audit`, it refuses shards that do not reproduce `split-summary.json`. The D87 shards are read
only.

For each split S of TRAIN, DEV-tune and DEV-test it writes:
- `S.tokens.u16`: a UORT stream of every kept record as BOS ... EOS, never across records;
- `S.mask.u8`: the response mask, one byte per token;
- `S.segment.u8`: the segment tags, one byte per token;
- `S.json`: records, tokens, scored targets (every non-BOS position), target bytes, response
  targets and bytes, and dropped bytes, plus the sha256 of each file.

Records run in the plan's order: shard paths sorted, then line order. That is the order claude-411's
split and audit use. It does not depend on the order the shards are given or on the thread count.
TRAIN is written as per-shard parts in parallel, then joined in shard order. It is never held whole
in memory.

For DEV-tune and DEV-test it also writes `S.novel.u8`, one byte per position, under claude-411's
rule:
- 0: the 13 tokens before the target (one record) occur as a 13-gram in TRAIN;
- 1: they do not (novel);
- 2: the target has fewer than 13 tokens before it in its record;
- 3: a BOS position, never scored.

`S.json` counts targets and bytes in each class.

## Byte rule (the BPB denominator)

The bytes of a target are the decoded UTF-8 length of its id, from one table built from the
tokenizer. BOS, EOS and UNK have length 0. The table is written as `id-bytes.u8`, 4,096 bytes, with
its sha256.

The rule holds for a record only if its targets' lengths sum to the record's text bytes minus its
dropped bytes, both as the ingest counts them. Dropped bytes are bytes the vocabulary cannot emit, so
no token predicts them. The unit checks this on every record of every split and refuses at the first
record that fails. Dropped bytes are reported apart and are never in the denominator.

## Gates

- G1: the instrument's tests pass, the old ones and these new ones:
  - **Reference.** On a multi-shard fixture, each split's three files equal, byte for byte, the
    files that claude-403's `Ingest` writer makes from the same records in plan order.
  - **Order.** The files do not change when the shards are given in reverse order or with 1 or 3
    threads.
  - **Novel fixture.** A planted 13-gram makes exactly one DEV-test target 0; the counts equal the
    audit's `novel_targets` on the same fixture, and the hand-checked classes 2 and 3 are right.
  - **Bytes.** Byte sums equal the fixture's text bytes, and a record whose text holds an
    unemittable byte reports it as dropped.
  - **Refusal.** A planted byte-table error is refused.
- G2: set-aside negatives, each failing, then restored:
  - TRAIN parts joined in reverse shard order: the reference test fails;
  - the novel rule taken over 12 tokens: the novel fixture fails;
  - special ids given their surface length: the byte rule refuses the fixture.
- G3, on wick:
  - two runs give `cmp`-identical files;
  - DEV-test's novel counts equal claude-411's audit (`/workspace/out/c411/audit2/prompt`):
    all-target novel 14,795,227 of 21,349,576, short context 63,852, response-mask novel
    14,190,847 of 18,751,676;
  - DEV-test has 5,321 records and TRAIN has 2,091,560,001 tokens, both as the audit counts them;
  - TRAIN's token count is below the counter's 2^31 stream bound;
  - `uor-r4-discrete`'s `read_uort_tokens` reads every stream.
- G4: `uor-harness loop pr-check --local` reads `verdict: PASS`.
- G5: no corpus text, token stream, mask or id list is committed. Only the `S.json` summaries
  (counts and digests) and logs are.

There is no model, no scoring and no milestone claim, so there is no pre-registration sheet (§4.5).
The D87 copies are read only. Nothing on `/mnt/plexpool` is written.
