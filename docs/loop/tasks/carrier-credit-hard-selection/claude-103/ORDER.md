# ORDER: carrier-credit-hard-selection, claude-103

Lab: claude. Item: [carrier-credit-hard-selection](../../../../roadmap/items/carrier-credit-hard-selection.md). Base: `6b5da6248`.

## Goal

The item's done-when reads:

> A hard-selection credit step replaces the sampled choice with no softmax, and the three-arm sheet is rerun with the result recorded.

This unit decides where that rerun can lawfully happen, and how. The record is the decision card [DECISION.md](DECISION.md), D68, and it is approved only after its own fresh-context decision review. The building and the GPU run follow under this order once the scope fix in the card has landed on `main`.

## Cancel probe (run first)

Is the work still needed, and can the sheet be rerun as written? Run on 25 Oct, against `origin/main` `87fe8d0ff`.

- **Mechanism: still needed.** `docs/mechanisms/carrier-credit.md` is PARKED. Its first reopen condition is a rule-compliant replacement for the sampled choice, and nothing on `main` provides one.
- **Instrument: the sheet cannot be replayed.**
  - Its objective is a log-mean-exp over softplus of a log mass, which CHARTER §3.2 bans.
  - Its modules are compiled out (`#[cfg(any())]` in `crates/uor-r4-training/src/lib.rs`), and its runner requires the `historical-reference` feature.
  - saved437 is only in the iCloud cold store.
  - This is a fault of the instrument, not a negative for the mechanism (§4.2).
- **Instrument: the lawful home is the live discrete learner.** A new schedule must live inside `crates/uor-r4-discrete`, because `Device::apply` is crate-private and moves commit only through `PreparedDriver::transition`. That crate is outside this item's `scope:`.
- **Verdict:** do the work. Fix the instrument and the scope first, as the decision card sets out.

## Files in scope

- `docs/harness/prep/carrier-credit-hard-selection*`
- `docs/labs/carrier-credit-hard-selection/**`
- `docs/loop/tasks/carrier-credit-hard-selection/**`
- After the card's scope fix lands on `main`: `crates/uor-r4-discrete/**`, and the notebook row in `docs/research/notebook/m1-discrete-attention.md`.

## Done when

- The decision card has passed its fresh decision review, and is logged on #2819 as `[claude] owner log:` (D69).
- The scope fix is merged through an `infra-loop-harness` unit.
- The hard-selection schedule exists in `crates/uor-r4-discrete`, integer and on CUDA, with finite controls. The default schedules' config and lineage bytes are unchanged, and the full discrete suite plus the driver-source pins are green.
- The prep sheet has a FIT and a D67 pre-run review. The arms run on one RTX 5090, and the result is recorded: the REPORT, a notebook row, and the mechanism page's tried table.
- This PR writes `Closes #2819` only if all of the above hold.

## Stop and ask if

Stop and decide, never stop and wait.

- The decision review rejects the card. Revise it, or record a different lawful reading, and review that one fresh.
- The selection rule needs a host-side reduction. Move it onto CUDA; never run training arithmetic on the host.
- A pin or a default config byte changes outside the new variant. Find the cause before going further.
