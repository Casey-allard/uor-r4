# Original HTML address / radix primitive reproduction

Run:

```sh
node check-source-codecs.cjs /absolute/path/to/view-source_https___spiralcore.netlify.app.html
```

The script requires the exact original HTML SHA-256 `770142b4f343842cd0693d362d49f95272a5ded029d9682211a5215d64810fd8` and extracts actual source using the start-inclusive / end-exclusive marker pairs recorded in `results.json`. Extracted implementations are not rewritten or patched. The isolated VM receives only `Uint8Array` and these two pure source fragments; it does not load the DOM, oscillator, waveform decoder or any UOR model path.

Scope: symbolic address presentation round trips and the original base63 integer codec. This is not a full RFC conformance suite, physical networking test, actual current golden-route waveform recovery, compression benchmark, learned representation or attention test. The later live HTML recorder uses `SpiralGoldenRecovery`; this script does not execute that recorder or its waveform decoder. The base63 codec tested here remains in the source as the legacy score codec.

Two executions passed: initial in-memory command and this retained reproduction. The retained run used Node v26.4.0, exited 0 and returned 0.372155 seconds tool wall time. No source or research state was changed. `results.json` is the retained run's complete stdout. The initial run had identical counts and result but did not print the extraction hashes.
