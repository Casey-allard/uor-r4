# REPORT: m3-nonrelation-prose-panel, deepseek-58 (pin the saved world=v2 compiler artifact in the pre-registration sheet)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2856 (folds #1718). Base: `49164086b`. `References #2856`; the item stays `ready`.

## What did not work, first

- **The artifact is not in the repository and was not in the HF store index snapshot under a compiler-looking path.** The 20 October store index does not list `sieve/compiler-save-op-v25-rawtable` or `op-model-v25`; the only sieve row is `data/hot/sieve-panel.tar` (14,571,520 B, `sieve + panel bundle`). Guessing a `sieve/` prefix would have produced a wrong pin, so the bundle was downloaded and opened before anything was written into the sheet.
- **The bundle manifests record blake3, not SHA-256.** The sheet needs the `artifact_sha256` the compiler identity carries, so the sha256 values were computed from the extracted members (and the tokenizer tar was fetched separately, because the sieve bundle does not contain the tokenizer).
- **An earlier review had already named this as the blocker.** The deepseek-41 `NOT_FIT` from the previous FIT review listed exactly two prerequisites; the bar one was resolved in that same repair, and this unit resolves the remaining artifact pin without touching the review's recorded verdict.

## What changed

| File | Change |
|---|---|
| `docs/labs/m3-nonrelation-prose-panel/preregistration-DRAFT-2026-10-22.md` | new §2b artifact pin (bundle, compiler artifact, trunk files, tokenizer tar and file, identity fields, pre-run hash gate); status line notes the pin and the fresh FIT request; Verdict gains a re-review-request subsection while the historical `NOT_FIT` record stays unchanged |
| `docs/labs/m3-nonrelation-prose-panel/README.md` | the open-items list now records the pin as done and names what is still open (fresh FIT, owner confirmation, typed release) |
| this folder | ORDER and REPORT |

## Measurements

Verified on the host from downloaded copies (no GPU, no model execution):

- Bundle `data/hot/sieve-panel.tar`: 14,571,520 B, SHA-256 `9d8da1253ebd8cdc62780afe54de418c811088b63f6dcac9ac8daf9a7b3d234a`.
- Saved compiler artifact `sieve/compiler-save-op-v25-rawtable/compiler.json`: 5,861,613 B, SHA-256 `34cdf896abf5d823e185edc269454c6ad92d97b083ad7ef8f093944931783f06`; schema `uor-r4.relation-compiler/1`; `label_schema` `m-world-v2.relations-acts/1`; internal `tokenizer_sha256` `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- Trunk `sieve/op-model-v25/model/model.safetensors`: 8,439,228 B, SHA-256 `04ce0d7a253cca6aeedbcf8d38837fed9fcf3a6b430e13f10eb098ac055db4ce`; `model/config.json`: 227 B, SHA-256 `e1009111a801518160db3b58a1c9981302ad2649fda296f3cf35e6d0af567627`.
- Tokenizer tar `data/tokenizer.json.tar`: 112,640 B, SHA-256 `2893ab5ce68aaae1624e12aa6f8b5c0b117529fc2388058645a6af018bd3d8bb`; member `tokenizer.json`: 109,457 B, SHA-256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` — **equal to the compiler's `tokenizer_sha256`**, so the pair is self-consistent.
- Cross-check: the pinned tokenizer digest `d36d3e87…` is the same tokenizer digest used by the other current sheets (e.g. `m4-lawful-likelihood-bpb`), so this is the canonical 4,096-token file, not a stale copy.

## Checks run at the head

`hf download` of both store paths, `tar -tf`/`tar -xf` member inspection, `shasum -a 256` on every pinned member, `python3` JSON field read of `compiler.json` (schema/label_schema/tokenizer_sha256), `loop report --check`, `loop pr-check --local`; a doc-only counted record is needed (D46 fresh-context path).

## Skills used

- `solution-integrity`: the pin was built from downloaded bytes and verified field-by-field (including the internal tokenizer digest matching the file digest) instead of copied from an old record; the historical NOT_FIT verdict was preserved rather than rewritten.
- `scope-guard`: only the item's lab folder and task folder changed; no compute, no release, no bar, no hold, no training.

## Still open

- **A fresh non-author FIT** of the pinned sheet (not antigravity, not deepseek) is requested. A FIT clears only the configuration in §2b and releases nothing by itself.
- **Casey's typed D40 release** (CHARTER §5.1) is still needed before the baseline GPU run; then the run records the `false_assert` and `false_overwrite` rates in `docs/research/notebook/m3-durable-memory.md`. None of that is done here.

## Plain summary

- Pinned the saved `world=v2` compiler, tokenizer and op-model trunk by file, path and SHA-256 from the private HF store, verified on the host with the compiler's own tokenizer digest matching the tokenizer file; added the pre-run hash gate and requested a fresh non-author FIT. The prior `NOT_FIT` prerequisite is resolved; the baseline GPU run still needs the FIT and Casey's typed release.