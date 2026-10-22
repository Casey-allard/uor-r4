# ORDER: m4-lawful-likelihood-bpb, deepseek-10 (repair: lawfulness wording, the rank-weight precedent, and the emissions claim)

Lab: deepseek. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (#2861). Base: `effe1e9c1b3961608092eda425017036c6901bff`.

## Goal

Answer the counted same-lab fresh-context CHANGES record on PR #2940 at `effe1e9c1` (posted by this lab's fresh-context reviewer, which is allowed only because the PR is doc-only): (1) stop calling the construction lawful while the §3.2 card is open; (2) complete required edit 5 by citing the Flock rank-weight normalizer as an existing read-weight normalization instead of calling a rank-weight table "no-normalization"; (3) correct the emitted-probabilities claim (the model does emit Q48 masses through the banned exp-table softmax at `model.rs:3811/4541`, and the artifact would emit potentials only under this proposal). Also take the reviewer's lows: log2 precision wording, count-baseline Q48 conversion, crate-hosting consistency, REPORT size figure, and post the revised note on #2861.

## Done when in deepseek-9 (history)

- deepseek-8 `0a4549689` added the design-only note; Claude's CHANGES at `e9748f1` produced deepseek-9, which fixed the zero-mass case, named the bits computation, placed division, chose crates and softened the geometry claim.

## Cancel probe (run first)

Class **instrument** (a design record). The CHANGES record is counted at the current head `effe1e9c1`, so the repair is needed. All three blocking findings are real: the note contradicts itself on lawfulness while the §3.2 card is open; it labels the softmax-free learned rank table "no-normalization" although `stack/flock.rs` ships `RAW_RANK_WEIGHTS_Q16`/`RECIPROCAL_Q32`/`rank_table_q31` and `archive/mechanisms/flock-rank-reads-2026-10-20.md` defines `w_r ∝ 1/(r+1)`; and `model.rs:3811/4541` shows the model already emits Q48 masses through an integer softmax over `tables.exp`.

## Files in scope

The item's seven globs and this task folder. This repair edits `docs/labs/m4-lawful-likelihood-bpb/README.md` and the task folder only.

## Done when

- The note is explicitly conditional: it calls nothing lawful until the §3.2 card is answered; the heading and property 6 say so.
- Required edit 5 is complete: the Flock rank-weight normalizer is cited as an existing read-weight normalization (`archive/mechanisms/flock-rank-reads-2026-10-20.md`; `stack/flock.rs` constants), and the board's Normalization GAP is left standing as a model-mechanism gap.
- The emissions statement is exact: the current integer model emits Q48 masses through the banned exp-table softmax, so it emits no **lawful** probabilities; the proposed construction's artifact would emit potentials only.
- The lows are stated: log2 fixed-point width/rounding and its error bound; the MLE baseline's exact Q48 conversion and zero-count behaviour; the crate-hosting split phrased consistently; the REPORT's size figure measured, not estimated.
- The revised note's summary is posted on #2861.
- `loop gates --full` GREEN; `report --check` and `pr-check --local` pass.

## Stop and ask if

- The PM's §3.2 reading or measurement classification is needed before the note can call itself lawful (it stays conditional and the cards are open).
- A bar, status, constraint or spend change is needed; GPU work becomes implied.