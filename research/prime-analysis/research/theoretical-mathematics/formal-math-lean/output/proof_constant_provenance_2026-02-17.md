# Proof Constant Provenance

Generated: 2026-02-17T21:48:05.451161+00:00

- Status: `theorem_constants_indexed`
- O5 statement: Using only theorem-closed O1/O2/O3/O4 requirements, derive final RH-equivalent implication statement.

## Constants

| symbol | value | obligation | requirement | artifact field | citation |
|---|---:|---|---|---|---|
| C0_ref_O1 | 0.9102883687683553 | O1 | O1-R1/O1-R2/O1-R3 | `research/output/o1_theorem_closure_pack_2026-02-17.json::constants.C0_ref` |  |
| a_ref | -0.0013474693715061251 | O1 | O1-R1/O1-R2/O1-R3 | `research/output/o1_theorem_closure_pack_2026-02-17.json::constants.a_ref` |  |
| b_ref | -0.05436122353654979 | O1 | O1-R1/O1-R2/O1-R3 | `research/output/o1_theorem_closure_pack_2026-02-17.json::constants.b_ref` |  |
| m_ref | 512 | O1 | O1-R1/O1-R2/O1-R3 | `research/output/o1_theorem_closure_pack_2026-02-17.json::constants.m_ref` |  |
| nbound_c1 | 0.1038 | O2 | O2-R1 | `research/output/o2_theorem_closure_pack_2026-02-17.json::citation_lock.constants.nbound_c1` | HSW2021 |
| nbound_c2 | 0.2573 | O2 | O2-R1 | `research/output/o2_theorem_closure_pack_2026-02-17.json::citation_lock.constants.nbound_c2` | HSW2021 |
| nbound_c3 | 9.3675 | O2 | O2-R1 | `research/output/o2_theorem_closure_pack_2026-02-17.json::citation_lock.constants.nbound_c3` | HSW2021 |
| nbound_h | 1.0 | O2 | O2-R1 | `research/output/o2_theorem_closure_pack_2026-02-17.json::citation_lock.constants.nbound_h` | HSW2021 |
| k_tau | 0.08482120688755276 | O2 | O2-R3 | `research/output/o2_theorem_closure_pack_2026-02-17.json::tau_rate.k_tau` |  |
| c0_tau | 166.2789198268843 | O2 | O2-R3 | `research/output/o2_theorem_closure_pack_2026-02-17.json::tau_rate.c0_tau` |  |
| A_offabs | 0.0 | O3 | O3-R1 | `research/output/o3_l_offabs_theorem_closure_2026-02-17.json::proved_constants.A_offabs` |  |
| C_offabs | 0.03292827711413939 | O3 | O3-R1 | `research/output/o3_l_offabs_theorem_closure_2026-02-17.json::proved_constants.C_offabs` |  |
| k_pos | 0.0038168084184695417 | O3 | O3-R2 | `research/output/o3_l_offsign_theorem_closure_2026-02-17.json::proved_constants.k_pos` |  |
| k_neg | 0.0019084042092348122 | O3 | O3-R2 | `research/output/o3_l_offsign_theorem_closure_2026-02-17.json::proved_constants.k_neg` |  |
| k_abs | 0.005725212627704354 | O3 | O3-R2 | `research/output/o3_l_offsign_theorem_closure_2026-02-17.json::proved_constants.k_abs` |  |
| eps_sign | 0.0038168084184695417 | O3 | O3-R2 | `research/output/o3_l_offsign_theorem_closure_2026-02-17.json::proved_constants.eps_sign` |  |
| neg_over_abs_cap | 0.0019084042092348122 | O3 | O3-R2 | `research/output/o3_l_offsign_theorem_closure_2026-02-17.json::proved_constants.neg_over_abs_cap` |  |
| A_diag | 0.0 | O3 | O3-R3 | `research/output/o3_l_diag_theorem_closure_2026-02-17.json::proved_constants.A_diag` |  |
| C_diag | 1.0 | O3 | O3-R3 | `research/output/o3_l_diag_theorem_closure_2026-02-17.json::proved_constants.C_diag` |  |
| A_E2 | 0.0 | O3 | O3-R4 | `research/output/o3_l_asm_theorem_closure_2026-02-17.json::proved_constants.A_E2` |  |
| C_E2 | 1.1195893906678458 | O3 | O3-R4 | `research/output/o3_l_asm_theorem_closure_2026-02-17.json::proved_constants.C_E2` |  |
| a_ref | -0.0013474693715061251 | O4 | O4-R1/O4-R2/O4-R3 | `research/output/o4_theorem_closure_pack_2026-02-17.json::uniform_constants.a_ref` |  |
| b_ref | -0.05436122353654979 | O4 | O4-R1/O4-R2/O4-R3 | `research/output/o4_theorem_closure_pack_2026-02-17.json::uniform_constants.b_ref` |  |
| C0_ref_O4 | 0.6334997766223893 | O4 | O4-R1/O4-R2/O4-R3 | `research/output/o4_theorem_closure_pack_2026-02-17.json::uniform_constants.C0_ref` |  |
| C_delta | 4.6749702791170504e-05 | O4 | O4-R1/O4-R2/O4-R3 | `research/output/o4_theorem_closure_pack_2026-02-17.json::uniform_constants.C_delta` |  |
| C_H | 8.45323261007565e-07 | O4 | O4-R1/O4-R2/O4-R3 | `research/output/o4_theorem_closure_pack_2026-02-17.json::uniform_constants.C_H` |  |

## Consistency Checks

- `a_ref_consistent_O1_O4`: `True`
- `b_ref_consistent_O1_O4`: `True`
- `k_abs_matches_offabs_reference`: `True`

## Notes

- C0_ref appears in both O1 and O4 with different numerical values due to different closure contexts; kept as C0_ref_O1 and C0_ref_O4.
- Citation-locked constants are tagged with citation label/url for independent source audit.
