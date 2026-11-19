# Blind source-boundary premise check

Two independent English readers saw the full texts, with no gold answers, token spans or workspace access. Their versions appeared in opposite orders. Both used the same stem after the opening sentence.

> This is the note for the first job. Nora can spray paint cans. Which objects does Nora's note say she can work on? Quote the object wording.

| First sentence | Reader A | Reader B |
|---|---|---|
| The work order says to disinfect now and to repaint later. | “paint cans” | “paint cans” most likely; “cans” also possible |
| The work order says to repaint now and to disinfect later. | Ambiguous between “cans” and “paint cans” | “cans” most likely; “paint cans” also possible |

“Spray” can take “paint cans” as its object, while “spray paint” can take “cans.” The context shifts the preferred reading. It does not identify a unique original start on both texts. Reject this pair for single-answer scoring. No tokenizer collision or selector score is claimed. Keep it as a candidate for an ambiguity/abstention control with separately defined acceptance. A positive source discriminator requires independent agreement on two genuinely different writable starts and evidence that the reference packet is identical.
