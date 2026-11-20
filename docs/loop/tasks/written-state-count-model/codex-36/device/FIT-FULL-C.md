# [codex] Full-C compact device readback review

TEST FITNESS: FIT

I reviewed the pruner, retention audit and CUDA reader as their non-author. I authored the upstream spool producer. Wick strictly rehashed the full staging manifests and compact candidate. The independent host audit rehashed the frozen TRAIN-derived source and candidate, checked complete retained key and follower words against the source, independently priced both thresholds, reconstructed spectra, and passed native cold load. Cascade exit is zero. The 148,096,362,864-byte original count spool has SHA-256 `1afd1c27bac4e963f4885ed275fa45c3e0a4014d3a6242f5deba82f04110ce26`. The bound TRAIN stream has SHA-256 `cd3adfe949ee34a62bc8d5e48592e091fcece796618d7910d1667d014a4bae2a`.

| Stage object | SHA-256 |
|---|---|
| Compact candidate | `064b691a54895771df1a957b2f97c91163990b6c6b6e09504c43285a97c49ef8` |
| Independent audit and identical staged witness | `16b7925328f392d085bb5297f2e93131068669b70787b8c5afc1d1a87739c0dd` |
| Source manifest | `f441692151f4bc2a48e2553ebe886e6c4eb683331078cfe6a06587e405869da1` |
| Build and input manifest | `d39f5ea8b3fb721911151193795898e2e522001c5d73e3e226958e704503007c` |
| Device executable | `9c576eba1e78807cc5fea60f55b15c5c4d1ab5c828ba882ed488b76e32b70dc3` |
| Native witness executable | `8fc728a6d110952484e128f1d9ac61859fdec0465112ca8fd784fda5b58677c1` |
| Eleven order descriptors, 528 bytes | `b26cf5251d882e804f7033720482357d12b02357730191ff9820c56f0e3e8223` |
| Eighteen original-ID probes, 432 bytes | `d994d53a0ef1af5a2e06dcba80b4167539cda287c2d59df38bd4bce0fc90b0bf` |
| Cap ledger | `ff92fec9f08db4d794090515521e85549fd50a332447a104aa5f59342e6e1c79` |

Threshold 318 occupies 3,912,334,481 bytes, leaving 601,850 bytes under the compact cap. Threshold 317 would occupy 3,914,255,830 bytes, including 8,049 additional contexts, and exceeds the cap. The immutable stage is `device-candidates/full-c` under Wick's D36 custody root. Its executables and source manifest compare byte for byte with the successful C1 replay. I independently decoded the staged packet using host integer words. Every original token ID, prefix length, and zero padding word matches the audit. All eleven descriptor cardinalities match the retained groups and followers. Orders 4–10 exercise descriptor byte addresses above 2³¹. The CUDA reader carries descriptor offsets and byte strides in unsigned 64-bit addressing, directly over the uploaded compact bytes.

Two fresh hardware processes must each run the same 18 probes, with 11 retained and seven dropped contexts, and emit 198 order records each. Require identical outputs from the two processes and a separate ordinary native comparison of every raw and adjusted target count, context total and suffix Q48 word, including causal zero padding. Require absent higher-order mass to equal the prior suffix mass on all seven dropped-context controls and require the final native raw counts, totals and masses to match the independent audit. The retained C128 finite transfer check rejected stale identity and an altered output word. That control and the C1 transfer use the same device image. Any failed identity, shape, status or word comparison leaves this artifact's device gate open.

The separate GO must bind the exact candidate and stage to Wick's RTX 4090 `GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41`, sm_89, under the idle check and shared lock in the pinned launcher. Each process streams its 3,912,334,481-byte resident payload using an 8 MiB host staging buffer. Descriptors, probes, result and status require 7,368 device bytes, so the post-context admission including 64 MiB reserve is 3,979,450,713 free bytes. The native comparison may fault about 4 GB of read-only mapped candidate pages into RAM. Record elapsed time and host RSS separately for GPU and native processes. This review admits only the finite compact readback after GO. R-only fitting, paired held evaluation and the physical fitted deployment charge remain separate.
