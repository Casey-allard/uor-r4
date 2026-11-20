# [codex] Active S2 count control within the shared byte cap

The lossless full-table codec measured in [D26](../../codex-26/count/README.md) remains 64,839,481 bytes over the cap and uses a standalone reader. This deployment keeps a fixed subset of whole context groups from the same 1%-TRAIN table and runs on the existing opt-in ordinary client. It drops 12,661,248 rows, and on the first token of the map prompt it backs off from D23's order seven to order two. The same ID wins this example under both artifacts. Predictive effects of pruning remain unmeasured.

The complete active D11 is **125,998,700 bytes**, eight bytes below the **125,998,708-byte** cap. The package carries **1,973,111 records in 1,509,774 complete groups**, including all 570,778 rows of orders zero through two. Every retained group keeps its original coordinates, successor IDs and count ranks, with no numerical conversion, and orders 3–10 each have their own frozen byte allowance. The public M1 seed-1 trunk payload and original descriptors compare byte for byte and structurally, respectively, after separating the S2 header bindings. The original 96-byte provenance span is copied unchanged. [Rule frozen before serving](REGISTRATION.md) · [Per-order census and output hashes](deployment.json)

| Component | Bytes |
|---|---:|
| D11 file header and alignment | 3,136 |
| Exact M1 trunk payload | 86,533,188 |
| Trunk end alignment | 60 |
| Count provenance | 96 |
| Selected count records | 39,462,220 |
| **Total** | **125,998,700** |

Each export reopened the finished file and compared it to the in-memory package built by copying the selected source spans. Two independent processes produced byte-identical D11 files and receipts. Repackaging the D23 planted carrier kept all fourteen rows and reproduced the original fixture file byte for byte. The ordinary text client emitted red `[677,1]` and blue `[924,1]` and reported EOS for both. The routes switched from order ten at the first token to order two at EOS. Text and raw-ID paths agreed on prompt IDs, generated IDs, Q32 masses and all recorded S2 route, count and top-eight fields. Raw-ID JSON omits the text path's stop field.

The real TRAIN-derived artifact loaded in the same ordinary client. “What is a map?” selected order two, with counts 256, 128, 256 and 4,096 for IDs 201, 223, 287 and 799. It emitted 799 (“ We”) with Q32 mass 3,598,518,889 and reached the one-token cap. A second process reproduced the complete JSON byte for byte. Replaying its framing IDs reproduced every integer field. The planted examples are format and serving conformance checks. The real prompt is a finite diagnostic with no prescribed gold token. [Ordinary client observations](client.json)

This establishes one active count control at the shared cap and on the seed-1 trunk used by the relation work. P2 still requires four active comparators, a common deployable byte ledger for their necessary state, GPU differential replay after pruning and a registered paired sealed DEV-test score. The latency measurement and P1 mixture also remain open. No training, GPU dispatch or panel scoring took place in this unit.

## Reproduce and verify identities

From the repository root, set `S` to the Codex scratch root and use a fresh directory `R` inside it. Compile the packer offline. Its only model inputs are the pinned D23 source and public M1 base. The D23 device provenance and custody appear in its storage record. [D23 custody](../../codex-23/evidence/storage.json)

```sh
unit=docs/loop/tasks/written-state-count-model/codex-27/count
CARGO_TARGET_DIR="$R/packer" cargo build --locked --offline --release --manifest-path "$unit/Cargo.toml"
CARGO_TARGET_DIR="$R/client" cargo build --locked --offline --release \
  -p uor-r4-integer --features serving-wasm --bin uor-chat
pack="$R/packer/release/budget-s2-pack"
chat="$R/client/release/uor-chat"
base="$S/public-m1-predictor/seed1.d11"
tok="$S/public-m1-predictor/tokenizer.json"
source="$S/active-s2-23/real/active.d11"
fixture="$S/active-s2-23/run-1/validation/fixture-active.d11"
"$pack" "$base" "$source" "$R/active.d11" > "$R/deployment.json"
"$pack" "$base" "$source" "$R/repeat.d11" > "$R/repeat.json"
cmp "$R/active.d11" "$R/repeat.d11"
cmp "$R/deployment.json" "$R/repeat.json"
cmp "$R/deployment.json" "$unit/deployment.json"
"$pack" "$base" "$fixture" "$R/fixture.d11" > "$R/fixture.json"
cmp "$R/fixture.d11" "$fixture"
"$chat" --discrete "$R/active.d11" --tokenizer "$tok" --say 'What is a map?' --max-tokens 1
"$chat" --discrete "$R/fixture.d11" --tokenizer "$tok" --say 'red saw the oak' --max-tokens 2
"$chat" --discrete "$R/fixture.d11" --tokenizer "$tok" --say 'blue saw the oak' --max-tokens 2
```

The packer ran using Rust 1.97.1 (8bab26f4f) on arm64 macOS. The primary client executable was the local D26 release build under `pre-gate-source-26/target-views/release/uor-chat`, SHA-256 `19b00209b5368e95ec0cc327f4e29813078316f84991233533273cc4c4993ca9`. The raw client responses and exported carriers are in local custody under `query-assertion-27/count-run`. The committed response receipt removes only the local artifact path fields.

A fresh release build of the ordinary client from the D27 worktree, SHA-256 `5d1aaec765c3e2c0f965bd418da85189d9125c0b07024b22e3c4df68376d7f85`, reproduced all three ordinary-text JSON outputs byte for byte. The installed harness reported zero selected rules for the native-space filter, so that invocation confers no check result.

The [follow-up finite GPU differential](GPU.md) records device replay on the same capped table and the original planted controls.
