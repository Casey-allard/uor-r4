# [codex] Count-only deployments fall below the ft/dev prediction floor

All twelve count-only deployments missed the seed-1 ft/dev floor by more than their paired bands. The strongest, the full-minus-fit depth-four allocation, predicted **2,306 of 14,287** original targets against **7,002** for the published trunk. Across the same targets it gained 780 correct predictions and lost 5,476, for a net loss of 4,696 against a paired band of 159. For this count-only configuration, the DEV-tune likelihood improvement did not carry over to the ft/dev reply-token floor. P1(d) remains open for the integrated deployment with its proposed Pick and floor experts. [Full paired receipt](wick/result.json) · [Matched-byte DEV-tune scores](../count/RESULT.md)

| TRAIN counting dose | Context allocation | Entire D11 bytes | Correct / 14,287 | Gains | Harms | Net vs trunk | Paired band |
|---|---|---:|---:|---:|---:|---:|---:|
| 1% | Exact depth one | 93,180,496 | 1,136 | 353 | 6,219 | −5,866 | 163 |
| 1% | Depth two | 93,180,496 | 1,522 | 332 | 5,812 | −5,480 | 157 |
| 1% | Depth three | 93,180,496 | 1,553 | 366 | 5,815 | −5,449 | 158 |
| 1% | Depth four | 93,180,496 | 1,568 | 396 | 5,830 | −5,434 | 158 |
| 10% | Exact depth one | 105,048,364 | 1,156 | 401 | 6,247 | −5,846 | 164 |
| 10% | Depth two | 105,048,364 | 1,986 | 469 | 5,485 | −5,016 | 155 |
| 10% | Depth three | 105,048,364 | 2,038 | 544 | 5,508 | −4,964 | 156 |
| 10% | Depth four | 105,048,364 | 2,031 | 569 | 5,540 | −4,971 | 157 |
| 100% minus TRAIN fit | Exact depth one | 125,998,708 | 1,168 | 408 | 6,242 | −5,834 | 164 |
| 100% minus TRAIN fit | Depth two | 125,998,708 | 2,084 | 540 | 5,458 | −4,918 | 155 |
| 100% minus TRAIN fit | Depth three | 125,998,708 | 2,220 | 710 | 5,492 | −4,782 | 158 |
| 100% minus TRAIN fit | Depth four | 125,998,708 | 2,306 | 780 | 5,476 | −4,696 | 159 |

Gains count rows where only the count deployment is correct. Harms count rows where only the trunk is correct. The band is the ceiling of twice the square root of their sum. The same frozen TRAIN-fitted candidate files were retained throughout. All twelve arms have the same original ft/dev ordinals, canonical prefixes and eligible target IDs. Their corpus-derived next-token predictions use the same fixed floor as the DEV-tune experiment. This is teacher-forced top-one evaluation on the original M1 seed-1 rows. [Sheet](SHEET.md) · [Independent FIT](FIT.md) · [D83 close-grade evidence](../../../../../labs/m1-close-saved-model/README.md)

The fresh Wick CUDA trunk run reproduced the historical 7,002/14,287 reference. The original per-row records have SHA-256 `9c88ee8d20518a0063f1a8eabb0b37ed84861d52cce561604093404840c8caac`. On every eligible row the Rust verifier reconstructed the original target position and canonical prefix, then agreed exactly with the CUDA tape on greedy ID, winner mass, actual-target Q32 mass, Q12 bits and all four route-key words. This covers **171,444 arm-rows**. All twelve tapes matched their capture-time digests. A different arm's native words and a corrupted response mask both triggered the expected refusals. The encoded row-order digest, seed-1 D11 hash, baseline digest, fixture identities, actual D11 sizes, artifact hashes and device tapes are in the receipts. [Baseline](wick/baseline.json) · [Full receipt](wick/result.json) · [Checksums](wick/inputs.sha256) · [Checked tapes](wick/checked-tapes.txt) · [Wrong state](wick/reject-wrong-native.log) · [Corrupted mask](wick/reject-corrupt-mask.log)

The RTX 4090 launcher reported 2 minutes 47.64 seconds of wall time and 1,491,188 KiB peak RSS, including waits for the shared lock. The baseline device scorer reported 87,234 microseconds. The separate Rust reload and tape verification took 10.20 seconds and peaked at 894,496 KiB. [Wick device and tape log](wick/device.log) · [Launcher timing](wick/time.txt) · [Reload timing](wick/grade-time.txt) · [Launcher](run-wick.sh) · [Verifier](grade-wick.sh)

The same frozen count-only deployment can improve the language-stream likelihood and remain far below the response-token floor. DEV-test remains sealed and P1(b) still gates P2. The private store independently returned the original baseline, all twelve GPU tapes, the raw fixture and the verifier results byte-for-byte. After preflight strengthened the raw input and native-state bindings, a rebuilt scorer reproduced the full-minus-fit depth-four ft/dev tape exactly. [Immutable archive receipt](../count/STORE.md) · [Validated-scoring replay](../count/PREFLIGHT.md)
