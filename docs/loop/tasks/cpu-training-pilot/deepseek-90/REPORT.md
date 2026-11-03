# REPORT: public M1 CPU trunk parity, deepseek-90

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base:
`e2470ec4d09e873352df128133116f73d3ababda`.

## What did not work

- **The existing CPU discrete reader refused the public trained artifact.**
  `CpuDiscreteProposalScorer` implements the four-table Route/Flock/Relation
  family. It rejected the 19-table M1 container instead of silently dropping
  the 15 post-rotation tables.
- **The trained trunk is not trainable in this unit.** The new API is
  read-only. The four-table trainer and writeback continue to refuse the
  19-table artifact because they cannot rewrite its extra sufficient-statistic
  tables.
- **The first synthetic test build used the wrong kappa representation.** The
  snapshot exporter expects the `blake3:`-prefixed operator and composition
  identities; stripping the prefix made the test artifact invalid before the
  reader ran. The fixture now uses the compiled identities byte for byte.
- **Reusing a mixed-target build root reproduced the repository's known
  feature-artifact collision.** A fresh target per test/release invocation
  passed. This was a build-cache instrument fault, not a source failure.
- **This is not a language result.** It proves exact agreement with the public
  M1 integer evaluator on an existing trained artifact. It does not train a
  new model, generate or grade complete replies, or establish fresh-phrasing
  quality.

## What changed

- Added `CpuDiscreteTrunkReader`, a read-only independent CPU path for
  `uor-r4.lut-discrete-trunk/1`.
- The reader first runs the independent CPU Route, Flock or Relation base read.
  It then runs the pure-integer recurrence, causal pointer successor,
  joint-pointer candidate selection, Hamilton rotation and post-rotation
  `OuterL1V1` writer from `uor-r4-discrete::trunk`.
- Exposed the exact query, admitted cells, top-8, token, read code, recurrence
  code and pointer successor as `CpuDiscreteTrunkDecision`.
- Kept the training parser at four tables. The new 19-table entry point is
  separate, and tests require both `CpuDiscreteRelation::from_bytes` and
  `CpuDiscreteProposalScorer::from_bytes` to continue refusing the trunk.
- Extended `cpu-public-discrete-parity` to compare every full 32-coordinate CPU
  query and complete decision with `IntegerDiscreteModel`.

## Measurements

Public seed-1 artifact:
`04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`.

| Reading | Result |
|---|---:|
| DEV rows | 14,287 |
| Q12 query coordinates checked | 457,184 |
| Query-coordinate mismatches | 0 |
| Admitted-cell mismatches | 0 |
| Top-8 mismatches | 0 |
| Token mismatches | 0 |
| Admitted targets | 9,012 |
| Top-1 | 7,002 |
| Top-8 | 7,485 |
| Decisive checksum | `b6d9d8b8cdc7c81e` |
| Release wall | 2.069 s |
| Rows/s, one thread | 6,904.414 |
| Peak RSS | 1,450,320 KiB |

The aggregate readings equal the published M1 seed-1 result. The synthetic
control also compares query, cells, top-8 and token against integer serving,
and corruption controls refuse truncation, an invalid recurrence step
candidate and a sparse-table token outside vocabulary.

## Checks run at the head

- `cargo check --locked -p uor-r4-cpu-train --all-targets`: passed.
- Focused `cpu_trunk` tests in a fresh release target: 2 passed.
- `cargo test --release --locked -p uor-r4-cpu-train --lib`: 135 passed,
  0 failed, 4 ignored.
- `cpu-public-discrete-parity` on the pinned 14,287-row DEV fixture: passed
  with the readings above.
- Targeted `rustfmt --check` on the three changed Rust files: passed.
- `git diff --check`: passed.

## Skills used

No available Codex skill applied. The existing CPU readers, D11 trunk parser,
integer serving evaluator and report rules were read directly.

## Still open

- CPU training and artifact writeback over the 15-trunk-table M1 model.
- A public trained FBS artifact carrying learned Hamming, turn-memory and
  point-key records; the private result store still requires credentials.
- Complete-reply correctness, fresh-reply phrasing, and a configuration that
  improves a held-out ranking bar.
- GPU qualification. This CPU parity result does not replace a GPU bar.

## Plain summary

The public trained M1 model now runs end to end through an independent CPU
read and all 19 tables. Every one of 14,287 DEV decisions matches integer
serving exactly, including all 457,184 Q12 query coordinates. This closes the
CPU read/trunk parity gap; it does not yet make that 19-table model trainable
on CPU or prove useful conversation.
