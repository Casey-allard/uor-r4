# [codex] D28 evidence custody

## Sealed TRAIN and finite device checks

The finite deposit preserves the 68-prompt frozen TRAIN input and labels, the independently reproduced 226-record fit packet and binding, the confirmatory inputs and blind judgments, both planted device fixtures with their host oracles, the finite device readback, the original frozen bank, and the device executables, launch inputs and source snapshots used to interpret the finite results. The device fit and score outputs compare byte for byte with the independent planted host oracles.

| Storage | Locator | Readback |
| --- | --- | --- |
| Wick | `/home/callard/uor-codex/20261119-value-onset-28/custody-finite-v1` | All 78 archived files rehashed from the retained copy. |
| Private Hugging Face dataset `caseyallard/uor-r4-store` | Revision `e8f278ff26e29d9f1457adc7ee86e38261e050ea`, path `results/codex/2026-11-19-value-onset-28/finite-v1` | Downloaded the pinned delivery, decoded the original archive and rehashed every restored file. |

The delivery manifest SHA-256 is `165f894e6d191cb8282489432155f478c6f99b444f8749d53700fb935e9c7dfb`. The original-file manifest SHA-256 is `2f1d8afcee73d5afe56b32dfe82a2f68083dcb9228482f1944200dc7ac6f9cc7`. The compressed archive SHA-256 is `43072a648abb8b68ef1f4c4fdb7eeda1e07f933e16e46a7125930fe154648e7c`.

From a downloaded delivery directory, decode `finite-originals.base64.txt` with `openssl base64 -d -A` to `finite-originals.tar.zst`. Validate `files.sha256` and `finite-originals.tar.zst.sha256`, extract into a fresh directory with `zstd -dc finite-originals.tar.zst | tar -xf - -C RESTORE`, and check `original-files.sha256` from that directory.

## Confirmatory originals and complete carriers

The confirmatory deposit preserves 420 ordinary response JSONs, 210 per process, across the six active carriers. It includes all six raw grade and visit sets, the staged host packets and oracles, the twelve held device outputs, the original GPU archive, primary and cold TRAIN banks, all three sealed banks, both fixture and panel inputs, the sampled ordinary model response, the executable and source bindings, and the validated analysis and per-arm funnels. All 210 ordinary primary and cold response pairs compare byte for byte. Each of the six primary and cold device outputs matches its corresponding host grade packet. The source overlay includes the canonical candidate and collision checks in the final fitter.

| Storage | Locator | Readback |
| --- | --- | --- |
| Wick | `/home/callard/uor-codex/20261119-value-onset-28/custody-outcomes-v1` | All 1,098 files in `raw/` were rehashed. All six complete D11 files retained under `models/` match the registered carrier hashes. |
| Private Hugging Face dataset `caseyallard/uor-r4-store` | Revision `535d12e6ca8d6ffa738f48ad73301cee84b277d9`, path `results/codex/2026-11-19-value-onset-28/outcomes-v1` | Downloaded the pinned delivery, decoded its four chunks and rehashed all 1,098 restored files. |

The delivery manifest SHA-256 is `a6659178c0425688626df2308afa7b978fdbf51bda8ac1936fd3ecab449f015e`. The original-file manifest SHA-256 is `9dfa5383cd31bb29b16a65e865f8a1c8791b7415095777e576e8b20dcc292618`. The 1,485,563-byte compressed archive SHA-256 is `b4b8cfc1152c47a94f2c0b607d424da0b2d9080f551164ba66a1674ca00a3b51`. Concatenate the `chunk-*.txt` files in filename order before decoding with `openssl base64 -d -A`, then verify the archive and original-file manifests with the same restore procedure as the finite deposit.

The carrier manifest binds each complete 86,604,922-byte D11. The Hub archive carries the six exact 65,536-byte prefixes, the tokenizer, and the reconstruction instructions and checksums in `carrier-chain/`. All six files share the exact 86,539,386-byte tail of the frozen legacy carrier `02b922d37308593de956bf9c7c8ee0dbb259d2152528d6993eb9c89be84f65e3`. Wick retains that original at `/home/callard/uor-codex/20261119-requested-occurrence/artifacts/seed1/assertion-agreement.d11`. Wick linked the verified frozen legacy file and reconstructed the other five from the retained chain. The restored Hub prefixes also reproduced all six workstation originals byte for byte with the same canonical tail. The chain manifest SHA-256 is `84528a2e052a1eef435e454039d0d0d45b24b0602699e0cc5178290a78bc7cbe`. [Registered carrier hashes](carriers.sha256)
