# REPORT: CPU copy-key-2 parity, deepseek-87

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `1499c09ed`.

## What did not work, first

- Before this unit, the integer artifact format accepted copy key 2 but the
  independent CPU Hamming port still built every query and key from one token.
  The served-forward audit could therefore compare against the wrong CPU read
  without refusing the artifact.
- No readable trained copy-key-2 artifact is available on this host. The exact
  parity evidence uses the shared synthetic FBS/Hamming fixture.
- This proves execution parity, not learning, complete replies or language
  capability.

## What changed

The CPU `CopyHeads` implementation now records whether the artifact names copy
key 1 or 2 and applies the serving half order:

- query at `t`: `x_t`'s first half and `x_{t-1}`'s second half;
- key at `s`: `x_{s-1}`'s first half and `x_{s-2}`'s second half;
- row 0 and key position 1 reuse `x_0` for the missing prior token, matching
  the integer session.

Copy key 1 retains its existing one-token expression. The served-forward
manifest now reports `copy_key` explicitly.

## Measurements

- Full CPU all-target suite: **131 library tests passed, 4 ignored**; all
  driver and auxiliary binaries passed.
- `copy_key_two_heads_match_serving_at_both_hooks`: exact Q12 query, token and
  kept-cell parity over a 16-position synthetic copy-key-2 artifact.
- `copy_key_two_tells_two_facts_after_the_same_token_apart`: key 1 ties the
  two facts at distance 0; key 2 keeps the ordered `(C, X)` fact and moves the
  `(B, X)` fact to a positive distance.
- `copy_key_two_audit_matches_serving_exactly`: the public served-forward audit
  records `copy_key=2` and reports zero query, token or kept-cell mismatches.
- Existing copy-key-1 tests remain green.

## Checks run at the head

- `cargo test --locked -p uor-r4-cpu-train --all-targets --features
  uor-r4-integer/serving-wasm,uor-r4-integer/fbs-synthetic`: passed.
- Strict package Clippy with `--all-targets --no-deps -- -D warnings`: passed.
- Package formatting and `git diff --check`: passed.

## Skills used

No available Codex skill applied. The merged integer copy-key implementation,
CPU Hamming port, synthetic artifact builder and served-forward audit were
inspected directly.

## Still open

- A real trained copy-key-2 artifact still needs exact CPU/serving parity when
  the private HF export becomes readable locally.
- Complete replies, fresh phrasing and useful local chat remain `NOT_RUN`.
- The discrete K1/K4 corpus comparison is still the next materially new CPU
  training unit.

## Plain summary

The serving format learned to distinguish two-token Hamming keys, but the CPU
audit still used the old one-token key. That mismatch could make an audit
meaningless. The CPU now uses the same ordered pair of tokens as serving, and
three tests show the query vectors, tokens and kept cells agree. This closes
the portability gap; it does not show that such a model talks better.
