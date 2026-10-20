## A. Current UOR-R4 joint model, parameter count from integer config shapes [SOURCE-derived]
total parameters = 1,678,466; tied embedding = 1,048,576 (62.5%); non-embedding = 629,890; weight-terms per token (all affine maps incl. tied head) = 1,672,448
packed 4-bit size = 0.80 MiB; as served today (i16 codes) = 3.20 MiB; F32 = 6.40 MiB  -> all fit in the 12 MiB P-cluster L2

## B. Per-token weight traffic, residency, bandwidth ceiling and physics-floor energy (M1, DRAM mid = 10 pJ/bit)
Energies in millijoules per token. 'mult' = arithmetic with multipliers; 'no-mult' = shift/add or LUT.
| model | fmt (b/w) | access | bytes/token | resident | M1 ceiling tok/s | M1 Pro ceiling tok/s | E_DRAM | E_SRAM | E_arith mult | E_arith no-mult | no-mult saving of total |
|---|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1.7M (UOR-R4 joint) | fp16 (16) | 100% | 3.36 MB | L2 | cache-resident | cache-resident | 0 | 0.0361 | 0.000839 | n/a | n/a |
| 1.7M (UOR-R4 joint) | fp16 (16) | 10% | 336 KB | L2 | cache-resident | cache-resident | 0 | 0.00361 | 8.39e-05 | n/a | n/a |
| 1.7M (UOR-R4 joint) | int8 (8.5) | 100% | 1.78 MB | L2 | cache-resident | cache-resident | 0 | 0.0192 | 0.000168 | 0.000151 | 0.09% |
| 1.7M (UOR-R4 joint) | int8 (8.5) | 10% | 178 KB | L2 | cache-resident | cache-resident | 0 | 0.00192 | 1.68e-05 | 1.51e-05 | 0.09% |
| 1.7M (UOR-R4 joint) | int4 (4.5) | 100% | 944 KB | L2 | cache-resident | cache-resident | 0 | 0.0101 | 0.000168 | 0.000151 | 0.16% |
| 1.7M (UOR-R4 joint) | int4 (4.5) | 10% | 94.4 KB | L2 | cache-resident | cache-resident | 0 | 0.00101 | 1.68e-05 | 1.51e-05 | 0.16% |
| 1.7M (UOR-R4 joint) | int2 (2.5) | 100% | 524 KB | L2 | cache-resident | cache-resident | 0 | 0.00564 | 0.000168 | 0.000101 | 1.16% |
| 1.7M (UOR-R4 joint) | int2 (2.5) | 10% | 52.4 KB | L2 | cache-resident | cache-resident | 0 | 0.000564 | 1.68e-05 | 1.01e-05 | 1.16% |
| 1.7M (UOR-R4 joint) | ternary (1.67) | 100% | 350 KB | L2 | cache-resident | cache-resident | 0 | 0.00377 | 0.000168 | 3.37e-05 | 3.41% |
| 1.7M (UOR-R4 joint) | ternary (1.67) | 10% | 35 KB | L2 | cache-resident | cache-resident | 0 | 0.000377 | 1.68e-05 | 3.37e-06 | 3.41% |
| 30M | fp16 (16) | 100% | 60 MB | DRAM | 1023 | 2864 | 4.8 | 1.55 | 0.015 | n/a | n/a |
| 30M | fp16 (16) | 10% | 6 MB | DRAM | 10233 | 28635 | 0.48 | 0.154 | 0.0015 | n/a | n/a |
| 30M | int8 (8.5) | 100% | 31.9 MB | DRAM | 1926 | cache-resident | 2.55 | 0.821 | 0.003 | 0.0027 | 0.01% |
| 30M | int8 (8.5) | 10% | 3.19 MB | DRAM | 19263 | cache-resident | 0.255 | 0.0821 | 0.0003 | 0.00027 | 0.01% |
| 30M | int4 (4.5) | 100% | 16.9 MB | DRAM | 3639 | cache-resident | 1.35 | 0.435 | 0.003 | 0.0027 | 0.02% |
| 30M | int4 (4.5) | 10% | 1.69 MB | DRAM | 36385 | cache-resident | 0.135 | 0.0435 | 0.0003 | 0.00027 | 0.02% |
| 30M | int2 (2.5) | 100% | 9.38 MB | L2 | cache-resident | cache-resident | 0 | 0.101 | 0.003 | 0.0018 | 1.16% |
| 30M | int2 (2.5) | 10% | 938 KB | L2 | cache-resident | cache-resident | 0 | 0.0101 | 0.0003 | 0.00018 | 1.16% |
| 30M | ternary (1.67) | 100% | 6.26 MB | L2 | cache-resident | cache-resident | 0 | 0.0673 | 0.003 | 0.000603 | 3.41% |
| 30M | ternary (1.67) | 10% | 626 KB | L2 | cache-resident | cache-resident | 0 | 0.00673 | 0.0003 | 6.03e-05 | 3.41% |
| 125M | fp16 (16) | 100% | 250 MB | DRAM | 246 | 687 | 20 | 6.44 | 0.0625 | n/a | n/a |
| 125M | fp16 (16) | 10% | 25 MB | DRAM | 2456 | 6872 | 2 | 0.644 | 0.00625 | n/a | n/a |
| 125M | int8 (8.5) | 100% | 133 MB | DRAM | 462 | 1294 | 10.6 | 3.42 | 0.0125 | 0.0113 | 0.01% |
| 125M | int8 (8.5) | 10% | 13.3 MB | DRAM | 4623 | 12936 | 1.06 | 0.342 | 0.00125 | 0.00113 | 0.01% |
| 125M | int4 (4.5) | 100% | 70.3 MB | DRAM | 873 | 2444 | 5.62 | 1.81 | 0.0125 | 0.0113 | 0.02% |
| 125M | int4 (4.5) | 10% | 7.03 MB | DRAM | 8732 | 24436 | 0.562 | 0.181 | 0.00125 | 0.00113 | 0.02% |
| 125M | int2 (2.5) | 100% | 39.1 MB | DRAM | 1572 | 4398 | 3.12 | 1.01 | 0.0125 | 0.0075 | 0.12% |
| 125M | int2 (2.5) | 10% | 3.91 MB | DRAM | 15718 | 43984 | 0.312 | 0.101 | 0.00125 | 0.00075 | 0.12% |
| 125M | ternary (1.67) | 100% | 26.1 MB | DRAM | 2353 | cache-resident | 2.09 | 0.672 | 0.0125 | 0.00251 | 0.36% |
| 125M | ternary (1.67) | 10% | 2.61 MB | DRAM | 23531 | cache-resident | 0.209 | 0.0672 | 0.00125 | 0.000251 | 0.36% |
| 1B | fp16 (16) | 100% | 2 GB | DRAM | 31 | 86 | 160 | 51.5 | 0.5 | n/a | n/a |
| 1B | fp16 (16) | 10% | 200 MB | DRAM | 307 | 859 | 16 | 5.15 | 0.05 | n/a | n/a |
| 1B | int8 (8.5) | 100% | 1.06 GB | DRAM | 58 | 162 | 85 | 27.4 | 0.1 | 0.09 | 0.01% |
| 1B | int8 (8.5) | 10% | 106 MB | DRAM | 578 | 1617 | 8.5 | 2.74 | 0.01 | 0.009 | 0.01% |
| 1B | int4 (4.5) | 100% | 562 MB | DRAM | 109 | 305 | 45 | 14.5 | 0.1 | 0.09 | 0.02% |
| 1B | int4 (4.5) | 10% | 56.2 MB | DRAM | 1092 | 3054 | 4.5 | 1.45 | 0.01 | 0.009 | 0.02% |
| 1B | int2 (2.5) | 100% | 312 MB | DRAM | 196 | 550 | 25 | 8.05 | 0.1 | 0.06 | 0.12% |
| 1B | int2 (2.5) | 10% | 31.2 MB | DRAM | 1965 | 5498 | 2.5 | 0.805 | 0.01 | 0.006 | 0.12% |
| 1B | ternary (1.67) | 100% | 209 MB | DRAM | 294 | 823 | 16.7 | 5.38 | 0.1 | 0.0201 | 0.36% |
| 1B | ternary (1.67) | 10% | 20.9 MB | DRAM | 2941 | 8231 | 1.67 | 0.538 | 0.01 | 0.00201 | 0.36% |
| 3B | fp16 (16) | 100% | 6 GB | DRAM | 10 | 29 | 480 | 154 | 1.5 | n/a | n/a |
| 3B | fp16 (16) | 10% | 600 MB | DRAM | 102 | 286 | 48 | 15.5 | 0.15 | n/a | n/a |
| 3B | int8 (8.5) | 100% | 3.19 GB | DRAM | 19 | 54 | 255 | 82.1 | 0.3 | 0.27 | 0.01% |
| 3B | int8 (8.5) | 10% | 319 MB | DRAM | 193 | 539 | 25.5 | 8.21 | 0.03 | 0.027 | 0.01% |
| 3B | int4 (4.5) | 100% | 1.69 GB | DRAM | 36 | 102 | 135 | 43.5 | 0.3 | 0.27 | 0.02% |
| 3B | int4 (4.5) | 10% | 169 MB | DRAM | 364 | 1018 | 13.5 | 4.35 | 0.03 | 0.027 | 0.02% |
| 3B | int2 (2.5) | 100% | 938 MB | DRAM | 65 | 183 | 75 | 24.1 | 0.3 | 0.18 | 0.12% |
| 3B | int2 (2.5) | 10% | 93.8 MB | DRAM | 655 | 1833 | 7.5 | 2.41 | 0.03 | 0.018 | 0.12% |
| 3B | ternary (1.67) | 100% | 626 MB | DRAM | 98 | 274 | 50.1 | 16.1 | 0.3 | 0.0603 | 0.36% |
| 3B | ternary (1.67) | 10% | 62.6 MB | DRAM | 980 | 2744 | 5.01 | 1.61 | 0.03 | 0.00603 | 0.36% |
| 7B | fp16 (16) | 100% | 14 GB | DRAM | 4 | 12 | 1.12e+03 | 360 | 3.5 | n/a | n/a |
| 7B | fp16 (16) | 10% | 1.4 GB | DRAM | 44 | 123 | 112 | 36.1 | 0.35 | n/a | n/a |
| 7B | int8 (8.5) | 100% | 7.44 GB | DRAM | 8 | 23 | 595 | 192 | 0.7 | 0.63 | 0.01% |
| 7B | int8 (8.5) | 10% | 744 MB | DRAM | 83 | 231 | 59.5 | 19.2 | 0.07 | 0.063 | 0.01% |
| 7B | int4 (4.5) | 100% | 3.94 GB | DRAM | 16 | 44 | 315 | 101 | 0.7 | 0.63 | 0.02% |
| 7B | int4 (4.5) | 10% | 394 MB | DRAM | 156 | 436 | 31.5 | 10.1 | 0.07 | 0.063 | 0.02% |
| 7B | int2 (2.5) | 100% | 2.19 GB | DRAM | 28 | 79 | 175 | 56.3 | 0.7 | 0.42 | 0.12% |
| 7B | int2 (2.5) | 10% | 219 MB | DRAM | 281 | 785 | 17.5 | 5.63 | 0.07 | 0.042 | 0.12% |
| 7B | ternary (1.67) | 100% | 1.46 GB | DRAM | 42 | 118 | 117 | 37.6 | 0.7 | 0.141 | 0.36% |
| 7B | ternary (1.67) | 10% | 146 MB | DRAM | 420 | 1176 | 11.7 | 3.76 | 0.07 | 0.0141 | 0.36% |

## C. Sensitivity: DRAM energy scenario (low 5 / mid 10 / high 20 pJ/bit), dense access, M1
| model | fmt | E_total low | mid | high (mJ) | arithmetic share (mid) |
|---|---|---:|---:|---:|---:|
| 125M | fp16 | 16.5 | 26.5 | 46.5 | 0.24% |
| 125M | int4 | 4.64 | 7.45 | 13.1 | 0.17% |
| 125M | ternary | 1.73 | 2.77 | 4.86 | 0.45% |
| 1B | fp16 | 132 | 212 | 372 | 0.24% |
| 1B | int4 | 37.1 | 59.6 | 105 | 0.17% |
| 1B | ternary | 13.8 | 22.2 | 38.9 | 0.45% |
| 3B | fp16 | 396 | 636 | 1.12e+03 | 0.24% |
| 3B | int4 | 111 | 179 | 314 | 0.17% |
| 3B | ternary | 41.5 | 66.5 | 117 | 0.45% |
| 7B | fp16 | 924 | 1.48e+03 | 2.6e+03 | 0.24% |
| 7B | int4 | 260 | 417 | 732 | 0.17% |
| 7B | ternary | 96.8 | 155 | 272 | 0.45% |

## D. KV/event-memory read traffic per generated token (2 bytes/element)
| config | KV bytes/position | ctx 256: KV bytes/token | ctx 4096: KV bytes/token | ctx 4096: E_DRAM mJ (10 pJ/b) | ctx 4096 KV / 4-bit weights |
|---|---:|---:|---:|---:|---:|
| UOR-R4 joint (1 layer; K64+V256) | 640 B | 164 KB | 2.62 MB | 0.21 | 2.78x |
| 125M (12 x 768, MHA) | 36.9 KB | 9.44 MB | 151 MB | 12.1 | 2.15x |
| 1B (24 x 2048, MHA) | 197 KB | 50.3 MB | 805 MB | 64.4 | 1.43x |
| 1B (24 x 2048, GQA kv=512) | 49.2 KB | 12.6 MB | 201 MB | 16.1 | 0.36x |
| 7B (32 x 4096, MHA) | 524 KB | 134 MB | 2.15 GB | 172 | 0.55x |
| 7B (32 x 4096, GQA kv=1024) | 131 KB | 33.6 MB | 537 MB | 42.9 | 0.14x |

## E. Calibrated platform view (energy = measured-class power x time), batch-1 decode on M1
Llama-2-7B Q4_0 (llama.cpp, M1 GPU): 14.19 tok/s x 3.79 GB = 53.8 GB/s (88% of 61.4 GB/s); ~0.46 J/token at ~6.5 W -> 15.1 pJ per streamed weight-bit (all-in, GPU domain)
UOR integer session, 3.695 ms/token at an assumed 3.0 W single P-core (+uncore) -> 11.1 mJ/token = 6628 pJ per weight-term (physics floor for its arithmetic: ~0.09 pJ)
UOR integer session, 3.695 ms/token at an assumed 5.0 W single P-core (+uncore) -> 18.5 mJ/token = 11047 pJ per weight-term (physics floor for its arithmetic: ~0.09 pJ)
UOR integer session, 3.695 ms/token at an assumed 6.3 W single P-core (+uncore) -> 23.3 mJ/token = 13919 pJ per weight-term (physics floor for its arithmetic: ~0.09 pJ)

## F. What 'no multiplier' can save vs bit-width and access sparsity (1B dense, M1, mid DRAM)
   fp16 access 100%: total(mult) 212 mJ = 100.0% of fp16-dense; removing multipliers changes it by +0.00%
   int8 access 100%: total(mult) 112 mJ = 53.0% of fp16-dense; removing multipliers changes it by -0.01%
   int4 access 100%: total(mult) 59.6 mJ = 28.1% of fp16-dense; removing multipliers changes it by -0.02%
ternary access 100%: total(mult) 22.2 mJ = 10.5% of fp16-dense; removing multipliers changes it by -0.36%
   int4 access  10%: total(mult) 5.96 mJ = 2.8% of fp16-dense; removing multipliers changes it by -0.02%
ternary access  10%: total(mult) 2.22 mJ = 1.0% of fp16-dense; removing multipliers changes it by -0.36%
