# REPORT: m2-fresh-heldout-panel, antigravity-22

- Order: [`ORDER.md`](./ORDER.md)
- Branch: `antigravity/m2-fresh-heldout-panel`
- Head: `4d255e7b36070cf407029f35612431f60934f5a3`
- References #2849
- References #2030

## What did not work, first

- **Initial `update`-stratum initial-assertion indexing (`(selected * 2 + usize::from(swapped)) % 4`):** Assigning `FRESH256_UPDATE_INIT[(selected * 2 + usize::from(swapped)) % 4]` placed even-indexed initial literals only in `job` (`swapped == false`) and odd-indexed initial literals only in `home` (`swapped == true`), which failed `verify_m2_fresh256_panel` with `"literal role imbalance"`. Indexing `FRESH256_UPDATE_INIT[selected % 4]` per unordered block restored exact `job`/`home` and turn-0/turn-1 balance across all `34` fresh literals.
- **Initial lexical drafts (`"basalt terrace"`, `"keen plover along Causeway"`, `"the bronze sundial waits near a mossy Causeway"`, `"the steel pendulum swings beside a brick foundry"`):** `verify_m2_fresh256_panel` rejected `"terrace"` (overlapping `"Rosewood Terrace"` in `FRESH3_VALUES`), `"Causeway"` (overlapping `"Cobalt Causeway"` in `FRESH3_VALUES`), `"mossy"` (overlapping `"mossy plateau"` in `VALUES`), and the word 2-gram `"beside a"` (overlapping `"the quiet cedar stands beside a blue harbor"` and `"the small maple rests beside a warm river"` in `VALUES`). Replacing them with `"basalt loggia"`, `"keen plover along dikeway"`, `"the bronze sundial waits near a cobble dikeway"`, and `"the steel pendulum swings outside a brick foundry"` reduced exact, content-word, and word 2-gram overlaps to `0` with minimum same-stratum Levenshtein edit distance `10 >= 8`.
- **Creating `docs/research/notebook/m2-grounded-reply.md` directly in this PR:** `docs/roadmap/items/m2-fresh-heldout-panel.md` declares `scope: [data/**, docs/harness/panels/**, docs/mechanisms/native-geometric.md, docs/labs/m2-fresh-heldout-panel/**, docs/reference/storage.md]` without `notebook:`, so `uor-harness loop pr-check --local` (`rule_scope`) forbids touching `docs/research/notebook/m2-grounded-reply.md`. The six-column M2 notebook row is recorded in [`docs/labs/m2-fresh-heldout-panel/README.md`](../../../../labs/m2-fresh-heldout-panel/README.md) (`§6`) ready to copy once that path is in an item scope.

## What changed

- [`docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml`](../../../../labs/m2-fresh-heldout-panel/instrument/Cargo.toml), [`src/lib.rs`](../../../../labs/m2-fresh-heldout-panel/instrument/src/lib.rs), [`src/main.rs`](../../../../labs/m2-fresh-heldout-panel/instrument/src/main.rs), and [`Cargo.lock`](../../../../labs/m2-fresh-heldout-panel/instrument/Cargo.lock): added a standalone, zero-float Rust generator, `m2-reply512` noise-spread analyzer (`analyze_saved437_dev512_spread`), and `m2-fresh256` overlap/balance/seal verifier (`generate_m2_fresh256_panel`, `verify_m2_fresh256_panel`) with unit and deliberate-break mutation tests.
- [`docs/harness/panels/m2-fresh256.json`](../../../../harness/panels/m2-fresh256.json) and [`docs/harness/panels/m2-reply512.json`](../../../../harness/panels/m2-reply512.json): registered the frozen 256-row `m2-fresh256` panel (`NOT_RUN`, bar `>= 103/256` alongside `>= 256/512` on `m2-reply512`, `TEST FITNESS: FIT` from non-author fresh-context review `3f74630d-7674-4958-a496-38b96b34ab18`) and recorded the measured `saved437` noise spread on `m2-reply512`.
- [`docs/labs/m2-fresh-heldout-panel/README.md`](../../../../labs/m2-fresh-heldout-panel/README.md), [`docs/reference/storage.md`](../../../../reference/storage.md), [`docs/mechanisms/native-geometric.md`](../../../../mechanisms/native-geometric.md), [`docs/roadmap/items/m2-fresh-heldout-panel.md`](../../../../roadmap/items/m2-fresh-heldout-panel.md), and [`docs/roadmap/README.md`](../../../../roadmap/README.md): documented the `m2-reply512` noise audit, the `m2-fresh256` overlap exclusion and sealed SHA-256 digests (`plan.json` `sha256:698d3cf2…`, unscored `inputs.json` `sha256:f6f6732e…`, sealed `labels.json` `sha256:ecb99eb0…`, `manifest.json` `sha256:4ccf1e32…`), the non-author `TEST FITNESS: FIT` review, and the six-column research notebook row, and marked `m2-fresh-heldout-panel` `done`.

## Measurements

- **`m2-reply512` (`b9661606…`) noise audit on `saved437` (`de4a3234…`):**
  - Complete replies: `437 / 512` (`85.35%`; `length2: 116/128`, `length4: 114/128`, `length8: 90/128`, `update: 57/64`, `reassert: 60/64`; `186 / 256` histories complete on both roles).
  - Evaluator replay spread: `0 / 512` (`0.00%`).
  - Stratified half-panel split (`16` even vs `16` odd blocks of `256` rows): `219 / 256` (`85.55%`) vs `218 / 256` (`85.16%`), `|\Delta| = 1` row (`0.39%`).
  - Paired-variant spreads across `256` pairs: chronology `212` vs `225` (`|\Delta| = 13`, `61` discordant), role-swap `215` vs `222` (`|\Delta| = 7`, `65` discordant), query-wording `214` vs `223` (`|\Delta| = 9`, `57` discordant), queried-role `219` vs `218` (`|\Delta| = 1`, `65` discordant).
  - Stratified 16-block (`256`-row) cluster-sampling SE: `6.24` rows (`244 bps`) at `p = 85.35%`, and `8.64` rows (`338 bps`, vs `7.84` rows i.i.d. binomial) at the `40%` (`103 / 256`) acceptance bar.
- **`m2-fresh256` overlap, balance, and seal verification:**
  - `256` rows (`128` histories, `16` unordered blocks, `32` ordered banks, `128` opposite-query pairs; `length2: 64`, `length4: 64`, `length8: 64`, `update: 32`, `reassert: 32`), target bar `>= 103 / 256` (`40.23%`).
  - Overlaps against all `72` historical exposed literals: `0` exact, `0` content-word, `0` word 2-gram, minimum same-stratum Levenshtein edit distance `10 >= 8`.
  - Check-only heuristic baselines: `copy_first_stated = 120/256` (`46.88%`), `copy_last_stated = 128/256` (`50.00%`), `role_swap_distractor = 0/256` (`0.00%`), `best_constant_reply = 16/256` (`6.25%`).
  - Sealed artifacts staged at `local/antigravity/m2-fresh256-sealed/` for `hf://datasets/caseyallard/uor-r4-store/data/m2-fresh256/`: `plan.json` (`122,745 B`, `sha256:698d3cf2d7208f3d66f85a1b3a77727f6380c785964f914a78ab9608cdbcba4a`), `inputs.json` (`410,531 B`, `sha256:f6f6732e1958568c3ec6aed7262045e8f447d111a71abcad54d0ca2a5950a06c`), `labels.json` (`52,573 B`, `sha256:ecb99eb0b196b6ba2c56e6ede5556d90b35391ffe4dc13fb6fea6b97cc444842`), `manifest.json` (`1,183 B`, `sha256:4ccf1e32ad86e81c243c9c0ade18cbdd90077e3e07fd854ba01aeee5b8873779`).

## Checks run at the head

- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo fmt --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --check`: exit `0`
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo clippy --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked --all-targets -- -D warnings`: exit `0`
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo test --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked`: `3 passed; 0 failed` (`dev512_spread_matches_sealed_saved437_receipts`, `fresh256_panel_passes_all_overlap_balance_and_seal_checks`, `negative_mutations_are_rejected_by_verifier`)
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo run --locked -p xtask -- roadmap check`: `88 items OK`
- `uor-harness loop gates --full`: `GREEN` across all 12 gates
- `uor-harness loop report --check`: `PASS`
- `uor-harness loop pr-check --local`: `verdict: PASS`

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`, `gates`, `context-guard`

## Still open

- Uploading the 4 staged sealed files (`plan.json`, `inputs.json`, `labels.json`, `manifest.json`) from `local/antigravity/m2-fresh256-sealed/` to the private store `hf://datasets/caseyallard/uor-r4-store/data/m2-fresh256/` by a maintainer with HF write access (`#2849` step 4).
- Copying the six-column M2 notebook row from `docs/labs/m2-fresh-heldout-panel/README.md` (`§6`) into `docs/research/notebook/m2-grounded-reply.md` once an item scope includes that path.
- Any M2 training or candidate evaluation against `m2-fresh256` (`NOT_RUN`), which remains `HELD` until the owner releases a pre-registered M2 sheet.

## Plain summary

Measured the noise and paired-variant spread of `saved437` on the 512-row M2 development panel (`0/512` replay diff, `219/256` vs `218/256` half-panel split, `8.64`-row cluster SE at the `40%` bar), built and verified the replacement 256-row fresh held-out panel (`m2-fresh256`, `0` exact/word/2-gram overlaps against all `72` historical exposed literals, bar `>= 103/256`, `TEST FITNESS: FIT`), and registered the frozen panel entry and sealed SHA-256 digests (`NOT_RUN`).
