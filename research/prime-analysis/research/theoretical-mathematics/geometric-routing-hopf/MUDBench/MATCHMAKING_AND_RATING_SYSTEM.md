# MUDBench Matchmaking and Rating System

## Document Status
Version: 0.1
Status: Draft
Purpose: Define how agents are paired for matches and how their skill is quantified over time.

---

# 1. Why This Exists

You now have:
- arena system
- tournaments
- leaderboard reporting

Now you need something humans always underestimate:

**fair matchmaking and meaningful ratings**

Without this:
- strong agents farm weak ones
- weak agents never improve
- ratings become noise

This system exists to prevent that.

---

# 2. Core Principles

Matchmaking and rating must be:

- fair
- deterministic (where required)
- resistant to exploitation
- statistically grounded
- version-scoped

If your rating system can be gamed, it will be.

---

# 3. Separation of Concerns

This system has two components:

1. Matchmaking (who plays whom)
2. Rating (how skill is updated)

They are related but must not be tightly coupled.

---

# 4. Rating Model Options

## 4.1 ELO (Baseline)

- simple
- well understood
- pairwise comparison

Weakness:
- not ideal for multi-agent scenarios

---

## 4.2 Glicko / Glicko-2

- includes uncertainty (RD)
- adapts faster to new agents

Better for:
- evolving agent populations

---

## 4.3 TrueSkill (Recommended)

- supports multi-agent matches
- models skill as distribution
- handles team dynamics

This is the likely long-term choice.

---

# 5. Canonical Rating Representation

Each agent must maintain:

{
  "rating_mu": float,
  "rating_sigma": float,
  "matches_played": int,
  "last_updated": timestamp
}

Derived conservative score:

score = mu - k * sigma

This prevents overconfidence in new agents.

---

# 6. Initialization

New agents start with:

- default mu (e.g., 25)
- high sigma (uncertainty)

They must prove themselves.

---

# 7. Rating Updates

After each match:

- compute expected outcome
- compare to actual outcome
- update mu and sigma

Constraints:

- deterministic update given inputs
- no hidden randomness
- replay-auditable

---

# 8. Matchmaking Strategy

## 8.1 Skill-Based Matching

Primary rule:

- pair agents with similar rating

Goal:
- competitive matches
- meaningful rating updates

---

## 8.2 Exploration Factor

Occasionally allow:

- mismatched pairings

Purpose:
- detect misrated agents
- prevent local rating traps

---

## 8.3 Queue-Based Matching

Agents enter a queue.

Matchmaker selects:

- closest rating pairs
- valid scenario configuration
- available agents

---

# 9. Multi-Agent Matchmaking

For multi-agent scenarios:

- balance total team rating
- minimize rating variance within match
- avoid stacking strong agents

---

# 10. Rating Decay (Optional)

To handle inactivity:

- reduce confidence (increase sigma)
- optionally decay mu slightly

Prevents stale dominance.

---

# 11. Anti-Exploitation Rules

Prevent rating abuse:

- detect repeated same-opponent matches
- cap rating gain per opponent
- flag abnormal win patterns

---

# 12. Version Isolation

Ratings must be scoped by:

- benchmark_version
- scoring_version
- scenario set

Ratings across versions are NOT comparable.

---

# 13. Arena Integration

In persistent arena:

- ratings update continuously
- matchmaking runs asynchronously
- agent history accumulates

---

# 14. Tournament Integration

In tournaments:

- matchmaking is fixed
- ratings may update or remain frozen (configurable)

Tournament results must not corrupt global ratings unintentionally.

---

# 15. Confidence and Uncertainty

Agents with high sigma:

- require more matches
- should be matched more frequently

Low sigma:

- stable rating
- slower updates

---

# 16. Rating Visibility

## Public:

- mu
- conservative score
- rank

## Internal:

- sigma
- full distribution

---

# 17. Match Quality Metric

Each match should compute:

- rating difference
- expected competitiveness

Low-quality matches (huge mismatch) should be minimized.

---

# 18. Failure Handling

If a match fails:

- no rating update
- log failure
- reschedule if needed

---

# 19. Data Requirements

Each rating update must be based on:

- full replay
- validated scorecard
- deterministic outcome

No shortcuts.

---

# 20. Future Extensions

- adaptive matchmaking
- scenario-weighted ratings
- capability-specific ratings
- meta-learning detection

---

# 21. Closing Statement

Matchmaking decides who fights.

Rating decides who matters.

If either is flawed, your entire system quietly becomes meaningless.
