# Attempt 1: stopped at the registered preflight

Attempt 1 ran implementation pin
`99a0687f3a4a7cf87cc5bdaff07ce6330d5d3968` after FIT and fresh GO. The
release build and all controls passed.

The 4,096-bit trajectory signature reduced leave-one-out ambiguity from
**277 to 254** rows, but its top-1 fell from **300 to 297** against the
existing code readout. The registered preflight required trajectory top-1 not
to fall below code, so the run stopped before the seed-2 preflight, free
generation or the one-worker repeat.

The exact output and structured bar failure are under
[`evidence-attempt-1/`](evidence-attempt-1/). No configuration was retuned
after observing the result.
