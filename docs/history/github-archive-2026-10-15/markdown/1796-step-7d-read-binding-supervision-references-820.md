# #1796 — Step 7d: read-binding supervision (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T05:22:31Z
- **Updated:** 2026-10-09T13:09:06Z
- **Closed:** 2026-10-06T15:51:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1796
- **Merged:** 2026-10-06T15:51:16Z
- **Merge commit:** a2662a676f839126439e07c78a375307dcca5338
- **Base / Head:** main / UOR-Foundation:claude/step7d-read-binding
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Pre-registration: issuecomment-6009883286.

**Status:** draft. The pre-registered 96M A/B gives **REJECT** (result: issuecomment-6011946391). The option stays opt-in and off by default, and its off path is bit-identical.

| arm | v4 memory | unknowable | sieve-off Mqar | Instr (A = 78 / 74) | probe a/b/c |
|---|---|---|---|---|---|
| C-s1 / D-s1 | 24 → **26** | 17 → 17 | 106 → 107 | 74 → 73 | 7/0/9 → 6/0/8 |
| C-s2 / D-s2 | 21 → **26** | 19 → 18 | 106 → 106 | 67 → 65 | 9/0/10 → 6/0/8 |

- Memory fails the bar on both seeds (needs ≥ 28 and ≥ C + 4); Instruction fails on seed 2. Unknowable, Mqar and the open-panel guard (p = 1) pass. W = 0.1, chosen by the frozen rule.

**Exact-head evidence (7c682fdb, 2×RTX 5090):**
- the bootstrap's `cuda_stack_ops_parity` passed 33/33, including `test_read_supervision_parity`;
- the off path reproduces Step 6c C-s1 (`model_sha256` b754d656…), with the option absent and with `=0`;
- the regenerated store reproduces sha b0770fbd…;
- D-s1 equals the sweep's SW-0.1 model, bit for bit.

## What changes (opt-in, default off)

**`dialogue-recall-corpus generate binding_labels=1`** writes a sidecar beside each split's store. The stores themselves are byte-identical with or without it.
- `binding_labels.jsonl` has one entry per answer whose expected value is stated earlier in the dialogue:
  - `b`: the history token positions of that value;
  - `c`: the history positions of the question's forbidden values;
  - `q`: the answer input positions whose next token is a token of the value.
- `binding_labels.json` holds the counts and the SHA-256 of the token payload.

**`dialogue-train read_binding_supervision=W`**, with `read_binding_labels=DIR read_binding_source=LABEL [read_binding_layer=L]`:
- On steps whose batch holds a labelled answer, it adds `W · mean_q −log(m_q + 1e-6)`.
- `m_q` is the attention mass that the *binding head* of the supervised read layer (default: the last) puts on the bound value.
- The binding head is chosen per query, without gradient: the head with the most mass on bound ∪ competing.
- Labels are mapped to FullPrefix windows through each episode's document start. Loading checks the source's token count and payload SHA-256.
- The curve adds `train_binding_{steps,rows,nll,bound_mass,competing_mass,objective}`.

**`StackModel::read_supervised_loss`** carries the bound and competing sets as auxiliary value channels of the fused read:
- one pair of channels per group slot, shared by all heads;
- the channels share the read's exact scores, age, admission and NoRead normalization;
- they are removed before `read.out`, so the language loss and hidden states are unchanged.

## Tests
- `read_supervision_leaves_language_unchanged_and_reads_the_probe_masses`, for Dot, Lorentz and L2, with and without a pointer:
  - the language loss equals `weighted_loss` bit for bit, and its gradient is unchanged;
  - each row's masses and selected head equal `read_span_probe` on the row's prefix window;
  - the binding term equals a recomputation from those masses.
- `read_supervision_gradient_matches_finite_differences_and_raises_the_bound_mass`:
  - central-difference check over the query, key, age, null, embedding and recurrent weights;
  - the value and out projections get no gradient;
  - one step lowers the term.
- `read_supervision_refuses_bad_labels_and_weights`.
- `binding_labels_point_at_the_expected_value_and_leave_the_store_unchanged`: the store bytes are identical, the payload SHA matches, and every label's bound and answer positions decode to the value.
- CUDA: `test_read_supervision_parity` (`cuda_stack_ops_parity`) compares, for L2, Lorentz and Dot, with and without a pointer:
  - the CPU and CUDA losses, masses, selected heads and total gradient;
  - the CUDA language loss against `weighted_loss`, bit for bit.

Local (laptop CPU):
- `cargo test -p uor-r4-training --lib read_`: 99 passed.
- `--example geometric-stack`: 11 passed.
- `--bin dialogue-recall-corpus`: 17 of 18 passed. `the_leak_check_detects_an_injected_panel_turn` also fails on main: "Quill" is now a panel recall value, so this change does not cause it.
- A CPU smoke run of `dialogue-train` gave the same `model_sha256` with no option and with `=0`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6019071445 — 2026-10-06T15:00:36Z — Casey-allard

Exact-head self-review at `783faf93709f96d0900835024e6ae6f2c39c8a33` (owner rule: self-review allowed). Merging as a kept negative result (Step 7d REJECT on #820); opt-in, default off.

- Merged origin/main (no rebase). Resolved the BindingCapture clash: `supervision: None` in main's qk-dump constructor, `qk: None` in 7d's supervision-forward constructor. rustfmt applied.
- At this head: `cargo check -p uor-r4-training --lib --tests --example geometric-stack` → 0 errors; `cargo test -p uor-r4-training --lib -- read_supervision binding_labels_point_at_the_expected_value` → 3 passed, 0 failed; `rustfmt --check` clean on the touched .rs files.
- Off path: supervision field is None unless `read_binding_supervision>0`, so default training is unchanged. APPROVE.

### comment 6020033338 — 2026-10-06T15:50:12Z — Casey-allard

Exact-head self-review at `c8f3040bdaa46cacf5d59f2250654bf309576559`: empty commit on top of `783faf93` (tree identical, verified with `git rev-parse HEAD^{tree}`) to re-trigger required checks — GitHub kept reporting '5 of 5 required status checks are expected' for the old head although all five passed. The checks listed in the previous review apply unchanged. APPROVE.


---
_Comment/review ids in this file: 6019071445, 6020033338_

