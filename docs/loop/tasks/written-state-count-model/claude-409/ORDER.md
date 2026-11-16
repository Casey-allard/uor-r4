# ORDER: written-state-count-model, claude-409 (decision card: the written-state track, the plan row, and the native-space items S0–S6)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md). Track: written-state-count-model (no item file of its own yet).
Base: `origin/main` `ef1f4a641` (claude-408, #4045). Doc-only.

## Why this unit

Casey, 16 Nov 2026, set the goal: a 1B-param-equivalent model, trained on wick's RTX 4090 and served on the M1 laptop, with no matmul and no softmax, in the project's geometry (PLAN.md, owner directions, 16 Nov) [claude]. The same day the order of gaps before the 1B run was set at PLAN.md:38–39: (1) the Pick read, (2) conjunction memory, (3) a readout from the pick, (4) carry across windows, (5) the native-space contract, (6) the corpus trainer, D87 ingest and the 1B run [claude].

Step (1) is done (claude-408, #4045) [claude]. Before step (2), the lab's top session ran a principal-investigator review with two red teams on the whole path to the goal [claude]. Both red teams returned SURVIVES-WITH-FIXES, and the PI's final DECISION answers every blocker and major [claude]. Its central finding: the project cannot score 1e9 coordinates on one 4090 (900–2,000 GPU-days by survey B2.3), but it can count and error-write 1e9 integer cells in hours per pass [claude, est]. Steps (3)–(6) as written assume the scored 1e9-coordinate run, so they need a decision card before P0's other items start [claude].

Under CHARTER §5.1 and D68, a change of route, scope and stop rule is a decision card that the lab's top session approves after its own whole-project decision review; it does not wait on the owner, and it is logged for Casey, who can reverse it [claude]. A change to §3 or to the caps is not made here [claude].

This unit is P0 item 1 (the card) and P0 item 2 (the native-space ORDER items) of the DECISION [claude]. It writes no code and runs nothing on a GPU, so §4.5 asks for no sheet [claude].

## Native-space items (D85)

The full items are in [NATIVE-SPACE.md](NATIVE-SPACE.md). In short [claude]:

1. **State schema.** S0 tokens, S1 transport (the D11 discrete trunk), S2 n-let rows, S3 state rows, S4 conjunction rows, S5 session copy (Pick), S6 mixer, all integer, all inside `TrunkTables`.
2. **Shared operators.** `PrimeRouteKey`, the counting kernel, the integer Brier error-write, the Pick read and PRLL scoring, each one definition behind CUDA, the CPU oracle and `discrete_d11`.
3. **Replay oracle.** The SHA-256 of the canonical sorted (key, token, count) triple stream, equal across the GPU writer, the CPU oracle (on a 1–10% slice) and the serving loader; raw-word equality of every carried field on ≥ 64 prompts.
4. **Update rule.** Count, then integer Brier error-write, in canonical key order.
5. **Lossless format.** The canonical triple stream plus a frozen compact form with a lossless round-trip to the `SparseCountTable` oracle form.

These are a contract for P1–P5. This unit implements none of them, so it is an inventory and contract unit, not a training run [claude].

## Files in scope

- `docs/loop/tasks/written-state-count-model/claude-409/**` (ORDER, NATIVE-SPACE, REPORT);
- `docs/research/reviews/pi-review-2026-11-16/**`: the ten review files copied byte-for-byte, plus a README index;
- `docs/loop/tasks/written-state-count-model/claude-409/DECISION.md`: the decision card (`docs/integration/DECISIONS.md` is an owner path, so its index row is not written here);
- `PLAN.md`: one dated owner-directions row, status marks on the two 16 Nov rows, and one table under "Approved sequences".

## Stop list

- No code, no build beyond the harness, no GPU, no pod, no wick job.
- `CHARTER.md` and `AGENTS.md` are not edited. §3 and the caps are not changed.
- The owner's words in PLAN.md are not deleted or reworded; superseded steps are marked by a note that points to the D-entry.
- `docs/harness/native-space.json` is edited only if it has a per-line registry this track must join.
- No owner-queue item is acted on (P6 donor data, the §3.2 APM question, the label, the conditional §3.1 proposal).
- The review copies are not edited, except to redact a secret or email if the grep finds one.

## Done when

1. ORDER is the first commit on the branch and REPORT the last.
2. The decision card, the PLAN rows and the P0–P5 table are in, and the review folder holds the ten copies (byte-identical, checked with `cmp`) and its README.
3. The secret and email grep over `docs/research/reviews/` has no hits.
4. `UOR_LAB=claude uor-harness check` shows no new failure against `origin/main`.
5. `UOR_LAB=claude uor-harness loop pr-check --local` passes, or its output is reported.
