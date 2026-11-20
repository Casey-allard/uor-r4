# [codex] Second CUDA source-build replay custody

This supplement preserves the exact 9,369,600-byte second-pod archive `device-rebuild.tar`, SHA-256 `eefef05d072f881832c063882dac0fd513f00978d73eff9dec345e8251390fa3`. I extracted the retained archive, rehashed the build and result manifests, checked that GO binds the FIT and build seal, and compared all 12 rebuilt response files directly to their original expected bytes. The 18 input and expectation files also match their corresponding members of the [earlier D29 custody packet](STORAGE.md). This repeats the bounded D29 diagnostic on the newly built SM89 executables. The pod down receipt records deletion of instance `vast-61553201`.

| Storage | Locator | Verified contents |
|---|---|---|
| Wick | `/home/callard/uor-codex/20261119-rival-admission-29/rebuild-custody-v1` | The exact GPU tar is under `raw/`. Wick rehashed all 25 originals and 22 delivery files. |
| Private `caseyallard/uor-r4-store` | Revision `031bc7b94a27c830fc4d955de49229005dcee162`, path `results/codex/2026-11-19-rival-admission-29/rebuild-custody-v1` | Downloaded and rehashed all 22 delivery files. Decoded the five plain-text chunks, restored the exact GPU tar and rehashed all 25 original files. |

The original build manifest is `a626c321ce2c0f6c1a9439c7c50132be98d8b2d4ba8d48cef6a02cea42c177b2`. The original result manifest is `e6d55d3b97c9e8df00edb0c09f03f86caf793189a50da9f39215ed4b4d268489`. The delivery and original-file manifests are `3a16c52edc1475e1132c645f6f85850996c2f1b424939f992f675c54c222e678` and `ae246dc4c3e315e77335272817fe860a506561ce0744f3b7acf20d5ecbe7fc82`, respectively. The 416,932-byte reversible transport hashes to `9e01980fbcdf3bf2fd70ab6c0d798d315742c8ea8a894a71d31294b290146f80`. `TRANSPORT.md` describes exact restoration. The delivery includes the original build and result manifests, GO and [FIT for replay](../FIT-REPLAY.md), the pod disposition, and the independent verification receipts.
