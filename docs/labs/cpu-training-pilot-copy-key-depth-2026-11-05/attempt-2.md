# Attempt 2: stopped on the public-DEV manifest path

Attempt 2 used the corrected token and mask names, but the common runner still
passed `manifest.json` for public DEV rather than `dev-manifest.json`. It
stopped before writing any target reading.

The zero-byte `dev.json` and published attempt output are in
[`evidence-attempt-2/`](evidence-attempt-2/). The raw manifest, preserved as
`RAW-SHA256SUMS`, has digest
`ed23906414c6420e1162d3497ef60819f029d7cb06da07f124b14b9e4acfe833`.
The published subset verifies with `PUBLISHED-SHA256SUMS`.
