# Lab presence

One file per lab, `<lab>.json`, refreshed by `cli.py heartbeat`. Committed on purpose: a
presence file another lab cannot read is not presence.

```sh
export UOR_LAB=kimi
python3 scripts/harness/cli.py heartbeat -m m1 --next "what happens next"
python3 scripts/harness/cli.py multilab presence      # who is where, stale last
```

A file older than 30 minutes reads **STALE**, and stale is not "idle" — it means the lab may be
working without checking in, which the session contract forbids. A lab with no file here is
simply absent, and absence is not evidence of idleness either.

This directory holds no secret: it is a branch name, a milestone, a one-line description and a
timestamp. Never put context, transcripts or anything provider-specific in it.

## Never put test data here

A fabricated record in this directory is **indistinguishable from a real one**. On 12 October two
records written by a smoke test (`UOR_LAB=kimi` and `UOR_LAB=claude` set by hand to exercise the
reader) were committed, and they read exactly like two labs checking in. A reader would have
concluded that Claude and Kimi had published state, with line counts that neither had reported.

If you need to exercise the reader, point `$UOR_HARNESS_LIVE` at a temporary directory:

```sh
UOR_HARNESS_LIVE=$(mktemp -d) UOR_LAB=testlab python3 scripts/harness/cli.py heartbeat -m m1
```

The test suite does exactly this — every test class gets its own live tier — so a test run cannot
write here either.
