# m5-energy

4 records.

| Date | Record | Lab | Outcome | Summary | PRs |
|---|---|---|---|---|---|
| 2026-09-25 | [The plan I recommend: a local language model that stops wasting energy on matmul](../integration/low-energy-plan-2026-09-25.md) |  | plan | Proposal (not a decision) for ternary table-lookup serving with recurrence and exact memory, judged by measured J/token on the M1; energy figures are derived. | [#820](https://github.com/UOR-Foundation/uor-r4/pull/820) |
| 2026-09-28 | [Evidence Lineage and Resealing Correction: `d11-cost-profile-1`](../integration/d11-cost-profile-1-lineage-correction.md) |  | review | Lineage correction: the sealed D11 cost-profile root was edited in place and resealed; it records the hashes, the overwritten fields and the duplicated evaluation record. | [#1479](https://github.com/UOR-Foundation/uor-r4/pull/1479), [#973](https://github.com/UOR-Foundation/uor-r4/pull/973) |
| 2026-10-11 | [The metric the product needs: measured tokens/s, GPU on a pod — and the addressed read loses 1.65× there](../labs/tokens-per-second-2026-10-11/README.md) | deepseek | negative | Measured generated tokens per second on GPU: swapping an MLP for an addressed lookup memory was 1.65x slower, though it cut multiply counts 90 percent. A same-day correction relabelled the arms as MLP-vs-lookup, not read-vs-address. |  |
| 2026-10-18 | [D38 evidence and injective-draft cost provenance — 18 October 2026](../../docs/labs/d38-cost-provenance-docs-2026-10-18/README.md) | unknown | process | Docs fix: energy evidence must separate cache residency from sparsity, and old cost figures for the injective-init draft are relabeled as planning guesses, not measurements. |  |
