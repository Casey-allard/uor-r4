# D36 CUDA reader for UORKNC02

`lookup.cuh` exposes the batched CUDA entrypoint for callers with a resident,
immutable compact artifact. `device.cu` is the finite AOT launcher. The
standalone Rust witness prepares only metadata and probes, then compares
result words after the device process completes.

All tape words are little-endian. One order descriptor has `u64 keys`,
`u64 rows`, `u32 contexts`, `u32 followers` and three `u64` Q48 discounts.
The eleven descriptors take 528 bytes. Each 24-byte probe contains a `u16`
length followed by eleven `u16` token slots. History is oldest first, the
target follows it, and unused slots are zero. Each probe returns eleven
32-byte order records, each with `u32 raw`, `u32 adjusted`, `u64 raw_total`,
`u64 adjusted_total` and `u64 q48`. The kernel API also returns one `u32`
status per probe. The launcher requires all statuses to be zero before
writing the records. The Q48 field floors zero to one for the exposed order.
The internal chained mass is carried without injecting that floor.

From the Wick source root, build the locked witness and the SASS-only
executable for Wick's GPU. Scan the compiled image and linked libraries as
in the D31 dispatch before sealing it.

```sh
unit=docs/loop/tasks/written-state-count-model/codex-36/device
CARGO_TARGET_DIR="$stage/target" cargo build --locked --offline --release \
  --manifest-path "$unit/Cargo.toml"
/usr/local/cuda/bin/nvcc -O3 -std=c++17 -I crates/uor-r4-discrete/src \
  -gencode arch=compute_89,code=sm_89 "$unit/device.cu" -o "$stage/d36-count-device"
"$stage/target/release/d36-device-witness" prepare "$witness" "$counts" "$packet"
```

For the reviewed all-retained, 8 MiB and exact-floor arms, stage packet
directories named `packet-all`, `packet-8m` and `packet-floor` with their
corresponding `prod-c128-*.json` witness inputs. The dispatch verifies exact
hashes and expects the pruner's final artifacts in `../pruner/final`.
The stage holds `build-and-input.sha256`, `source.sha256`, `FIT.md`,
`GO.json` and `GO.sha256`. The parent GO fields are
`build_and_input_sha256`, `source_sha256`, `fit_sha256` and `device_uuid`.
Run `dispatch.sh STAGE NEW-OUTPUT-DIR` after the [finite sheet](SHEET.md)'s
review and device admission.

## Wick finite result

The original CUDA and Rust executables remain sealed under the
[finite GO](GO.json). Both fresh processes in each arm agreed on every word.
The primary processes took 0.11–0.12 seconds and peaked at 167,416 KiB host RSS.
The original-ID probes covered 26 retained and 14 dropped contexts.

| C128 artifact | Probes | Compared order records per process |
|---|---:|---:|
| [All retained](evidence/all-primary.json) | 11 | 121 |
| [8 MiB cap](evidence/8m-primary.json) | 18 | 198 |
| [Mandatory floor](evidence/floor-primary.json) | 11 | 121 |

The [stale pin](evidence/stale.stderr) was rejected before dispatch.
Changing the first raw output from 128 to zero triggered the expected
[comparison failure](evidence/altered.stderr). The
[receipt manifest](evidence/receipts.sha256) binds both fresh runs and the
negative controls to their Wick custody files.

## Candidate-dose bridge

The v0.2 witness maps an immutable compact artifact instead of reading it
into an owned byte vector. It has a 4,000,000,000-byte artifact bound and an
18-probe limit. The same sm_89 CUDA executable is used unchanged. The v0.2
Wick build reproduced the metadata, probes and verification receipts of all
three finite C128 arms and rejected both of their planted controls.
The original finite build tree remains sealed separately on Wick.

The [candidate sheet](CANDIDATE-SHEET.md) defines the C1 and full-C scope.
`dispatch-candidate.sh STAGE ARTIFACT NEW-OUTPUT-DIR` expects the exact
previous CUDA executable, the v0.2 witness, the candidate's
`candidate-witness.json` audit and a prepared `packet/`. The source and build
manifests, FIT and GO must live in the stage. The GO also binds the candidate
and audit identities as `candidate_sha256` and `witness_sha256`. The launcher
releases the shared GPU lock before running the separate native witness.

The C1 candidate passed the [separate FIT](FIT-C1.md) and [GO](GO-C1.json).
Both fresh RTX 4090 processes matched every word across 198 order records
from 18 probes, including 11 retained and seven dropped contexts, and
produced identical tapes. The device processes took 0.13–0.15 seconds
with 166,764 KiB maximum host RSS. The independent mmap comparisons took
0.31–0.41 seconds with 239,008 KiB maximum RSS. The
[primary receipt](evidence/c1/primary.json) and
[Wick custody manifest](evidence/c1/receipts.sha256) record this bounded
count-component replay.

The full-C threshold-318 compact artifact passed the
[full-C FIT](FIT-FULL-C.md) and [GO](GO-FULL-C.json). It occupies
3,912,334,481 bytes, leaving 601,850 bytes under the compact cap.
Threshold 317 would occupy 3,914,255,830 bytes. Adding the registered
trunk, tokenizer and fit/envelope reserve gives 3,999,398,150 bytes.
The actual fit and assembled deployment have their own measurement.
[Cap ledger](evidence/full-c/cap.json)

Both fresh Wick RTX 4090 processes produced identical tapes and matched
the native reader on 198 order records from 18 probes. This included all
11 retained and seven dropped-context controls. Device wall time was
5.51–5.60 seconds with 166,076 KiB maximum host RSS. The native mmap
witness took 4.82–9.89 seconds with 3,822,308 KiB maximum RSS.
[Primary receipt](evidence/full-c/primary.json) ·
[Wick custody manifest](evidence/full-c/receipts.sha256)
