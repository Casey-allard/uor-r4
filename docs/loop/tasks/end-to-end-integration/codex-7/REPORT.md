# REPORT: end-to-end-integration, codex-7

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `loop claim end-to-end-integration --lab-topic read-out-interface` was refused because
  `crates/uor-r4-integer/src/stack/**` overlaps claude-397's PR #4022. This unit adds a module and
  two adapters, no dependency and no `Cargo.lock` change.
- **The interface is the common result, not a common trait or input.** The two reads take different
  inputs (a Q31 history and row; tokens and a head), and forcing one `select` signature over both
  would be a false abstraction. `ReadOut` is the seam a stage consumes; a trait waits for a third
  consumer or a stage that composes reads. That is a deliberate narrowing of the ADR's wording, not
  a claim that the wording was met in full.
- **Only the copy head is adapted.** The type and ask heads, flock, route and relation are not; the
  Hamming adapter covers `CopyHeads` with the margin weights.
- **The served Q31 read now consumes the result.** `serve` calls `read_out`, writes its payload and
  traces its pick; the existing `apply`/trace tests are the bit-identical check. No other caller
  changed, and no output moved. `choose` is untouched.
- **The result matches the ADR's types and order.** `payload` is `Vec<i128>` (the ADR's type, and
  the crate's accumulator width); `support` is the read's own deterministic order — for a selection,
  ascending `(distance, position)`, which is what `choose` returns. It is not re-sorted, because the
  read's rank order is part of its answer.
- **The `read_out`-vs-`serve` test is no longer two independent implementations**, since `serve` now
  calls `read_out`. The independent bit-identity is the shipped `apply`/trace test, which compares
  the stepped read to the batch operator; the adapter test's remaining value is the payload/row
  equality on three successive rows.
- **`first_pick` maps the pick to a one-position support** (`pick - 1`), with the row as `payload`;
  a consumer must read the payload, not weights. A future consumer that expects weights from every
  read would be wrong; `ReadOut::check` refuses a result with both.
- **Positions are `u16`**, which holds a context up to 65,536; a larger context would need `u32`
  and a new result version.
- **No session wiring and no output change;** `serve` now consumes `read_out` (same step, same
  `+2^31` refusal, same written row), checked by the shipped `apply`/trace parity test. No model or
  quality claim.

## What changed

- `crates/uor-r4-integer/src/stack/read_out.rs` (new): `ReadOut` with `first_pick`, `selection`,
  `check`, and four unit tests.
- `crates/uor-r4-integer/src/stack/mod.rs`: `mod read_out;` and `pub use read_out::ReadOut;`.
- `crates/uor-r4-integer/src/stack/q31_read.rs`: `ServedQ31Read::read_out` (the pick and the row),
  `serve` now consumes it, and a test that its written row equals `serve`'s over three rows
  (payload at `i128`, the ADR's type).
- `ReadOut::pick()`: the 1-based pick or 0, so a consumer can recover the operator's pick without
  knowing the support convention.
- `crates/uor-r4-integer/src/stack/hamming.rs`: `CopyHeads::read_out` (choice plus margin weights)
  and a test against `choose` and `copy_weight_q31`.

Size: about 180 lines of source and tests, and the ORDER/REPORT.

## Measurements

- `read_out` unit tests: 4 passed (first pick, selection, malformed results, `u16` bound).
- Q31 parity: `read_out`'s payload equals `serve`'s written row on three successive rows, and the
  histories behave identically on the third.
- Hamming parity: `support` equals `choose`'s positions and every weight equals
  `copy_weight_q31(margin, distance)`.
- Library: **671 passed**, 1 ignored; the 9 Q31-read tests include the `apply`/trace parity that
  now runs through `read_out`.

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib` — 671 passed.
- `rustfmt --check` — `read_out.rs` and `q31_read.rs` clean; `hamming.rs` shows only its pre-existing
  diffs (the added lines are formatted).
- `cargo clippy --locked --no-deps -p uor-r4-integer --features serving-wasm --lib --tests` — no
  warning in the added lines.
- `git diff --check`, the harness contract rule, the REPORT check and the exact-head review — at the
  head.

## Skills used

- `solution-integrity`: refused the false common input and made the result the seam; the adapters are
  tested against the shipped functions, not against a restatement.
- `scope-guard`: one module and two small adapters; `choose` untouched, and `serve` now consumes
  `read_out`.
- `test-freeze`: adds tests inside existing files and changes no existing test; the commit carries a
  `TEST-CHANGE:` line.

## Still open

- A `Read` stage that consumes `ReadOut` in `IntegerStackSession`, and the Q31 read wired through it
  (bit-identical to the current serve path).
- The type and ask-head adapters, then flock/route/relation when a second consumer needs them.
- The I1b index wiring (the 8.6× exact read) behind the same result.
- I2 context wiring; I3/I5 deferred; I4 withdrawn.

## Plain summary

The two served reads now have a common result: a Q31 first pick returns one position and the row it
writes, a Hamming copy head returns its positions and margin weights, and both match the shipped
functions exactly. The seam is the result, not a forced shared input; wiring it into the session is
the next unit.
