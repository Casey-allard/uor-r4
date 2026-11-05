# ORDER: cpu-training-pilot, deepseek-188 (input-preserving query ports)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `d634c77e5`.

## Goal

Test whether the existing writer-query port-root objective can improve an
exportable one-table artifact when the learned token input/writer table remains
raw. Deepseek-185 showed that table-wide anchoring caused the earlier decoding
collapse through the input role; this unit isolates the final-gain query
movement from that known defect.

This is D84 CPU development research. It is not GPU qualification, a language
result or a complete-reply result.

## Fixed construction

Add an explicit raw-baseline mode to `cpu-query-port-driver` while preserving
its anchored default and existing report behavior. In raw mode:

1. start the trainer and all controls from the unmodified public artifact;
2. change only port final-gain coordinates `0..8` and `W-8..W`;
3. run eight deterministic ±1-code steps on the training corpus;
4. compare raw-frozen, trained, equal-move random-gain and moved-count-matched
   shuffled-cell controls;
5. require exact CPU/serving parity after accepted moves;
6. evaluate target admission, top-1, top-8 and target-root distance on the
   fixed DEV panel.

No table rows, recurrent weights, serving code or artifact format change.
Every trained/control artifact remains a valid single-table `UORLUT01`.

## Data and power

- public dual-c131 seeds 1 and 2;
- train-control-v3, 7,906 tokens and 2,708 response rows after context
  filtering;
- dev-panel-v3, 1,925 tokens and 616 response rows after context filtering;
- 16 Rayon workers;
- exact input pins and absent output roots recorded in the pre-registration.

## Tests

- anchored mode remains the default and retains its output names;
- raw mode performs no table substitution;
- raw-frozen bytes equal the source artifact in the raw arm;
- port-only changes, exact parity, matched-control refusal and deterministic
  replay remain enforced;
- malformed mode arguments fail closed.

## Bar

On both artifact seeds the trained arm must:

- beat raw-frozen, equal-move random gain and moved-count-matched shuffled-cell
  roots on held-out target-port distance;
- retain target admission, top-1 and top-8 at or above raw-frozen;
- preserve exact CPU/serving parity;
- reproduce its artifact and steps in-process.

KEEP only if both seeds pass. A miss is a REJECT for this raw-conditioned
eight-step configuration.

## Done when

1. CODE and focused tests are complete.
2. FIT-reviewed pre-registration and D67 GO are recorded.
3. Both registered artifacts run.
4. REPORT, compact evidence, notebook and roadmap/current state record the
   comparison.
5. Reviewed merge, fresh-main verification and claim release complete.

## Stop rule

Stop on source, input, artifact, parity, determinism or output-root drift; any
need to edit serving/CUDA/float-training/exporter/Claude-owned source; a
non-`UORLUT01` output; or a run above 20 minutes or 1 GiB peak RSS.
