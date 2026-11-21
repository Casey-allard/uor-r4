# [codex] D37 full-C count and reserved-R S5 fit

Fit A1 and A1s from the same order-ten compact count features and independent
causal identity Pick reads for all 15,947,584 non-BOS original targets in
4,060 reserved TRAIN records. EOS is included. The full-C TRAIN SHA-256 is
`cd3adfe949ee34a62bc8d5e48592e091fcece796618d7910d1667d014a4bae2a`.
The 3,912,334,481-byte physical count SHA-256 is
`064b691a54895771df1a957b2f97c91163990b6c6b6e09504c43285a97c49ef8`.
Original R is
`5875b173ad19aefbdd38929af8d88c15344333e941f44128d5e0a67b50102a0a`.
The seed-one D11 trunk is
`04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`.
The independently verified count-capture manifest is
`b7fd6424e0f13c201836c0686138aafb49231d059f35fb098e80d150a81eefd8`.
Both cold-width captures matched the primary `UORCBF01` tape at
`add9aa306fd70d87e02048e2a2a32d23f6f9e9be89bb8ff8011c64690042f4dd`.
The tape includes eleven u32 raw target counts, eleven u64 raw context
totals and one u64 final KN Q48 mass per scored target.
Its position is the zero-based non-BOS ordinal of the pinned UORT.

The registered Pick input uses at most 192 earlier original IDs, reset at
each BOS. The CUDA trace gets the last prefix ID as its forward placeholder.
Target ID is read only after the successor has been chosen, to decide the
hit. The producer recomputes that hit and records the full Pick tape with
prefix, route, tie and count parity on selected original ordinals including
the first targets, EOS, length-192 windows and batch seams. The typed rule
classifies the same observed distance under both arms. Standard classes are
closed, distance zero, and other fired distances. The split has closed,
zero, one, two, and three-or-more classes. Full row tapes retain Q48 native
expert words, including the pick-one mass on a hit and zero on a miss.
A cardinality, binding, replay or class discrepancy stops the affected run.
A planted class-boundary distance change must alter the five-class bucket.
The fixed finite optimizer result covers actively scored buckets 527 and 879.

The optimizer uses Kn start and 96 integer updates, with 97 Q12 scores and
all Q32 weight words preserved. Replay reloaded weights against the device
bucket and target mass at each retained ordinal, including closed-gate
injections and suppression of positive Pick contributions. On success the
arm receipts preserve 7,392 and 12,320 weight words respectively, bound to
the fit manifests without quantization. A fit yields one eligible state per
arm. A failed replay yields an instrument failure. Arm choice and the
original-ID argmax test require separate client and paired DEV-tune receipts.
A0, C1, C2 and C3 retain their registered definitions. The primary identity
dispatch covers A1 and A1s; control fits get their own admission and are
required before the registered selection. No DEV-tune or DEV-test target is
read here.

Run on one idle Wick RTX 4090 under the shared GPU lock, with GPU JIT disabled.
Require the exact admitted sm_89 optimizer, finite result and reviewed loaded
Pick image. Generate a fresh mode-specific output directory and retain partial
failures. Check at least 10 GiB of host memory and 8 GiB of GPU memory
available and at least 8 GiB of output capacity. Identity Pick is the longest
operation. Historical same-R C1 feature preparation took 270 s and the GPU
optimizer 2.37 s per arm. Full-C count features are already captured. Wall
time, RAM and device profile for this preparation remain unmeasured.
