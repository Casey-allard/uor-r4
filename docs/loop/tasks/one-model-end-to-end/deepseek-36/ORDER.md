# ORDER: one-model-end-to-end, deepseek-36 (record opening-weight result)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `a3ac3f326`.

## Goal

Record the completed deepseek-35 opening-weight run against its pre-registered
first-match outcome map. The fixed weight-8 configuration must be closed or
unlocked exactly as that map requires, with no weight sweep and no seed 2 unless
the positive content row was reached.

## Cancel probe

- PR #3856 merged the reviewed `--opening-weight` implementation at
  `a3ac3f326`.
- The fixed run sealed 91 entries, passed full readback and deleted its owned
  pod.
- The opening readings decide this configuration now; no further implementation
  or compute stands between the readings and one result record.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/deepseek-36/**`
- `docs/research/notebook/float-training.md`

## Done when

1. All four checkpoint readings record first-target top-1/top-8 and rank
   median, EOS top-1 and median, free EOS stops, complete rows and guard.
2. The selected checkpoint is classified by applying the opening sheet's
   first-match map, not by replacing it with the general SFT map.
3. The result names the exact artifact, config, init, panel, grade, seal and
   store identities needed to reproduce the classification.
4. One notebook row records the result and its closed configuration.
5. The single result PR passes exact-head review and the protected merge queue.

## Stop and ask

Do not change the architecture, serving path, selector, outcome bars, training
code or the closed recipe. Any readback, hash or selected-checkpoint mismatch
stops the result. Do not run another weight or seed from this order.
