# Admission-anchor audit source implementation plan

Source-only unit under owner autonomous-source direction and PM6263591101 Option A.
Design: ../specs/2026-10-19-admission-anchor-audit.md.
DeepSeek owns PR2777 sheet; AGY owns consumer. No CUDA qualification, rental, model evaluation
or training release follows from this plan. The exact final FIT sheet and source will be bound
before any numerical operation.

Current registration pin: FIT recording #2780 delivered at
`185f7171204507896fd5439ab0c93767db60c495`; sheet SHA256
`fcb770e66d38f607d25e16dade14ac7a6223dd8af0ccc9ec22c80db71f2c3450`.
The design is FIT; controller/evaluator source delivery, numerical qualification,
costed PM dispatch and Casey's typed D40 release remain separate.

1. Metadata slice: attention_anchor.rs + lib.rs export. Construct genuine TRAIN successor
   anchor from original windows, canonical CSV original eligible ordinals and separate contiguous
   anchor ordinals; validate source/context/window/hash/membership. RED then GREEN host tests.
2. Lifecycle slice: attention_audit.rs host receipt/state machine. Separate attempted, provisional,
   committed and rejected domains; baseline/64/final-tail scheduling; whole-interval rejection,
   generation/epoch identity, strict half-rate and consecutive-rejection stops. RED/GREEN tests
   use synthetic metadata only and preserve failure evidence.
3. Device slice: narrow attention_anchor_device.rs production CUDA evaluation and aggregate metric.
   Exclude DEV/reference evaluation; preserve immutable before/after checkpoint/epoch identity.
   Host refusal tests and synthetic CUDA-boundary checks; real device qualification NOT_RUN.
4. Controller slice: distinct registered mode/schema and attention controller integration.
   Authenticate new sheet/anchor/review/owner scope; preserve generic eval-corpus and old releases.
   Synchronous speculative intervals, durable commit/reject, new Device binding on restoration,
   append-only failed evidence and advanced attempted cursor. No selective-isolation heuristic.
5. Verification/delivery: focused host tests, appropriate Rust/static checks, required exact-head
   architecture/spec/Rust/security reviews, canonical record/current-state and protected delivery.
   Report implementation separately from real CUDA qualification, timing and scientific result.
   Restore/preserve source and own claim delta with verified custody before own-only cleanup.

Each slice stays in this single integrated source PR. Source-only tooling does not reset D21
or move a headline. Do not ship a metadata slice as a trained/admission-qualified mechanism.
Only one implementation agent owns a code slice at a time; spec review precedes quality review.
