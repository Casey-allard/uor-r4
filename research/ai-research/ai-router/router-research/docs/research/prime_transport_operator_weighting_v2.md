# Prime Transport Operator Weighting V2

## Purpose

This step learns weights only over the lawful outgoing entries of `H_v6`.

There is:

- no arbitrary candidate generation
- no post-hoc filtering
- no projection
- no fallback
- no scoring outside operator support

Learning only decides how strongly to use the six lawful generators already in
the operator:

- `I`
- `T_b`
- `T_x`
- `T_c`
- `T_y`
- `T_z'`

## Weighting Mechanism

The learned mechanism is a structured additive selector over lawful operator
entries only.

It scores the six generators from native context pieces:

- token id
- `b`
- `phi`
- `twist`
- query semiprime orientation
- binding semiprime orientation
- discourse style
- return-gap bucket
- focus
- speaker
- topic
- query-token flag
- tag-token flag
- explicit `spin_h` bits

This is operator-native because:

- the selector only chooses among the six lawful `H_v6` edges
- every evaluated next state is produced by a real operator generator
- no transition outside operator support is representable

## Baselines

The comparison includes:

- `v3` approximate reference
- `r5` rebuilt reference
- prior operator weighting result `v6`
- `H_v6` fixed lawful schedule baseline
- `v8` learned lawful operator weighting
- tiny transformer baseline

The `H_v6` baseline uses a fixed lawful generator schedule:

- generator index = `token_id mod 6`

This is not learned. It is a mechanical lawful operator baseline.

## Required Honesty Section

### Was learning applied only over lawful operator support?

yes

The learned selector only scored the six lawful outgoing `H_v6` edges:

- `I`
- `T_b`
- `T_x`
- `T_c`
- `T_y`
- `T_z'`

No arbitrary candidate states were generated.

### Were any illegal transitions introduced or scored?

no

Measured from
[prime_transport_operator_weighting_v2.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_operator_weighting_v2.csv):

- `illegal_transition_fraction = 0.0`
- `outside_support_fraction = 0.0`

So all learned motion remained inside lawful operator support.

### Did learned weighting make meaningful use of the richer lift structure?

yes

For `geometry_native_operator_model_v8`:

- `usage_T_z_prime = 0.552734375`
- `lawful_spin_change_fraction = 0.13333333333333333`

So the learned selector is making real use of the refined lawful lift rather
than ignoring it.

## Results

On the bounded `v3` discourse/query task:

- `geometry_native_sequence_model_v3_reference`:
  accuracy `0.9978`, query `0.9878`, loss `0.0077`
- `geometry_native_sequence_model_r5`:
  accuracy `0.7503`, query `0.7927`, loss `0.4760`
- `geometry_native_operator_model_v6`:
  accuracy `0.6813`, query `0.6951`, loss `1.9000`
- `geometry_native_operator_model_v7_fixed_policy`:
  accuracy `0.6868`, query `0.6980`, loss `4.3263`
- `geometry_native_operator_model_v8`:
  accuracy `0.6909`, query `0.7016`, loss `3.4019`
- `tiny_transformer_baseline_v8_reference`:
  accuracy `0.6958`, query `0.6707`, loss `0.7199`

So the richer lawful operator support helps somewhat relative to the prior
operator-weighting result:

- accuracy improves from `0.6813` to `0.6909`
- query accuracy improves from `0.6951` to `0.7016`

It also slightly beats the fixed lawful `H_v6` schedule on both accuracy and
query accuracy. But this is still a small improvement, and it remains well
below `r5` on the bounded task.

## Generator Usage

For `geometry_native_operator_model_v8`:

- `I = 0.0`
- `T_b = 0.0`
- `T_x = 0.0`
- `T_c = 0.447265625`
- `T_y = 0.0`
- `T_z' = 0.552734375`

Class movement:

- `original_class_fraction = 0.0`
- `lifted_class_fraction = 1.0`
- `lawful_spin_change_fraction = 0.13333333333333333`

Compared with the earlier operator-weighting result, this selector no longer
collapses onto `T_x`. It now uses the richer lawful lift structure materially,
but still over a narrow subset of the operator algebra.

## Required Outputs

The CSV is:

- [prime_transport_operator_weighting_v2.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_operator_weighting_v2.csv)

It reports:

- test accuracy / loss
- query accuracy
- parameter count
- generator usage frequencies
- fraction staying in the original class
- fraction entering lifted lawful classes
- fraction changing spin class lawfully
- illegal transition fraction
- outside-support fraction

## Single Next Step

Add the first lawful radial-class transport component, because the richer lift
is now being used and still only produces a small gain, while the current
operator algebra has no native radial transport at all.
