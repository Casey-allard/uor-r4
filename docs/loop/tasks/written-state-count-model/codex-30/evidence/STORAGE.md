# [codex] D30 exact evidence custody

The combined Wick custody preserves 164 ordinary client transcripts across D26, D29 and both cold reruns, plus 82 read-disabled D26 transcripts across the first and cold control captures. The paired transcript directories compare byte for byte. The bundle includes writer and role outputs, panel inputs and build identities, original carrier and tokenizer files, the invoked ordinary client and host helpers, source snapshots, both projection and scoring GO records, the separate count authorization and receipts, and the original CUDA transfer archive and provider logs. The failed initial read-disabled CLI capture is preserved alongside the corrected runs.

| Storage | Locator | Verified readback |
|---|---|---|
| Private Wick combined evidence | `/home/callard/uor-codex/20261120-context-swap-30/custody-v1` | Rehashed all 831 entries in `SHA256SUMS.final` and checked the file count. Rechecked the CUDA transfer digest, host device-evidence manifest, panel identity manifest and both transcript directory pairs. |
| Private Wick count artifacts | `/home/callard/uor-codex/20261120-context-swap-30/count/custody-v1` | Rehashed both expanded and compact artifact pairs, both receipts, the staged inputs, authorization, executable and cubin. Both artifact pairs and receipts compare byte for byte. |
| Private Hugging Face receipts | Dataset `caseyallard/uor-r4-store`, revision `b20ed546bcfe69c72b1e07f4cf007617c37b434c`, path `results/codex/2026-11-20-context-swap-30/receipts-v1` | Downloaded at the recorded revision and rehashed 13 files. The uploaded manifest matched byte for byte. |

The combined manifest SHA-256 is `bbfe4095b8dfcac0b5c6c091c3c08d06472f022fa1c3023ec6e49ad023f1ac18`. The receipt manifest SHA-256 is `8d8462229f6d918558b9ab65c432ae6bfe6b867b27c17bdd0748cc780500615b`. The original CUDA tar inside the combined bundle is `raw/device/remote-original.tar`, SHA-256 `40dbd5bc24d37b9c9d1c09c789a8b324061c620d345be1ceaaae8b5011919060`. The separate device copy verified all 111 original archive members after extraction. [CUDA restore and run details](../device/EVIDENCE.md) · [C128 count report](../count/REPORT.md)

The subsequent manuscript snapshot was independently verified on Wick against `REPORT.sha256`. Its report SHA-256 is `fc5a6d23999521d4a8a5882bcca6ec523497c7e770e7a85f55b6805059e599c1`.

The archived preflight panel manifest predates the role binary addition and the read-disabled CLI correction. The preserved preflight `Cargo.toml` and `client-no-read.sh` under `track/preflight-panel/` match their original digests. All recorded panel Rust sources, fixture inputs and the writer binary match the preflight manifest. The early panel README digest differs from the retained current README. The ordinary client reproduces the SHA-256 recorded in the projection GO. The original seed-one, D26 and D29 carriers and tokenizer reproduce their runtime identity receipts.

To verify the combined copy on Wick:

```sh
cd /home/callard/uor-codex/20261120-context-swap-30/custody-v1
sha256sum --quiet -c SHA256SUMS.final
sha256sum --quiet -c REPORT.sha256
diff -qr raw/ordinary raw/ordinary-cold
diff -qr raw/disabled raw/disabled-cold
```
