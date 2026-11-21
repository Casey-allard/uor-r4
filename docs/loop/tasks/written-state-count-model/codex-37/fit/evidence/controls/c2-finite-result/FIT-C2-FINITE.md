# FIT for the finite D37 C2 device selector

Reviewer [codex] tune37, 20 Nov 2026. Independent of the C2 source author. **TEST FITNESS: FIT** under a separate parent GO and Wick lock for the staged synthetic UORT only.

The Wick control-stage-v2 source and artifact manifests verified with strict SHA-256. Stage manifest `e718ced93a91b1b0f9969c6901c9e604c0d3aaf1924f1044a9204958e36b5a0c`; finite dispatch `e7361e752de2ddb70828c5e0a51c26e3ce078fc54f840886c69afed1ce716815`; sheet `780827878fcbcaf71466e821ac01f016e91f31a2a919e3a39b315387c8c3b229`; C2 device selector `5a9d132b5f7957750ab351607d1e9d06f707e56bb55a6d4411a99201d6c20d7f`. The C2 SASS scan found zero FADD, FMUL, FFMA, MUFU, HMMA or MMA mnemonics.

The host validates UORT framing and constructs prefix intervals from the latest BOS and the 192-ID window bound. The sm_89 kernel reads only those prefix positions, searches from newest to oldest and emits the window-relative predecessor or the absent word. It emits one word per original non-BOS ordinal through EOS. The independent Rust oracle checks every fixture word, with explicit witnesses for the latest of repeated exact pairs and eviction of an older match. The fixture also exercises BOS reset, EOS and empty history. Hash-pinned fixture bytes, fresh output, no active foreign GPU process, and the registered shared lock bound the dispatch.
