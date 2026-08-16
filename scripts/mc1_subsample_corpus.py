#!/usr/bin/env python3
"""Retired pre-#729 recorded-corpus prefix writer.

This historical helper selected a prefix ending at a story-run boundary and
rewrote ``stories`` as ``max(kept_story_id) + 1``. That changed the derived
80/20 story cutoff instead of preserving the finalized source corpus's exact
train/held partition, so outputs from different requested sizes were not the
fixed-partition controls the scaling analysis assumed.

Use the registry-aware Rust transaction instead:

    r4 transformerless subsample-recorded-corpus \
        --src-meta SOURCE/corpus.meta --src-recs SOURCE/corpus.records \
        --out-meta OUTPUT/corpus.meta --out-recs OUTPUT/corpus.records \
        --records N

The tombstone intentionally exits before parsing arguments, reading an input,
or touching an output path.
"""

import sys


def main() -> int:
    print(
        "mc1_subsample_corpus.py is retired; use "
        "`r4 transformerless subsample-recorded-corpus`",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    sys.exit(main())
