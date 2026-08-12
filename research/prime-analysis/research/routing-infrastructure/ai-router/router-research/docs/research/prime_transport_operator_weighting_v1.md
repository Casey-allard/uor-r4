# Prime Transport Operator Weighting V1

## Purpose

This step learns weights only over the lawful outgoing entries of `H_v5`.

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
- `T_z`

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

This is operator-native because:

- the selector only chooses among the six lawful `H_v5` edges
- every evaluated next state is produced by a real operator generator
- no transition outside operator support is representable

## Baselines

The comparison includes:

- `v3` approximate reference
- `r5` rebuilt reference
- `H_v5` fixed lawful schedule baseline
- `v6` learned lawful operator weighting
- tiny transformer baseline

The `H_v5` baseline uses a fixed lawful generator schedule:

- generator index = `token_id mod 6`

This is not learned. It is a mechanical lawful operator baseline.

## Required Honesty Section

### Was learning applied only over lawful operator support?

Yes.

The learned selector only scored the six lawful outgoing `H_v5` edges:

- `I`
- `T_b`
- `T_x`
- `T_c`
- `T_y`
- `T_z`

No arbitrary candidate states were generated.

### Were any illegal transitions introduced or scored?

No.

Measured from
[prime_transport_operator_weighting_v1.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_operator_weighting_v1.csv):

- `illegal_transition_fraction = 0.0`
- `outside_support_fraction = 0.0`

So all learned motion remained inside lawful operator support.

## Results

On the bounded `v3` discourse/query task:

- `geometry_native_sequence_model_v3_reference`:
  accuracy `0.9978`, query `0.9878`, loss `0.0077`
- `geometry_native_sequence_model_r5`:
  accuracy `0.7503`, query `0.7927`, loss `0.4760`
- `geometry_native_operator_model_v5_fixed_policy`:
  accuracy `0.6868`, query `0.6980`, loss `4.3263`
- `geometry_native_operator_model_v6`:
  accuracy `0.6813`, query `0.6951`, loss `1.9000`
- `tiny_transformer_baseline_v6_reference`:
  accuracy `0.6958`, query `0.6707`, loss `0.7199`

So this first lawful operator-weighting step is architecturally clean but not a
performance gain. It slightly underperforms the fixed lawful `H_v5` schedule on
accuracy and query accuracy, while remaining better calibrated than that fixed
baseline on loss.

## Generator Usage

For `geometry_native_operator_model_v6`:

- `I = 0.2321614583`
- `T_b = 0.0`
- `T_x = 0.5578125`
- `T_c = 0.0018229167`
- `T_y = 0.0002604167`
- `T_z = 0.2079427083`

Class movement:

- `original_class_fraction = 0.45390625`
- `lifted_class_fraction = 0.54609375`

The selector is using only lawful motion, but in this bounded task it collapses
mostly onto `T_x` and `T_z`, with almost no usage of `T_c` or `T_y` and no
usage of `T_b`.

## Required Outputs

The CSV is:

- [prime_transport_operator_weighting_v1.csv](/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_operator_weighting_v1.csv)

It reports:

- test accuracy / loss
- query accuracy
- parameter count
- generator usage frequencies
- fraction staying in the original class
- fraction entering lifted lawful classes
- illegal transition fraction
- outside-support fraction

## Single Next Step

Add nontrivial spin transport to the lawful lift structure, because the current
operator support is lawful and multi-class but the first weighting pass is not
using enough distinct operator directions to improve behavior.
