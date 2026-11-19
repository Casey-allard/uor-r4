# STEP 0: can the geometric Pick arm (A1) run on D87? (claude-439)

Lab: claude. Checked 19 Nov on the laptop, read only. No model ran and no text was decoded.

**Verdict: yes.** D87's token ids are the D11 trunk's token ids. A saved trunk carries the token
table that the Pick read needs. Two lines of the ORDER are corrected below, before the sheet.

## 1. The tokenizer

- D87's streams were built with tokenizer sha256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`. The source is claude-420's
  `run1/train.json`, `dev-tune.json` and `dev-test.json` (`tokenizer_sha256`); TRAIN is `c6cba29e…`.
- The D11 trunk loader expects `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377` (`discrete_d11.rs`, the count tests; `count-replay.rs`).
- The files differ, so every part that decides an id was compared with `jq -cS`. The copies read
  were `local/claude/claude-370/in/driver-data/tokenizer.json` (`d36d3e87…`) and
  `local/codex/public-m1-predictor/tokenizer.json` (`a9fb13f7…`); each copy's sha256 was checked
  first.

| Part | Result |
|---|---|
| `model.vocab` | identical, 4,096 entries |
| `model.merges` | identical, 3,837 pairs, both stored as arrays |
| other `model` settings (BPE, `unk_token`, no byte fallback, no dropout) | identical |
| `normalizer`, `pre_tokenizer`, `post_processor`, `added_tokens`, `truncation`, `padding` | identical |
| `version` | differs: missing in `a9fb13f7…`, `"1.0"` in `d36d3e87…` |
| `decoder` | differs: missing in `a9fb13f7…`, ByteLevel in `d36d3e87…` |

The decoder maps ids back to text and the version is a label, so neither changes an id. Codex's
`context-score` already refuses to run unless these two files share vocabulary, merges,
pre-tokenizer and added tokens. This check also covers the normalizer and the post-processor.

## 2. The snapshot

- No saved D11 artifact is in Pick mode. The trunk that the count work binds is
  `local/codex/public-m1-predictor/seed1.d11`, sha256 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` (`BASE` in
  `count-replay.rs` and `count-score.rs`). Its schema is `uor-r4.lut-discrete-trunk/1` and its
  `read_mode` is `relation`. It has 4,096 tokens, 4 blocks, 5 stages and a context of 192.
- `pick_read` (`discrete_d11.rs`) takes the token blocks, the level-0 gates and a transport.
  - With `read_position` 3 the transport is `Identity`, which needs no further section.
  - `PrefixHolonomy` would need `discrete.position_frames`, which the trunk lacks.
  - So A1 reads `seed1.d11`'s token blocks with `Identity` transport. The 16-bit code per block
    comes from `root_code`, giving 4 codes per token and a pair distance from 0 to 128.
- The sheet names this snapshot again with its sha256 and records why it is the one used.

## 3. Corrections to the ORDER

The indexing below is the ORDER's: the target is at `t`, the query pair is `(t − 2, t − 1)`, and a
candidate pair is `(p − 1, p)`. `pick_read` calls the last prefix position `t`, which is the ORDER's
`t − 1`.

1. **"Its gate decides whether it fires" is wrong.**
   - In `pick_read` the pick is the latest candidate with the least XOR + popcount distance, and it
     depends on the codes alone.
   - The level-0 gates only weigh the successor's blocks in the read vector, which this expert does
     not use.
   - `pick_read` returns a pick whenever the prefix holds at least three tokens. So "fires" means:
     the target is at `t ≥ 3` of one D87 document, inside the read's window. "Did not fire" means
     `t < 3`.
   - The three bucket classes stay: did not fire, fired at distance 0, fired at distance > 0.
2. **G2's second negative changes with it.** "The Pick column ignoring the gate" cannot fail
   anything, because the gate never enters the column.
   - In its place: a Pick that compares only `p` with `t − 1` and drops the `(p − 1, t − 2)` half.
     It must fail a fixture where `t − 1`'s token occurs twice earlier and only the pair picks the
     right copy.
   - The earliest-position tie negative stays as written.

Nothing else in the ORDER changes. The C2 control, exact token pairs, needs no tokenizer
or snapshot, and runs either way.
