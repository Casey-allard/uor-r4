# [codex] ORDER — connector continuation and EOS after exact copy

Lab: codex
Item: [written-state-count-model](../../../../roadmap/items/written-state-count-model.md) (#4063).

References [#4063](../../../../roadmap/items/written-state-count-model.md), P3. Owner requests the WordStart baseline, fixed entity endpoints, connector continuation and EOS after exact copy individually and jointly. Cancel probe: #4090 contains neither a continuation treatment with WordStart fallback nor an EOS emission on copy completion. Verify baseline against its sealed rows.

Keep released seed artifacts, prefixes, WordStart decisions, query/fact anchor pairs and endpoints. Carry integer source, token ID, cursor, fence and release reason through actual-feedback CUDA decoding. Use a bounded native source decision based on observable prefix state and training-only evidence. EOS must be an emitted token on witnessed completion of a nonempty copy; preserve native decoding on abstentions and guard cutoffs. Use independent Rust representation/replay and AOT CUDA, integer routes and lossless raw ID/state serialization. No gradient or floating optimizer. Freeze the definition in the sheet before evaluation. Compare per case across four arms on Wick, both seeds and transfer, reporting first wrong token, exact-through-EOS, sampled witnesses, repeat and historical controls. No production default or milestone change. Stop and repair on parity, leakage, invalid provenance, or baseline drift.
