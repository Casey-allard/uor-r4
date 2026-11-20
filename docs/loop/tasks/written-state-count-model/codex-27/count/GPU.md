# [codex] Finite RTX 4090 replay of the capped S2 table

After the [host packaging and serving pilot](README.md), an RTX 4090 independently reproduced all recorded reading decisions and 4,096 exact Q32 masses per prefix on each of two processes. The exercised inputs were the one fixed map prefix read from the 125,998,700-byte capped carrier and the ten original D23 planted prefixes. This checks a finite reader differential. It leaves held predictive quality, timing and the P2 comparisons open.

| Item | Bound identity |
|---|---|
| Active 125,998,700-byte D11 | `81a36d7ed720952ce72f0b0535a1ad40afe33b69a3f02a0a7ecb483b8d3d0750` |
| Extracted 39,462,220-byte rows | `6b4eef7e5ca905738009d4504f1b46210fb715ddafd34371a0b7f15db0aa2eca` |
| Original D23 AOT SM89 executable | `4fde13cb7e9edafd284a6ac120d6260e92241a3421050419117b99b07897e4ee` |
| Original disassembly | `0ae4f4c8d7a8259752be7278abf47ebc0dded82f46c02e577302e5eb64368c69` |
| Input manifest | `dda663beeeffdc2cf81e8390ec5a87f354e458febdde53262fb06a07b8838dba` |
| Output manifest | `5d639ae3a001f55882bfd36a6f637a9274f0a193e310c171095c1efcefee6cff` |

Wick extracted the rows from the pinned final carrier and independently reconstructed the request and expected distribution with D23's Rust oracle. The oracle checks the ordinary host trace's chosen key, order, ordered ID/count pairs, whole Q32 support, top eight and winner. D23's planted rows, queries and oracle compare exactly to their original D23 manifest. Run instances on `vast-61542024` used NVIDIA GeForce RTX 4090 `GPU-be79bd7b-6b2b-839a-782a-78666ce01210`, driver `615.78.08`, compute capability 8.9.

The map prefix selected order two and key `0201c2ca000000000000000000000000`, reproducing counts `(201,256), (223,128), (287,256), (799,4096)`, the top-eight ranking and the top-one token 799 with mass 3,598,518,889. The primary, cold and sorted local-oracle map JSONs share SHA-256 `fa4b713642e8d23510494ea55fb4164a64451f179e6fec7c860e34a8392d5c27`. The equivalent planted outputs share `21f43a21658f5e6cfe0480a720e9528055e61c693df21353ac41b701e7bd1e78`. Each pass checked 45,056 exact mass words and every recorded route, read and ranking on the eleven prefixes. Every mass vector sums to 2^32. The device wrapper exited zero. Host readback independently checked all six output files and compared both downloaded device oracle files to the original local oracles.

The GPU reader receives the serialized row slice and the fixed prefix IDs. This evidence covers the retained row lookup, backoff and integer mass arithmetic on those prefixes. The ordinary client pilot separately covers loading the full D11 and tokenizing text. No count table was trained, evaluated on a held panel or selected against a query here. [D23 differential and build scope](../../codex-23/REPORT.md) · [Custody](../evidence/STORAGE.md)
