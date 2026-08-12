# Lean Contract Bridge

Generated: 2026-02-17T21:57:15.713466+00:00

- Lean file: `research/formal/lean/PrimeRiemannBridge.lean`

## Contracts

### L1ArtifactContract

- Transfer bound contract for |E| with bridge term.

| field | value | source | json path |
|---|---:|---|---|
| c1.a_ref | -0.0013474693715061251 | `research/output/o1_theorem_closure_pack_2026-02-17.json` | `constants.a_ref` |
| Ctr | 0.6878610001589391 | `research/output/o4_theorem_closure_pack_2026-02-17.json` | `uniform_constants.C0_ref` |
| Ctr_components.C0_ref_O4 | 0.6334997766223893 | `research/output/o4_theorem_closure_pack_2026-02-17.json` | `uniform_constants.C0_ref` |
| Ctr_components.b_ref_abs | 0.05436122353654979 | `research/output/o1_theorem_closure_pack_2026-02-17.json` | `constants.b_ref` |

### L2ArtifactContract

- Bridge growth bound contract for |H(x)| <= CH*(log x)^2.

| field | value | source | json path |
|---|---:|---|---|
| CH | 8.710122086201695 | `research/output/o5_integrated_implication_draft_2026-02-17.json` | `integrated_o5_theorem.constants_snapshot.C_H` |
| A_H | 2.0 | `research/output/o5_integrated_implication_draft_2026-02-17.json` | `integrated_o5_theorem.constants_snapshot.A_H` |

### O2Constants

- Locked explicit zero-count constants.

| field | value | source | json path |
|---|---:|---|---|
| nbound_c1 | 0.1038 | `research/output/o2_theorem_closure_pack_2026-02-17.json` | `citation_lock.constants.nbound_c1` |
| nbound_c2 | 0.2573 | `research/output/o2_theorem_closure_pack_2026-02-17.json` | `citation_lock.constants.nbound_c2` |
| nbound_c3 | 9.3675 | `research/output/o2_theorem_closure_pack_2026-02-17.json` | `citation_lock.constants.nbound_c3` |
| nbound_h | 1.0 | `research/output/o2_theorem_closure_pack_2026-02-17.json` | `citation_lock.constants.nbound_h` |
| citation_url | https://arxiv.org/abs/2107.06506 | `research/output/o2_theorem_closure_pack_2026-02-17.json` | `citation_lock.url` |

### O3Constants

- Direct O3 closure constants.

| field | value | source | json path |
|---|---:|---|---|
| A_offabs | 0.0 | `research/output/o3_l_offabs_theorem_closure_2026-02-17.json` | `proved_constants.A_offabs` |
| C_offabs | 0.03292827711413939 | `research/output/o3_l_offabs_theorem_closure_2026-02-17.json` | `proved_constants.C_offabs` |
| k_abs | 0.005725212627704354 | `research/output/o3_l_offsign_theorem_closure_2026-02-17.json` | `proved_constants.k_abs` |
| A_diag | 0.0 | `research/output/o3_l_diag_theorem_closure_2026-02-17.json` | `proved_constants.A_diag` |
| C_diag | 1.0 | `research/output/o3_l_diag_theorem_closure_2026-02-17.json` | `proved_constants.C_diag` |
| A_E2 | 0.0 | `research/output/o3_l_asm_theorem_closure_2026-02-17.json` | `proved_constants.A_E2` |
| C_E2 | 1.1195893906678458 | `research/output/o3_l_asm_theorem_closure_2026-02-17.json` | `proved_constants.C_E2` |

## Consistency

- `o2_citation_url`: `https://arxiv.org/abs/2107.06506`
- `o2_citation_label`: `HSW2021`
- `l1_ctr_candidate_nonnegative`: `True`
- `l2_ch_candidate_nonnegative`: `True`
