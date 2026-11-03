# ORDER: served-forward audit release adoption, deepseek-94

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `c1b0c417819b0b0411ef551826d2563ddb60381e`.

## Goal

Close the remaining release-path gap for the exact served-forward audit. The
Studio trained-model staging path and the integer release-bundle packager must
refuse to publish a UORLUT artifact unless it carries a deterministic audit
receipt that passes independent validation against the artifact hash and byte
count.

The receipt is evidence that the CPU full path and `IntegerStackSession`
matched every Q12 query coordinate, token and kept cell. It is not GPU
qualification or a language result.

## Files in scope

- `scripts/audit_served_forward_gate.py`
- `scripts/validate_served_forward_receipt.py`
- `scripts/stage-studio-site.sh`
- `scripts/studio/build-and-publish.sh`
- `scripts/package_release_bundle_integer.py`
- `scripts/tests/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-94/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/harness/claims.json`

## Done when

1. A reusable receipt validator checks the audit schema, deterministic/no-timing
   contract, artifact SHA-256 and byte count, token-source provenance,
   position/Q12/token/kept-cell counts, zero mismatches, equal decision hashes,
   no instrument fault and no recorded mismatch detail.
2. It accepts either a local artifact path or a pinned artifact hash and byte
   count for a remote-only Studio model. When a receipt hash is supplied, the
   validator checks it before trusting the report.
3. A bundled Studio trained model must name a local `audit` receipt and its
   `audit_sha256`; validation completes before either the artifact or tokenizer
   is copied. A Hugging Face-only trained card must carry the same receipt and
   pin the artifact hash and byte count.
4. The release-bundle packager finds every UORLUT artifact under the bundle root,
   requires an adjacent `<artifact>.audit.json`, validates it against that
   artifact, and records the receipt path and hash in the release manifest.
5. Tests refuse missing, wrong-hash, changed-byte-count, tampered-counter,
   decision-hash mismatch and instrument-fault receipts. A valid receipt passes,
   and the Studio staging test proves no artifact is copied on refusal.
6. Existing gate/run tests, release-script tests, strict Python syntax checks,
   shell syntax checks and local loop policy pass. If a package-wide baseline is
   already red, record the exact failure instead of claiming it passed.
7. The REPORT records the fail-closed cases, the valid controls, the command
   checks and the remaining release paths or limitations.

## Stop if

- Validation requires re-running the model audit inside a packaging script.
- Receipt validation needs network access.
- A release path would publish before validation succeeds.
- The change requires editing owner-owned workflow files; leave the command
  change for the owner and report it.
