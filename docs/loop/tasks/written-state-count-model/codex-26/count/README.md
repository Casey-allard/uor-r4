# [codex] Retained order-ten count packaging audit

The measured codec exceeds the **125,998,708-byte cap** by **64,839,481 bytes**. The exact seed-1 D11 base and the independently readable sidecar occupy **190,838,189 bytes** combined. The existing ordinary client reads the original D23 S2 /2 section. The packed format runs through the standalone audit reader only. This scope was a host serialization check on frozen state, with no training, predictor execution, GPU, panel scoring or access to DEV-test.

| Measured component | Bytes |
|---|---:|
| Actual seed-1 Relation D11 base | 86,536,132 |
| Packed header, including 96 original provenance bytes and 12 page-range bounds | 152 |
| On-disk page directory, 98,083 entries × 20 bytes | 1,961,660 |
| Packed page bodies, including lengths and bit padding | 102,340,245 |
| **Actual packed sidecar** | **104,302,057** |
| **Total** | **190,838,189** |

The source D23 S2 /2 carrier occupies 379,223,660 bytes, including a final 292,687,276-byte section with 14,634,359 ordered 20-byte records. Its declared span starts at payload-relative byte 86,533,248, or file byte 86,536,384. Keeping that exact active D11 prefix instead of the standalone seed-1 base and replacing the trailing section would yield 190,838,441 bytes before rewriting the section descriptor or accounting for a new format identifier. The measured total charges two actual files. Future integration into a single D11 requires a separately measured reader, descriptor and deployment envelope.

## Format and readback

Separate order ranges preserve the order digit and restore the discarded zero padding exactly. Within each order, pages restart every 128 distinct contexts and never split a context; a context has at most four successors. Each directory entry stores the first normalized key as a little-endian 128-bit word and a little-endian 32-bit page offset. Each page begins with a 16-bit little-endian key-stream length. An unsigned little-endian base-128 stream stores each normalized key delta with two low bits for successor count minus one. A second stream stores each original 12-bit successor ID and its 5-bit rank as one 17-bit word, least-significant bits first. The rank continues to mean the declared count 2^rank. A predecessor query binary-searches one order's restart directory and traverses at most one page, or at most 128 context groups and 512 successor words.

After file closure, the audit reconstructed every original ordered tuple, with 217,676 additional predecessor/exact queries against streamed boundaries or binary search in the original flat rows. The latter included 1,024 order-ten pairs sharing the nine most recent tokens. All 1,024 original contexts hit, and changing the oldest token alone produced 1,024 exact misses. The malformed checks rejected a short header, a truncated last page, a nonzero restart delta, a zero rank and an invalid initial page offset. These are byte and query checks. Runtime throughput, device parity and client loading are unmeasured. Two release builds with the same encoder independently produced byte-identical output. [Per-order audit log](audit.txt)

This page tests one layout, with no pruning or held-key selection. The cap leaves 39,462,576 bytes above the base. The 17-bit successor words alone require at least 31,098,013 bytes before page padding, leaving at most 8,364,563 bytes for all context keys, indexes, provenance and any further predictor state. The full sidecar remains above cap before adding any S5 or S6 state.

## Live #4107 evidence and prerequisites

The 100% run2 source JSON at PR head `67774541fcf5f7d65e0b8e78151a2d22d7cad3a0` reports order ten, reserved TRAIN fitting, and 10,807,417 DEV-tune targets over 21,928,475 bytes. Its overall BPB is 1.432620 for pure count (A0), 1.360806 for geometric Pick (A1), 1.432607 for the read-disabled control (C1), 1.365580 for exact token pairs (C2), and 1.360932 for permuted codes (C3). The registered result is `geometric_gain`. The A1−C3 paired band is [−232, −24] micro-BPB overall and [−319, −37] on novel text. The copy-positive B2 check passed. This resolves the stale first-run status in codex-22 for the S5 instrument on DEV-tune. [Run2 source JSON](https://github.com/UOR-Foundation/uor-r4/blob/67774541fcf5f7d65e0b8e78151a2d22d7cad3a0/docs/loop/tasks/written-state-count-model/claude-439/run2/pick-p1000.json) · [PR report](https://github.com/UOR-Foundation/uor-r4/blob/67774541fcf5f7d65e0b8e78151a2d22d7cad3a0/docs/loop/tasks/written-state-count-model/claude-439/REPORT.md)

P1 still needs the frozen deployed order-ten mixture and registered same-target comparison against P0, with the dialogue comparison measured separately. P2 still needs its four active readers, a common feasible D11 byte ledger including each reader's real indexes and needed state, and the registered paired, sealed DEV-test comparison. Neither gate was exercised by this audit.

## Reproduction and custody

Built on arm64 with `rustc 1.97.1 (8bab26f4f 2026-07-14)`. From this directory, verify the source SHA-256 and use a fresh output path.

```sh
Q=/Users/casey.allard/uor-r4/local/codex/pre-gate-source-26/count
SOURCE=/Users/casey.allard/uor-r4/local/codex/active-s2-23/real/active.d11
shasum -a 256 "$SOURCE"
rustc --edition 2021 -O -D warnings main.rs -o "$Q/s2-pack-audit"
"$Q/s2-pack-audit" "$SOURCE" "$Q/new.s2p"
```

The tool expects the pinned source fixture at the offsets listed above. The binary files remain in local custody.

| Object | SHA-256 |
|---|---|
| `active-s2-23/real/active.d11` | `6da3ac9fcfd8456f803284042f2eece9eca9d471e57affbe2cb3e1b726745be7` |
| Original trailing source section (provenance and all tuples) | `d5c08d40e1a827ed0e1f1f6383e6c7c6aa57666468fd3fe59c272cfef4e353c1` |
| `public-m1-predictor/seed1.d11` | `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` |
| `run2/packed.s2p` | `0874e970d7da2176bdce891985189011cf140646175017dc2dcefb87061ce608` |
| `run2/s2-pack-audit` | `03d941767ce498919ce1410cd2d04910a5ac93fb9c1bb09590a566824588ce16` |
| GitHub #4107 `run2/pick-p1000.json` at cited head | `28d06be01c22439fb715d6ba516845c8ee5a0530c0b0c09ec9d062461d4a4a53` |
