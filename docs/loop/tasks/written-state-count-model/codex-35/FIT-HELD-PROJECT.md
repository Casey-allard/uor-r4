# [codex] D35 independent FIT for blind HELD candidate projection

**FIT for candidate projection with the pinned sterile Wick runner under a separate project-held GO.** This is a zero-decode projection against the frozen D26 parent. The ordinary input contains only row ID and user text. The diagnostic adds the original query mention and the factual person span, and the native trace must resolve exactly that assertion. Cases without that diagnostic span stay in ordinary coverage and are omitted from both supplied packets and supplied captures. No held worksheet, answer key or labels are linked into the runner. The Wick release depfile includes the runner, batch and common Rust modules and tokenizer sources, with no freeze module or worksheet dependencies. The working and staged runner sources match by SHA-256. The runner binary had none of the four distinctive HELD worksheet names, the offline key filename, or the causal-stratum marker in a literal scan. I did not examine real HELD output or key records.

| Wick input | SHA-256 |
|---|---|
| `sterile-target/release/d35-panel` | `6cc05dc0e5b3b2a4d18a47534b0cfa4ce95f3141bc4666608e19bb7e743557de` |
| `sterile-source/panel-sterile-sources.sha256` | `3e59026b3bb255dd243eeb079b1c66f16e9df544287ff98da637bcf38d917dfb` |
| `native/pinned-target/release/uor-chat` | `4dbbf3a404d4523098d96cd8a70efe6550cf6bcdaf0cc8dfec73fc9465aefe43` |
| `native/source-executable.sha256` | `e2af67276842812b3dff10517a4393759f9e3971b2e237e4a56cfdf85eba9601` |
| Frozen HELD blind manifest | `01ad16eca30eb36f2d05adb026645a816dab3c6586326d1f82a80f70bd782cd7` |
| Frozen TRAIN input manifest | `35b26571ae5dd73226f2a3828377af4919b1c8ba904c3f414a5b039bdeefe4c9` |

Use the D26 and tokenizer pins from the phase FIT. Bind the runner source and executable, native pins, both input manifests and both TRAIN projection packet/index manifests from FIT-TRAIN to the GO. The combined directory receives copies of previously reviewed TRAIN packet, index and omission files. Verify their bytes against the pinned TRAIN-only directory before and after projection, and reject combined packets that differ. Verify the HELD packet/index manifests without opening per-row decisions. Treat the projection traces, including historical selections, as sealed blind evidence. The offline key stays sealed through the later native capture and CUDA score receipts. This FIT is for projection only.

The original `frozen` root is authoritative, with offline manifest `c13c9bd8818d1fd9d76354d52fd05c2d33565ef9f598a950801d688bcf23b3d7`. The panel author independently examined target lengths only and measured 24 of 216 crossed rows at or below the frozen eight-token writer budget. The rank mechanism and original split were already frozen at that point. No rows are excluded and the original blind experiment remains intact. Alongside the original all-row comparison, report descriptive counts by original target length at most eight versus above eight and by the actual source-through-fence writer envelope. Keep any supplemental short-object panel separate under its own prospective provenance and admission.
