# MUDBench World Implementation Plan

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the practical implementation plan for the first playable MUDBench world, with a strong bias toward an AberMUD 6 style text-world while preserving benchmark integrity and agent evaluability.

---

# 1. Why This Exists

The existing MUDBench documents define:

- the benchmark mission
- the simulation architecture
- the world model
- the scenario system
- the agent interface
- scoring and validation rules

What they do **not** yet define in practical terms is the first actual world to build.

This document fills that gap.

It defines the first concrete world implementation target so the project stops being a benchmark framework in the abstract and becomes a playable text-world that agents can actually inhabit.

---

# 2. World Implementation Goal

The first MUDBench world should be:

- text-first
- room-based
- command-driven
- deterministic
- easy to inspect
- easy to replay
- small enough to build quickly
- rich enough to feel like a real MUD

The design target is **AberMUD 6 style**, not in the sense of historical code compatibility, but in the sense of:

- compact room graph
- direct command grammar
- concise descriptions
- item-centered problem solving
- dangerous but legible world structure
- low-interface overhead
- high action clarity

The goal is not nostalgia for its own sake.
The goal is to use a proven interaction style that works naturally for agent evaluation.

---

# 3. Product Interpretation

MUDBench is both:

1. the MUD runtime itself
2. the benchmark harness around that runtime

That means the first world must satisfy two roles at once:

## 3.1 It must feel like a MUD
Agents should:
- move through rooms
- inspect text descriptions
- pick up and use items
- interact with NPCs
- solve goals through action sequences

## 3.2 It must behave like benchmark infrastructure
The world must:
- emit structured observations
- remain deterministic under seed
- produce replay-auditable events
- expose scoring hooks
- support scenario resets

The world is not a separate game bolted onto the benchmark.
It is the benchmark environment.

---

# 4. First World Design Philosophy

The first world should favor:

- clarity over breadth
- strong interaction loops over lore
- compact structure over huge content
- meaningful verbs over decorative complexity
- repeatability over procedural chaos

The first world should not try to be a giant fantasy universe.
It should try to be a **tight, inspectable, dangerous little world**.

Think:
- village edge
- forest path
- cave entrance
- ruined cellar
- locked gate
- patrol zone
- merchant corner
- dangerous interior chamber

Enough to feel real.
Small enough to debug.

---

# 5. AberMUD 6 Style Design Targets

The first world should mimic the useful design properties of classic Aber-style worlds.

## 5.1 Room-Centric Play
The core unit of world interaction is the room.

Each room should have:

- short title
- compact description
- exits
- visible entities
- hazards if relevant

Descriptions should be concise enough for fast agent parsing.

## 5.2 Verb-Driven Commands
The initial command grammar should stay close to classic MUD action style.

Examples:

- north / south / east / west
- look
- inventory
- get torch
- drop key
- use key
- attack goblin
- talk guard
- give coin guard

Structured APIs may still exist underneath, but the public game feel should remain MUD-like.

## 5.3 Compact, Dangerous Geography
The map should be small but nontrivial.

It should include:

- loops
- dead ends
- chokepoints
- guarded paths
- locked progression
- optional side paths

## 5.4 Utility Items Matter
Items should not be decorative.
They should unlock movement, safety, or goals.

Examples:

- torch for dark rooms
- key for locked gate
- ration for healing or trade
- rope for traversal
- dagger or club for combat survival

## 5.5 NPCs Are Functional
NPCs should not exist only for atmosphere.

Each initial NPC should matter to play through:

- merchant
- guard
- hostile creature
- quest giver
- optional informant

---

# 6. Implementation Target: World v0

The first playable world should be called:

**world_v0_aberstarter**

This is the minimum viable real world for MUDBench.

It should support:

- 20 to 30 rooms initially
- 3 to 5 regions
- 8 to 15 meaningful items
- 5 to 8 NPCs
- 2 to 4 hazards
- 1 to 3 benchmark scenarios layered over the same map

This is intentionally smaller than the later benchmark target.
The goal is to produce a real playable vertical slice first.

---

# 7. Proposed Initial Regions

## 7.1 Outskirts
Safe starting area.
Used for orientation and simple resource pickup.

Possible rooms:
- village road
- abandoned cart
- dry well
- watch post

## 7.2 Market Edge
Semi-safe interaction area.
Used for trade, information, or optional tool acquisition.

Possible rooms:
- trader stall
- storage lane
- collapsed shed

## 7.3 Forest Paths
Navigation and memory region.
Contains loops, hidden object placement, and mild threats.

Possible rooms:
- split trail
- thorn path
- fallen tree
- hidden clearing

## 7.4 Cave Mouth / Cellar Ruin
Progression region.
Introduces darkness, lock/key gating, combat pressure, and objective retrieval.

Possible rooms:
- cave mouth
- damp passage
- dark chamber
- locked alcove
- flooded step

## 7.5 Guarded Gate / Exit Route
Resolution region.
Used for final objective completion or escape.

Possible rooms:
- ruined gate
- guard post
- stone arch
- outer track

---

# 8. Proposed Initial Core Loop

The first world should support a simple but meaningful loop:

1. explore safely
2. identify constraints
3. acquire necessary item(s)
4. avoid or defeat threat
5. unlock progression path
6. retrieve objective
7. return or escape
8. score outcome

This loop should test:

- navigation
- memory
- planning
- tactical judgment
- efficiency

That aligns directly with the benchmark scoring dimensions already defined elsewhere.

---

# 9. Recommended First Scenarios

The first world should host multiple scenarios on one shared map.

## 9.1 Retrieval Scenario
Goal:
- find a hidden item and return it safely

Tests:
- navigation
- memory
- efficiency

## 9.2 Locked Progression Scenario
Goal:
- acquire key/tool, unlock path, reach target chamber

Tests:
- planning
- delayed dependency handling
- exploration

## 9.3 Guard / Trade Scenario
Goal:
- obtain access through interaction rather than force

Tests:
- social reasoning
- item management
- route choice

## 9.4 Threat Avoidance or Combat Scenario
Goal:
- survive a hostile region while reaching target

Tests:
- tactical competence
- risk assessment
- healing/resource usage

The same map can support all four with different scenario configs.

---

# 10. Recommended Initial Verb Set

The first implementation should keep the verb set deliberately small.

## 10.1 Required Verbs
- north / south / east / west
- look
- inventory
- get <item>
- drop <item>
- use <item>
- attack <target>
- talk <target>
- give <item> <target>

## 10.2 Optional Early Verbs
- equip <item>
- flee
- examine <object>
- unlock <target> with <item>

## 10.3 Deferred Verbs
These should be postponed until v1+:
- crafting
- spells
- advanced emotes
- unrestricted speech parsing
- nested container systems
- clan / guild systems

Keep the surface area small enough that both agents and validators can handle it cleanly.

---

# 11. Observation Style

The runtime may internally expose structured observations, but the world presentation should feel like a real MUD.

Recommended text presentation:

```text
Cave Mouth
A low opening breaks the hillside. Cold air leaks from the darkness within.
Exits: west, down
You see: torch, goblin
Health: 87
Inventory: coin, rope
Recent: The goblin snarls.
```

This gives:

- human readability
- classic MUD feel
- strong model legibility
- easy structured extraction

The public world should feel textual first.
The benchmark internals can still remain structured.

---

# 12. Map Design Rules

The first world map must obey these rules:

## 12.1 Every Region Must Matter
Do not add decorative zones with no benchmark value.

## 12.2 Every Critical Path Must Be Legible
Critical dependencies should be discoverable through text, not hidden behind arbitrary guesswork.

## 12.3 At Least One Loop Must Exist
This tests map memory and exploration discipline.

## 12.4 At Least One Gated Path Must Exist
This tests planning and item dependency handling.

## 12.5 At Least One Social Alternative Must Exist
There should be a path solvable by interaction instead of only combat.

## 12.6 At Least One Hazard Must Punish Thoughtless Play
Examples:
- dark room without torch
- poison thorn path
- guard aggression
- narrow route with hostile creature

---

# 13. NPC Implementation Rules

NPCs should start simple and deterministic.

## 13.1 Merchant
Role:
- trades one or two critical items

## 13.2 Guard
Role:
- blocks or regulates access
- can be bypassed by trade, dialogue, or alternate path

## 13.3 Hostile Creature
Role:
- creates tactical pressure
- punishes careless route choice

## 13.4 Informant / Guide
Role:
- exposes textual hints
- useful for social reasoning scenarios

## 13.5 Quest Giver
Role:
- sets explicit benchmark objective where needed

NPC behavior should be finite-state and deterministic, not “AI” in the modern sense.

---

# 14. Item Design Rules

Every early item should justify its existence.

Recommended initial items:

- torch
- key
- rope
- ration
- dagger
- club
- coin
- herb
- objective artifact
- note or hint token

Each item should have one or more of:

- access value
- combat value
- healing value
- trade value
- scenario value

No junk loot in v0.
Junk loot is how humans entertain themselves while ruining signal quality.

---

# 15. Hazard Design Rules

Hazards should be sparse but meaningful.

Recommended v0 hazards:

- dark room penalty without torch
- thorn path damage
- guarded chokepoint
- unstable floor or water crossing penalty

Every hazard must be:

- deterministic
- replay-visible
- textually explained enough to learn
- tied to a countermeasure when possible

---

# 16. Combat Design Rules

Combat in v0 should be simple, sharp, and inspectable.

Required features:

- deterministic initiative or fixed resolution order
- visible health changes
- simple attack action
- optional flee
- weapon influence if equipped or used

Combat must not dominate the first world.
It is one tool of pressure, not the entire game.

---

# 17. World State Implementation Requirements

The first world implementation must expose a world state that is:

- serializable
- resettable
- hashable for validation
- replay-reconstructable

State should include:

- room graph
- item locations
- NPC state
- agent state
- scenario-specific objective state
- hazard activation state

This is mandatory for benchmark trust.

---

# 18. Hosted-Server Interpretation

Long-term, MUDBench can absolutely become a hosted server model:

- agent authors connect local or hosted agents
- the service schedules them into world instances
- arenas run continuously
- new instances scale as capacity fills

Example future model:
- one world instance supports up to N agents
- additional load spawns another instance
- agents are assigned based on version, queue, or scenario set

That is a valid long-term product direction.

However, the first implementation should **not** optimize for cloud scale first.

First build:
- one local instance
- one deterministic world
- one strong vertical slice

Then scale.

Do not start with Kubernetes dreams before you have a cave and a torch.

---

# 19. Recommended v0 Build Order

## Phase A — Static World Graph
Build:
- rooms
- exits
- descriptions
- item placement
- static NPC placement

## Phase B — Basic Interaction
Add:
- movement
- look
- inventory
- get / drop / use
- talk

## Phase C — Gating Mechanics
Add:
- locked path
- key usage
- darkness / torch requirement
- one trade interaction

## Phase D — Tactical Layer
Add:
- one hostile NPC
- attack / flee
- simple health changes
- consumable healing item

## Phase E — Scenario Layer
Add:
- retrieval scenario
- locked progression scenario
- one social alternative scenario

## Phase F — Benchmark Hooking
Add:
- scoring signals
- replay events
- deterministic validation
- baseline-agent test runs

This order favors real playability early.

---

# 20. Success Criteria for World v0

The first world is successful if:

1. it feels like a real text MUD
2. agents can navigate it through text commands
3. at least one nontrivial objective can be completed
4. at least one scenario tests delayed dependency planning
5. all runs are replayable
6. scoring hooks behave deterministically
7. baseline agents can produce differentiated results

If all seven hold, you have a real benchmark world rather than a design document pretending to be one.

---

# 21. Relationship to Existing Specs

This document operationalizes and narrows the broader design intent already defined in:

- `WORLD_SPEC.md`
- `SCENARIO_SPEC.md`
- `SCENARIO_LIBRARY.md`
- `SIMULATION_ENGINE_SPEC.md`
- `AGENT_INTERFACE_SPEC.md`
- `BENCHMARK_SCORING_SPEC.md`

It should be treated as the first concrete world-building target, not as a replacement for those higher-level specifications.

---

# 22. Future Evolution

After the first world is stable, later worlds may add:

- larger region graphs
- faction systems
- richer social interaction
- rotating benchmark instances
- hosted multi-instance deployment
- arena-specific persistent states
- Aber-style content packs with shared engine rules

But those belong after the first world proves itself.

---

# 23. Closing Statement

The first MUDBench world should feel like an old-school text MUD with modern benchmark discipline.

That means:

- concise rooms
- meaningful items
- small but dangerous geography
- deterministic rules
- replayable outcomes
- measurable agent behavior

Build the cave, the torch, the key, the goblin, and the locked door first.

Everything more ambitious can wait until that works.
