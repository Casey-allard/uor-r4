# D25 forensic context readback

The 21 frozen D25 assertion prefixes had one post-gate vector. The pre-gate
Relation words and recurrence codes varied. The joint-pointer selection chose
token ID 1 on all rows, and the packed roots of that token's codebook entry
and the post-gate vector were both 8195 for every row. This localizes the
shared output to the gate's token selection in this panel. The query token
alone was already constant (ID 16), while the recurrence, Relation vector,
pointer and joint score still carried variation into the selection.

| Split | Rows | Distinct pre-gate vectors | Distinct recurrence codes | Distinct causal successors | Distinct winning scores | Distinct winners | Distinct post-gate vectors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| TRAIN | 8 | 6 | 8 | 5 | 8 | 1 | 1 |
| Held | 9 | 6 | 9 | 6 | 9 | 1 | 1 |
| Exposed | 4 | 4 | 4 | 4 | 4 | 1 | 1 |
| Combined | 21 | 12 | 21 | 11 | 21 | 1 | 1 |

In the served D11 path, `apply_joint_pointer_gate` selects a token using
the query, Relation read, recurrence and causal successor. It uses each
selected token block as the destination for a signed BFS action from the
corresponding Relation root, then snaps to an orbit root. The 21/21 equality
between the measured winning codebook roots and actual output roots confirms
that this step yields the winning token prototype on the frozen panel. The
selected token can vary with a different prefix. This readback did not test
other prefixes or another scoring or selection rule.

The Wick RTX 4090 observation held the shared lock and checked for an empty
GPU compute-process list before launch. The three immutable context packets
matched the original staged checksums. All three post-gate files matched the
original outcome files byte for byte. This readback performed no fit.

| Receipt | SHA-256 |
| --- | --- |
| SM89 executable | `7b83cc81c246e724d054e5ec525775154cb4a98f43c67070e204af526015784a` |
| Build manifest | `a6252f7f411dd3c8a981ab9d2ef88a575b9ce584710c1e7b9c1fc2365ffb701d` |
| Readback manifest | `411426cddfbde7544ab20f73c93a5be81ba84d86cad69d13fd2a2a765972e6d8` |
| TRAIN raw trace | `7e5513b4f4b818440e8d84a46939a16368db2e07e6b54956d85c494278864122` |
| Held raw trace | `56ec285ec27532d3f8be52aee14796ab9db2f4de21574c836cf4f01d5c361f1b` |
| Exposed raw trace | `63b1a9b51277c5a2956511fa57e7df9d65751b043cc07cfa1c9aa61c2b48e74c` |

Local readbacks and manifests are in the D25 `forensic-context-01` directory.
