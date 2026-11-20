# [codex] Reserved-TRAIN A1 refit and exact reload

**The v4 reserved-TRAIN fit and cold reload passed.** The real fitted payload stores all 7,392 Q32 weights in 59,180 bytes. The loader reproduced 575 exact original-target CUDA mass words. Neither DEV stream was accessed. These are a fitted-state result and a target replay on R. Whole replies and held quality have their own acceptance gates.

## What stopped production

On the leased RTX 5090, the v2 A1 executable passed its finite numerical and malformed-input checks. Its linked-library and PTX checks also passed. The supplemental disassembly of the actual loaded sm_120 image found that the assembler had lowered the typed integer quotient through `I2F.U32.RP`, `MUFU.RCP` and `F2I.FTZ.U32.TRUNC.NTZ` in both the responsibility and maximization kernels. The disassembly also contained all-zero `HFMA2` idioms. The arithmetic lowering blocked the archive-backed production fit despite its exact finite result. The disassembly SHA-256 is `b0460c8405151a25f4e0df8934f796a42a03be260d87965366788ac39215d116`. The extracted sites hash to `0d6eaf30c11e64593cffe369af367c071093452208f8a7e9b468c701772f449d`.

The corrected [CUDA source](device.cu) replaces the variable divide with integer restoring division. The reduced-width trial quotient is an upper bound. Its correction compares the product against the original full-width numerator. The corrected source SHA-256 is `76e5aaf08732c2dcc70fa46be7a25bbf4a7d139472af76b5b7510ee8a3a64ffe`. Separate changes remove variable integer division from the KN source and the ordinary encode/carrier index operations. The existing shared index implementation matched 132,800 host encode quotients and 200,156 carrier remainders, including admitted widths and maximum unsigned operands. [Exact revised FIT](FIT.md)

## Observed checks

| v2 artifact or check | Observed value |
| --- | --- |
| GPU and job | RTX 5090, `vast-61565679`, build and finite job ended 03:24:59 UTC |
| Host executable SHA-256 | `9f1be86bae835b4ad3da80a3a275cc72e15e2cb816c3f2055e57aaf437c89dc8` |
| SM120 executable SHA-256 | `d153d7201d6436ee281b6941b94ddd75628ec7ecb60a277b3f51e85d7864b433` |
| Finite row tape SHA-256 | `bbde4182b1d12cce0c0c2d038eb0206295fcad31e74679d5a0332e19f3087681` |
| Finite trace SHA-256 | `7242ac91eb8968bf3980f2d453362097bc4b6c1333a8f3b60d7fb548e061c875` |
| Finite numerical comparison | 128 invented targets, 97 exact Q12 sums, 717,024 exact Q32 words |
| Finite truncated-input control | A 15-byte row tape was refused as `invalid row tape header`. No trace was created. |
| Supplemental diagnostic PTX | Four kernel entries, zero rejected tokens. This check missed the lowering above. |

On the replacement RTX 5090 `vast-61568255`, the v3 loaded-image audit found zero data-dependent floating or tensor opcodes in the rebuilt A1 executable, KN, raw lookup and ordinary Pick images. The 128-target CUDA finite differential again matched all 97 Q12 sums and 717,024 Q32 words. The truncated-row check again refused the input. The separate 17-step client finite fixture matched all 4,095 candidate words per step and rejected a truncated row. The job ended at 03:39:09 UTC. The same audit found 11 disallowed opcodes in the n-gram counter, so that build did not produce a full C1 artifact or run the reserved-R fit.

The v4 build fixed the counter's histogram stride at the actual 256×256 launch. Its loaded n-gram counter, raw, KN, ordinary Pick and A1 images passed the disassembly check. V4 reproduced the exact C128 count artifact and its 12,678 lookup probes. V4 repeated the A1 and client finite comparisons and both malformed-row refusals. The combined finite job exited 0 at 03:43:42 UTC. The A1 executable SHA-256 was `9ee5329eedb146fed090a76e96cabf3c69497cdee3b37ad8bdb513e54ec03ae3` and the Rust driver was `e146430310a1e6465e779ae6d03d7ab7de0364204380e5f1c122ce5d22338997`. The real fit ran only after the full C1 artifact passed independent restoration. [Counter repair review](../count/FIT-REPAIRED.md) · [Production GO](GO.json)

## Production fit and original-target replay

The v4 RTX 5090 job ran from 03:52:37 UTC to 03:57:14 UTC. It rebuilt C1 raw and adjusted support against all 22 historical order entries, then used only the 15,947,584 original targets from the 4,060 reserved R records. Each of 97 recorded Q12 sums matched the historical run. The sum went from 254,640,228,133 before updates to 235,787,974,827 after 96 CUDA passes. All 306 distinct archived fitted Pick words matched. The archived 1% report carries only that column, so the new 97-state device trace supplies the other fitted words. A0, C1, C2 and C3 remain the established quality controls for separate comparisons. [A1 scope and bars](SHEET.md) · [Historical A1 receipt](../../claude-439/run2/pick-p10.json)

All 575 selected positions across the first 64 original R records matched the compact reader and reloaded D11 Pick words, including all four Pick blocks, all per-order raw counts and totals, the KN Q48 and CUDA mixture Q48. The full-tape and bounded record-local count lookups agreed. The set has 128 closed, 30 exact-distance and 417 near-distance positions. The closed-gate control rejected all 128 injected hits. Among 24 fired hits, 22 exact-distance and two near-distance, suppressing the same bucket's Pick contribution removed precisely its nonzero Q48 amount. The score trace retains original positions, token IDs and the fitted bucket.

| Phase | Wall time | Peak host RSS |
| --- | ---: | ---: |
| R CUDA feature preparation, CPU packing and exact probe checks | 4:30.21 | 6,556,504 KiB |
| CUDA 96-pass fit and final probe scores | 0:02.37 | 1,975,376 KiB |

The raw and KN R lookups completed 46.889 seconds into preparation. The A1 Pick pass completed at 263.125 seconds. GNU time measured each phase's process RSS. The prior finite calculation used the same executable words and kept its own scope.

| Production output | SHA-256 |
| --- | --- |
| 59,180-byte `fit.weights` | `40af50ff9b01bc857c9f86aefcd3e1f0a1f82d7ac62573167d26e4bba5062f29` |
| `fit.manifest.json` | `98999a62b090218062ef61fb7faf0e7a04ef6662c71adb5bcdacbe1aa90d8ca4` |
| 97-checkpoint `fit.trace` | `1226d637f99b6abf169d39ba390e66b0ceaefcb072b20b37ddb1af5f4b0f8a30` |
| R expert tape `train.rows` | `38709e7829137aaf2b99c5c54637b65cbb6a7a59dbcf4f7bd22393ee5d1d8cba` |
| `probes-scored.json` | `3b592b5c3a24edcd0adf9b7de9e79f643bf6df39b9d695c9274aac177bf95ffa` |
| Producer result manifest | `672749caeff3ae6172af855afd179c81836c8cfcc4ff211588358076201bc36c` |

The measured count payload, independently loaded seed-one trunk and stored fitted weights sum to **2,316,336,147 bytes**, before client package metadata or per-session memory. Wick rehashed the small deployable state and original-target probe packet under manifest `64364d2e11eed399eccf8a4c5be0c51f8172305b2adfea9c7d5ab9c9c576322e`. A fresh native format process there read the full captured trace and raw probe words and independently reproduced the weight file, fit manifest and scored-probe packet byte for byte. The cold manifest is `5d40fb83367b0027e702ac1926126ef2282a06623bdaeb208eda2b4b5e226757`, under the private Wick root `/mnt/plexpool/uor-r4-corpus/runs/20261120-complete-c1-31/repaired-v4/a1-cold-seal`.

Wick also restored and strictly rehashed the complete protected producer packet at 03:59:24 UTC. It includes the 1,913,710,096-byte R expert tape, 5,736,984-byte device trace, original-target tapes, exact action and resources. The private root is `/mnt/plexpool/uor-r4-corpus/runs/20261120-complete-c1-31/repaired-v4/a1-v4/full`, with custody manifest SHA-256 `9008e926d1f1fde6695edd2cfdec2052ec64bab0ebd55199bbbd2fe162bc54aa`.
