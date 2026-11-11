# Reply-control bake-off V3 socket

V3 supersedes V1 and V2 before any candidate is scored. It keeps their trace
input and online keep-or-EOS output, and corrects both fixture failures:

- the cycle row carries its observed `short_cycle_period`;
- generic rule rows use an inert EOS rank of 8, so a threshold candidate only
  acts when the fixture supplies a trigger.

The controller receives only trace features, never reference ids. It may keep
the served token or emit EOS 1. Exact complete replies are the primary metric.
