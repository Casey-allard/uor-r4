# [codex] Blind pair ranking and anchor-only control

The native baseline and repair share the frozen content scorer. Each emits the query-role and fact-role energies for every token; eligible pairs share an exact token ID, with at least one intervening token. The baseline minimizes their summed energy with ties ordered by ascending query and fact positions. The repair also requires the same whole-word predicate already enforced by the downstream span resolver. The two tokenizer word spans must be disjoint, their raw IDs identical and the selected interior offsets equal. The comparison admits punctuation singleton spans, as before. Each full-ID sequence is compared before ranking using the frozen tokenizer class bank. No values or positions from the grader enter either blind route.

The Wick implementation compiles the original content CUDA code with an optional pair check. Its START/CONT span check is compared with the serving Rust definition. Per bare prompt and blind arm, every site cost and all three selected anchor words agree between native and AOT SM89. The downstream span, value and continuation cubins match the previously qualified images byte-for-byte. Every intermediate device choice, mask and emitted source address matches the frozen Rust path. Each blind file binds a bare tape digest, a released D11 and the three fitted-state digests. The nonempty fixtures include a colliding name fragment, a later valid winner at the same energy, an empty eligible set, an already-valid winner, an overlapping word and a singleton nonword.

The oracle command consumes a separately projected row with only a prefix fingerprint and two anchors. Both are requested word-end positions. All real predictions and repeats were sealed before the projection was constructed. The projection files were subsequently checked against identical ordered grade and bare prefixes. Oracle supplied no value location or connector length. The same downstream decisions and checks ran on every oracle prompt.

All 24 output files reproduced independently, one per arm, panel and released seed. Each of 13,824 baseline document rows has bit-identical frozen spans, WordStart and final CE decision to #4093 or the independently frozen challenge prediction. The baseline and treatment retained the same site costs throughout. Downstream source selection recovered 441/563 nulls on focal transfer, 433/555 on challenge, 266/461 on primary and 178/452 on the earlier transfer. The other former abstentions selected an incorrect address. [Source census](source-census.json) · [Finite device evidence](finite/fixture.txt) · [Pinned device programs](finite/cubins.sha256) · [Blind file fingerprints](main/blind.sha256) · [Oracle fingerprints](main/oracle.sha256)

The standalone scorer binary builds with feature `entity-pair-chain` on the SM89 machine. Both blind commands consume only a bare tape and frozen state.

```sh
entity-pair-chain fixture
entity-pair-chain baseline MODEL CONTENT_STATE SPAN_STATE EXACT_STATE BARE NEW_OUTPUT
entity-pair-chain repair MODEL CONTENT_STATE SPAN_STATE EXACT_STATE BARE NEW_OUTPUT
entity-pair-chain oracle MODEL CONTENT_STATE SPAN_STATE EXACT_STATE BARE PROJECTED_ANCHORS NEW_OUTPUT
```

The final-source Wick rebuild, after a tuple-destructuring edit in the legacy GPU wrapper, produced a different executable digest and the same four AOT cubins. All finite cases passed again. All 24 full prediction files from the rebuilt executable matched the sealed originals byte-for-byte. [Final executable digests](final-source/binaries.sha256) · [Final prediction digests](final-source/prediction.sha256)
