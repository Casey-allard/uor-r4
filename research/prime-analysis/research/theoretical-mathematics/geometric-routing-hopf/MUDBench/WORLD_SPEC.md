
# MUDBench World Specification

## Document Status
Version: 0.1
Status: Draft
Purpose: Define the structure, rules, and entity systems that govern the MUDBench text-world simulation.

---

# 1. World Design Philosophy

The MUDBench world is not primarily designed as entertainment content.

It is designed as a **controlled evaluation environment** where AI agents must:

- explore
- remember locations
- plan actions
- interact with entities
- manage resources
- adapt to other agents

The world must therefore be:

- deterministic when seeded
- structured enough for evaluation
- complex enough to test reasoning

---

# 2. World Topology

The world consists of a **graph of rooms**.

Each room may connect to other rooms through directional exits.

Example directions:

- north
- south
- east
- west
- up
- down

Rooms form a navigation graph.

Minimum MVP requirements:

- 50 rooms
- branching paths
- loops
- dead ends
- multi-region structure

Example room structure:

Room
- id
- name
- description
- exits
- entities
- hazards
- attributes

---

# 3. Regions

Rooms are grouped into **regions**.

Regions help structure the world for exploration metrics and scenario design.

Example regions:

- village
- forest
- ruins
- dungeon
- marketplace

Regions may have:

- thematic NPCs
- unique hazards
- special items
- quest objectives

---

# 4. Entities

All interactive objects in the world are entities.

Entity categories:

- rooms
- agents
- NPCs
- items
- environmental objects

All entities share common attributes.

Entity
- id
- type
- location
- attributes
- components

---

# 5. Agents

Agents are the AI participants in the environment.

Agent properties:

- unique identifier
- inventory
- health
- location
- status effects
- action history

Agents receive:

- textual observations
- structured environment state
- action outcomes

Agents may interact with:

- items
- NPCs
- other agents
- environment objects

---

# 6. NPCs

NPCs represent non-player characters.

NPC types may include:

- merchant
- guard
- monster
- villager
- quest giver

NPC attributes:

- behavior type
- hostility level
- inventory
- dialogue set

NPC behavior may include:

- wandering
- guarding
- attacking
- trading
- providing quests

NPC behavior should be deterministic within seeded runs.

---

# 7. Items

Items are objects that agents may interact with or collect.

Item properties:

- id
- name
- description
- weight
- value
- attributes

Item categories:

- weapons
- armor
- tools
- consumables
- quest items

Example actions:

- take
- drop
- use
- give
- equip

---

# 8. Inventory System

Agents and NPCs may hold items.

Inventory properties:

- capacity limit
- weight tracking
- stackable items (optional)

Inventory actions:

- take item
- drop item
- give item
- equip item

---

# 9. Movement System

Agents move between rooms via exits.

Example actions:

move north
move east

Movement may fail if:

- exit does not exist
- path is blocked
- special requirement is unmet

Movement may produce:

- room description
- entity list
- hazard notification

---

# 10. Combat System

Combat introduces tactical reasoning tasks.

Combat actions:

- attack
- defend
- flee
- use item

Combat properties:

- health
- damage
- weapon modifiers
- turn order

Combat resolution should remain deterministic under seeded runs.

---

# 11. Quest System

Quests provide structured objectives.

Quest properties:

- quest id
- objective
- target entities
- reward
- completion condition

Example objectives:

- retrieve item
- defeat enemy
- deliver item
- explore location

Quest completion contributes to evaluation scoring.

---

# 12. Hazards

Hazards create environmental challenges.

Examples:

- traps
- poison gas
- unstable structures
- hostile territory

Hazards may:

- damage agents
- block movement
- require tools or planning to bypass

---

# 13. Observation Format

Agents receive observations after each action.

Observation should include:

- current room description
- visible exits
- nearby entities
- inventory summary
- status effects
- recent events

Example observation structure:

Observation
- location
- description
- exits
- entities
- inventory
- health
- messages

---

# 14. Action Space

Agents must choose actions from a **constrained action set**.

Core actions:

- move <direction>
- look
- take <item>
- drop <item>
- use <item>
- attack <target>
- talk <npc>
- give <item> <target>

Actions outside the allowed set are rejected.

---

# 15. Multi-Agent Interaction

Multiple agents may exist in the same world.

Agents may:

- cooperate
- compete
- attack
- trade
- block paths

Multi-agent interaction is essential for:

- social reasoning
- strategic behavior
- emergent tactics

---

# 16. Persistence

In **benchmark mode**, the world resets each run.

In **arena mode**, the world may persist across sessions.

Persistent state may include:

- item locations
- quest completion
- NPC state
- world hazards

Persistence must not compromise benchmark fairness.

---

# 17. World Generation

The initial version should use a **fixed world map**.

Procedural generation may be added later.

Reasons for fixed map in MVP:

- reproducibility
- easier evaluation
- simpler debugging

---

# 18. Evaluation Hooks

The world must expose signals used by the evaluation engine.

Examples:

- rooms explored
- quests completed
- enemies defeated
- items collected
- survival time
- cooperation events

These signals feed benchmark scoring.

---

# 19. Safety Constraints

Agents must not be allowed to:

- execute arbitrary code
- modify world rules
- create new entities
- alter scoring logic

These constraints protect the benchmark environment.

---

# 20. Future Extensions

The world system may later support:

- procedural regions
- faction systems
- economy simulation
- diplomacy
- crafting systems
- large-scale maps

These features should remain modular.

---

# 21. Closing Statement

The MUDBench world is a structured problem space.

It must be complex enough to challenge agents but simple enough to remain interpretable.

Every feature in the world should exist because it tests a capability dimension.
