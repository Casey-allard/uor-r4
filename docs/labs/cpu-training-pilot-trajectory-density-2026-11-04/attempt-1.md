# Attempt 1: stopped at the registered preflight

Attempt 1 ran implementation pin `1bf1a72fa` after fresh FIT and GO. The
release build, controls, Clippy, rustfmt and input checks passed.

The combined 4,096-bit termination gate retained all **1,568** generated
training rows, including **84** terminal and **1,484** nonterminal decisions.
It selected two neighbours at offset radius two and recalled **16/84** terminal
rows while stopping **11/1,484** nonterminal rows.

The shuffled-label gate selected its own threshold, produced a different gate
hash, changed **58** leave-one-out firing decisions, recalled **18/84** terminal
rows and stopped **13/1,484** nonterminal rows. The true and shuffled
ungated-content audit covered **1,145** decisions with zero mismatches.

The registered terminal-recall floor was 26/84, so the run stopped before seed
2, complete generation and the one-worker repeat. False stops remained inside
the 14-row limit.

Exact output is in [`evidence-attempt-1/`](evidence-attempt-1/); its original
sealed `SHA256SUMS` is
`f56a5dc8be880189fb71776cac9f9979c6a7a0a6daa4a1c249230bcfa6c90467`.
