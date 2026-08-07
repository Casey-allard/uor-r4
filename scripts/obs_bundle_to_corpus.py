#!/usr/bin/env python3
"""Convert an observation shard bundle into the c_meta/c_recs corpus pair.

The from-text observation driver spills 88-byte v4 records into per-shard
files routed by `shard_of(sample_id(window))`, so `merged.bin` is in
*shard* order, not corpus order. `compiler::load_corpus_from` requires
corpus order: it reconstructs `input[i]` as `next[i-1]` within a story and
resets the story position on every story-id change, so shard-ordered bytes
would produce an interleaved, meaningless token stream.

This script sorts the records into (story ordinal, span_start) order and
writes the pair the measurement harnesses load via `R4_CORPUS_META` /
`R4_CORPUS_RECS`. The record bytes themselves are copied verbatim — only
their order changes — so the token content is exactly the bundle's.

The sort is stable and total (span_start is unique within a story), so the
output is deterministic: the same bundle always yields byte-identical
output, and the printed BLAKE3 κ pins it.

Usage:
    scripts/obs_bundle_to_corpus.py <obs-dir> <out-prefix>

    obs-dir     directory holding merged.bin (and manifest.json, read for
                provenance only)
    out-prefix  writes <prefix>_meta.bin and <prefix>_recs.bin

Example:
    scripts/obs_bundle_to_corpus.py obs-text /tmp/obs
    R4_CORPUS_META=/tmp/obs_meta.bin R4_CORPUS_RECS=/tmp/obs_recs.bin \\
      cargo test -p uor-r4-graph-certify --test long_range_ceiling -- --ignored --nocapture
"""

import hashlib
import json
import os
import struct
import sys

RECORD_SIZE = 88
STORY_OFF = 0
SPAN_START_OFF = 72
# `load_corpus_from` accepts a 25-byte meta whose trailing byte is the
# completion flag; bytes 16..24 carry the generator rng state, which has no
# meaning for a converted bundle. A fixed tag keeps the output deterministic
# and greppable rather than pretending to be a generator checkpoint.
META_TAG = 0x4F42533243505355  # "OBS2CPSU"


def blake3_or_sha256(path):
    try:
        import blake3  # type: ignore

        h = blake3.blake3()
        prefix = "blake3"
    except ImportError:
        h = hashlib.sha256()
        prefix = "sha256"
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            h.update(chunk)
    return f"{prefix}:{h.hexdigest()}"


def main(argv):
    if len(argv) != 3:
        print(__doc__)
        return 2
    obs_dir, out_prefix = argv[1], argv[2]

    merged = os.path.join(obs_dir, "merged.bin")
    raw = open(merged, "rb").read()
    if len(raw) % RECORD_SIZE:
        print(
            f"error: {merged} is {len(raw)} bytes, not a multiple of {RECORD_SIZE}",
            file=sys.stderr,
        )
        return 1
    count = len(raw) // RECORD_SIZE

    manifest_path = os.path.join(obs_dir, "manifest.json")
    if os.path.exists(manifest_path):
        manifest = json.load(open(manifest_path))
        declared = manifest.get("total_records")
        if declared is not None and declared != count:
            print(
                f"error: manifest declares {declared} records, merged.bin holds {count}",
                file=sys.stderr,
            )
            return 1
        print(f"input_cid      {manifest.get('input_cid')}")
        print(f"partition_rule {manifest.get('partition_rule')}")

    records = [raw[i * RECORD_SIZE : (i + 1) * RECORD_SIZE] for i in range(count)]

    def key(record):
        story = struct.unpack_from("<I", record, STORY_OFF)[0]
        span = struct.unpack_from("<I", record, SPAN_START_OFF)[0]
        return (story, span)

    records.sort(key=key)

    keys = [key(record) for record in records]
    if len(set(keys)) != len(keys):
        print("error: duplicate (story, position) pairs; bundle is not a clean stream", file=sys.stderr)
        return 1
    stories = len({story for story, _ in keys})
    # Every story must expose a contiguous 0..L-1 position run, or the
    # reconstructed `input` stream would silently splice across a gap.
    runs = {}
    for story, span in keys:
        runs.setdefault(story, []).append(span)
    for story, spans in runs.items():
        if spans != list(range(len(spans))):
            print(f"error: story {story} positions are not contiguous from 0", file=sys.stderr)
            return 1

    recs_path = f"{out_prefix}_recs.bin"
    meta_path = f"{out_prefix}_meta.bin"
    with open(recs_path, "wb") as handle:
        for record in records:
            handle.write(record)

    meta = bytearray(25)
    struct.pack_into("<Q", meta, 0, count)
    struct.pack_into("<Q", meta, 8, stories)
    struct.pack_into("<Q", meta, 16, META_TAG)
    meta[24] = 1
    open(meta_path, "wb").write(bytes(meta))

    print(f"records        {count}")
    print(f"stories        {stories}")
    print(f"{recs_path}  {blake3_or_sha256(recs_path)}")
    print(f"{meta_path}  {blake3_or_sha256(meta_path)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
