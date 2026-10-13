# Prime Transport Geometry-Native Divisibility Bridge v12

## Purpose

Stress-test the v11 divisibility-bridge recovery under a stronger
reduced-schema-alignment mismatch, while keeping the same geometry-native
engine and the same arithmetic transition design spirit.

## Stronger Setting

The v12 setting keeps the same family as v7-v11 but makes the mismatch harder
in one bounded combined step:

- longer test sequences: `42` instead of the shorter reduced-alignment runs
- one extra latent entanglement term in the binding and query rules
- a noisier lossy projection back into the old schema through a more entangled
  proxy role and proxy entity mapping

This is directly comparable to v7-v11 because the token family, training
regime, and downstream geometry-native engine remain the same.

## Mechanism

The v12 bridge keeps the v11 arithmetic-transition idea intact:

- prime-coded atomic role increments
- semiprime bridge states
- small divisibility hubs

The only extension is a second small hub term inside both the query bridge and
binding bridge so the transition can mediate the added mismatch factor rather
than only the original one.

This is still geometry-native and bounded. It is not a learned adapter and it
does not alter the main sequence engine.

## Comparison

Reduced-alignment transfer line:

- v7 failure:
  - geometry-native accuracy `0.479204952717`
  - geometry-native query accuracy `0.544108569622`
- v8 single-rule chart realignment:
  - geometry-native accuracy `0.568933844566`
  - geometry-native query accuracy `0.624918460846`
- v11 divisibility bridge:
  - geometry-native accuracy `0.883501827717`
  - geometry-native query accuracy `0.969032287598`
  - tiny transformer accuracy `0.477366715670`
  - tiny transformer query accuracy `0.510322570801`
- v12 stronger-mismatch divisibility bridge:
  - geometry-native accuracy `0.552269339561`
  - geometry-native query accuracy `0.584205031395`
  - geometry-native test loss `6.065035820007`
  - tiny transformer accuracy `0.452380955219`
  - tiny transformer query accuracy `0.464435160160`
  - tiny transformer test loss `2.171264886856`

## Interpretation

The v11 recovery is not fully robust under the stronger mismatch.

What remains true:

- v12 still beats the unrecovered v7 result
- v12 still beats the tiny transformer baseline on the stronger setting
- the bridge still provides useful reduced-alignment recovery

What weakens:

- v12 drops sharply relative to v11
- v12 no longer beats the simpler v8 chart realignment on transfer accuracy or
  transfer query accuracy

Numerically:

- accuracy drops from `0.883501827717` in v11 to `0.552269339561` in v12
- query accuracy drops from `0.969032287598` in v11 to `0.584205031395` in v12

So the honest reading is that divisibility-mediated transition structure still
helps, but the particular fixed bridge used here is not robust enough to
maintain the large v11 gain once the mismatch is strengthened.

## Answers

### Does v11’s divisibility-bridge recovery remain robust under a stronger mismatch?

Partially, but not strongly.

The bridge survives the harder setting in the sense that it still beats v7 and
still beats the tiny transformer baseline. But the large v11 recovery shrinks
substantially and falls below v8.

### Is the gain still geometry-native in a meaningful sense?

Yes.

The gain still comes from bounded arithmetic transition structure using
prime-coded increments, semiprime bridge states, and small divisibility hubs.
The main geometry-native engine remains unchanged, and there is no generic
dense adapter or transformer block.

### Does this strengthen the claim that adaptive geometry needs divisibility-mediated transition structure?

It sharpens the claim rather than simply strengthening it.

v11 showed that divisibility-mediated transition structure can be a powerful
missing ingredient. v12 shows that the idea remains directionally right, but
that one fixed bridge rule is not enough for harder mismatches. So the
direction still looks promising, but robustness is not solved.

### What is the next smallest stronger test if v12 works?

The next smallest stronger test should keep the same stronger v12 family but
allow bounded support-window bridge calibration or bridge-family selection,
instead of one fixed bridge rule. That would test whether adaptive
divisibility-mediated transition selection can recover the lost robustness.

## Bottom Line

The stronger v12 mismatch weakens the v11 divisibility bridge substantially.
The bridge still helps and still beats the transformer baseline, but it no
longer dominates the chart line. That means divisibility structure remains a
credible transition mechanism, but fixed divisibility hubs are not yet a fully
robust answer.
