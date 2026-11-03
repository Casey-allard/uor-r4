# REPORT: served-forward audit release adoption, deepseek-94

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base:
`c1b0c417819b0b0411ef551826d2563ddb60381e`.

## What did not work

- **A receipt is evidence, not a signature.** Validation binds the report to an
  artifact hash and rejects internal contradictions, but the release operator
  and repository account are still trusted. This unit adds no signing key or
  independent attestation service.
- **The synthetic Studio demo remains exempt.** It is an explicitly untrained
  protocol fixture generated during the build, not a trained model release.
- **The optional arbitrary public-URL card remains unpinned.** It does not
  copy a local artifact and retains its existing warning that it has no pinned
  hash. Models in the trained-model manifest are now gated.
- **No release was published.** The tests stage into temporary directories and
  pack a temporary fixture; they do not push a site or create a release.

## What changed

Added `scripts/validate_served_forward_receipt.py`. It independently validates
a deterministic served-forward report against either:

- a local artifact path, computing SHA-256 and byte count; or
- a remote card's pinned artifact SHA-256 and byte count.

The validator checks the complete audit schema, no timing, token-source
provenance and hashes, position/Q12/token/kept-cell workload, zero mismatches,
instrument-fault bit, equal CPU/serving decision hashes and empty mismatch
details. A supplied receipt SHA-256 is checked before the report is trusted.

The trained-model path in `scripts/stage-studio-site.sh` now requires `audit`
and `audit_sha256` on every local or Hugging Face-only entry. It validates the
receipt before copying either model bytes or tokenizer bytes. A local model also
publishes `<id>.uorlut.audit.json`, and `models.json` records its hash and URL.
A remote card records the same receipt hash while keeping the artifact remote.
After copying, the staged artifact, tokenizer and receipt are re-hashed; a
source changed during the copy fails and the partial files are removed.

`scripts/package_release_bundle_integer.py` scans every bundle file for UORLUT
magic. Each artifact now requires an adjacent `<artifact>.audit.json`; the
receipt is validated against the artifact and its workload and decision hashes
are included in the release manifest.

## Measurements

- Valid receipt: passed with 3 positions, 192 Q12 coordinates, 3 token
  decisions and 24 kept cells.
- Valid local Studio staging: copied the artifact, tokenizer and receipt, and
  wrote the audit hash and URL into `models.json`.
- Valid remote Studio card: accepted only with pinned artifact hash/byte count
  and a matching receipt.
- Missing receipt: refused before the trained artifact was copied.
- Tampered parity with a recomputed receipt hash: refused before copy.
- Artifact hash, artifact byte count, receipt hash, Q12 count, token mismatch,
  kept-cell mismatch, decision hash, instrument-fault bit, token provenance and
  mismatch-detail changes were all refused.

No model was trained or evaluated for this unit, and no release was pushed.

## Checks run at the head

- Full `scripts/tests` suite: **16 passed**.
- Python byte compilation for the validator and release packager: passed.
- `bash -n` for `stage-studio-site.sh` and `studio/build-and-publish.sh`:
  passed.
- `git diff --check`: passed.
- Local loop report and PR checks: recorded at the exact review.

## Skills used

No available Codex skill applied. The existing audit gate, Studio stager,
release packager and owner-path rules were inspected directly.

## Still open

- Decide whether the build-generated synthetic demo should also require a
  receipt.
- Replace the arbitrary public-URL slot or require a pinned hash and receipt for
  it at publish time.
- Add these checks to owner-owned workflows where automatic publication is
  enabled.
- Audit/sign the receipt itself if the trust model moves beyond the repository
  account and release operator.

## Plain summary

Trained Studio models and release-bundle UORLUT files can no longer be published
without a passing deterministic full-path audit receipt. The gate rejects
missing, altered, misbound and parity-failing receipts before copying the model.
This closes the remaining release-script adoption gap for trained local and
remote Studio entries and packaged UORLUT artifacts; it does not add receipt
signing or publish anything.
