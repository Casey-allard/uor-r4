# [codex] Non-author FIT for D37 finite two-shape optimizer

FIT for the invented-row finite CUDA check on the admitted Wick RTX 4090. This review ran no CUDA executable. The finite stage qualifies the optimizer and its two bucket shapes only after the dispatcher returns success and seals the results.

| Reviewed object | SHA-256 |
|---|---|
| Sheet | `4f4eb0d2a4a5c222a731df040a765554c437161fc93722219392abf96d312583` |
| Dispatcher | `0f55e875b14583aca2b23886f3212fe3d77ee536fb74596964e484f8a0e651ea` |
| Stage inventory | `4f97e8a8e1a4e4e54dd99e8b920054bb8bbe3d864e3fb29be997da0315b30a89` |
| Device executable | `3b7c91ad698a2ef1eb81eca2d3c2f42fd9a188e1a55899bea827bc8bc0326aaf` |
| Host oracle executable | `b9bdfbad9305d19fda31440a7e6b27655f71c1e5205ba3dbb68afeedf159e5ec` |
| CUDA source | `2eb6b1224dbf9408d8d31742432265b2c0be301ec325969c8476974b192c8d1b` |
| Host oracle source | `2db2114e69af74311b77f26056c6c4a7e38618b3b00eaeefd3dbee98319c790a` |
| Device SASS | `a7beb3b095d4a57c00b9985904d1d1d09bc3610174858758142594a92fb344e0` |

I checked the complete stage inventory on Wick and regenerated the SASS digest from the admitted executable. The executable embeds two sm_89 cubins and exposes no PTX. The instruction census for initialize, accumulate, maximize and probe_mass contained integer, memory and control instructions. The four FLO instructions were FLO.U32 integer leading-bit operations. No floating-point arithmetic or tensor instructions appeared. The live Wick device reported the admitted UUID and capability 8.9. The dispatcher checks the GO and its pinned inputs, takes the shared lock, checks for foreign compute, disables PTX JIT and requires a fresh output.

I independently read both 15,384-byte row tapes. Their invented rows include the closed and exact buckets, and exercise distinct positive-distance classes at ordinals 3 and 4. Ordinal 10 has Pick mass 2^48 and bucket 527 or 879 according to the carrier. The probe ordinal is exactly the LE64 encoding of 10. The truncated tape is the identical prefix with eight fewer bytes. Its required diagnostic comes from full length validation before CUDA allocation or trace creation. The Rust verifier recomputes 97 independently scored states using the shared typed integer operator and rejects the first incorrect state row. It also checks the final selected bucket and Q48 mass. The total comparisons are 717,024 and 1,195,040 Q32 words plus 97 Q12 sums per arm. Shell syntax passed.

Real-R preparation and CUDA fitting need their own review and GO. The retained real-preparation build log contains compile errors and is outside this finite admission.
