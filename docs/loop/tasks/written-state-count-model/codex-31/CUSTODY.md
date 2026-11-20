# [codex] D31 private custody and lease close

Wick holds the D31 evidence under `/mnt/plexpool/uor-r4-corpus/runs/20261120-complete-c1-31`. The archive roots have mode `700`. Paths in the table are relative to that directory. The verifier checked file hashes after restoration, before the corresponding Vast pod was deleted. Corpus, candidate rows, probes and executable images remain in private custody. [Count evidence](count/REPORT.md) · [A1 evidence](fit/REPORT.md) · [Client evidence](REPORT.md)

| Capture | Relative manifest on Wick | Manifest SHA-256 |
| --- | --- | --- |
| First finite controls and rejected machine code | `pod-finite/runs/count31/custody.sha256` | `16df68bea6887e94328e33f1c54bd0ba45c1fc9ea0825dc742888add6ebc259d` |
| Intermediate repaired build with the histogram audit failure | `repaired-v3/restored/runs/count31-repaired/v3/custody/artifacts.sha256` | `55ef483b1f7045ca91fb5a18435faf4e53713cd7f2346ad76db888e3eb9d1386` |
| Final source, AOT images, disassembly and finite comparisons | `repaired-v4/finite-restored/runs/count31-repaired/v4/finite-custody/artifacts.sha256` | `9d14ea3f7bfe7ab891b381c72b5942b159d797870c3ef473aa101bf113c82ef0` |
| Full C1 and independent cold lookup readback | `repaired-v4/count-v4/cold-custody.sha256` | `6c611e6b37aeeb940525e6cfc90e1a6912604f6e8ec5b1b51ba5a2dfbed98ed9` |
| A1 producer and full training row and fit tapes | `repaired-v4/a1-v4/full/runs/count31-repaired/v4/a1-custody-full/artifacts.sha256` | `9008e926d1f1fde6695edd2cfdec2052ec64bab0ebd55199bbbd2fe162bc54aa` |
| Both full CUDA panels and their candidate and score tapes | `repaired-v4/client-v4/full/runs/count31-repaired/v4/client-custody-full/artifacts.sha256` | `c4d2c7da71e722a4c02a0557c43fbcdf729c77c83dc107cdb20efbd2f3375524` |

The full count file is `repaired-v4/count-v4/runs/c1/compact.counts` alongside the pod result manifest `results.sha256` in `repaired-v4/count-v4`. The A1 weights are `repaired-v4/a1-v4/full/runs/count31-repaired/v4/a1-real/fit.weights` alongside `fit.manifest.json`, `train.rows` and `fit.trace`. The primary and cold device receipts and step tapes are under `repaired-v4/client-v4/small/panel-primary` and `repaired-v4/client-v4/small/panel-cold`. The full device archive contains 789 verified files and 201,516,631 bytes of client-run material. The ordinary client outputs and timing are under `normal-5ea09ea47/real-client`.

| Vast instance | Released UTC | Deleted UTC | Ledger uptime | Quoted rate | Estimated compute cost |
| --- | --- | --- | ---: | ---: | ---: |
| `vast-61565679` | 2026-11-20 03:31:31 | 2026-11-20 03:31:36 | 905 s | $1.042/h | $0.262 |
| `vast-61568255` | 2026-11-20 04:03:28 | 2026-11-20 04:03:35 | 1,744 s | $1.042/h | $0.505 |
| Total | | | 2,649 s | | $0.767 |

The cost estimate uses the quote and provider-reported uptime. The provider invoice remains unverified. The live listing at 04:03:37 UTC showed zero project pods at $0/hour.

The archived C1 action preserves the original malformed `GO.sha256` and the corrected `GO.strict.sha256`. The executed dispatch had SHA-256 `8309f1480502580bca649d9f8368596f8c88e1c6004afe9d7f3002302bca6dc4`. Its separate stage and build checks passed at launch. The complete authorization manifest was verified after the run. Future dispatches require well-formed checksum manifests, the authorized scope and GPU identity, and all required input and build digests from `GO-COUNT.json` before creating a run directory. [Historical repaired FIT](count/FIT-REPAIRED.md) · [Current dispatch](count/dispatch.sh)

Host-only fixtures with GNU `sha256sum` reproduced the historical acceptance of a manifest containing one valid entry and one malformed entry. The repaired dispatch blocked malformed stage and GO manifests, an omitted GO pin, a missing required digest, a wrong scope and changed C1 input before reaching the resource gate. A complete fixture reached the resource gate, which the test deliberately blocked before any run directory was created.
