# Host diagnostics 2026-10-17 — Stage-A K-sweep re-score and D34 successor tables (kimi/pm1)

Execution of the two host-only FIT'd pre-registrations from [#2572](https://github.com/UOR-Foundation/uor-r4/pull/2572), released on [#2029 comment 6230337501](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6230337501). Executed by the DeepSeek implementer seat under the Kimi PM desk. **$0 GPU, no model forward, no training; `model_compute_executed: false`, `applied_updates: 0`, `host_only: true` in both records.**

## Outputs and identities

| Object | Identity |
| --- | --- |
| Sealed archive (iCloud) | `icloud:UOR-R4/results/kimi/diagnostics-20261017.tar`, 400,324,096 bytes, MD5 `e6e5deff87cb7c0ade76b38afd37cfaa` (upload verified by download round-trip) |
| `stage-a-rescore-record.json` (`uor-r4.stage-a-rescore/1`) | SHA256 `258a00f0b7312f9d68be528deca26bbd692351482de313ae616bbd99f0e530c2`, 4.6 MB |
| `d34-successor-table-record.json` (`uor-r4.successor-table-control/1`) | SHA256 `2717d34bcfcdd1486991925ecab066266a6b56ba34b89727c37f78c821fe3bec`, 5.3 MB |
| `run_diagnostics.py` (stdlib-only analyzer) | SHA256 `79b19c201c7b10ac40054a287e306489d7b813eb7f7b0f132f01d7a29f72736c` |
| Scratch README | SHA256 `51ced15fcd81ef4626ad133f4e37d2804ca3ad64a88daa0bc863a0365933b1f3` |

Both records carry 14,287 per-row entries. Determinism: a second full run produced records identical except `generated_at`. Base `fb4519226`.

Artifact verification: Stage-A archive 33,802,240 bytes, MD5 `b274beb321ce9232bd3690659fee7890` (index-verified before extraction; 38/38 manifest blake3 entries verified). Q4 receipt `4fff1d3c…` and checkpoint `4477eb5c…` verified directly (cursor 16,404 / 407 accepted updates). DEV tokens `0919e90f…` / mask `a838bb1a…`; TRAIN tokens `3dee858f…` / mask `b5e7352d…`; `ft-balp.tar` sha256 `b1d22113…`. Partition binding `e4ef44fb…` reproduced exactly (400 documents; 14,287 eligible; 3,847 `P_succ`; 10,440 `P_comp`; 1,335 conflict; buckets `[50, 2872, 900, 25]`; zero at d=1).

## Diagnostic 1 — Stage-A K-sweep re-score ([sheet](../../harness/prep/stage-a-ksweep-rescore-2026-10-17.md))

- **H1 integrity PASS:** 36/36 sweep points reproduce exactly against the sealed README and `report.json`; all four K=8 full-panel controls exact; Q4 K=8 sub-panel controls exact (`P_succ` 29/170/435/11/20; `P_comp` 120/577/1485/54/85); monotone admission PASS; `k8_qualification = ALL_FOUR_EXACT`.
- **Primary H2 (Q4 K=8 vs K=16 on `A_8`, n = 1,920):** top-1 **149 → 138**; discordance `b = 11, c = 0`; floor `max(24, 6.63) = 24`; **H2 supported (admission-only)** — the primary does not rise by the floor.
- **Secondary H2 (32 fixed-set checkpoint/K steps):** 0 ≥floor rises, 0 ≥floor falls; descriptive only.
- **H3 (top-8):** `P_succ` 170 → 140 (Δ −30); `P_comp` 577 → 446 (Δ −131); difference of deltas 101 ≥ floor 39; **H3 supported**.
- **Decision row 6** (H2 admission-only, flat/falling); H3 row 7; **K\* = 8** (`J(8) = 917`, `J(9) = 909`).

## Diagnostic 2 — D34 successor tables ([sheet](../../harness/prep/d34-successor-table-control-2026-10-17.md))

Host tables built on the model-matched 1,049,856 TRAIN targets (conservation OK; 1,266 distinct query tokens):

| Panel | T2 top-1 | T2 top-8 | T3 top-1 | U/U_succ top-1 | T1b |
| --- | ---: | ---: | ---: | ---: | ---: |
| Full (14,287) | **6,300** | 11,263 | 5,047 | 1,226 | — |
| `P_succ` (3,847) | **1,245** | 2,373 | 882 | 194 | 718 |
| `P_comp` (10,440) | 5,055 | 8,890 | 4,165 | 1,032 | 1,466 |
| Conflict (1,335, descriptive) | 589 | — | 542 | 448 | 954 |

T1 is 3,847/3,847 on `P_succ` by construction (tautological panel-definition check). The unigram reference reproduced the sealed 1,226/14,287 exactly.

**Decision: row 1 — UNAVAILABLE.** The Relation arm has no frozen checkpoint and was not simulated; the Route interim was NOT_RUN (host-only commission; the optional GPU forward was not released).

## What this establishes for the learned-attention line

The context-free T2 table scores **6,300/14,287** top-1 full-panel and **1,245/3,847** on `P_succ`, against the trained Q4 Route artifact's sealed 149/14,287 (29/435 admitted on `P_succ`). Under the merged learned-attention-link sheet (`m1-learned-attention-link-2026-10-17.md`, FIT), T2 is the ordinary comparator the first trained Relation artifact must beat (admission-restricted, `F_{T,R}` floor) before any learned-relation claim; these are the numbers it must clear. They are diagnostic counts at this configuration, not a mechanism verdict (D22).

## Deviations

1. D34 Relation arm UNAVAILABLE (no checkpoint; M2 HELD) — not simulated. Route interim NOT_RUN (no GPU release).
2. The original Q4 receipt is not inside the Stage-A archive; it was verified directly and matches the derived K=8 aggregate on every shared field (the receipt carries batch aggregates, not per-row records).
3. `ft/dev/manifest.json` says `split: train`; its token/mask hashes, sizes, partition and binding match the pinned DEV artifact — the pinned hashes are authoritative.
4. `cloud-store` deletes the staged tar after MD5-verified extraction; integrity rests on the 38-file blake3 manifest.

Line: M1 qualification · count 0/3 unchanged · headline 37/232 suspended unchanged. Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE.
