# Attempt 1: superseded by session identity

Attempt 1 ran implementation pin
`a463a1c8d028348fed0c95494e5282eac7c5756e`. Both public seeds already
passed complete CPU/serving content parity, but final integration audit found
that serialized sessions did not bind the attached readout.

The implementation now stores the canonical readout hash in the model and
session, emits a distinct `UR4SESS2` codec when a readout is attached, and
refuses restore into a model with a missing or different readout. Because this
changes the binary, the original run is retained here with its report,
checksums and resource ledger but does not decide the unit.

The registered rerun uses pin `fe253c5ca`.
