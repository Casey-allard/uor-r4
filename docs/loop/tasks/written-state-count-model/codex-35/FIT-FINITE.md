# [codex] D35 independent FIT for the planted finite device witness

**FIT for the planted finite launch only.** The non-author review used the charter, D35 ORDER and sheet, CUDA source, Wick runner, AOT inventory and the planted inputs. Wick verified both manifests with strict SHA-256 checks. The staged CUDA source, fixture and build and run scripts match the worktree. The image targets SM89 and contains two application kernels and no embedded PTX. The application SASS inventory uses integer arithmetic. Device execution was pending at review.

| Reviewed Wick input | SHA-256 |
|---|---|
| `device/aot/stage.sha256` | `c312e15c344b6b9a19e359515b388829d13ef9019f1ea6986d7d73440594c043` |
| `device/aot/paired-value-rank` | `f689d43b6eb108650a0f53a1d4e61754fe0f81b5e0612d4e677010167fb16be2` |
| `device/finite/packets.sha256` | `5f91984325e566d06adba2e625cdaa8069f3595758b8c770a004db6884d0ca06` |

The fixture hard-codes the bank and all ordered pair words separately from the CUDA implementation. The scalar contextual points (6, 0, −2) have equal local geometry. The planted target-to-rival differences are +6 and +8. The first row must select original onset 2 forward and onset 6 in reverse, and moving those endpoint points between positions flips those choices. The equal-coordinate row must abstain with both distances at 6. Extreme coordinates require an exact forward zero and a reflected minimum of `32 × (65535 + 4294967295) + 6` for the positive extreme; 32-bit accumulators would corrupt the result. Empty and explicit singleton choice/abstention rows test metadata transfer, with the singleton floor supplied by the host. Independent byte comparisons cover the full bank and score output on primary and cold runs.

The adverse controls inject an out-of-range signed contextual bank word, add a singleton marker to a three-member pool, and change a planted expected winner. The runner checks two exact rejection reasons and requires the mutated winner bytes to differ. The first two controls exercise input validation. The third exercises the byte comparison and does not by itself validate writer replay. The finite witness exercises the CUDA displacement, orientation, reduction, strict comparison and serialized results given planted coordinates and masks. Native projection, real original-token writer replay and split admission require their own evidence.

Use the reviewed AOT stage and finite packet manifest in one digest-pinned admission alongside this receipt and the lab GO. Confirm Wick's standing Codex lock and an idle GPU inside the runner, use a fresh result directory, and preserve the exit results and manifest. A failed comparison or adverse control is an instrument result. A pass clears the finite device prerequisite for a separate TRAIN review.
