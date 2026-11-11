# Reply-control bake-off V2 socket

This supersedes the first reply-control bake-off before any candidate was
scored. Its first cycle fixture omitted the observed
`short_cycle_period`, so the rule could not test the declared input.

V2 keeps the same input and online output as `../reply-control/SOCKET.md`:

- one ordered `TraceRow` sequence per case;
- current-row decision only, `Keep` or `EmitEos`;
- EOS 1 is the only token a controller may inject;
- no reference ids reach the controller;
- the current cycle rule stops on the observed `short_cycle_period` after
  appending the third repeated token.

The scorer reports exact complete replies, truncations, EOS-position errors,
empty replies and cap stops. The primary metric remains exact complete replies.
