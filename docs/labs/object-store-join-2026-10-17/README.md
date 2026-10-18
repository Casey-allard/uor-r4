# Pinned cold join: objpeer join

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-join.

## What changed

- New objpeer join command for the cold-join path: it reads a local decompressed artifact, verifies
  its SHA-256 and length against --expect-sha256/--expect-bytes before touching the store, then folds and
  saves the store atomically. A wrong pin exits 2 (BindingMismatch) and writes nothing.
- Output: JOIN object_bytes/roots/records/sha256/store/millis (JSON with --json).
- No protocol or wire change; compression stays outside the protocol (download compressed, decompress
  locally, then join).

## Verification

- packaged_cli_join_pins_a_local_artifact: pinned join succeeds, store exists, second join is idempotent.
- packaged_cli_join_rejects_a_wrong_pin_without_writing_the_store: exit 2, no store file, BindingMismatch.
- Package: 57 passed / 0 failed; Clippy -D warnings and fmt clean.

## Why it matters

Cold joins are a documented loss (5,725,408 B cold store pull vs 39,920 B compressed for the 2 MiB
artifact). join makes the supported recipe explicit and pinned: compressed download, local fold on
match, then warm deltas. The next review step is tools/cold_join.sh to measure both paths end to end.

## Benchmark: tools/cold_join.sh (2026-10-18)

The requested end-to-end measurement now exists at
`crates/uor-r4-object-store/tools/cold_join.sh` (host-only; no pod, no model compute):

    SIZE_MB=2 DATA=text   bash crates/uor-r4-object-store/tools/cold_join.sh
    SIZE_MB=2 DATA=random bash crates/uor-r4-object-store/tools/cold_join.sh

Measured on this laptop (single run, loaded host; loopback), 2 MiB artifacts:

    text (compressible)  cold pull wire 33,777 B / 553 nodes / 6 rounds / 0.079 s
                         gzip         7,223 B (ratio 0.00344) + 0.029 s decompress
                         join         553 records / 34 ms / store 28,672 B
                         warm delta   252 B / 0 nodes / 0.297 s

    random (incompressible) cold pull wire 5,734,720 B / 98,298 nodes / 31 rounds / 0.613 s
                            gzip         2,097,823 B (ratio 1.00032)
                            join         98,298 records / 116 ms / store 4,554,752 B
                            warm delta   252 B / 0 nodes / 0.340 s

The script parses the real PULL_STATS/JOIN fields, cmp-checks the pulled bytes, requires the
warm delta to fetch 0 nodes, and fails closed if the compressed download is not smaller than the
cold pull wire for compressible data. The cold-pull number here (5,734,720 B random) matches the
earlier incidental cold-pull figure in the record; the text case is much smaller because the
native store deduplicates the repeated line (553 nodes) while the cold pull still carries protocol
framing and ids.

Limits: loopback and single-run wall times are not WAN measurements; the join path needs the
decompressed bytes on local disk (gzip only reduces the transport); incompressible data gains
nothing from the compressed download and is carried by the native store at ~2.2x wire/object.
