# D30 boundary and role panels

The boundary panel contains six novel pairs, 30001–30012, and two exposed
D29 pairs, 30051–30054. Only the earlier passage changes within each pair.
The factual sentence, query and complete original protocol-2 tape from the
factual person onward are identical. Each passage licenses either a modifier
inside the item phrase or a word attached to the verb. Both readings require
different whole-word starts in the same assertion. The controls omit the
disambiguating information or the factual sentence. Their designated refusal
is a comparison tape, with no annotated gold source or unique object answer.

The calibration-free role panel crosses which name is requested, whether
its fact comes first, and whether the request clause comes before the
exclusion clause. Both families keep a third named fact and a query-side
incidental reference. Each family includes a direct one-person request,
a request without a designated person, and an incidental-only mention.
Record both original half-open endpoints and any original assertion heads.
An incidental mention can be a competing candidate in either role slot.
The two readers receive only the blinded user-text TSVs.

The phrase “away team jerseys” in row 30005 has an alternative spelling
with a hyphen. The passage explicitly selects the uniform worn on trips,
which licenses the compound in the literal, unhyphenated clause. Keep this
pair visible in the per-pair readout.

## Prepare and verify

Use the original assertion-agreement carrier and tokenizer described in
D26, then the independently hashed recurrence and D29 baseline carriers
named in the D30 sheet. Build the inherited D25 extractor and the three D30
binaries from their standalone locked Cargo manifests.

```sh
cargo build --locked --offline --release --manifest-path "$D25/Cargo.toml"
cargo build --locked --offline --release --manifest-path "$D30/Cargo.toml"
"$D30/prepare.sh" "$EXTRACTOR" "$PANEL_BINARY" "$PARENT" "$TOKENIZER" "$OUTPUT"
```

The inherited panel builder checks both registered starts against the
native candidate mask and copies literal original IDs through the original
fence and EOS. The D30 pass independently re-encodes each literal in
protocol-2 assistant position and verifies byte-for-byte IDs, complete
local suffix equality and distinct gold tapes. The exact user texts for
the four exposed prompts are those of D29 rows 29050–29053. The bundled
D25 and D26 historical splits are unused by D30. Keep the grades, donor
interpretation and writer oracles on the host. Only the target-blind
candidate packets and original projection inputs enter device scoring.

After the registered release, check the client SHA against the release
receipt. Invoke `../ordinary/run.sh` from this directory with the scratch
root, client, tokenizer, D26 carrier, D29 carrier, and `d30-role-read` binary.
It captures ordinary protocol-2 replies from both carriers, checks the
role traces, and compares the cold captures byte for byte. Use
`client-no-read.sh` with the same client, D26 carrier, tokenizer, a fresh
output directory, and both blind TSVs. Repeat into a second fresh directory
and compare both captures. Run the writer check with the device readback
from the three registered score packets.

```sh
"$WRITER" "$RECURRENCE" "$BASELINE" "$TOKENIZER" "$OUTPUT/panel" \
  "$DEVICE_SCORE_READBACK" "$ORDINARY_ROOT" "$D26_DISABLED" "$WRITER_JSON"
```

The writer reads the device's original positions and uses the deployed
integer copy transition to record every emitted ID and cursor word through
EOS or the 16-token cap. It verifies the nominal GPU source selection
against the reloaded recurrence carrier, and checks each supported forced
gold and opposing source through its own exact tape. It verifies complete
ordinary capture records against native role, assertion and cursor replay.
For each attempted reply, it records the first differing expected and actual
ID, including early EOS, extra tokens and missing IDs. The reciprocal pair
check compares the swapped traces with the independently forced opposite
positions on the same target tape. Actual ordinary reads and controlled
supplied-boundary reads are retained independently.
