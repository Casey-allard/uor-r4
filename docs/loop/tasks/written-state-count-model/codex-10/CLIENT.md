# [codex] A text turn through the normal discrete CLI

The released M1 seed-1 model gives the same reply, “Got it, I'll remember that.”, to all three inputs below. It emits ID 1155 first and EOS 1 at position 9. On the first two the requested answer starts with ID 1074. The output therefore diverges before the shared first word of the requested values. The third example asks for an explanation. These are ordinary single-turn client examples. They use the released greedy predictor, with no labels, oracle, learned copy bank or scored change to the predictor.

| Input to the one-shot text client | Expected behavior |
|---|---|
| Tava has green box. Tavi has green bag. Leave out Tava. What item did the record mention for Tavi? | Return green bag |
| Tava has green box. Tavi has green bag. Leave out Tavi. What item did the record mention for Tava? | Return green box |
| How do you know when a question is unclear? | Generate a relevant explanation |

`uor-chat --discrete MODEL --tokenizer TOKENIZER --say TEXT --max-tokens 16` encodes a complete one-turn prefix with BOS and literal dialogue roles (default v2), loads the artifact, feeds each committed prediction back to the integer evaluator, and stops on EOS or the length limit. The JSON includes the prompt IDs, raw emitted IDs including EOS, the text, stop reason and both file fingerprints. The two role-swap prompts matched the full archived transfer token arrays byte-for-byte, including BOS, at case IDs ending in `late-before-none-shared-Tavi` and `late-before-none-shared-Tava`. An initial build omitted BOS by calling the suffix encoder. Comparison with the Wick tape exposed the missing BOS. A regression test now checks the complete prefix, and all four records here come from the corrected build. The raw-ID command preserves its fixed-length parity transcript. Replaying the first text turn through the raw-ID CLI matched every ID up through the observed EOS. A synthetic binary fixture checks exact prefix encoding, EOS truncation, fixed-length raw-ID generation and rejection of a tokenizer whose bytes were altered. The two historical M1 pairs use recorded digests. New text-capable artifacts can bind the tokenizer digest in their header.

The fourth record repeats the first input through the newly frozen 379,223,660-byte S2 carrier on the client. Its artifact digest changed to `640fa6884d258343e38804282405820999f40f23554848b0d2561b9833d478d9` and its reported schema changed to the S2 trunk. The prompt and all ten emitted IDs were identical. The decoder has not yet been fitted to mix the new count rows into prediction. [Count replay](COUNT-RESULT.md)

[Three baseline records and S2 read-back](CLIENT-TURNS.json) · [Command source](../../../../../crates/uor-r4-integer/src/bin/uor-chat.rs) · [Normal-entry tests](../../../../../crates/uor-r4-integer/tests/discrete_chat_cli.rs)
