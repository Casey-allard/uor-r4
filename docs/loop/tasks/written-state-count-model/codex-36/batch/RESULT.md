# [codex] Resident compact batch finite readback

**PASS.** On Wick's RTX 4090, eight fresh processes each wrote the full
3,752-byte tape for the 26 original-ID targets in the planted four-record
fixture. The 112-byte header matched the magic, target count and three pinned
identities in every output. All 598 feature words per output matched the
ordinary compact CPU oracle byte for byte, yielding 4,784 comparisons
across the eight outputs. Every tape SHA-256 was
`5ab0805021b32409e5232420b2b9b2ab219b69c455115990ee3b2b5427595a1e`.

| Rows per batch | Launches in each process | Primary internal time | Cold internal time |
|---:|---:|---:|---:|
| 1 | 26 | 45 ms | 51 ms |
| 3 | 9 | 33 ms | 38 ms |
| 10 | 3 | 30 ms | 30 ms |
| 26 | 1 | 47 ms | 30 ms |

Each process recorded one 4,517,125-byte compact upload taking 6 ms.
External wall time ranged from 0.15 to 0.18 seconds. Peak host RSS across
the processes was 180,216 KiB. Live post-context device free memory was
24,838,406,144 bytes. These timings describe the tiny finite fixture.
The full R throughput has its own pending qualification.

Before GPU dispatch, the host tests rejected planted cross-record history
and a lost batch prefix by inspecting the actual probe words. The host
format controls also rejected malformed framing, stale UORT identity,
changed header fragments, a zero batch width and attempts to overwrite
an existing tape. [Host captures](evidence/)

The exact [GO](GO.json) binds both non-author FITs, the [dispatch](dispatch.sh),
and the pinned Wick source, build and inputs. The original hardware outputs,
including all eight feature tapes, remain at
`/mnt/plexpool/uor-r4-corpus/runs/20261120-whole-context-36/batch/gpu-finite`.
The [sealed capture manifest](evidence/gpu-finite/receipts.sha256) has
SHA-256 `ef3ff3a0d377ce25072fcfd41ee8073e5530ceab29d3fa64e08f4758a21b0097`.
