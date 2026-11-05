# Attempt 1: stopped at the registered preflight

Attempt 1 ran implementation pin
`c128b6b62` after FIT and fresh GO. The release build and controls passed.

The phase-transition density gate retained all **1,568** generated training
rows, including **84** terminal and **1,484** nonterminal decisions. It selected
two neighbours at offset radius two and recalled **14/84** terminal rows while
stopping **7/1,484** nonterminal rows.

The registered terminal-recall floor was 26/84, so the run stopped before seed
2, complete generation or the one-worker repeat. The shuffled-label control
recalled **16/84** with **11/1,484** false stops and changed **48** leave-one-out
firing decisions, so the control was non-vacuous.

Exact output is in
[`evidence-attempt-1/`](evidence-attempt-1/); its `SHA256SUMS` is
`5b5af4570dd10bc1935bd52e70da80611b3de9a18674b05d9f9a99a123a97859`.
