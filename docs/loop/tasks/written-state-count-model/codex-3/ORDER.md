# ORDER: geometric source-to-writer copy on Wick

Lab: codex
Item: [written-state-count-model](../../../../roadmap/items/written-state-count-model.md) (#4063)

Casey requested a native geometric copy action carrying source position, exact token ID and span cursor through the writer, with oracle, learned-selector and copy-disabled Wick arms. The cancel check at 78e7e7c00 found #4084 has counterfactual writer inputs and forced feedback, with no source-bound copy writer. Open M2/P3 is the capability chain. Scope includes the integer D11 research serving interface, standalone discrete CUDA scorer and tests, fixed role panel evaluation and the serving notebook.

State: unsigned integer words bind a selected source occurrence and cursor, original prompt fence, exact source ID, and active/released state. Copy reads only original prompt memory, emits the exact integer through the writer, advances once on emission, then returns to native generation. Pin a prefix-only stopping rule before evaluation. Use one transition contract on the integer serving path and AOT CUDA behind raw-word per-step differential. The oracle chooses the occurrence only. A learned selector must have demonstrably fitted state and must choose without asked-span labels. Copy-disabled uses the unmodified release prediction. Fixed artifact geometry and selector state serialize losslessly; transient cursor words can be snapshotted for exact replay. Frozen base weights; any new fit uses a disjoint training set and pre-registered GPU integer updates. No float master or optimizer.

Done when both released seeds run on the same fixed role panel with independent source/ID/cursor and prediction replay, the first divergent step per row, paired old-versus-new answer IDs and free continuation. Preserve all attempts. Stop on label leakage, parity failure or missing provenance; repair instruments before interpreting outcomes. No paid hardware, production default switch, phase promotion or gold-token feedback.
