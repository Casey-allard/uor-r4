# [codex] D29 paired original-address readout

Calibration recovered 10 ordinary exact replies and lost 0 against D28 fitted-whole. It introduced 7 wrong supported source selections and 0 unsupported factual writes. Relative to the shuffle, it recovered 0 and lost 0. The true negative table had 19 cross-family-supported signatures out of 30, and the sorted-key shuffle changed 18.

The registered advancement bar failed. The calibrated arm released new wrong sources and its exact-reply count did not exceed the shuffled control.

| Carrier | Lane | Requested entity | Original assertion heads | Exact through EOS | Correct source | Wrong source | Unsupported write | EOS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | ordinary | 36/49 | 37/49 | 0/47 | 0/47 | 0 | 0 | 47/49 |
| baseline | person | 49/49 | 49/49 | 0/47 | 0/47 | 0 | 0 | 47/49 |
| baseline | assertions | 49/49 | 49/49 | 0/47 | 0/47 | 0 | 0 | 47/49 |
| calibrated | ordinary | 36/49 | 37/49 | 10/47 | 10/47 | 7 | 0 | 49/49 |
| calibrated | person | 49/49 | 49/49 | 18/47 | 18/47 | 3 | 0 | 49/49 |
| calibrated | assertions | 49/49 | 49/49 | 18/47 | 18/47 | 3 | 0 | 49/49 |
| shuffled | ordinary | 36/49 | 37/49 | 10/47 | 10/47 | 11 | 0 | 49/49 |
| shuffled | person | 49/49 | 49/49 | 18/47 | 18/47 | 7 | 0 | 49/49 |
| shuffled | assertions | 49/49 | 49/49 | 18/47 | 18/47 | 7 | 0 | 49/49 |

The assertion-head column compares both original token endpoints and order. A wrong source means a released original address outside the supported value starts. An unsupported write means an original source was released for a conflict or absent target.

- baseline. Forced gold matched 47/47 through EOS, forced competing matched 0/47, and disabled matched 0/47. The ordinary lane emitted 0 exact replies from original starts beyond three connector pieces.
- calibrated. Forced gold matched 47/47 through EOS, forced competing matched 0/47, and disabled matched 0/47. The ordinary lane emitted 3 exact replies from original starts beyond three connector pieces.
- shuffled. Forced gold matched 47/47 through EOS, forced competing matched 0/47, and disabled matched 0/47. The ordinary lane emitted 3 exact replies from original starts beyond three connector pieces.

## Paired changes and early values

- baseline to calibrated, ordinary. Recovered `[29010,29011,29016,29017,29018,29109,29111,29113,29115,29141]` and lost `[]`. New wrong source `[29000,29001,29002,29108,29110,29112,29114]` and new unsupported write `[]`.
- baseline to calibrated, person. Recovered `[29010,29011,29016,29017,29018,29104,29105,29106,29107,29108,29109,29110,29111,29112,29113,29114,29115,29141]` and lost `[]`. New wrong source `[29000,29001,29002]` and new unsupported write `[]`.
- baseline to calibrated, assertions. Recovered `[29010,29011,29016,29017,29018,29104,29105,29106,29107,29108,29109,29110,29111,29112,29113,29114,29115,29141]` and lost `[]`. New wrong source `[29000,29001,29002]` and new unsupported write `[]`.
- baseline to shuffled, ordinary. Recovered `[29010,29011,29016,29017,29018,29109,29111,29113,29115,29141]` and lost `[]`. New wrong source `[29000,29001,29002,29005,29007,29008,29009,29108,29110,29112,29114]` and new unsupported write `[]`.
- baseline to shuffled, person. Recovered `[29010,29011,29016,29017,29018,29104,29105,29106,29107,29108,29109,29110,29111,29112,29113,29114,29115,29141]` and lost `[]`. New wrong source `[29000,29001,29002,29005,29007,29008,29009]` and new unsupported write `[]`.
- baseline to shuffled, assertions. Recovered `[29010,29011,29016,29017,29018,29104,29105,29106,29107,29108,29109,29110,29111,29112,29113,29114,29115,29141]` and lost `[]`. New wrong source `[29000,29001,29002,29005,29007,29008,29009]` and new unsupported write `[]`.
- shuffled to calibrated, ordinary. Recovered `[]` and lost `[]`. New wrong source `[]` and new unsupported write `[]`.
- shuffled to calibrated, person. Recovered `[]` and lost `[]`. New wrong source `[]` and new unsupported write `[]`.
- shuffled to calibrated, assertions. Recovered `[]` and lost `[]`. New wrong source `[]` and new unsupported write `[]`.

Stage 4 cleared 4 genuine early gold candidates in the ordinary calibrated lane. These original candidates were unresolved in D28. Their row, fact addresses, source addresses, signed distances, radii and downstream outcomes are retained in the structured readout.

- Row 29000 cleared original source 18 under fact span `[11,15]` with negative distance 24 and radius 96. Baseline and calibrated released sources were `null` and `17`.
- Row 29001 cleared original source 18 under fact span `[11,15]` with negative distance 28 and radius 96. Baseline and calibrated released sources were `null` and `17`.
- Row 29012 cleared original source 16 under fact span `[11,13]` with negative distance 68 and radius 100. Baseline and calibrated released sources were `null` and `null`.
- Row 29013 cleared original source 16 under fact span `[11,13]` with negative distance 76 and radius 100. Baseline and calibrated released sources were `null` and `null`.
- Row 29000 actual token IDs changed from `[1155,302,311,14,340,1919,1203,383,16,1]` to `[486,1254,2294,1]` through stop. The first mismatch changed from `{"offset":0,"expected":1254,"actual":1155}` to `{"offset":0,"expected":1254,"actual":486}`.
- Row 29010 actual token IDs changed from `[1155,302,311,14,340,1919,1203,383,16,1]` to `[1861,1689,85,1]` through stop. The first mismatch changed from `{"offset":0,"expected":1861,"actual":1155}` to `null`.
- Row 29108 actual token IDs changed from `[1155,302,311,14,340,1919,1203,383,16,1]` to `[3680,2084,1]` through stop. The first mismatch changed from `{"offset":0,"expected":738,"actual":1155}` to `{"offset":0,"expected":738,"actual":3680}`.
- Row 29109 actual token IDs changed from `[1155,302,311,14,340,1919,1203,16,1]` to `[738,1455,1]` through stop. The first mismatch changed from `{"offset":0,"expected":738,"actual":1155}` to `null`.

## Identical-local boundary pairs

- chair rows `[29052,29053]` have identical original IDs from the fact head onward and different preceding words. Gold local sources are `[4,5]` for `["home office chairs","office chairs"]`. Baseline, calibrated and shuffled supplied-head local sources are `[null,null]`, `[null,null]` and `[null,null]`.
  Each arm gives the two members identical supplied-head candidate scores.
- magazine rows `[29050,29051]` have identical original IDs from the fact head onward and different preceding words. Gold local sources are `[5,6]` for `["back issues","issues"]`. Baseline, calibrated and shuffled supplied-head local sources are `[null,null]`, `[null,null]` and `[null,null]`.
  Each arm gives the two members identical supplied-head candidate scores.

## Role order and next IDs

- baseline, role. Requested 4/16, excluded 0/16, joint 0/16. Exact ordinary 0 and supplied person 0.
- baseline, role_direct. Requested 2/2, excluded 1/2, joint 1/2. Exact ordinary 0 and supplied person 0.
- calibrated, role. Requested 4/16, excluded 0/16, joint 0/16. Exact ordinary 4 and supplied person 12.
- calibrated, role_direct. Requested 2/2, excluded 1/2, joint 1/2. Exact ordinary 1 and supplied person 1.
- shuffled, role. Requested 4/16, excluded 0/16, joint 0/16. Exact ordinary 4 and supplied person 12.
- shuffled, role_direct. Requested 2/2, excluded 1/2, joint 1/2. Exact ordinary 1 and supplied person 1.

The factor matrix keeps requested-person choice, fact order and request/exclusion clause order for both role families. The per-row TSV gives each original pair of entity endpoints, assertion heads, released source, first actual ID, first mismatching ID and terminal EOS. The historical D21 role result provides context. This comparison uses the fixed D28 fitted-whole parent and registered D29 prompts. [D21 historical role result](../../codex-21/REPORT.md)

[Per-row original addresses and reply tape diagnostics](paired-rows.tsv) · [Complete paired and signed evidence](paired-summary.json)
