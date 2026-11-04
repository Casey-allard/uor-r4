# Attempt 1: stopped at registered preflight

Attempt 1 ran implementation source
`c8180c1576970069dcff395889460defad2a2d6c` after the registered FIT
and GO reviews. The release build, focused controls, source identity and
five public inputs passed.

The seed-1 preflight recalled only 19/84 generated terminal rows against the
registered 26/84 floor. Its class-preserving shuffled control also recalled
19/84. Nonterminal false stops remained below the 14-row limit at 9/1,484
true and 11/1,484 shuffled. Because the preflight is a stop rule, the pipeline
did not run the seed-2 preflight, public generation, or the worker-count
repeat.

The complete output is retained in
[`evidence-attempt-1/`](evidence-attempt-1/) with `SHA256SUMS` and the five
input hashes. Measured local compute is recorded in the
[resource ledger](resource-ledger.json). Nothing was retuned after observing
the result and the 26/84 floor was not changed.
