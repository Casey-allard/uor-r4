# Replay the unchanged writer on the six value policies

Use the original seed-1 assertion-agreement carrier, tokenizer, original onset bank, CUDA fitted bank and CUDA append-only augmentation. `extract` checks the frozen carrier digest. `pack` verifies all component digests, reloads the finished model and requires byte identity when reconstructing the baseline. Both whole-word and legacy policies consume the same signed state for a given bank. The original role bank, span state and paired writer are embedded in every carrier. The registered holdout uses unannotated user text in the ordinary client.

The `stage` command emits two little-endian files per policy, one device-input row and one host expected-output row per visited assertion, plus the map to original row and lane. The three lanes use the ordinary selection, supplied original person, and supplied person plus assertion indices. A lane with no matching factual head contributes zero device visits. Each packet contains the original selected head boundary, query, fence and legal candidates. The legacy variant drops only candidates farther than its three-token connector window from this packet. Its result must match the unchanged old reader. Fitted and augmented client replays must regenerate the identical input bytes for a given enumeration. This is an exact differential for native value scoring on captured candidates. Original whole-word discovery, role binding and ordinary generation execute in the client.

Run the local process after the FIT and GO, using fresh roots:

```sh
d28-value-replay extract BASELINE ORIGINAL.bank
d28-onset-fit seal BASELINE TRAIN_STAGE CUDA_FITTED.raw FITTED.bank
d28-value-replay augment ORIGINAL.bank FITTED.bank CUDA_AUGMENTED.raw AUGMENTED.bank
d28-value-replay pack BASELINE TOKENIZER BANK legacy|whole NEW.d11
d28-value-replay stage NEW.d11 PANEL NEW_STAGE
dispatch/clients.sh CHAT d28-value-replay TOKENIZER SIX_MODEL_DIR PANEL RUN
```

The native writer grade reselects each original ordinary source. It requires the ordinary client's original prompt IDs, loaded artifact, roles, copy decision, all emitted IDs, every cursor word and stop reason to match. It separately supplies a full value from the original prefix or a valid competing start, then checks their complete forced tapes. Unsupported cases lack a literal output key and are counted by inappropriate source release. The raw captured tapes and model-specific stage words survive even when no ordinary reply improves.
