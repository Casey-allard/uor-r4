# ORDER: m4-lawful-likelihood-bpb, deepseek-9 (repair: the counted CHANGES on the design note)

Lab: deepseek. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (#2861). Base: `e9748f179e5427b992cd41c7777b830eb67b7e00`.

## Goal

Answer the counted `uor-review v1` CHANGES record on PR #2940 at `e9748f1` in the design note: the zero-mass/infinite-bits case, how bits are computed, division and the D11 contract, the crate choice, and the geometry claim plus its missing citations, together with the two lows (tie rule/permutation and uniform-mass exactness; count-table smoothing).

## Done when in deepseek-8 (history)

- deepseek-8 ORDER `0a4549689` added the design-only unit: `docs/labs/m4-lawful-likelihood-bpb/README.md` (potential-ratio lawful likelihood), posted on #2861, with no code and no measurement. Review at `e9748f1` returned CHANGES.

## Cancel probe (run first)

Class **instrument** (a design record). The CHANGES record is counted at the current head `e9748f1`, so the repair is needed. Each of the five required edits is a real defect in the note, not a rewording: `floor(phi*2^48/sum)` can floor a valid token to zero mass (infinite bits); the note never names a `log2`; it does not say whether the artifact or a GPU scorer divides, or that `phi*2^48` needs u128; it names no crate; and it claims a geometric normalization the mechanism board does not support.

## Files in scope

The item's seven globs and this task folder. This repair edits `docs/labs/m4-lawful-likelihood-bpb/README.md` and the task folder only.

## Done when

- Every token gets at least mass 1 (base-1 plus proportional split of `2^48 - V`, ties by token id), property 1 reads `>= 1`, and a V=4096 worst-case control with the true token at the minimum has finite bits.
- The bits computation is named (integer fixed-point log2, stated error bound) and the float alternative is ruled out by §3.7.
- The division is placed: the artifact emits potentials only and the ratio runs in the GPU scorer (or `stack_div_u128` under audit), with u128 arithmetic for `phi*2^48`.
- A crate is chosen with the reason (integer crate scoring path plus the discrete crate's eval-corpus reporting).
- The geometry claim cites `docs/mechanisms/README.md`'s Normalization row and the flock rank-weight normalizer, and is softened to what the board supports.
- Both lows are stated in the note: token-id ties with a tied control, and exact uniform masses only when V divides `2^48`; the count baseline's smoothing is stated.
- The two PM questions are asked on #2028 (is a linear full-alphabet ratio inside §3.2; is `-log2(mass)` a measurement), and the note says they are open.
- `loop gates --full` is GREEN, and `loop report --check` and `loop pr-check --local` pass.

## Stop and ask if

- The PM's reading of §3.2 or of the measurement classification is needed before the note can call itself lawful (it is asked, and the note holds it open).
- A bar, status, constraint or spend change is needed; the held-out stream must be named now; GPU work becomes implied.