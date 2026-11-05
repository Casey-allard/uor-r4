# Attempt 1: stopped on the public-DEV input path

The release screen binary built and both new panels were generated. The
public-DEV command then named `tokens.u16` and `response_mask.u8`; that corpus
uses `dev-tokens.u16` and `dev-response-mask.u8`, so the process exited before
writing any target reading.

The zero-byte `dev.json` and published attempt output are in
[`evidence-attempt-1/`](evidence-attempt-1/). The raw manifest, preserved as
`RAW-SHA256SUMS`, has digest
`222d12634ac2c8d9f9bcda3edee332e1503f2d477737a03a4c2a234cd9667572`.
The published subset verifies with `PUBLISHED-SHA256SUMS`.
