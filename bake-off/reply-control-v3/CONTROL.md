# V3 dumb control

The control copies every supplied row and makes no stop decision.

| Rule | Result |
|---|---|
| `empty_input_has_empty_output` | pass, TOO WEAK |
| `served_eos_is_kept_and_terminal` | fail |
| `injected_eos_is_terminal` | pass, TOO WEAK |
| `current_cycle_stop_uses_the_observed_cycle_mark` | fail |
| `output_is_only_served_tokens_or_eos` | pass, TOO WEAK |
| `output_never_exceeds_the_reply_cap` | fail |
| `flow_emits_nothing_after_a_stop` | fail |
| `controller_receives_only_trace_features` | pass, TOO WEAK |

Real arm-r panel score: 607/1,141 exact complete, 294 truncated, 490 EOS
position errors.
