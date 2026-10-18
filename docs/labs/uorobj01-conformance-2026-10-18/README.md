# UOROBJ01 non-Rust conformance run  2026-10-18

The object-store backlog asked for a non-Rust UOROBJ01 verifier proof: `port_agent.py` already
implements pull/verify/audit in pure Python (hashlib/socket/struct only), but no transcript or
reproducible command existed. This delivers one, using the Rust `objpeer serve` as the peer.

## What the smoke does

`spiralcore-base/wire/tools/uorobj01_conformance_smoke.sh`:

1. builds (or uses `OBJPEER`) the Rust server, generates a 256 KiB local artifact and starts
   `objpeer serve` on loopback;
2. runs `port_agent.py audit` from an empty Python store — the protocol reports the target stale,
   and the CLI exits 3 (documented contract);
3. runs `port_agent.py pull` pinned with `--expect-sha256` and `--expect-bytes`, then `cmp`-checks
   the pulled bytes against the original;
4. runs `port_agent.py audit` again from the warmed Python store — the target is verified and the
   CLI exits 0.

No Rust client is involved in steps 2–4; only the Python agent speaks the wire protocol.

## Measured transcript (host, loopback)

    UOROBJ01_CONFORMANCE artifact_bytes=262144 sha256=696e1428d972dad3...
    UOROBJ01_CONFORMANCE cold_audit stale=1 exit=3
    PORT_PULL_OK records=215 bytes=262144 wire=275323 sha=696e1428d972dad3
    UOROBJ01_CONFORMANCE pull bytes=262144 byte_identical=true
    {"checks": 1, "verified": 1, "stale": 0, "unknown": 0, "wire_bytes": 200, ...}
    UOROBJ01_CONFORMANCE warm_audit verified=1 exit=0
    UOROBJ01_CONFORMANCE_PASS checks=1 pull_bytes=262144 byte_identical=true stale_then_verified=true

The smoke fails closed on: missing `PORT_PULL_OK`, byte mismatch, unexpected audit exit codes, wrong
JSON counts, or a server that never becomes ready. Workdir is a gitignored `local/uorobj01-conformance.*`
path removed by trap.

## Why this matters

The object-store protocol (UOROBJ01 in `crates/uor-r4-object-store/PROTOCOL.md`) is now demonstrated
against a second, non-Rust runtime rather than only the reference client. That is interoperability
evidence for the protocol and for the Python audit tooling, not a new transport or a model result.

## Limits

- Client-side conformance only: `port_agent.py` has no serve mode, so the reverse direction (Rust
  client against a Python server) is not covered.
- Loopback, 256 KiB text artifact, single run; no WAN or fleet-scale claim.
- The Python store is its own JSON record index; the audit checks closure against that index, while the
  pull itself re-hashes the assembled payload and checks the pinned SHA-256/length.
- Host-only; no pod, no model compute.

No protocol or Rust source change; the tool only drives existing interfaces.
