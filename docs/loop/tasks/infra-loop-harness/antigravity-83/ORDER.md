# ORDER: infra-loop-harness, antigravity-83

Lab: antigravity. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file scope maintenance, as in claude-90, claude-108, and antigravity-80). Base: `12facdf94`.

## Goal

Narrow the `scope:` of four `status: ready` roadmap items under a D68 decision review so their pre-registration sheet / `TEST FITNESS: FIT` and evaluation units can be claimed in parallel while `PR #3231` holds `docs/mechanisms/hamilton-gate.md` and `PR #3230` holds `crates/uor-r4-discrete/**`:

1. `docs/roadmap/items/m1-normalization-mechanism.md` (`#2842`, milestone M1): remove `docs/mechanisms/hamilton-gate.md` (which overlaps `PR #3231` and is not edited by the `m1-normalization-mechanism` sheet + `TEST FITNESS: FIT` close unit).
2. `docs/roadmap/items/m4-flock-coupling-prereg.md` (`#2860`, milestone M4): remove `crates/uor-r4-discrete/**` (its 7/7 host controls in `flock_coupling_controls.rs` are merged on `main`; only the sheet + `TEST FITNESS: FIT` + notebook row remain), and add `notebook: docs/research/notebook/m1-discrete-attention.md` and `docs/loop/tasks/m4-flock-coupling-prereg/**`.
3. `docs/roadmap/items/r-prime-zeta-vs-nulls.md` (`#2881`, milestone research): remove `crates/uor-r4-discrete/**` (its constructor, CLI, and 7/7 host tests are merged on `main`; only the sheet + `TEST FITNESS: FIT` and zero-update evaluation battery remain), and add `notebook: docs/research/notebook/m1-prime-zeta.md` and `docs/loop/tasks/r-prime-zeta-vs-nulls/**`.
4. `docs/roadmap/items/r-read-off-vs-read-on.md` (`#2885`, milestone research): remove `crates/uor-r4-discrete/**` (its kernel switch, 7/7 host controls, and battery script are merged on `main`; only the sheet + `TEST FITNESS: FIT` and zero-update evaluation battery remain), and add `notebook: docs/research/notebook/m1-discrete-attention.md` and `docs/loop/tasks/r-read-off-vs-read-on/**`.

## Cancel probe (run first)

At `12facdf9471aa228459e9dd6c2cef91b2ce9bdf8`:
- `uor-harness loop claim infra-loop-harness --lab-topic unblock-m1-m4-sheet-scopes` refused (`already claimed by PR #3169`). Checked `#3169`, `#3230`, and `#3231` via `gh pr diff --name-only`: zero file overlap with `docs/roadmap/items/{m1-normalization-mechanism,m4-flock-coupling-prereg,r-prime-zeta-vs-nulls,r-read-off-vs-read-on}.md` or this task folder. Hand-written under the umbrella item exception (`run-antigravity.txt` Part 3.2).
- **Cancel probe result (`instrument`):** Confirmed roadmap scope bottleneck; proceed with the D68 scope narrowing.

## Files in scope

- `docs/roadmap/items/m1-normalization-mechanism.md`
- `docs/roadmap/items/m4-flock-coupling-prereg.md`
- `docs/roadmap/items/r-prime-zeta-vs-nulls.md`
- `docs/roadmap/items/r-read-off-vs-read-on.md`
- `docs/loop/tasks/infra-loop-harness/antigravity-83/**`

## Done when

- All four item files have their scopes narrowed to their docs/prep/labs/task paths with notes recording the D68 scope change, a fresh-context D68 decision review approves the change and is logged for Casey on `#2904`, `#2842`, `#2860`, `#2881`, and `#2885`, and `uor-harness loop gates --full` is green.

## Stop and ask if

- Never stop and wait. If any sheet + FIT unit requires crate changes first, keep the crate glob on that item and narrow only the others.
