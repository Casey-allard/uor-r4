# scale-probe instrument

The laptop side of P5 scale-probe: counting only, integer arithmetic only. Every inconsistency
is a nonzero exit that names the file (or split and row) and the reason. Outputs are never
overwritten. A position is scored iff `p >= 1 && tokens[p] != 0 && mask[p] == 1`.

Build from the repository root:
`~/.cargo/bin/cargo build --offline --locked --manifest-path docs/labs/scale-probe/instrument/Cargo.toml`.

## `mix`

Builds one nested training mix. Per root, takes the shortest document-boundary prefix `[0, L)`
with `L * D >= total * N`, so each root's 1/4 is a prefix of its 1/2, which is a prefix of the
whole. Prefixes are concatenated in the given root order; `out/manifest.json`
(`uor-r4-scale-probe-mix/v1`) records each root's provenance, counts taken and the written shas.

```
scale-probe mix out=local/claude/mix-q1 fraction=1/4 \
  root=oasst2:local/claude/rtc/oasst2/train:<tokens_sha256>:<mask_sha256> \
  root=wiki-00000:local/claude/rtc/wiki-00000/train:<tokens_sha256>:<mask_sha256>
```

## `grade`

Scores one checkpoint's rows files (`{split, memory, positions, targets, nearest, writer}`)
against TRAIN count tables: the unigram top-1 and T2, the previous-token bigram top-1 (ties to
the lower id; an unseen key falls back to the unigram top-1). The key is `tokens[p-1]`, so a
document's first scored token is keyed on its BOS (token 0). Writes `out/grade.json`
(`uor-r4-scale-probe-grade/v1`) and `out/nearest-hits.bin` (one bit per row, LSB-first, splits
in argument order). A split's `row_identity_sha256` is SHA-256 over its `(position u64 LE,
target u32 LE)` pairs; the pooled one is SHA-256 over, per split in order, the name length
(u64 LE), the name bytes and the split's 32-byte identity.

It also writes `out/t2-hits.bin` and `out/unigram-hits.bin`: one bit per row for "the T2 top-1
equals the target" and "the unigram top-1 equals the target", in the same layout as
`nearest-hits.bin`. `grade.json` records their names and SHA-256s (`t2_hits_file`,
`t2_hits_sha256`, `unigram_hits_file`, `unigram_hits_sha256`).

```
scale-probe grade out=local/claude/g-w256-q1-s1 train=local/claude/mix-q1:<tok>:<mask> \
  panel=oasst2:local/claude/rtc/oasst2/heldout:<tok>:<mask> rows=oasst2:rows-oasst2.json
```

## `seen`

Marks which panel rows may quote TRAIN. It takes the same `out=`, `train=`, `panel=` and
`rows=` arguments as `grade`, and writes `out/seen.json` (`uor-r4-scale-probe-seen/v1`) and
`out/excluded.bin`.

- TRAIN 8-grams are the runs `tokens[i..i+8]` inside one document: no BOS (token 0) at
  offsets 1 to 7. Each id takes 12 bits of one key, the first id in the high bits.
- A row at position `p`, in the document whose BOS is at `s`, is `short` if `p - s < 8`. Its
  8-token context would reach into the previous document. Equivalently, a BOS lies in
  `tokens[p-7..p]` (clipped at 0).
- A row that is not short is `seen` if `tokens[p-8..p]` is a TRAIN 8-gram, else `unseen`.
- `excluded.bin` has one bit per row, set iff the row is short or seen. Rows are in pooled row
  order, LSB-first, zero-padded to whole bytes, as in `grade`.

`seen.json` records the row identities (the same encoding as `grade`), the number of distinct
TRAIN 8-grams, the counts of seen, short and unseen rows per split and pooled, and the SHA-256
of `excluded.bin`.

`rows=NAME:all` takes every scored position of panel `NAME`, in corpus order, with target
`tokens[p]`. Only `seen` accepts it; `grade` refuses it by name.

```
scale-probe seen out=local/claude/seen-q1 train=local/claude/mix-q1:<tok>:<mask> \
  panel=oasst2:local/claude/rtc/oasst2/heldout:<tok>:<mask> rows=oasst2:rows-oasst2.json
```

## `pair`

Compares cell A (seeds 1, 2, and 3 when `a3=` is given) with cell B, or with a count table,
row by row:
`band(s) = max(⌈2·√(b(s)+c(s))⌉, max_s d(s) − min_s d(s))`, the spread taken over every seed
given; a seed is `ahead` if `d(s) > band(s)`, `behind` if `d(s) < −band(s)`, else `inside`; the
verdict is the seeds' common position, or `mixed`. Inputs must share every row identity; a
cell's seeds share TRAIN. A third seed goes to both cells (`a3=` with `b3=`) or, against a
table, to cell A alone; pair refuses any other combination.

With `table=`, pair reads the grade's `t2-hits.bin` or `unigram-hits.bin`. It refuses an older
grade that lacks them. It also recounts each split's and the pooled `vs_t2` or `vs_unigram`
`(b, c)` from the bitsets and refuses a grade that disagrees.

`clean=SEENDIR` adds an `unseen` object: the same readings over only the rows whose excluded
bit is 0. The seen directory must match a1's grade: the same TRAIN (`train.tokens_sha256`), the
same pooled identity, the same split names, order and identities, and an `excluded.bin` of the
right length whose SHA-256 matches `seen.json`. Its `seen.json` SHA-256 is recorded as
`clean_seen_sha256`. When the two cells train on different mixes, put the larger mix as cell A
and build the seen directory against it: a row unseen in the larger mix is unseen in its
prefixes.

```
scale-probe pair out=local/claude/w512-vs-w256.json a1=g-w512-s1 a2=g-w512-s2 \
  b1=g-w256-s1 b2=g-w256-s2 clean=local/claude/seen-q1
scale-probe pair out=local/claude/w512-vs-t2.json a1=g-w512-s1 a2=g-w512-s2 table=t2
```
