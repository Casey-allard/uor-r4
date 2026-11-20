# [codex] Repaired complete C1 count test fitness

TEST FITNESS: FIT

I reviewed the counter repair from the separate A1 role. The loaded v3 n-gram image failed instruction admission. It emitted 11 disallowed opcodes in the histogram despite its integer source. The exact v4 source archive below replaces the runtime-derived histogram strides with the same registered launch geometry in both CUDA and Rust. The loaded-image check and the final exact C128 control still gate the full C1 run. [Count sheet](SHEET.md) · [Original independent FIT](FIT.md) · [D31 order](../ORDER.md)

The only histogram launch uses 256 blocks by 256 threads and allocates 256 by 4,096 per-block words. Every original thread therefore still visits `begin + 256 × block + thread + 65,536 × n` below the same end. These residues cover all target positions once. Bucket clearing and writing still visit each of the 4,096 slots once per block, by stride 256. The reduction sees the same block count and slot order. With the admitted ceiling of 2^31 stream tokens, even the first terminating increment lies below 2^32 and per-block totals fit u32. The change preserves order, original IDs, count support, widths, chunk bounds, target checks, and signatures. [CUDA histogram](../../../../../../crates/uor-r4-discrete/src/ngram_count.cu) · [Single bound launch](../../../../../../crates/uor-r4-discrete/src/ngram_count_device.rs)

This refresh retains the registered C128 artifact and split limits. On the exact v4 AOT build, reproduce all D30 order words, physical count bytes and lookup tape, with 12,678 checked C128 probes and byte-identical JSONL. Require zero unadmitted numerical instructions in the loaded machine code, sm_120 ELF and no fallback PTX. Fail closed before C1 on any discrepancy. C1 still compares all available independent historical raw/adjusted digest domains, the top-order raw readback, complete native compact reconstruction, all frozen GPU lookup probes, the exact artifact size and off-pod host readback. No fit outcome, held language claim or quality comparison follows from this count test. [Build and dispatch](dispatch.sh)

| Reviewed v4 source | SHA-256 |
| --- | --- |
| Source archive | `372fded6915d8ab2677df4fb7b6c5ae768bbabd2226740d559f51b52681a236d` |
| Full source manifest | `cea9ed08d8efd552545597ae458bb03da3ac00f38ce31dfbca49acf0bf391e35` |
| Count sheet | `601cf28a364de162e800a7b05ab0ecaae71ba14d9f158efa99c8aff086b8bf8e` |
| `ngram_count.cu` | `e693ef6a51d14ac4c04559aa564ec648ffb3f9b1ae8aad25c400cfd06eae8f0b` |
| `ngram_count_device.rs` | `1db2c79ae8c2fe211fa4925c8db09e20464e998e253781c1d69df0fba3dddf07` |
| `kn_score.cu` | `cb423de7a83250100c81db4443748eb197bf81717ff8bf2ac17c5e887a777fe5` |
| `kn_score_division.cuh` | `5b2eaa7f1e3844f2412674d15327174f064bb60b86883cd13a5b4133667a2d95` |
| `kn_lookup.cu` | `c4544edd0405316d8324574c336ef9ffcdda3260a5991eba684e592a0e982380` |
| `full-compact-count.rs` | `9737c83530943582c81e94c2cd93ff286e95c7ef17263554092e28ae02ea3f7f` |
| `dispatch.sh` | `8309f1480502580bca649d9f8368596f8c88e1c6004afe9d7f3002302bca6dc4` |
