# [codex] Optional P1(c) certificate code FIT

**FIT for the optional Wick certificate** after the top-session capacity review and with the existing Wick GPU lock. Scope is the registered proof for the exact frozen count family, using the already verified D87 DEV-tune frame and fit. The measured P1(c) result stands on its original evidence.

| Reviewed artifact | SHA-256 |
|---|---|
| Certificate source archive | `6f7ec23cc9d419b435c8c81f40feb9a52f638b4562b21bdf9943a98fe8d3ed4d` |
| CUDA SM89 worker | `d975d2fc0531c5eab5bd7d1bcc29817a0ae3e7fd291ef2f5645284444f8e430c` |
| Rust certificate orchestrator | `237697ff3ee03ffad52c29625b398631ebfba6ed9739cf6ff12d6a4c7b5c13dc` |
| Independent certificate verifier | `5004ce3bb994bf00686a26eeb9b30cd02694db59617afdb0d8c8e425b6503f77` |
| Frozen likelihood source included by the synthetic oracle | `26c59f3abbfb8a83258dfdcf7aed6867828588fd2a40f0c1b33cd33fd88b1d9c` |
| Frozen primary verifier called by `--reverify` | `0e3e688c746072430925baf50099dd13c480b67274e625f4fd15f508ba29b5ec` |

The staged archive identifies commit `43e630cf5411170da4e5b1d70378b06d17036031`, tree `75dedf876cffedd2189c3644f5dd7d78a6a07d1a`. I checked archive and executable digests and compared the staged build sources byte for byte to the corresponding archive members. The method is pinned by CERTIFICATE.md SHA-256 `4b1e8d56293fe7b8870b5206e2d96349b7fbe5e75c3742e65d008acfc42f1634`.

- The CUDA worker computes the exact floored mixed Q48 anchor mass and Q12 score, rounds each directional term upward, and reduces every lane allowed by the bucket mask, including a live lane with zero row mass or floor weight in the anchor. The final Rust verifier reconstructs ordered bucket residuals, rejects missing lanes and negative residuals, and rounds the total descent allowance upward. The proof uses the simplex vertex maximum of the linear upper bound on the directional gain, then the registered 3/2 logarithm factor and 4 Q12 units per row. For fewer than 2^24 rows, the directional total is below 2^105 and its scaled descent numerator is below 2^118. The per-row maximum yields a second lower bound by monotonicity of the fixed Q12 scorer.
- The initial draft allowed an unavailable interior best-pass trace word to set the feasible upper bound. The candidate uses saved feasible endpoints and allows a pass-zero anchor only after device rescoring. It records the observed score and missing interior state separately. Pure KN and the restricted near-KN endpoint are feasible in the full simplex. Every certificate run on corpus data must re-run the pinned primary verifier with the current fit files and require exact receipt agreement. The device inputs and the independent arithmetic check bind the sealed payload, current fit, and exact executables. The audited CUDA worker supplies the complete corpus directional sums. The independent Rust row oracle rescored the synthetic case.
- The 131-row finite CUDA witness matched the widened Rust arithmetic in all buckets, failed with the floor-for-ceil fault, and passed again with the restored worker. Clean and restored 33-row synthetic device receipts matched byte for byte. I ran the exact staged independent verifier on the existing synthetic identity-mode packet with `--reverify`. Its output reproduced the saved receipt byte for byte at SHA-256 `4e7bc7aed1e003c68052bc4a12906a8d071838301bd25541689fefb5cb371506`, with interval [11,287, 11,500] Q12. The deliberately faulty receipt with the matching mutant worker digest failed its direction check. The exact checkout's Rust verifier suite passed 20 tests.

Recheck the pinned hashes, use the same frozen frame and primary receipt, and allow for simultaneous parent and worker staging buffers in the Wick host reservation. Apply the exact branch thresholds only if the verified full interval lies in one regime. The synthetic interval is finite instrument evidence.
