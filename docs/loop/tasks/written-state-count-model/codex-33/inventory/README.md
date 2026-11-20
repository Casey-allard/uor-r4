# [codex] D33 whole-context retention inventory

Keeping C1 orders zero through three already costs **128,880,606 compact bytes**, or **215,478,532 deployment bytes** with the current D11 component sizes. That exceeds the historical seed-one **125,998,708-byte** prototype total by **89,479,824 bytes** with all higher-order contexts removed. Prototype comparisons that enforce this low-order floor therefore require a different shared budget or a separately reviewed change to the floor or format. The registered P2 requirement remains equal table bytes across all four active arms. [Seed-one prototype ledger](../../codex-22/equal-bytes.md) · [P2 gate](../../../../../research/reviews/pi-review-2026-11-16/DECISION.md)

The inventory used the validated `CompactCountLookup::visit_counts` on the pinned physical D31 C1 table. For each order ≥4 and support threshold T, it retains a context exactly when the sum of its original raw follower counts is ≥T. Retention keeps the complete context and all its raw and adjusted words, in canonical key order. It always keeps orders 0–3. Nothing reads a language panel or chooses a threshold using performance. This is a byte and support census, with no pruned artifact, recomputed discounts, fit, language evaluation or address comparison.

| Minimum raw support at orders 4–10 | Compact bytes | Projected deployment bytes | Retained raw support at orders 4–10 | Retained adjusted support at orders 4–10 |
|---:|---:|---:|---:|---:|
| 1 | 2,229,740,835 | 2,316,338,762 | 100.00% | 100.00% |
| 2 | 544,346,506 | 630,944,432 | 53.24% | 33.77% |
| 4 | 276,123,394 | 362,721,320 | 38.04% | 18.55% |
| 8 | 200,944,233 | 287,542,159 | 30.29% | 12.32% |
| 16 | 167,482,412 | 254,080,338 | 24.59% | 8.56% |
| 32 | 150,493,951 | 237,091,877 | 20.15% | 6.08% |
| 64 | 141,069,300 | 227,667,226 | 16.54% | 4.37% |
| 128 | 135,612,446 | 222,210,372 | 13.27% | 3.11% |
| 4,096 | 129,108,805 | 215,706,731 | 2.90% | 0.44% |

[The complete receipt](retention.json) records all 13 thresholds, each of the eleven orders, retained context and follower counts, raw occurrence support, adjusted predecessor support, and exact projected table-section data bytes. The percentages above use the unpruned sums over orders 4–10 in each domain. They count occurrences across orders and do not represent unique training positions or served probability mass. The unpruned inventory reconstructed the exact 2,229,740,835-byte file size and matched D31's distinct contexts, distinct followers, raw targets and adjusted targets in every order. The independent C1 oracle already matched all of D31's canonical count digests. [C1 oracle result](../oracle/RESULT.md)

The projection charges 836 fixed compact header bytes, `ceil(12k/8)+4` bytes per retained order-k context and 10 bytes per follower except order ten, which stores a shared count in 6 bytes per follower. The current D11 ledger adds the 86,536,132-byte seed-one trunk, 59,180-byte A1 weights, 2,299-byte fit manifest, 48-byte P1 header and outer header, which changes with the decimal width of the embedded state length. The manifest length and weight format are assumptions for candidates. Pruned counts change the fit binding, and a usable deployment requires a new manifest and appropriate R-only fit. The historical 125,998,708-byte anchor leaves 39,400,783 compact bytes with these current component sizes.

The **4,000,000,000-byte full-C engineering ceiling** allows 3,913,402,073 compact bytes on the current ledger at the cap. D31's historical full-dose census projects 3,058,504,360 data bytes for orders 0–3. After charging those orders and all 836 compact header bytes, at most **854,896,877** bytes remain for orders 4–10. Full-C retention and the common realizable P2 table budget still require their own occupied-byte census. C1 retained-support percentages cannot determine either. [D31 full-dose projection](../../codex-31/REPORT.md)

## Custody and reproduction

Wick's AMD Ryzen 9 9950X ran the read-only inventory in 4.50 s wall at 2,179,252 KiB peak RSS, including the 2.23 GB compact input buffer. The [evidence manifest](evidence.sha256) binds the result, timing and exact source/executable manifest. The retained private Wick root is `/mnt/plexpool/uor-r4-corpus/runs/20261120-count-identity-33/inventory`. From the D33 worktree source on Wick, run with a fresh private output path:

```sh
unit=docs/loop/tasks/written-state-count-model/codex-33/inventory
input=/mnt/plexpool/uor-r4-corpus/runs/20261120-complete-c1-31/repaired-v4/count-v4/runs/c1/compact.counts
CARGO_TARGET_DIR="$result/target" cargo build --offline --locked --release --manifest-path "$unit/Cargo.toml"
/usr/bin/time -v -o "$result/inventory.time" "$result/target/release/d33-retention-inventory" "$input" > "$result/retention.json"
cmp "$result/retention.json" "$unit/retention.json"
```
