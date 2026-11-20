# [codex] D35 independent FIT for CUDA TRAIN fitting

**FIT for the two pinned TRAIN-only packets on the reviewed Wick CUDA image.** Both packet and index manifests verify strictly. I checked the raw D351 words for equality across geometry after excluding only the 32 contextual words per candidate, verified that all fact-prefix contextual words agree within each row, and checked the packet's target markers against the frozen TRAIN labels in index order. Against the native TRAIN projections I checked exactly one supplied assertion per row, the exact completed gold span and at least one rival for each admitted row, the candidate spans and order, and the qualifying failure condition and recorded candidate ranges for all omissions. The indexes and omission lists are byte-identical. No HELD output or key was inspected.

Each 28,812-byte packet contains 36 admitted rows and 72 ordered gold-to-rival records. The frozen TRAIN panel contains 144 rows, and 108 were omitted by the predeclared eligibility rule. Admission retained 12 rows per connector condition but only two of six TRAIN families, with 24 from one and 12 from the other. This narrows interpretation of the learned bank. The intended output per geometry is 12 header bytes plus 72 ordered 384-byte displacement records, or 27,660 bytes.

| Wick input | SHA-256 |
|---|---|
| `train-projection/fact-prefix/packets.sha256` | `480758c76f4e0e4123d087027e66060fd45ce459c3db3d6e1b2e23ce667b8137` |
| `train-projection/fact-prefix/train.bin` | `fd166857468928486701ccabc790b30a7e39f7d1a4fc0eafff56817952dfdf59` |
| `train-projection/candidate-prefix/packets.sha256` | `79269e6dea1ede00d3cb4b825df08e3f422d963568678958cb1bbac1c37a8c05` |
| `train-projection/candidate-prefix/train.bin` | `371b740ef30d80c2a725dad9d9f876fd9b9456eec7fa984f851fa9c6b797d673` |
| Both geometry `indexes.sha256` | `17ccfd41b6fa9785fd3c2858882810099bc54f8b7fd2e62313b3c395fc0f7694` |
| Both geometry `train.omitted.json` | `e5fc543bc8fd41ba4f6cfd8d7da58e9ee043f3d4c8ade6aa6ff149478e4452e0` |
| `train-projection/go.json` | `63575672dc30524f0b9064c28c376465669c8797da13e60ef6fd6e3be9899c97` |
| `frozen/train/manifest.sha256` | `35b26571ae5dd73226f2a3828377af4919b1c8ba904c3f414a5b039bdeefe4c9` |
| `native/source-executable.sha256` | `e2af67276842812b3dff10517a4393759f9e3971b2e237e4a56cfdf85eba9601` |
| `device/aot/stage.sha256` | `c312e15c344b6b9a19e359515b388829d13ef9019f1ea6986d7d73440594c043` |
| Finite PASS `device/finite-output/results.sha256` | `635b1333c31a62a2f2d53917a065545cf45fd43affb40d83daf43c2802b47585` |

From the Wick run root, after the lab binds this receipt, these manifests and the separate fit GO to `fit-admission.sha256`, run:

```bash
bash device/aot/run-wick.sh fit train-projection fit-admission.sha256 "$fit_admission_sha" fit-output
```

Require the bound Wick lock and idle GPU, fresh result directory, identical primary and cold bank bytes, the expected byte count in both geometries and an independent arithmetic/serialization comparison before packaging the resulting banks. This FIT clears only GPU construction from the pinned TRAIN slabs.
