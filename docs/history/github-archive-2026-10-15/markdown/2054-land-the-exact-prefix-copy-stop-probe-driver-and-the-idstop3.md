# #2054 — Land the exact-prefix copy-stop probe driver and the idstop3 non-firing classification

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T16:44:00Z
- **Updated:** 2026-10-09T16:46:37Z
- **Closed:** 2026-10-09T16:45:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2054
- **Merged:** 2026-10-09T16:45:13Z
- **Merge commit:** ff0008e2600efd66579aea76c19676415f3d9b24
- **Base / Head:** main / UOR-Foundation:deepseek/idstop3
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Lands the measurement instrument and the measured negative from the idstop3 round. **No serving-time behaviour changes**: the only non-documentation file is a new example.

## What this adds

1. `crates/uor-r4-training/examples/bind-probe-exact.rs` — the exact-prefix binding probe driver that produced the 72/84 token-identity-stop result (`docs/evidence/token_identity_stop_breakthrough_2026-10-09.txt`). It existed only on the pod network volume (`bindprobe/idstop/`), so the current best result could not be reproduced from `main`. It is a plain example: cargo discovers it (no `Cargo.toml` entry), and candle-core's non-cuda stub lets it build without `--features cuda`, exactly like the existing `binding-probe` bin and `geometric-stack`/`m-world` examples. The mechanisms it drives (`CopyStopMode::Identity`, `CopyStop::identity`, `StackModel::set_pointer_copy_stop` / `set_pointer_copy_trace` / `set_pointer_gate_floor`) are already on `main`.
2. `docs/evidence/idstop_nonfiring_classification_2026-10-09.txt` — the classification, its reason breakdown, the default-unchanged proof and the exact build/run commands a successor needs to re-run it.

## Measured result (arm-ptr, gate floor 0.5, identity stop, 84 cells, trace channel added)

2x2: fires+exact **20**, fires+not exact **3**, no fire+exact **52**, no fire+not exact **9**; exact 72/84 = 0.8571, Wilson [0.7667, 0.9163].

The question was whether the 9 non-firing cells are recoverable by widening the strict "open". **They are not, for two independent measured reasons:**

- The population a wider open would target is **EMPTY (0/84)**: in all 79 cells that emitted the value's first token, the pointer's argmax source at that step held exactly that id. The rule can open at any step, not only the first, so there is nothing for a wider window to add.
- Recoverability by any rule that only decides where the reply ends is **0/9**: 4 of the 9 never emit the value at all (EOS after a template fragment), and the 5 that do emit it end on the value with the template text `" My name is "` **before** it — right value, right termination, wrong prefix. No prefix truncation of any of the 12 non-exact replies normalises to the expected value.

Supporting measurements: 61 of 84 cells end by EOS while a copied run is still alive (the rule only acts on a break); the strict rule's own template firing here is 1 of 23 firings. Default unchanged: 9 files / 288 records re-run with the rule off at floor 0 and compared field-by-field with the sealed references — 0 differing fields, 0 missing, 0 added records. The parser self-check reproduces arm-ptr 35/84 with channels 0.5149/0.8351/0.5610/0.4901, arm-a-w05 10/84, arm-c-ctrl 24/84. An offline replay of `CopyStopRun` (EOS/short-cycle stops first, the dropped id restored) reproduces the recorded fire decision on 84/84 cells and the recorded span on 23/23 firings.

Inferred, not measured, and deliberately not built here: the 5 template-prefixed cells could be repaired by a **span-extraction** instrument (emit only the span the pointer selected), ceiling +5/84; it needs its own pre-registered test.

## Checks executed at this head

- `~/.cargo/bin/cargo check -p uor-r4-training --example bind-probe-exact` (rustup-managed cargo 1.97.1, local): `Finished dev profile ... in 1m 27s`, exit 0, only pre-existing warnings.
- `rustfmt --edition 2021 --check crates/uor-r4-training/examples/bind-probe-exact.rs`: clean (the landed copy is the measured source plus two re-wraps; recorded in the evidence file with both md5s).
- Pod build at `main` 433a03f66 with `--features cuda --example bind-probe-exact`: BUILD-OK, binary sha256 `56ebcae040dd2dabe8791bf89ec6d7685e13ae1dee3bc07bd380017cff2974f6`; from it, the 84-cell trace run, the 9 default runs and the sealed-reference comparison above.

References #2029. Refs #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

