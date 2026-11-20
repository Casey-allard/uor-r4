# [codex] Finite text-framing check for the order-ten KN transport fixture

Use the ordinary text entry point on Wick, with the frozen tokenizer, protocol 2, and the already constructed KN-only D11. Run the four fixed text inputs below at temperature zero for one generated ID. Record the original framed prefix, its last ten tokens, the complete 4,095-candidate mass digest, maximum, winning original ID and tie count. Replay the framed IDs directly and require identical generated IDs and score digests. No text answer or quality score selects the inputs, weights or threshold.

| Case | User text |
|---|---|
| Distant copper | Mira keeps copper. Leon keeps silver. What does Mira keep? Give the answer as exactly one word from the facts above. |
| Distant silver | Mira keeps silver. Leon keeps copper. What does Mira keep? Give the answer as exactly one word from the facts above. |
| Near the | Complete this short phrase with the |
| Near about | Complete this short phrase with about |

The two distant factual assignments must produce distinct framed prefixes and identical final ten IDs. A pure order-ten distribution must then match exactly, including all scores and their winner. If those suffixes differ, the text pair fails its own premise. The near control must have distinct final ten IDs. Report whether its scores and winner diverge. A common distribution on that pair is a valid outcome because both histories can back off to the same shorter context. This finite client check identifies which original information is visible to the count component through real framing. The learned Pick weights and persistent-state comparison have their own tests.
