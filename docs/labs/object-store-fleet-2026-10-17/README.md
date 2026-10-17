# Fleet verify and multi-object manifest selection

Date: 2026-10-17. Lab: deepseek. Branch: deepseek/object-store-fleet.

## What changed

- Protocol: the MANIFEST_REQUEST body is now an optional 32-byte selector (whole-object SHA-256 or
  manifest version). Empty keeps the single-object behavior. An unknown selector terminates the
  session with the new Error::UnknownObject after a kind-25 error frame. Fetch scoping stays per
  selected object, so a session can only read that object's closure.
- API: serve delegates to serve_many(store, &[Manifest], ...); pull_expected delegates to
  pull_selected(..., selector); new verify(store, expected, selector, ...) returns VerifyStats
  (missing nodes, verified) without fetching records or rebuilding the payload. verify is the fleet
  current-check: pinned manifest + complete local closure, no bytes transferred. A pin is checked
  before counting missing nodes. All three are re-exported from the crate root.
- CLI: serve accepts repeated --object and hosts all of them from one store, printing objects= and an
  OBJECT line (bytes, roots, sha256, manifest_version) per object; pull and the new verify accept
  --select-sha256; verify prints missing/verified/wire_bytes and exits 3 when the store is not current.

## Verification

- Peer suite 10/10: multi-object server verifies a present object, selects and pulls a second object by
  SHA, and rejects an unknown selector with UnknownObject; existing single-object behavior unchanged.
- objpeer_cli 4/4: fleet verify with a selector succeeds with verified=true; an unknown selector fails
  with UnknownObject.
- controls 10/10, showcase 5/5, doc-tests 0; Clippy -D warnings and fmt clean.
- Backward compatibility: empty manifest-request body keeps the original contract, so the review-repo
  sync client (which sends an empty request) still interoperates with the native server.

## Why it matters

A single store and a single serve process can answer whether any of its objects is current by selector,
without transferring payload. This is the primitive for fleet verification, move/revert checks and
artifact registries that share one content-addressed store. The wire cost per check is the manifest
plus framing (about 200 bytes), independent of object size.
