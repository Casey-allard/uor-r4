# ORDER: in-space-trunk-to-text, claude-155

Lab: claude. Item: [in-space-trunk-to-text](../../../../roadmap/items/in-space-trunk-to-text.md). Base: `5ae96c7da`.

## Goal

Casey, 28 Oct 2026, in claude's chat, after claude-154 read cell D:

> "Yes, pre-register the prompt register as the next unit: learned query slots over the user's turn, kept as separate vectors and reaching the readout, with the opening read and a plain bag as controls."

Earlier in the same conversation, the brief for it:

> "ok we still need to design a mechanism that works and is cheap for this next"

**What claude-154 found.** The opening read handed the trunk the user turn's first 8 tokens, the very tokens that name 607 openings by lookup. Openings fell to `206.0` and `156.3`, against a bar of `347.3`. Taken out of the trained model, the read changed only `25` and `19` of 14,287 tokens. The trunk ignored it.

A candidate cause: the 8 points arrived averaged into one vector, entering only after layer 2.

**The prompt register** (`--prompt-register Q`) keeps the user's turn as Q separate learned slots, and lets each reach both the stream and the readout. At width 256 there are `lanes = 64`. After layer 2, with `x` the stream and `ids` the window:

1. **The span** of position `t` is the turn anchor `a(t)` through `t`. The anchor is the opening read's rule: after the last `<|bos|>` or `<|eos|>`, or the window's start. The span is empty when `ids[t]` is either of those.
2. **Keys:** for each position `s`, a `lanes`-bit sign code. The sign is taken of each lane's real part after `x_s`'s lane is turned by a learned key quaternion (identity at the start), straight through as in the Hamming read.
3. **Queries:** each of the Q slots has a learned `lanes`-vector `c_q`, whose sign is its code. Its start is deterministic and differs by slot.
4. **Distance:** `d(q, s)` is the number of lanes where slot `q`'s code and `s`'s key differ, `Σ (code − key)² / 4`.
5. **Selection** (`--register-select learned`, the default):
   - for each `t` and `q`, the `K` positions of the span with the smallest `d`, ties to the earlier position (`--register-picks K`, default 8), chosen detached;
   - each pick weighs `w = relu(m_q − d) / max(m_q, 1)`, with a learned margin `m_q` that starts at `lanes / 2`.

   With `--register-select all`, the **plain bag**, every position of the span weighs 1, and Q must be 1.
6. **Values:** `x_s`, turned lane by lane by a learned value quaternion per slot (identity at the start).
7. **Slot `q` at `t`:** `S_q(t) = Σ w·v / Σ w`, or zero when the weights sum to 0.
8. **The stream:** `x ← x + Σ_q U_q ⊙ S_q(t)`.
9. **The readout,** after the final norm and the Hamming readout: `n ← n + Σ_q V_q ⊙ S_q(t)`.

Every slot keeps its own `U_q` and `V_q`, learned per coordinate and zero at the start. So at step 0 the model is exactly the model without the register. There is no exponential and no matrix product: per-lane quaternion products, sign codes, Hamming counts, a top-K, a relu margin, weighted means and elementwise weights.

**The controls:**
- **The plain bag** `--prompt-register 1 --register-select all`, run here. It sees every word of the turn, uniformly weighted.
- **The opening read**: claude-154's (o), sealed, with the same base recipe.

The item's own done-when (unchanged by this unit):

> On the pinned DEV panel, decoded as the whole table's nearest token:
>
> 1. Reply-opening top-1 is at least `197` of 1,226. That is always naming DEV's commonest opening token (`161`) plus three binomial standard deviations (`36`). The same checkpoint's whole-panel top-1 must be at or above the M1 count table's `7,002`.
> 2. On that same checkpoint, the relative reach from `--reach`, `R16 = shift(16) / shift(0)`, is at least `0.05`. The 27 Oct best checkpoint reads `0.011`. An untrained trunk reads more, but it cannot meet (1).
> 3. Greedy replies to the six fixed DEV prompts are recorded beside the true replies, at every checkpoint, in the lab record.
>
> Lead with tokens per second, decode and context separately.

## Cancel probe (run first)

Run 2026-10-28 against `origin/main` `5ae96c7da92754ab080469f6d5f8720d01f8dd74`. **Not cancelled (instrument: a search of main and the open PRs).**

- **No register or slot read exists in this trunk.** `git grep` of `crates/` for a prompt register, query slots or a slot read finds none in `uor-r4-float-train`. The trunk's reads are `flock_read`, `flock_read_content`, `flock_read_hamming` and `flock_read_opening`.
- **Other matches are another line.** "Query slot" and "class-slot reads" appear only in `uor-r4-core`'s native learner (`native_geometric/learner/query_read.rs`, `construction_causal_return_attention.rs`), not in this trunk.
- **No overlap.** The open PRs are this unit's draft, #3343, and antigravity's #3323, which touches no file in this scope.

## Files in scope

- `crates/uor-r4-float-train/src/flock_read_register.rs` (new), and its module line in `src/lib.rs`
- `crates/uor-r4-float-train/src/geometric_stack_native.rs`: the two hooks
- `crates/uor-r4-float-train/src/in_space_long.rs`: config, start, optimizer, save and load, `--inspect` (with the per-slot drop)
- `crates/uor-r4-float-train/src/bin/train-in-space.rs`: the three flags
- `docs/harness/prep/in-space-trunk-to-text*`
- `docs/loop/tasks/in-space-trunk-to-text/**`
- `docs/research/notebook/float-training.md`
- `docs/roadmap/items/in-space-trunk-to-text.md`: its status, if a run meets the item's done-when

## Done when

1. **The register,** with tests:
   - **Host:** the spans and their picks on a hand-built window, ties included. A negative gate, picks taken from the whole window rather than the span, must fail it. Save and load round-trip, and a bad file is refused.
   - **CUDA:**
     - zero `U` and `V` give no change, at both points;
     - one slot's output equals its weighted mean, computed independently;
     - `all` weighs the whole span uniformly;
     - gradients reach every learned part and never the table.
   - With no new flag, every file a run writes is unchanged.
2. **The per-slot drop:** `--inspect` on a register checkpoint also prints its DEV reading with each slot's `U_q` and `V_q` set to zero, and with all of them zero.
3. **The pre-registered runs** (one sheet, a fresh FIT and pre-run review, CHARTER §4.5):
   - (r) `--prompt-register 8`;
   - (g) `--prompt-register 1 --register-select all`.

   Both run on claude-152's (a) recipe (the cell objective, width 256, 4 Hamming heads, the pinned TRAIN), at seeds 1 and 2. Openings are read as the mean of the last three checkpoints. The references are (a) and claude-154's (o), both sealed.
4. **The REPORT** leads with tokens/s. One notebook row, and the result on main.

## Stop and decide (never stop and wait)

- **A probe projects an arm past 2 hours:** profile first, and record why in the sheet. The pick tensor is `(rows, Q, K, width)`, so lower `K` before anything else.
- **The drop shows the register unused** (the reading without it equals the reading with it, as with claude-154's read): report it as the finding. It is not an instrument failure.
- **A run diverges or crashes:** stop it and classify it, then rerun that seed into a new folder.
