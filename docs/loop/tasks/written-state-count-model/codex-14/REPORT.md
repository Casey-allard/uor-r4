# [codex] P1(c) on Wick, the full-count fit and its distance from the reference

## What did not work, first

Starting all 13 count experts at equal weights still uses more bits overall than pure Kneser–Ney (KN) at full TRAIN dose. CUDA reproduced the earlier 25 objective words exactly, including the final 128,726,996,235 Q12 bits. Fitting on even records and testing on odd records and vice versa also cost more than pure KN overall. The split gap exceeded half the in-sample gain. Of 1,182 active non-KN expert weights observed across nonempty buckets, 970 stayed at one throughout the deciding near-KN fit. The convergence band measures what this optimizer did in 24 passes. A separate mathematical argument is required to bound the optimum.

The first exporter implementation rejected a zero-KN synthetic row that the prior public API accepted. The corrected exporter enforces the positive mass rule on its protected frame while preserving the old caller. The original nine ceiling tests, all 21 host KN tests and the detached collector suite passed after correction. The finite detector also caught two deliberate source faults. Reading adjusted counts in place of raw counts changed an expected ML mass; removing the remainder from a weight update failed the exact normalization check.

## What changed

At source `b0328fb996f225ef1ebbc4bf62172aa15a5a7c6b`, the Wick collector seals the original query-order KN mass, eleven raw ML masses and a uniform mass, with bucket, byte credit, novelty and record parity for each non-BOS target. The fitter performs responsibilities, bucket updates, objectives, pure-KN scoring and both held folds on the RTX 4090. The private frame can be reopened independently of the original count kernel. The reference experiment fits its principal weights on the same DEV-tune rows it scores. Its outputs are reference weights and bit counts. Deployable S2/S5/S6 fitting uses a disjoint TRAIN-only fitting set and its own artifact and evaluation. [Order](ORDER.md) · [Method](SHEET.md) · [Exact-code FIT](CODE-FIT.md) · [Pre-run decision](PRE-RUN.md)

## Measurements

Each row uses the same cut for TRAIN and DEV-tune. BPB is total Q12 bits divided by 4,096 times credited bytes. The three measured cuts have different target sets. Only the last row reads the P1(c) branch.

| TRAIN dose | DEV-tune targets | Primary pass | KN BPB | Near-KN 13-expert BPB | Near-KN 2-expert BPB | Uniform-start 13-expert BPB | Held-parity BPB | Novel KN / primary BPB |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1‰ | 8,311 | 40 | 2.305596 | 2.293807 | 2.304686 | 2.283304 | 2.352220 | 2.445081 / 2.433820 |
| 10% | 1,066,742 | 32 | 1.691806 | 1.691201 | 1.691590 | 1.690356 | 1.691832 | 1.986543 / 1.985981 |
| 100% | 10,807,417 | 24 | 1.432318 | 1.432076 | 1.432197 | 1.433182 | 1.433325 | 1.747151 / 1.746890 |

Full TRAIN was 2,091,560,001 native words. DEV-tune had 21,928,475 credited bytes, including 15,995,822 novel bytes across 7,566,174 novel targets. The exact reduction was 21,783,936 Q12 units overall and 17,056,206 on novel text. The measured primary/reference ratio was 0.9998307 overall, above the strictly greater than 0.90 geometry-critical cutoff. The registered four-drop sensitivity projection from pass 24 gave 128,626,506,270 Q12 bits, still on that side of the cutoff. Both near-KN arms met containment overall and novel on every cut. Records include the full endpoint trajectories, best observations, restricted-start scores, floor dwell, fold controls and individual receipts. [Cold](receipts/cold-direct/verified.json) · [Cold reopen](receipts/cold-reopen/verified.json) · [10% sizing](receipts/sizing-10pct/verified.json) · [Full](receipts/full-d87/verified.json) · [Full trajectories](receipts/full-d87/trajectories.json) · [Full floor dwell](receipts/full-d87/floors.json)

The full collection took 396.91 s on Wick, reached 16,066 MiB of sampled per-process GPU memory and 11,141,376 KiB peak host RSS. The subsequent full GPU fit took 1.63 s including input verification and staging; those phases use different memory lifetimes. The fitter reported 1,145,586,202 bytes for its frame arrays, and peak fit-process host RSS was 2,434,476 KiB. One-second process sampling missed fit residency. A second full fit under Nsight Systems reproduced the entire result JSON, SHA-256 `6cee2002b4700b1ed6884e88d6d73b349be32f74855c3cf6c25bbded181312e6`. The captured trace identifies 96 full-mask and 48 restricted-mask CUDA responsibility kernels, as well as update and reduction kernels. This is a cached, controlled single-machine measurement of an offline instrument. The run spent $0 on rental. [Sizing decision](receipts/sizing-decision) · [Full dispatch](receipts/full-decision) · [Device samples](receipts/full-d87/observed-device.json) · [GPU kernel trace summary](receipts/profiled-reopen/kernel.csv)

### How much could better count weights still save?

For this order-10, 176-bucket, 13-lane Q32 mixture with its exact mass floors, the minimum attainable all-target score on these specific rows is enclosed by **127,285,173,165 to 128,627,655,182 Q12 bits**. Dividing by the identical KN score gives **0.98939548 to 0.99983067 × KN**. Even an optimal mixture in this family can save at most **1.06046%**. The lower bound is above 0.90 × KN by 8.9395 percentage points. This settles the geometry-critical branch for the frozen count family on DEV-tune. Novel-only optimality, larger count models and the behavior of any geometric candidate remain separate questions.

The best feasible upper witness was the near-KN endpoint at pass 24. The tightest lower witness was the *uniform-start* endpoint at pass 24, despite its worse realized score. On the saved full frame the integer CUDA certificate summed directed, rounded-up tangent terms for every present expert in every bucket, including floor-weight lanes. The Rust verifier reran the original fit verifier against the pinned binaries and reconstructed the interval from those sums with widened arithmetic and explicit log-rounding allowance. On finite examples it recomputed contributions independently, exhaustively compared finite minima and rejected a deliberately rounded-down witness. The full-data lower bound relies on the identified device implementation and aggregate witnesses; the independent verifier rereads the protected frame and fit identities and reconstructs the aggregate proof. [Bound and assumptions](CERTIFICATE.md) · [Independent pre-run review](certificate/CODE-FIT.md) · [Reconstructed interval and anchors](receipts/certificate/verified.json)

## Checks run at the head

The four protected input-file hashes, all adjusted per-order digests, the historical uniform and restricted fits and the pure-KN scores matched on each dose. On full data the KN mass tape SHA-256 is `ae8518d552bc42d8f1c3602680bd77586d0cd9c237962fbf5e2b36704b6bb2fd`. The new raw count/total lookup digest is `2e11bea6fb1a22faf719f46009309ef2258d0092c02d1769d6c7fc319d5113a2`. The full packed payload holds 1,178,008,453 bytes, SHA-256 `1559c7f4e0b6b4ead7e3a240b5726b9e3462ef2b1b239e327642ae3984f526be`. It remains on Wick in the private `~/uor-codex/20261118-full-count-ceiling/runs/full-d87/frame` directory. The separate 1‰ collections had byte-equal payloads, manifests, seals and fit result files. The independent verifier rejoined every original query row with its BOS-derived fold, novelty and byte count. The finite independent Rust/CUDA comparison matched all 6,388,144 witness bytes over 1,202 synthetic targets. [Frame identity](receipts/full-d87/manifest.json) · [Finite fitter checks](fit/README.md) · [Independent replay and historical controls](verify/README.md)

## Skills used

The native-space-cycle and uor-research-architect reviews scoped operator identity, device execution, hypothesis, controls and evaluation. This reference fit binds the count baseline. Native language-model training and serialized client replay have their own acceptance criteria.

## Still open

P1(a) needs retained triples and loader replay for the final compact carrier. P1(b) needs a frozen artifact within 4 GB at or below 1.03 times the sealed DEV-test reference. P1(d) needs the 7,002 dialogue-token baseline within its paired band. The next deployable experiment keeps count support fixed, compares S5 on and off, fits lawful S6 masses on disjoint TRAIN records and ranks the full legal union before choosing the output. The previous raw Pick plus greedy-count switch could reach only 5,666 of 14,287. P2's equal-byte geometric state comparison still requires P1(a,b). The independent role diagnostic still targets selection of the requested occurrence in ordinary role-reversed questions. The frozen writer produced 448/448 exact supported answers when supplied just that occurrence. [Current sequence](../../../../../PLAN.md#approved-sequences) · [Count reachability and role-to-writer evidence](../codex-13/REPORT.md)

A second lab opened [#4103](https://github.com/UOR-Foundation/uor-r4/pull/4103) after this order was public. Its 96-pass schedule and different bound can be compared at matched source, KN and pass words. This report draws its results from the Wick run at the stated head.

## Plain summary

On the full tuning set, starting near the best known count predictor settled the earlier failed measurement. The fitted mixture used almost exactly as many bits as that predictor. The roadmap therefore places the burden of the larger quality gain on geometric state and learned reads. The model still has to show that gain on held-out text and produce useful replies through the ordinary client.

The public report carries the device witness and exact reconstructed result. The approved evidence store contains the pinned compressed raw fit receipts, finite controls, timing and Nsight capture, with a byte-verified re-download. The protected target frame stays in the private Wick directory. [Store receipt](receipts/store.json) · [Raw certificate](receipts/certificate/device-certificate.json) · [Roadmap decision](DECISION.md)
