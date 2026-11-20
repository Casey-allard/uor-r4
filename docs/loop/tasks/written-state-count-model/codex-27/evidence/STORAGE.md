# [codex] D27 custody

The capped count artifact and its planted control were retained as complete files on Wick. The private Hub deposit carries the registered packer source, per-order byte ledger, original client JSON, panel source and sealed raw input and semantic records. The Wick copy also retains those text receipts.

| Storage | Locator | Verified contents |
|---|---|---|
| Wick | `/home/callard/uor-codex/20261119-query-assertion-27/custody-count-panel-v1` | 49 input, source, and receipt files plus two whole count carriers. Rehashed each file on Wick. |
| Private `caseyallard/uor-r4-store` | Revision `45c826009e91ec5a0fd8d6acbf72ecc7f282b4d8`, path `results/codex/2026-11-19-query-assertion-27/count-panel-v1` | Downloaded and verified all 49 file digests and compared both manifest files byte for byte. |

The text manifest SHA-256 is `b73d772bc146756d2678ee6d5f3891a4319cb65f35511e7db0a0745f6a879f21`. The carrier manifest SHA-256 is `c9c6d9053a03b7e22394cbe63452ef0ea527e494e3904f206fa36f1e3fb67641`. The active capped artifact is `carriers/active.d11`, SHA-256 `81a36d7ed720952ce72f0b0535a1ad40afe33b69a3f02a0a7ecb483b8d3d0750`, and the planted control is `carriers/fixture.d11`, SHA-256 `eebb8ff66ca5190e0ddfbe0ae4c6d701b3adcbbe52162c77681dd3d67cae03e7`. The Hub holds the exact count reproduction source and frozen census. [Count reproduction](../count/README.md)

## Finite count device follow-up

Wick preserves the exact GPU inputs and outputs in `/home/callard/uor-codex/20261119-query-assertion-27/count-device`, with the original six output files under `output-v1`. Wick reverified the eight manifest-bound input files and six output files. The private Hub carries the pinned inputs and outputs manifests, device identity, executable and build pins, the fixed map trace, and all three exact map outputs at revision `20ef8c6e41cf2ddb143a12b3129ecd1dd262bb25`, path `results/codex/2026-11-19-query-assertion-27/count-device-v1`. All 13 text files read back at that revision and match manifest `778aa06a35e382172a63a9d4f4940e9cf6e233f38618fe4380ce0ed015ccebd3` by SHA-256. [Finite replay scope](../count/GPU.md)

## Selector staging and device readback

Wick preserves the 66 manifest-bound files under `/home/callard/uor-codex/20261119-query-assertion-27/custody-outcomes-v1`, including the finite and held device packets, binary selector inputs and primary/cold outputs, both native staging tapes and independent oracles, planted host output, dispatch records and source. The file manifest is `bb8baa3f69cbf6f098eccaee0800422cfea1794890a17c6651f36603d2cb292b`. Wick also checked four complete relation carriers against `carriers/identities.sha256`.

The private Hub carries 35 registration, binding, raw native JSONL and device receipt files at revision `7d9ed2ed4abb216980f23cef054e50b3ba6e6402`, path `results/codex/2026-11-19-query-assertion-27/selector-v1`. All 35 were downloaded and rehashed on the workstation against manifest `2b44ba879cec459258223d92117092600f8da272b0ae4bf215cd9a232dd0fe39`.

## Final manuscript and ordinary-client readback

The final packet preserves all 1,104 original ordinary-client JSONs from the twelve parent, arm and run combinations. It also preserves the job input text, the original ordered and recurrence grades, value-witness arrays, calibrated summaries, unsupported audits, onset replay source, input cases and original coordinate replay. The manuscript comes from project revision `01ba347dbb10852d9b5a98a53cb241593a9a882c` and includes the final report, evidence analysis, onset localization, the per-row TSVs and serving notebook. The report binds 552 primary/cold pairs.

| Storage | Locator | Readback |
|---|---|---|
| Wick | `/home/callard/uor-codex/20261119-query-assertion-27/custody-ordinary-v1` | Rehashed all 2,236 raw files and 20 delivery files. The manuscript and transport manifests are under `delivery/`. |
| Private `caseyallard/uor-r4-store` | Revision `620fab3317fecc888372c7e7224ac01a3a2cfd66`, path `results/codex/2026-11-19-query-assertion-27/ordinary-v1` | Downloaded and rehashed all 20 delivery files. Decoded both text chunks to the 156,164-byte compressed archive, extracted it, and rehashed all 2,236 original files. |

The delivery manifest SHA-256 is `942a83d1e3ee08e54ce85391e475ae1bccfcfe360bb587beabbb77aaf86f3889`. The original-file manifest SHA-256 is `e17ba16f61efeba65fb5950684b45e740af8d2031943521b60f85a650a7ad3c3`. The compressed archive SHA-256 is `bb4e9d59a27beb4ebbbdd1d106db4f68d6abe5633a516b349bfeb975f27fa050`. `TRANSPORT.md` gives the commands for exact restore and checksum verification. The original coordinate replay `raw/grades/final-onset-replay.json` is `422bf04aa096855b37ff2004b0fcf52bdea4ec3dd738e11b822735dd7ed78642`.
