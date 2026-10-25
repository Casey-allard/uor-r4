# ORDER: m3-nonrelation-prose-panel, claude-98 (repair: the D23 interlock counts a doc-comment word in claude-94's compiler)

Lab: claude. Item: [m3-nonrelation-prose-panel](../../../../roadmap/items/m3-nonrelation-prose-panel.md). Base: `b19b15f00`. Repairs claude-94 (#3188, merged as `b19b15f00`).

## Goal

claude-94's `crates/uor-r4-training/src/sieve_frame_compiler.rs` says in its module doc that the compiler has "no score, probability, softmax or matrix product anywhere". The D23 interlock counts that word as a site, even inside a `//!` doc comment, so the file is 1 above its baseline of 0. That turns four `uor-r4-harness` lib tests red on `main`:
- `prlintrules::tests::an_acknowledging_body_produces_nothing`;
- `prlintrules::tests::naming_the_mechanism_produces_an_info_not_an_error`;
- `prlintrules::tests::the_real_interlock_is_callable_and_clean`;
- `prgates::check_pr_tests::every_ported_gate_and_the_dispatcher_agree_with_python` (all 11 of its differences are that one `prlint.softmax_ban` finding).

`loop pr-check`'s banned-presence rule skips comments, so it passed at #3188's head. The interlock's own rule is stricter. The fix rewords the doc sentence, with no change to code or behaviour.

## Cancel probe (run first)

Class: instrument (a red interlock caused by a doc word). At `b19b15f00`, both must hold, or the unit is cancelled:
- `sieve_frame_compiler.rs:14` holds the word;
- the four tests fail at `b19b15f00` with the interlock reporting that file `0 -> 1`.

## Files in scope

- `crates/uor-r4-training/src/sieve_frame_compiler.rs` (the module doc only)
- this task folder

## Done when

- The module doc says the same thing without the banned word, and the file has no D23 site.
- The four tests pass. `cargo test -p uor-r4-harness --lib` shows no other new failure: the one remaining failure, `prep::tests::the_real_sheets_are_readable`, is #3190's d41 sheet missing a verdict line. That file is antigravity's, and it is noted on #2028.
- `cargo test -p uor-r4-training --lib` still passes 72.
- `loop report --check` and `loop pr-check --local` pass, and a fresh-context review counts at the exact head.
- The REPORT says `References #2856`.

## Stop and ask if

- Any code line, not just the doc sentence, would have to change.
