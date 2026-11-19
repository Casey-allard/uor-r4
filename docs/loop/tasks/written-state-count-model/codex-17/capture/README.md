# [codex] Label-blind legacy candidate capture

This standalone Rust adapter opens the base and role D11 through the existing loader. It binds the actual packed 4-block codebook, tokenizer and legacy span-state word classes. It joins the frozen codex-15 chain's per-row blind selection, endpoint, occurrence list, raw 13-word CUDA assertion witness and chosen-writer receipt using original token addresses and SHA-256 digests. It reuses the codex-17 candidate inventory and replays only the unchanged deterministic cursor using the archived ordinary token IDs. It performs no model forward call.

## Run

```sh
cargo test --locked --offline
cargo run --locked --offline --quiet -- synthetic
cargo run --locked --offline -- \
  BASE_D11 PAIRED_D11 FROZEN_SPAN_STATE BLIND_JSON PUBLIC_SELECTION CHAIN_DIR NEW_DIR
```

Keep the input files under a verified immutable release. For the already-public seed-1 integration fixture, use the following files under the original local Codex root.

- Base D11 `public-m1-predictor/seed1.d11`
- Role D11 `ordered-role-transfer-20261118/assets/seed1-paired.d11`
- Legacy classes `span-connectors-20261118/results/span-01/seed1-state.json`
- From `fresh-boundary-validity-20261119/raw/primary-seed1`, use `blind/held-blind.json`, `public.json` or `no-read.json`, and `chain/`

Verify the custody receipt and cold replay before invoking the adapter. The command accepts only unchanged `public` and `no-read` selection arms. Grade-derived oracle, forced source, bounded override and label files have no entry point. A blinded requested-span receipt can be added later with an independently pinned schema.

## Outputs

| File | Purpose |
|---|---|
| `candidates.json` | Label-free rows with original ordinal, prefix digest, original tape length and half-open requested and excluded role spans. Each retained assertion has its original half-open fact span, first delimiter position, both masks and a list of every legacy intersection candidate. Cap-rejected candidates remain present with `writer_reachable=false`. Candidates carry absolute start and first-word end, slot lengths and classes, original token IDs, packed u16 roots and little-endian root bytes. Other writer-reachable starts only appear in `outside_legacy_inventory`. |
| `archive-audit.json` | Auditing lane for the old source, raw role and writer words, the capture hash and all source file digests. Keep this file separate from scoring and fitting inputs. |
| `packed-codebook.u16le` and `legacy-word-classes.u8` | Exact original vocabulary order. Roots use the D11 loader's low-nibble-first packing and little-endian u16 bytes. The class table comes from the actual span-state file pinned by the CUDA assertion tape. |

The native Linux bridge can flatten candidate rows using `(ordinal, assertion_index)` plus the immutable capture digest. It should preserve original offsets and report inactive cap hits without giving them a score. A future training-only join can bind separately sealed labels to this exact key. Any target used in a fit must match a reachable original source. Semantic ambiguity, no support and held grading require their own sealed records. Labels, old chosen sources and writer fields stay outside the blind descriptor stream.

## Checks and remaining dependency

The four synthetic tests cover raw original-role and fact reversal, forged fact end, shifted fence, no source despite another writable offset, exact packed bytes, original legacy versus extended starts and an inactive cap hit. The synthetic JSON repeated byte-for-byte.

The already-public seed-1 archive check validated 32 public writer traces, 24 retained assertions and 49 legacy candidate addresses. All 49 reach their shared assertion delimiters. The separate disabled arm reproduced 32 archived writer traces with no requested span, assertion or copy source. Every consumed archive file matched the cold repeat byte-for-byte. No label or grade file appears in either adapter input audit. [Archive checksums](archive.sha256) · [Synthetic packet](synthetic.json) · [Finite tests](tests.log)

The old CUDA writer receipt covers the archived selected source. Other candidate reach uses the same Rust cursor and first delimiter as a structural check. Full AOT CUDA replay and subsequent fit and held collection are separate work. The adapter has no access to v4 TRAIN or HELD inputs.
