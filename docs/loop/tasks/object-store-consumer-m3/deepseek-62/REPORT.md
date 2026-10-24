# REPORT: object-store-consumer-m3, deepseek-62 (freeze the durable-memory panel)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2873. Base: `30a2aaa4f`. `References #2873`; item stays `ready`.

## What did not work, first

- **First pin was a double hash.** The writer printed `sha256(manifest.whole_sha256())` as "FROZEN_SHA256", which is a hash of the digest, not the artifact's SHA-256. The drift test caught it immediately: the committed file hashes to `01cb94df...` while the pin said `18e0fb17...`. Fixed by pinning `whole_sha256()` bytes directly (equal to `shasum -a 256` of the file) and keeping `Manifest::version()` as the separate pin.
- **The panel is template-generated, not hand-authored.** It is deterministic, diverse across 16 entity tuples, six sub-panels and their branches, and adequate as a frozen fixture, but it is not a linguistically rich corpus; the sub-panel composition and grading protocol are unchanged from the draft, and the sheet is still DRAFT for the FIT to judge.
- **No model or grading was run.** The bars need a selected M1/M2 artifact, a non-author FIT and Casey's release; this unit is instrument only.

## What changed

| File | Change |
|---|---|
| `docs/labs/object-store-consumer-m3/m3-durable-memory-panel-v1.jsonl` (new) | frozen canonical panel: 384 JSONL items, **179,320 bytes**; whole-object SHA-256 `01cb94df8a969450f76b00c2191548cbc0eb0fd823d9e929a1c4086853b440ad` |
| `crates/uor-r4-object-store/tests/m3_durable_memory_panel.rs` (new) | deterministic generator + one-time `#[ignore]` writer + drift test that regenerates, byte-compares, folds into `Store` and pins `Manifest::version()` = `0b0616fa...` |
| `docs/labs/object-store-consumer-m3/README.md` | panel marked **FROZEN** with file/size/SHA-256/Manifest version and the drift test; sheet status stays **DRAFT** pending M1/M2 selection, owner ratification, non-author FIT and typed release; stop condition updated |
| this folder | ORDER and REPORT |

## Measurements

| Check | Result |
|---|---|
| panel items / lines | 384 (64 per sub-panel) |
| panel bytes | 179,320 |
| `shasum -a 256` of the JSONL | `01cb94df8a969450f76b00c2191548cbc0eb0fd823d9e929a1c4086853b440ad` |
| `Manifest::version()` after fold | `0b0616fa124b689f4ca95cb049237ed974a1bee0c3d0f395f6d3a2a9eb48fbee` |
| `cargo test -p uor-r4-object-store --test m3_durable_memory_panel` | 1 passed, 1 ignored (the writer) |
| full `cargo test -p uor-r4-object-store` | all targets green |

## Checks run at the head

`cargo fmt --check` (clean after formatting), the panel target and the full object-store suite above, `loop report --check`, `loop pr-check --local`, `loop gates --full` (see closing line). No Rust source change outside the new test; no model, GPU, bar, hold or training change.

## Skills used

- `solution-integrity`: the first pin was wrong and the drift test caught it before the PR; the SHA-256 and Manifest version are separate, correctly-derived values; the panel artifact is regenerated before every suite run rather than trusted.
- `scope-guard`: one new test, one frozen data artifact under the lab record, and the lab README; no model, bar or owner-path file touched.

## Still open

- **The bars are still unrun:** they need a selected M1/M2 artifact, a non-author `TEST FITNESS: FIT` review of this frozen panel, owner ratification of the proposed thresholds, and Casey's typed release. The item stays `ready`.
- The panel is a deterministic fixture; a richer hand-authored corpus can supersede it before the FIT if the PM prefers, but it must then be re-frozen with new pins.

## Plain summary

- Froze the 384-item durable-memory panel as a canonical JSONL artifact and pinned its SHA-256 and object-store Manifest version; a finite CPU test regenerates it, proves the committed bytes match, folds it into the store and fails if either pin drifts. The pre-registration now names a frozen panel, but the sheet stays DRAFT because the bars still need a model, a FIT and the owner's release.