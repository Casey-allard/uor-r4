# ORDER: cpu-training-pilot, deepseek-241 (forced-copy panel attribution)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `973bc0750`.

## Goal

Measure whether the current `one-model-end-to-end` replies fail mainly at
opening selection or also after the correct value is supplied. Run the sealed
Claude-322 served artifacts on `m2-fresh256` through matched CPU arms that force
nothing, the correct first token, the complete correct value, the paired
other-relation value, and a deterministic shuffled value.

This is an independent diagnostic that supports Claude-325 and the lawful
pointer/forced-copy work. It does not train, alter a model, touch Claude's
branches, or claim GPU qualification.

## Cancel probe

- The exact CPU served forward and panel stop contract are available.
- `served-reply-panel` already renders `m2-fresh256`, scores frozen accepted
  answers, reports complete-distinct pairs and uses the artifact's served head.
- Claude-322's two sealed artifacts are available at
  `results/claude/one-model-end-to-end-b64l-d3mwcap6-2026-11-08/runs/s{1,2}/`
  and the REPORT records artifact prefixes `86e51d5e...` and `8838cd10...`.
- No existing report forces a correct value and then reports whether
  continuation reaches EOS.

## Design

Add one opt-in CPU diagnostic binary, `cpu-forced-copy-panel`, without changing
the scorer or training paths. It validates all artifact/panel hashes and uses
the same protocol-v2 rendering, frozen-answer scoring, nearest served head and
stop contract as `served-reply-panel`.

For each panel row, run:

1. `free`: normal greedy generation.
2. `first-target`: force only the target reply's first scored token.
3. `correct-value`: force the complete shortest accepted answer, excluding its
   terminal EOS, so the next decision tests continuation into EOS.
4. `other-relation`: force the accepted value of the other case in the same
   panel pair.
5. `shuffled-value`: force a deterministic other row's complete accepted
   value, selected by a recorded seed and required to differ from the target.

Prefixes use the reference protocol and response mask. Labels remain outcomes
only; no model state is updated.

Report per rendering, artifact, stratum and arm:

- complete replies and complete-distinct pairs;
- EOS, BOS, cap and context stops;
- exact first-line value, other-relation value and neither;
- forced-prefix survival to EOS;
- reply IDs/text and deterministic report hashes;
- artifact/query parity and wall time, peak RSS and CPU hardware.

The paired diagnostic is:

- `first-target` versus `free` isolates opening selection;
- `correct-value` versus `first-target` isolates continuation and termination;
- `correct-value` versus `other-relation` and `shuffled-value` tests whether
  completion depends on the role-correct source value.

## Files in scope

- `crates/uor-r4-cpu-train/Cargo.toml`
- `crates/uor-r4-cpu-train/src/bin/cpu-forced-copy-panel.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-241/**`
- `docs/research/notebook/cpu-training.md`

## Done when

1. The new binary builds offline and locked; tests cover renderer parity,
   target/other/shuffled prefix construction, exact scoring, stop accounting and
   fail-closed artifact/panel metadata.
2. Both Claude-322 backend artifacts run on all 256 `m2-fresh256` rows under
   both registered renderings without training.
3. The report records all five arms, per-stratum and paired results, exact
   artifact hashes, CPU/served parity, hardware and a cumulative resource
   receipt.
4. REPORT, notebook row, report check, policy check, full gates and a fresh
   exact-head review pass.

## Stop and ask

This is an attribution diagnostic, not a capability bar. Do not use panel
outcomes to tune a forced prefix, alter model weights, claim useful chat, M2 or
GPU qualification, or replace Claude's current training track. Do not modify
Claude-owned files.
