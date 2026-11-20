# [codex] Independent FIT for the revised ordinary client

**FIT** for rerunning the [registered ordinary client comparison](../SHEET.md) on Wick with the revised pinned executable. This review follows the [initial deployment FIT](../container/FIT-release.md). Its accepted container tool, embedded C1 artifact, tokenizer, prompts, CUDA output and comparison protocol remain the inputs to this run. The initial client release and its results remain in custody. The top session records the exact revised GO.

When a split sidecar is named, the client uses the artifact's open file descriptor to read at most 4,112 bytes for deployment recognition. An embedded envelope returns a conflict before any larger read or sidecar I/O. The loader checks classification from the full bytes again before binding. The legacy path parses the owned D11 model, drops the encoded trunk buffer, and then reads any sidecar. The embedded path retains the original artifact buffer for the compact table's borrowed rows. The feature-off path continues through the original D11 file loader.

| Reviewed build or control | SHA-256 |
|---|---|
| Wick `container/client-v2/source-executable.sha256` | `adf2aae2a57c48128d94a2382874da6c1ab93fa472028b34680856ce07ed233a` |
| Wick `container-client-v2-target/release/uor-chat` | `67d33ec2a36fdc8b13f14ef26a5750ed668618bbe8c37280c1e84d31f9ae4f6d` |
| Finite harness `container/finite.sh` | `13aca6363e640e766d03d24948449a95053eb02decbfc9baf91e61d220c8c440` |
| Independent finite results manifest | `4a7a5c17dc65bed5bbab3f197fe1b9ac83fe7b89f9da30d0772f6bb6fb1c9982` |
| Standalone D11 wire JSON, historical and revised | `01810ced93ca09484b037c0fdd6908e5d0344fe09eec31c616df8e99092d4c62` |
| Sparse envelope's copied 266-byte header | `a83757c64ca9c17cbc37ca6fb67c2bf6f53faccc6d0e56ff685a6610d1fecd5e` |
| Prior client sparse rejection stderr | `f6545be018f4aca5b53a673bc7f640549bbb1482c574135b133b335b4481a916` |
| Revised client sparse rejection stderr | `ce87132e8cd1aa5ab14b6cec423026ec3183cf0e2061816ab27220db7e13487a` |
| Revised client sparse rejection resources | `dc35bb279dcd14c3ec02058900b623e00eaf8e1b6937c5ad6cd3193ad5be68cd` |

I strictly verified the source and executable manifest from Wick's `container-client-v2-source`. I reran the full finite harness with the revised client. Six raw-ID arms preserved the archived token, component and trace fields. The six raw-ID and two text arms matched their split-sidecar equivalents after accounting for deployment identity fields. C128 count readback and the corruption controls also passed. The standalone D11 protocol-2 response matched the historical client byte for byte on the planted-prefix fixture.

The sparse envelope had a 1 GiB logical length and one allocated filesystem block. With a 192 MiB address-space limit and a deliberately absent sidecar, the prior client failed with an out-of-memory I/O error. The revised client rejected the embedded-sidecar conflict, exited 1 and emitted no stdout. GNU time recorded 3,916 KiB peak RSS for the revised rejection. The independent captures are in `client-lifetime-fit.gsOcXBWe` under the D33 Wick run root. The full C1 response panel still needs the revised 48-process replay.
