# [codex] D33 independent test fitness and Wick custody

The [order](ORDER.md) and [sheet](SHEET.md) are fit for finite host checks on Wick. Full C1 dispatch requires separately reviewed finite controls, a source and dependency scan, exact executable pins and the top-session GO. The integrated D11 checks can advance independently of the C1 count run.

The [accepted decision, §3.3](../../../../research/reviews/pi-review-2026-11-16/DECISION.md) names the D84 CPU trunk driver for finite slice parity and requires equality among the GPU writer, CPU oracle and ordinary D11 loader. The stock driver updates different sparse tables and can score held rows. This unit uses the same-purpose separately registered Rust integer comparator, a precedent established in [D10](../codex-10/COUNT-FIT.md). The top session explicitly registered it as a host count parity instrument. Under Charter §3.5 and §4.5, the CPU derives reference words from the pinned TRAIN slice only. The archived D31 5090 run supplies the qualified GPU words. All comparisons retain the D84 development-only CPU scope. The ordinary loader exercises integer client serving under §3.6. A 4090 repeat could test portability, while the open questions here require a source-based CPU count and native D11 readback.

## Re-verified on Wick

The AMD Ryzen 9 9950X Wick host had 38 GiB available memory and about 40 TiB free in the owned corpus filesystem at inspection. Recheck both at dispatch. The real C1 input consists of 5,402 whole records and 20,754,301 tokens. The C128 fixture consists of 128 records and 111,671 tokens. Both receipt metadata and actual input hashes agree.

| Actual file or manifest | SHA-256 |
|---|---|
| D33 ORDER.md | `ed728446bb68082cd0e8bf960feaab28f5406c4b42caa012bddbf47ff973ebf1` |
| D33 SHEET.md | `aa4ad8fcb273867ab9481cdee6ce65983b5d472ad2bc4c31227c954667c7e8c2` |
| C1 UORT | `8d94e88ac31305ad6973734181a9727f520da46a5ecc263b944247ceedd07030` |
| C128 UORT | `1466d3790e1c53bc4ac3982712f669967cce7b3ad1d7e0e34259568154d19cb6` |
| C128 D31 GPU receipt | `af7d8c9ee99b0fda69ec4c258051928ae9103ec4df7597d6212a94235deaa5e2` |
| C1 D31 GPU receipt | `34e3cd6e9d4ae35a78284a36911547f4ca4109b52e0e85e09fb3af54630f3df7` |
| C1 complete compact state | `10bf54111de5c0cb471ed3c96365710431f1ebae13aef85900134d5871e30955` |
| C1 count results manifest | `2d7da7a31230a846f3caa4037c9ffa8690061ebb10c557f813622a03c2bbace8` |
| C1 cold custody manifest | `6c611e6b37aeeb940525e6cfc90e1a6912604f6e8ec5b1b51ba5a2dfbed98ed9` |
| C1 GPU and cold lookup tapes, each | `6a9be270188d5e6a179bbdbeec048a087a5613821418ecb038958776095ad2a9` |
| Restored v4 finite custody manifest | `9d14ea3f7bfe7ab891b381c72b5942b159d797870c3ef473aa101bf113c82ef0` |
| Restored v4 finite manifest | `d405582d447abe4412c60366a91ff8a014324e57d4a9e3a04a8f0d945c4fc179` |
| Restored v4 GPU count executable | `44bc78aa7efd39b13670f3d0912a1bfce3090b0d3b168d6bbd08f6d96c896519` |
| Seed-one D11 trunk | `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` |
| Bound tokenizer | `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377` |
| Fit manifest | `98999a62b090218062ef61fb7faf0e7a04ef6662c71adb5bcdacbe1aa90d8ca4` |
| A1 weight file | `40af50ff9b01bc857c9f86aefcd3e1f0a1f82d7ac62573167d26e4bba5062f29` |
| Historical P1 sidecar | `075f0432ddca78ed48292104167f5056b88143ca3211954da98f04bf0dce64a9` |
| Full restored client results manifest | `a87eca99f938f41e41037a8707544b7d8b9dd1f0ed6b2fed8b1292ee1c9403a7` |
| Primary and cold CUDA client step tapes, each | `ffb4302a3677d94e156f6ad18f7639f2a8966010b0bed69cbf0f18afaff68ed9` |
| Primary and cold CUDA client receipts, each | `71c36b01b0063279fc7c4da8da93aa9dbc4d998f65aa2b7d22e56a24279eef56` |

The archived client candidate rows and mass words verified in the full restored tree. The small mirror contains summary tapes and a manifest that refers to files held in the full tree. Use the full tree to reproduce verification. Both CUDA step tapes and both receipts matched byte for byte. Both count lookup tapes also matched byte for byte. The v4 finite and pre-C1 C128 compact files and lookup tapes matched by direct comparison. D31's GO field `c128_sha256` identifies the finite compact file (`eea859b5d7272ef8d4be3a4ee623c92c2b11da866478b05765a7b20557b3788e`). D33 binds the original C128 UORT separately. D31's full-job GO manifest defect and subsequent strict restoration remain relevant to attribution. [D31 custody and limitations](../codex-31/CUSTODY.md)

The C1 input lives at `/home/callard/uor-codex/20261119-rival-admission-29/count/cr-v1/c1.tokens.u16`. The C128 input was verified at `/home/callard/uor-codex/20261120-context-swap-30/count/c128.uort`. Both GPU count receipts are under `repaired-v4/count-v4/runs/{c128,c1}/receipt.json` in the D31 root. The full device trace lives under `repaired-v4/client-v4/full/runs/count31-repaired/v4/client-real`.

## Oracle leg FIT

**FIT for the independently pinned C1 oracle on Wick.** The production executable derives tuples from original record boundaries and the predecessor position, sorts one bounded occurrence vector per order and reduces independently of the CUDA writer and project assembler. The source implements the P1 oracle function using a sorted vector with full context words and distinct predecessor type counts. Order 10 uses raw follower words under its separate adjusted hash domains. The parity computation has no training, language scoring, fit, device launch or compact-table read dependency.

| Oracle review binding | SHA-256 |
|---|---|
| Source | `e625a667e3cc9aaaae27ca9494852fcaea5ddfdc6810f7b92f3fd9b12a4218a0` |
| Cargo.toml | `d88f343d9d598e53633e8bb1b765b2efd6e7f54be02aa5414eff840dec740c03` |
| Cargo.lock | `6e3b55a7f1c539bc0ce81e13266bffdabb72f139015f30c9bbe73e5a89be3f73` |
| Release executable | `e7690f58e0a7ddbe29edd4303d584940d6c2ef7ec60822dec0bb5040e94e2373` |
| Oracle C128 tape reproduced by reviewer | `7710f5fb0b35b6a80554e194a0843e78803f4aca3e7affd8be5fd0180293a7ec` |
| Compact sorted JSON projection of all seven compared fields in 22 C128 rows | `69d9651d4f798d3dfec2dbeb8811f3de245f5437fe3766376de8ae5ea93bda01` |
| Private repeated-predecessor mutation log | `e1bb60226f27f306dd9d4d573e9880798a50f5e3345aa5f580b5e56fbc9f1a56` |

I reran the three locked offline unit tests, and independently ran the pinned release executable on Wick against the pinned C128 UORT and actual pre-C1 D31 GPU receipt. The resulting tape matched byte for byte. All seven fields of all 22 raw and adjusted order rows also match the historical and v4 finite C128 receipts. Finite controls exercised duplicate occurrences with two predecessor types, BOS preservation, exclusion of cross-record transitions, malformed token and ordinal inputs, a literal upper-bit order-10 key, separate hash domains and detection of a changed count. In an isolated Wick source copy, forcing the reducer to count repeated predecessors as distinct made the fixture fail with 3 types versus the planted 2. Its failed tape is retained under `oracle/independent-fit.lQk0Cigp` in the D33 run root.

I checked the locked dependency tree, scanned each resolved package's Rust sources for the charter's banned operation names and device backends, and inspected dynamic linkage. The sole name match was a comment in libc. This is a host executable with system libc and libgcc dependencies. Its JSON and SHA dependencies process the receipt and identity data, and the count arithmetic and comparison fields are integer. The reviewed release is under `oracle/d31` in the D33 Wick run root. Exact GO still belongs to the top session.
