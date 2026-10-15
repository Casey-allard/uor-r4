# #1845 — Add an opt-in entry-only loss scope and declared dose to the frozen-map fitter (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-07T16:02:26Z
- **Updated:** 2026-10-09T14:18:21Z
- **Closed:** 2026-10-07T17:17:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1845
- **Merged:** 2026-10-07T17:17:42Z
- **Merge commit:** d99e99922013ea5e423a34d1ff4362bea5a902ad
- **Base / Head:** main / UOR-Foundation:deepseek/entry-isolation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this changes

`geometric-frozen-map-fit` gains one opt-in knob and one declared-dose knob; both default to the retained campaign, and there is no change to any fitted artifact or to serving.

- `loss_scope`: `all` (default) or `entry_only`. Under `entry_only` every later-position weight is zero and the entry position keeps **its own** balanced weight, so an arm differs from the control in exactly one thing: whether later-position gradient reaches the four shared parameter groups.
- `updates`: declared dose, admitted only in multiples of 32 up to 128, so the final reload always finds a written checkpoint.

## Why

`docs/labs/deepseek-handoff-2026-10-07.md` §2.2(b) reads the exposure audit's `entry position weight 1/24` as the entry objective being drowned by 13,328 supervised positions of which it is 1,024. The governing code does not support that reading. `episode_loss_weights` divides by `counts[phase]`, so every *nonempty phase* receives the same total weight per episode:

| phase | positions/episode | per-position weight | phase total |
|---|---:|---:|---:|
| `entry` | 1 | 1/(3·1·8) = 1/24 | 1/24 |
| `later_copy_covered` | 8 | 1/(3·8·8) = 1/192 | 8/192 = 1/24 |
| `later_generate_only` | 4 | 1/(3·4·8) = 1/96 | 4/96 = 1/24 |

The entry phase already receives 1/3 of each episode's credit. `balanced = true` is unconditional at the single production call site, and `authenticate()` refuses to run unless the parent configuration records `phase_balanced_token_loss` — so all retained arms are phase-balanced. The harness's own `phase_weights_conserve_equal_nonempty_phase_mass` asserts exactly this.

That leaves one testable part of the hypothesis: whether the later phases push the shared groups against the entry decision. This knob runs that test without reweighting the entry (a reweight would confound the two). It is the arm requested on #820.

## Verification

- `cargo check -p uor-r4-training --example geometric-frozen-map-fit` — unchanged pass (pre-existing warnings only).
- `cargo test -p uor-r4-training --example geometric-frozen-map-fit` — 10 passed, 0 failed, including two new tests: `entry_only_scope_zeroes_later_credit_and_preserves_entry_weight` and `loss_scope_and_dose_default_to_the_retained_campaign`.
- `rustfmt --edition 2021 --check` on the touched file — clean.
- `loss_scope=all` keeps the exact prior arithmetic and evaluation order; the legacy branch is untouched; the default `phase_policy` string is byte-identical so existing sealed baselines still authenticate.

## Experiment (running)

Matched pair from the authenticated retained parent (`read-state-bridge-joint-fit-attempt2/checkpoint-0128`), identical seed 1001, `credit=clipped`, `read_state_pullback=legacy`, 128 updates, batch 8, fresh Adam, same four rates: `entry_only` vs `control`. Readout is `metrics-0128.json` → `entry_teacher_correct` / `entry_own_correct`, which are 0/512 in all six retained arms. Results and retained roots are reported on #820.

References #820.

## Result — `entry_only` at 128 updates

| metric | frozen parent | `entry_only` 128 | `control` 128 |
|---|---:|---:|---:|
| `entry_teacher_correct` | 0 / 512 | **0 / 512** | running |
| `entry_own_correct` | 0 / 512 | **0 / 512** | running |
| `first_divergent_teacher_correct` | 40 / 1024 | 56 / 1024 | running |
| `later_copy_covered_teacher_correct` | 351 / 4128 | 415 / 4128 | running |
| `first_divergent_own_prefix_reached` | 0 | 0 | running |

The entry position carried the whole episode's credit for 128 updates and did not move: zero of 512 rows changed at the entry decision. The zero-update evaluation of the frozen parent reproduces the sealed admission baseline exactly (40/1024 and 351/4128), so this is the retained parent and evaluation, not a lookalike. Entry-only credit did move the shared groups — later Copy-covered correctness rose 351 → 415 of 4,128 without ever being credited — so the entry objective is connected and credited, and still loses its own competition.

Report root `/workspace/uor-r4/deepseek/entry-isolation/arm-entry_only-u128-attempt2/` (sealed, exclusive), elapsed 948.8 s. Full discussion on #820.

## Matched control — complete

| metric | parent | `entry_only` 128 | `control` 128 |
|---|---:|---:|---:|
| `entry_teacher_correct` | 0 / 512 | 0 / 512 | **0 / 512** |
| `later_copy_covered_teacher_correct` | 351 / 4128 | 415 / 4128 | **502 / 4128** |
| `first_divergent_teacher_correct` | 40 / 1024 | 56 / 1024 | 32 / 1024 |

The control reproduces the sealed order1001 clipped arm exactly (entry 0/512, Copy-covered 502/4128), so the pair is same-executable and the cross-executable caveat is gone. Allocating all of the entry phase's credit changes the entry decision by zero rows and costs the later positions. Gold/winner mass ratio at entry across the 512 rows: 0.0004 mean at the parent, 0.0135 (`control`) and 0.0257 (`entry_only`) after 128 updates; the best row reaches 0.14. The entry token is off by ~2,500× at the parent and still 2 orders of magnitude away after training.

This PR also adds the explicit retraction to `docs/labs/deepseek-handoff-2026-10-07.md` §2.2(b), whose "entry objective is drowned" diagnosis these measurements falsify, so the next session reading that handoff does not repeat it.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

