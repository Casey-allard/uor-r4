# Trusted object pins: whole-SHA-256 and manifest-version anchors

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-object-pin.

## What changed

- ExpectedObject is now a set of optional pins: native object length (bytes), manifest version
  (SHA-256 of the canonical UOROBJ01 manifest) and whole-object SHA-256 (the digest an OCI descriptor or
  artifact store publishes). from_manifest keeps the previous behavior; from_sha256 pins a published
  digest; with_bytes adds a length; an empty pin set is rejected.
- pull_expected rejects with BindingMismatch before any record is fetched or imported when any supplied
  pin does not match the peer manifest, and refuses an anchor with no pins at all. ExpectedObject is
  re-exported from the crate root.
- The packaged objpeer CLI exposes --expect-sha256, --expect-version and --expect-bytes; READY now also
  prints the object sha256 and manifest_version so an operator can pin a peer out of band.

## Verification

- Peer test whole_sha256_pin_accepts_the_right_object_and_rejects_another: correct pin pulls and
  rebuilds exactly; a wrong pin returns BindingMismatch with zero records imported.
- CLI test packaged_cli_enforces_a_whole_sha256_pin: correct --expect-sha256/--expect-bytes succeeds with
  byte-identical output; a wrong digest fails with BindingMismatch.
- Package suites: lib 10, objpeer_cli 3, peer 9, showcase 5 - all pass; Clippy -D warnings and fmt clean.

## Trust model

A pin proves object identity and integrity (the bytes are the pinned artifact) and fails closed before
any import. It is not transport authentication: there is no TLS or peer credential, so a pin protects
against the wrong or corrupted artifact, not against an actively malicious relay serving a different
object that collides with a pin (infeasible for SHA-256) or traffic analysis.
