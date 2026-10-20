# Current golden score/FBS primitive check

Run from this evidence directory, supplying the original attachment:

```sh
node check-current-golden-codec.cjs /absolute/path/to/view-source_https___spiralcore.netlify.app.html
```

The first argument supplies the exact input HTML; omission uses the original local Downloads path. The script pins its SHA-256 and extracts the original current `SpiralGoldenRecovery` score/pack/unpack definitions and the original typed FBS constructors. It emits the fragment SHA-256 values. Source JavaScript is executed solely to test the actual supplied byte codec; this introduces no model or product dependency.

The wrapper builds ordered FBS cells with the source functions, uses the current receiver's documented leaf-order/radix rule to recover bytes, and checks each carry/frontier count. It is deliberately not the full receiver: no waveform samples are generated or scored, no oscillator clock or geometry route search is called, and no browser or model path is executed. `routes` and `timing` definitions appear in the extracted prefix but are never invoked. Full waveform receipts in the HTML remain author-supplied evidence.

The recorded execution passed on Node v26.4.0 with exit 0: 353 exact byte round trips, 5,460 structural checks, 256 independent integer radix-bound checks, all 63 packed symbols, seven malformed refusals, and the production page-packing size check. The counter totals are check counts, not independent model trials. Input files were read only; only this evidence directory was written.
