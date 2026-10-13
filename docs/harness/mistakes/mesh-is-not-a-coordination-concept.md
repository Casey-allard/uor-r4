---
id: mesh-is-not-a-coordination-concept
title: "The mesh" is not a coordination concept in this repository, and looking for one finds geometry
date: 2026-10-13
severity: info
class: assumption
trigger: [mesh, connect to the mesh, lab mesh, agent mesh, coordination mesh, service mesh, tailscale, wireguard, lab-runner coord, coordination layer]
paths: [docs/harness/**, docs/labs/protocol.md]
milestones: [m1, m2, m3, m4, m5, m6, m7, m8]
mechanism: ""
guard: none: a negative finding has nothing to intercept; the check is "does a coordination mesh exist", which the search below settles
evidence: [docs/labs/multilab-harness-2026-10-12/README.md, docs/labs/protocol.md]
status: active
---

## What happened

The harness objective names an "existing coordination mesh" and asks for the coordination layer to be
connected to it: *"research the project's mesh concept and connect the coordination layer to it."* The
wording presumes the mesh exists and needs wiring.

It does not exist. This entry records that, with the method, so the next lab settles it in a minute
instead of re-deriving it — which is the whole reason a ruled-out assumption belongs in a registry
that lessons also live in.

## Symptom

A plan that treats GitHub as one participant among several, or that proposes a daemon, socket, broker
or peer connection to "join" the labs. Also the specific question *"are we on the mesh yet?"*, which
has been asked across labs and has no referent.

The symptom is **a design that needs a component which was never built and is not wanted**.

## Lesson

**A full-text search for `mesh` returns no coordination concept.** Re-verified on main: the hits are
geometric or unrelated —

| source | what it is |
|---|---|
| `docs/native_geometric_mechanism_map_973.md` | TensorFlow **Mesh**'s MoE gating, cited as a comparator |
| SpiralCore schema, mechanism map | nearest-neighbour / **simplicial** meshes |
| research simulations | `THREE.Mesh` |
| a vendored source catalog | **service** mesh entries, unrelated |
| `research/**/.vendor/numpy` | `meshgrid` |

There is no coordination mesh, no Tailscale or WireGuard mesh, and no bus other than GitHub.

**The candidate that looks like the mesh is explicitly forbidden.** `docs/labs/protocol.md` says of the
retired `lab-runner coord`: *"do not recreate it or run `lab-runner coord`"*. So the one
daemon-shaped thing this repository ever had was **removed on purpose**, and reviving it is not a
research question.

**What the objective's clause resolves to instead:** GitHub is the bus, `main` is the truth, and the
harness contributes what GitHub alone does not — a committed record of **who holds which branch** and
**what each lab is doing between merges**. Presence and claims are the coordination layer; they need no
socket, because two labs publishing to the same repository already share a transport.

Anything daemon-shaped was ruled out on **policy** grounds, not on effort.

## How to apply

Do not build or propose a mesh, broker, daemon or socket to connect the labs. Read coordination as:

```sh
python3 scripts/harness/cli.py multilab presence   # who is doing what, live tier first
python3 scripts/harness/cli.py multilab claims     # who holds which branch
python3 scripts/harness/cli.py heartbeat -m m1     # publish yourself
```

If a future direction genuinely needs something more than the repository, it needs an **owner
decision** first, and the decision should say what the repository cannot carry — not assume a mesh
already carries it.

To re-check the negative, search for the concept rather than the word:

```sh
grep -rni 'mesh' --include='*.md' --include='*.rs' --include='*.py' . \
  | grep -vi 'simplicial\|mesh_tensorflow\|service mesh\|meshgrid'
```
