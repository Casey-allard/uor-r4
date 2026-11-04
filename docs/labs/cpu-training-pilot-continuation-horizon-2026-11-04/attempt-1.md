# Attempt 1: stopped at the registered preflight

Attempt 1 ran implementation pin
`0a8631d4e033af39025b06c64fc50496782269f4` after FIT and fresh GO. The
release build and all focused controls passed.

The seed-1 horizon regressor learned more than its shuffled control: true
horizon absolute error was **7,184** against **9,525**, and it recalled
**40/84** terminal rows. It also emitted **66/1,484** nonterminal false stops,
far above the registered **14**-row limit. The run therefore stopped before
the seed-2 preflight, free generation or the one-worker repeat.

This is a preflight stop, not a language result. The exact output and
structured bar failure are under
[`evidence-attempt-1/`](evidence-attempt-1/). No configuration was retuned
after observing the result.
