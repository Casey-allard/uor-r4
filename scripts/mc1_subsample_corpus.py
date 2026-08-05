#!/usr/bin/env python3
"""M-C1 subsample control for the #399/#393 capacity-starvation hypothesis.

Produces a record-count-matched control corpus from a larger v3 (88-byte
record) observation corpus: the FIRST `--records` records of the source,
truncated back to the last complete story-run boundary at or before the
target, with a correctly rewritten 25-byte meta.

Background: a merged observation corpus (e.g. /tmp/wiki10k-obs/merged.bin,
the 16-shard wiki10k merge) is a concatenation of shards in which EVERY
story contributes a slice of sparse probed positions to EACH shard, stories
ascending within a shard. Story ids are therefore not globally monotonic
and a story is not globally contiguous; the honest truncation point is a
story-RUN boundary (the record where the story id changes), which is what
the corpus loader treats as a story transition when rebuilding `input`.

The meta `stories` field is written as `max story id + 1`: consumers index
`vec![true; c.stories]` by story id, so the field must exceed every id
present, and the 80/20 `train_cut` stays on the same story-id split as the
full corpus.

Usage:
  python3 scripts/mc1_subsample_corpus.py \
      --src-meta /tmp/wiki10k-obs/state.bin \
      --src-recs /tmp/wiki10k-obs/merged.bin \
      --out-meta /tmp/mc1_meta.bin --out-recs /tmp/mc1_recs.bin \
      --records 500000
"""

import argparse
import struct
import sys

RECORD_SIZE = 88  # v3: story u32 | next u32 | top8 tokens | top8 weights | 4 anchors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--src-meta", required=True)
    parser.add_argument("--src-recs", required=True)
    parser.add_argument("--out-meta", required=True)
    parser.add_argument("--out-recs", required=True)
    parser.add_argument("--records", type=int, default=500_000)
    args = parser.parse_args()

    meta = open(args.src_meta, "rb").read()
    if len(meta) != 25 or meta[24] != 1:
        print(f"source meta is not a finished 25-byte corpus meta: {args.src_meta}")
        return 1
    n_src, stories_src, rng = struct.unpack("<QQQ", meta[:24])

    target = args.records
    with open(args.src_recs, "rb") as f:
        buf = f.read((target + 1) * RECORD_SIZE)
    n_avail = len(buf) // RECORD_SIZE
    if n_avail <= target:
        print(f"source has only {n_avail} records; nothing to truncate")
        return 1

    def story(i: int) -> int:
        return struct.unpack_from("<I", buf, i * RECORD_SIZE)[0]

    # Truncate at the last complete story-run boundary at or before target.
    cut = target
    if story(target - 1) == story(target):
        i = target - 1
        while i >= 0 and story(i) == story(target - 1):
            i -= 1
        cut = i + 1
    if cut == 0:
        print("no story-run boundary at or before the target record count")
        return 1

    kept = {story(i) for i in range(cut)}
    stories_field = max(kept) + 1

    with open(args.out_recs, "wb") as g:
        g.write(buf[: cut * RECORD_SIZE])
    with open(args.out_meta, "wb") as g:
        g.write(struct.pack("<QQQ", cut, stories_field, rng) + b"\x01")

    print(
        f"M-C1 subsample: {cut} records (target {target}) of {n_src}; "
        f"{len(kept)} distinct story ids, meta stories field {stories_field} "
        f"(source {stories_src}); last kept story {story(cut - 1)}, "
        f"first dropped story {story(cut)}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
