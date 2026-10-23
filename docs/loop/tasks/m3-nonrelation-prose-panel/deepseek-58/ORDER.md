# ORDER: m3-nonrelation-prose-panel, deepseek-58 (pin the saved world=v2 compiler artifact in the pre-registration sheet)

Lab: deepseek. Item: [m3-nonrelation-prose-panel](../../../../roadmap/items/m3-nonrelation-prose-panel.md) (#2856; folds #1718). Base: `49164086b`. Partial unit: `References #2856`; the item stays `ready` because the baseline GPU run still needs a fresh FIT and Casey's typed release.

## Goal

The panel, frozen file and Rust grader are already on main and the owner-registered pass bar is quoted in the sheet. The only remaining `NOT_FIT` prerequisite from the deepseek-41 non-author review is that the saved `world=v2` compiler artifact is unpinned, so a FIT cannot fix the configuration it would clear. This unit pins the exact compiler artifact, its tokenizer and label schema, and the op-model trunk, by file, path and SHA-256 from the private HF store, on the host with no GPU, and re-requests FIT.

## Cancel probe (run first)

Host-only docs unit. At the base, `docs/labs/m3-nonrelation-prose-panel/preregistration-DRAFT-2026-10-22.md` names no compiler artifact path or hash (grep for `compiler.json`/`artifact_sha256` in the sheet returns nothing), while the review at the sheet's Verdict names that as prerequisite 2. Task stands.

## Files in scope

- `docs/labs/m3-nonrelation-prose-panel/**`, this task folder. Nothing else.

## Done when

- The sheet names: the HF-store bundle and its byte size + SHA-256; the saved `compiler.json` path/member, byte size, SHA-256 (`artifact_sha256`), `label_schema` and tokenizer digest; the tokenizer tar + `tokenizer.json` byte sizes and SHA-256; the op-model trunk `model.safetensors` and `config.json` byte sizes and SHA-256; and the pre-run hash gate.
- Every hash was verified on the host from a downloaded copy (no GPU, no model execution) and the compiler's internal `tokenizer_sha256` equals the tokenizer file's SHA-256.
- A fresh non-author FIT is requested in the sheet and on the issue; the historical `NOT_FIT` record is preserved; no compute is cleared and no bar/status/hold is changed.
- `loop report --check`, `loop pr-check --local` pass; a counted record (doc-only path) lands it.

## Stop and ask if

- The artifact cannot be located or a re-downloaded file does not match a recorded digest (then report the mismatch, do not invent a pin); any GPU run or owner release would be needed (that is not this unit).