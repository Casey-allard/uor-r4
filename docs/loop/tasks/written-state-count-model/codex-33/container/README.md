# [codex] D33 native count deployment

The D33 file starts with `UORLUT01`, followed by an eight-byte little-endian JSON header length. The header declares schema `uor-r4.lut-discrete-count/1` and the byte lengths and SHA-256 digests of the original D11 trunk and original D31 P1 package. Those two exact byte spans follow immediately in that order. P1 already carries the original fit manifest, weights and compact counts. The tokenizer stays external under the original trunk SHA binding.

Build the ordinary Linux `uor-chat` with `--features p1-count-serving`. Use a separate Cargo target directory for the standalone tool in this directory. A shared build directory produced incompatible compiled identities for the count-runtime dependency across the root and standalone Cargo workspaces. The isolated builds passed.

```text
d33-count-container pack SEED.d11 STATE.p1 DEPLOYMENT.d11
d33-count-container readback DEPLOYMENT.d11
uor-chat --discrete DEPLOYMENT.d11 --tokenizer TOKENIZER.json --say "hello" --p1-trace --temperature 0
```

The normal CLI text and raw-ID paths and the readback tool all load through `discrete_d11::count_deployment::DiscreteCountModel::load`. The embedded case reports the outer file identity and both component identities. Legacy `--p1-state` continues to work with its original trunk. Providing it alongside embedded P1 state is rejected. The underlying trunk loader and existing inference operators still load and use the same integer words. The compact lookup borrows the count bytes from the outer buffer and validates the count, P1 and outer digests. This deployment was exercised in the ordinary Wick CLI. Other D11 clients and platforms were outside the run.

## Wick evidence

The first exact finite runner compared the emitted artifact path literally between sidecar and embedded runs and stopped at that field. The corrected run normalized the path and identity fields and compared all other report fields exactly. The earlier partial run remains at `container/finite-pinned`. The corrected `finite-pinned-path-normalized` run preserved all six raw-ID decision traces and both text smoke results against their sidecar runs. The raw-ID arms also matched preserved D31 decision fields and switched the enabled planted response from 44 to 45 when the source token changed. Both disabled arms emitted EOS in the original 4,095-way tie. The deployed loader rejected changes to the trunk, fit manifest, weights, counts and both outer SHA bindings, plus a truncated file and a sidecar conflict. A rehashed altered trunk passed the outer hash comparison and failed at the original fit binding. C128 ordinary readback matched the archived CUDA and independent oracle census in all 22 raw and adjusted order rows.

The real deployment is 2,316,338,762 bytes, comprising 268 envelope bytes, 86,536,132 original trunk bytes and 2,229,802,362 original P1 bytes. Both embedded slices compared byte for byte with their original files. The outer SHA-256 is `ffd0a9c5be63c8e3eb7605b455cae3d042cfc58920fd52c71bf98de6ecaa3103`. The pack took 6.80 s at 2,949,168 KiB peak RSS. Reopening it and reconstructing all canonical rows through the ordinary loader took 13.74 s at 2,949,716 KiB peak RSS. All 22 raw and adjusted C1 census rows matched both the archived CUDA receipt and the independent original-token oracle, including target counts, domains and context totals carried in the canonical digests.

Wick custody is `/mnt/plexpool/uor-r4-corpus/runs/20261120-count-identity-33/container`. Its checked manifests and summaries include the following.

| File | SHA-256 |
| --- | --- |
| `GO.json` | `97ecee06bd0c39b5df896033d691e41c01b2cf323b88d33fd5ab31662ff7bdb6` |
| `pinned-source-executable.sha256` | `28a2aa6e63bb812af34bc92165474a189590f3a08bc5e183a790bfc425a50ff5` |
| `finite-pinned.sha256` | `95e9f931ebe113115a5471b1df3ed3484d09c5e269e48be65aeffb83fd7d0bbe` |
| `pack-c1.json` | `be6fd1d5caa9eaaf21b580a49bbe7b11ba2f686bcb48d0a229f89f32a8e0b7fd` |
| `readback-c1.json` | `509e000289d897569cd475238f60f23a6b232d6902a7b1622aa7a537043768dc` |

The contract check passed. The count-enabled integer library had 728 passing tests and one ignored. The ordinary CLI had 13 passing tests with the count feature and 13 in the retained stock build.

## Client allocation correction

Review found that the first count-enabled client kept the encoded seed bytes after parsing a legacy trunk and read a supplied sidecar before rejecting a deployment that already carried counts. The revised client releases the seed buffer before loading a sidecar. When a sidecar is specified, it checks up to 4,112 bytes from the open artifact before reading its body. The full artifact is checked again before binding any sidecar. The original packer/readback executable was `19e168944a74f6b0af0198c4151d794ee3d6a17f63e78b359590e658b14eef57`. The revised-source readback executable was `ffecc99339926882418c2a33781554fdb66f7d81c8a5ed108005e31f34c7bdc2`. Both read the identical deployed bytes and emitted byte-identical C1 JSON. [Final-source GO](GO-final-readback.json) · [Final readback manifests and receipts](evidence/client-v2/)

The revised client SHA-256 is `67d33ec2a36fdc8b13f14ef26a5750ed668618bbe8c37280c1e84d31f9ae4f6d`. The checked manifest is `client-v2/source-executable.sha256` under the container custody root, SHA-256 `adf2aae2a57c48128d94a2382874da6c1ab93fa472028b34680856ce07ed233a`. Twelve finite comparisons against the first build matched entire CLI JSON outputs for base, split and embedded layouts, both read modes and both input forms. A valid finite deployment with an 8 GiB sparse sidecar and an 8 GiB sparse file with a deployment header both rejected the sidecar explicitly under a 512 MiB virtual-memory limit.
