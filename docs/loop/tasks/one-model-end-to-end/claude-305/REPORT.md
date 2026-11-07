# REPORT: one-model-end-to-end, claude-305

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](./ORDER.md). No GPU and no spend: everything ran on the laptop.

## What did not work, first

- **The build script's prefix check was wrong, and it stopped the first run.** It compared each old root with the head of its new one byte for byte, header included. A `tokens.u16` file starts with a 64-byte header whose bytes 8–15 hold the token count (64 + 2 × count is the file size), so the header always differs when a root grows. Every file read "NOT PREFIX", and the script stopped before the mix.
  - The masks failed too, although they have no header. macOS `cmp -s -n` reported them unequal even though their leading bytes hash equal. The check used both the wrong span and an unreliable tool.
  - Class: instrument. The data was fine.
  - The repaired check is `prefix-check.sh`, in the seal. It compares shasums, with `head -c` and `tail -c`, past the token header. It shows all 12 old files (3 roots × train and held-out × tokens and mask) equal to the leading data of their D4 files (`prefix-check.txt` in the seal).
  - `build-d4-part2.sh` then ran the remaining steps on the roots already built. It rebuilt nothing.
- **Conversation is a smaller share of D4.** OASST2 is unchanged at 3,773,419 tokens, so its share of TRAIN falls from 13.5% (D3) to 5.1% (D4). The oasst2 sub-panel is where a drop would show first.
- **T2 is not a stronger baseline on D4.** It gets 256,443 panel rows against D3's 256,660 (−217). The bar's T2 is therefore no harder to beat; the seen share is what grows.
- **Not built:** the overlap census (`train-in-space --census 1`). The bar does not need it, and at 0.67–0.75 GB per million TRAIN tokens it would need about 50 GB, over the laptop's 16 GB. The seen set below is the overlap reading the bar uses.

## What D4 is

Three wiki roots rebuilt by `prepare-real-text` at claude-293's settings, with only `max_docs` tripled (900 → 2,700, 8,400 → 25,200, 4,350 → 13,050), plus OASST2's stored TRAIN root. The mix is `scale-probe mix fraction=1/1`, in D3's root order:

| Root | TRAIN tokens (D3) | TRAIN tokens (D4) | TRAIN docs (D4) | Held-out docs: panel (old) + new, unused |
|---|---|---|---|---|
| oasst2 | 3,773,419 | 3,773,419 | 5,071 | as stored + none |
| wiki-00000 | 8,011,538 | 22,752,102 | 2,573 | 36 + 91 |
| wiki-00020 | 8,255,877 | 23,962,093 | 23,975 | 430 + 795 |
| wiki-00040 | 7,927,279 | 23,551,634 | 12,386 | 239 + 425 |
| **Total** | **27,968,113** | **74,039,248** (2.65×) | | |

- `tokens.u16` `3a5b89f9fd99326eb18f57027eb38f2d8ef2d200e907d0c9a7e20553c78b0169`
- `response_mask.u8` `9fe9dedeccd599c0b9abd59c1864a414f999bb43f53529ba8e5013d35da10807`

**The panel is claude-293's four `heldout/` folders, unchanged:** 1,460,697 rows, with the row-identity sha `5f88a5b3…`, the same as D3's seen set. The larger build's new held-out documents are not used. They are neither in TRAIN nor in the panel.

## What changed

- The ORDER, with the cancel probe's result, and this REPORT.
- One row in `docs/research/notebook/float-training.md`.
- Data in the store (below). No source change.

## Measurements

**Cancel probe** (in the ORDER): `wiki-00020`, rebuilt at claude-293's own `max_docs`, is byte-identical on tokens, masks and `excluded.jsonl`. The manifests differ only in `path` strings.

**The panel against D4** (`seen` and `grade`; scale-probe built at `5388788e4`, whose instrument source last changed at `e052abf41`; with claude-295's `w256-d1-s1` no-memory rows for the row identities):

| Panel | Rows | Seen, D3 | Seen, D4 | T2 hits, D4 | Unigram hits, D4 |
|---|---|---|---|---|---|
| oasst2 | 184,542 | 26,155 | 29,200 | 29,531 | 4,601 |
| wiki-00000 | 396,045 | 53,638 | 67,414 | 71,051 | 6,851 |
| wiki-00020 | 443,252 | 85,342 | 104,764 | 78,401 | 14,538 |
| wiki-00040 | 436,858 | 62,177 | 81,207 | 77,460 | 11,431 |
| **Pooled** | **1,460,697** | **227,312** (15.6%) | **282,585** (19.3%) | **256,443** | **37,421** |

- **Distinct TRAIN 8-grams:** 59,915,754, against D3's 23,521,412.
- **Unseen rows:** 1,173,177; short rows: 4,935.
- **`excluded.bin`:** `511b0289…`.
- **Resources:** `seen` peaked at 1.47 GB in 3.9 s, `grade` at 0.37 GB in 1.23 s, `mix` at 0.56 GB in 1.80 s, and each root build at 0.10–0.32 GB in under 8 s.

**Control:** D3's seen set, rebuilt with this binary and these rows, is byte-identical to claude-295's `excluded.bin` (`94f6335f…`). The instrument has not moved.

**Seal:** `results/claude/one-model-end-to-end-d4-2026-11-07/`, 39 files.
- `SHA256SUMS` `51ae9b9dcb685646fa29b01030680f3907b51e91dff906c11332284071634a22`. This replaces the first seal's `d8b15e24…`, after `prefix-check.sh` was added at review.
- Read back from the store: all 39 sums OK.
- It holds the mix, `seen/`, `tables/` (grade.json and its hit bitsets), each new root's manifests and `excluded.jsonl`, and the scripts and logs.
- The new roots' TRAIN tokens are the mix's own bytes, so they are not stored twice. Every root rebuilds from the stored raw shards in under 10 s.

## Checks run at the head

- `loop report --check` on this REPORT.
- `prefix-check.txt`: 12 of 12 PREFIX.
- The D3 control: equal.
- `shasum -c` on the store read-back: 39 of 39 OK.
- D3's T2 of 256,660 is from claude-301's sheet (`one-model-end-to-end-d3-accum-2026-11-06.md`, "D3's tables").

## Skills used

None beyond the loop's own steps.

## Still open

- **The training run on D4** is the next unit, under its own sheet: (b64L) unchanged, on RTX 5090s, at two seeds.
  - At 16,384 steps × 64 × 192 it sees about 2.7 passes of D4, against 7.2 of D3. Two things change with the data: the amount of distinct text, and the mix (OASST2 falls from 13.5% to 5.1%). The sheet must read the oasst2 sub-panel on its own for that reason.
  - claude-301's (b64L) trained for 10,924 and 12,008 s, about $2.99 and $3.34 at its pods' rates. Its run-2 pods billed $4.84 and $5.48, which covers a second arm and scoring.
  - Its bar is against D4's T2 (256,443) and unigram (37,421), with `clean=` this seen set, and a data lever of (b64L)-on-D4 against (b64L)-on-D3.
- **The census** is not built (see above).

## Plain summary

The next experiment needs more real text to learn from. I built it from the three Wikipedia files we already had, about 2.65 times as much text as before. The test questions, and the way answers are scored, stay exactly the same, so the results compare directly. I checked byte by byte that the old data is unchanged inside the new data. The test documents stay out of training because which side a document lands on depends only on its id. About a fifth of test positions still repeat an 8-token run from training, which is why the unseen rows are scored separately. This cost nothing; it ran on the laptop in under a minute of compute. Training on it comes next, at about $3 of training time a seed.

References #3407
