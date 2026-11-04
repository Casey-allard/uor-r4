# Attempt 4: superseded by serving review

Attempt 4 ran implementation pin `819482378` after the attempt-3 control-target
breach. Both public seeds passed decision-for-decision serving parity, the
seed-1 repeat matched, and the separate control target stayed below 3 GiB.

The exact-head review then found that the new serving selector allocated a
signature vector and vote map per generated decision, that the claimed
raw-step test did not invoke the attached model's raw APIs, and that the
report omitted the required serving efficiency measurement. Its complete
output tree remains under `evidence-attempt-4/`.

Attempt 5 at implementation pin `486932e24` replaces the per-token allocation
with session-owned scratch, directly tests the raw and no-readout paths, and
records sidecar/no-readout throughput.
