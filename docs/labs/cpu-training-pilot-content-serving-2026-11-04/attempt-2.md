# Attempt 2: superseded by loopback identity

Attempt 2 ran implementation pin
`fe253c5ca89647264fb3e72cd38a962f1259a6d5`. Both public seeds passed
complete CPU/serving content parity, including canonical readout identity in
session snapshots, but exact-head delivery review found that OpenAI loopback
JSON omitted the sidecar path and file SHA-256 required by the order.

The implementation now threads those values through health, model, capability,
generation and final-stream JSON. A real CLI `--content-readout --serve`
launch and all associated loopback surfaces are covered by tests.

The attempt-2 report, sidecar evidence, checksums and resource measurements
are retained under `evidence-attempt-2/` and do not decide the unit. The
ledger overwritten by the later deciding run is restored byte-for-byte as
[`attempt-2-resource-ledger.json`](attempt-2-resource-ledger.json) from blob
`215c329ce`, the corrected pre-overwrite version. The registered rerun uses
pin `1be885d9d`.
