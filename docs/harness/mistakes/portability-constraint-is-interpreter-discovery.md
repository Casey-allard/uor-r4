---
id: portability-constraint-is-interpreter-discovery
title: The portability constraint is finding the interpreter, not the Python version
date: 2026-10-13
severity: info
class: assumption
trigger: [portability, bootstrap, python version, 3.9, 3.11, 3.12, interpreter, python3, zshrc, login shell, agent shell, install, onboarding, fromisoformat, Z suffix]
paths: [docs/harness/BOOTSTRAP.md, docs/harness/ONBOARD-PROMPT.md, crates/uor-r4-harness/**]
milestones: [m1, m2, m3, m4, m5, m6, m7, m8]
mechanism: ""
guard: none: the harness reports the interpreter it found and refuses below the floor; whether a lab's shell resolves the right one is a property of that machine
evidence: [docs/harness/BOOTSTRAP.md, archive/labs/harness-install-2026-10-13/README.md]
status: active
---

## What happened

Bootstrap work for four labs was planned around **Python version support** — whether the harness could
run on an older interpreter, and which stdlib features had to be avoided. That framing assumed a lab
either has a new enough Python or does not.

The real problem is that **`python3` resolves to a different interpreter depending on the shell**, so a
lab can have a new enough Python and still run the wrong one. Version support was never the binding
constraint; **discovery** was.

## Symptom

A check passes when run by hand and fails from a script, or the reverse. The specific tell is a
`datetime.fromisoformat` error on a `Z`-suffixed timestamp — a Rust-written record that reads fine in
one shell and raises in another.

Generalised: **two labs report different behaviour for the same command**, and neither is wrong.

## Lesson

Measured on this machine, minutes apart:

```
agent shell   python3 -> 3.15.0
login shell   python3 ->  3.9.6
absolute path          ->  3.9.6   (/usr/bin/python3)
```

The declared floor is **3.9**, and both interpreters pass the suite — so support is not the issue.
Discovery is:

- `python3` in a **non-login agent shell** picks up a toolchain ahead of `/usr/bin` on `PATH`.
- `python3` in a **login shell** does not, because the profile has not run.
- A lab that "has 3.15" and a lab that "has 3.9" may be **the same machine**.

This produced a real bug rather than a theoretical one. `datetime.fromisoformat` rejects the `Z`
suffix before **3.11**, so a `Z` timestamp written by the Rust side was unreadable on 3.9 — and the
failure was **silent**, because the field simply parsed as absent and the staleness rule it fed
stopped firing. A version feature, not a version requirement, and only visible because the suite runs
on both interpreters.

**Corollary that cost more time than the bug:** any timestamp the Rust side writes must parse on 3.9.
The fix was to normalise `Z` to `+00:00` before parsing, not to raise the floor.

## How to apply

The Python harness was retired on 20 October 2026, so the interpreter case is closed. The same
failure now shows up as **tool discovery for `cargo`**: on the owner's laptop `/opt/homebrew/bin/cargo`
is a Node CLI that shadows rustup's whenever Homebrew precedes `~/.cargo/bin` on `PATH`. Pin the tool
whenever a check matters:

```sh
which -a cargo                 # which cargo each shell finds first
~/.cargo/bin/cargo --version   # rustup's, always
uor-harness onboard            # reports which cargo it found
```

Do not raise the declared floor to dodge a compatibility bug. **A floor is a promise to the labs,
and raising it silently drops whichever lab is on the older interpreter** — which is exactly the lab
least able to notice.

When adding a new code path, ask *"does this parse on 3.9"* about anything involving dates, `Z`
suffixes, `|` type syntax at runtime, or `match`. The suite run on both interpreters is what catches
it; running on one is what let the `Z` bug through in the first place.
