# Witt Levels

Witt levels W8--W32 are the four scaling tiers of the UOR {@concept ring} substrate. Every
computation, identity, and proof in UOR is valid at one or more Witt levels. Understanding
Witt levels is essential for reading the algebraic identities and their associated proofs.

## Level Definitions

| Level | Ring | Width | Cycle Size | Label |
|---|---|---|---|---|
| W8 | Z/(2)Z | 1 bit | 2 | Boolean |
| W16 | Z/(4)Z | 2 bits | 4 | Crumb |
| W24 | Z/(16)Z | 4 bits | 16 | Nibble |
| W32 | Z/(256)Z | 8 bits | 256 | Byte |

Each level is a {@class https://uor.foundation/schema/WittLevel} named individual with:
- {@prop https://uor.foundation/schema/wittLength} -- Witt vector length
- {@prop https://uor.foundation/schema/bitsWidth} -- bit width 1, 2, 4, or 8
- {@prop https://uor.foundation/schema/cycleSize} -- ring cardinality (2^bitsWidth)
- {@prop https://uor.foundation/schema/nextWittLevel} -- the next level up (W8->W16, W16->W24, W24->W32)

W32 (the byte ring) is the primary ring for {@concept content-addressing}. W8 is the minimal
boolean ring. The tower W8 < W16 < W24 < W32 is captured by the `nextWittLevel` chain.

## The `WittLevel` Enum

In the generated `uor-foundation` Rust crate, `WittLevel` is an open-world newtype
struct `WittLevel { index: u32 }` rather than a sealed enum. This is deliberate:
the OWL ontology treats Witt levels as a known finite set today, but the newtype
structure allows the framework to scale beyond W32 without breaking changes.

The `#[non_exhaustive]` attribute on the Rust type signals this openness. The four
canonical instances W8--W32 are provided as associated constants.

## Universal vs. Level-Specific Validity

Not all algebraic identities hold at all Witt levels. The `ValidityScopeKind`
enum class captures four validity patterns:

- **Universal** -- the identity holds for all Witt levels (k in {0,1,2,3} and beyond)
- **ParametricLower** -- the identity holds for all k >= k_min (some minimum level)
- **ParametricRange** -- the identity holds for k_min <= k <= k_max
- **LevelSpecific** -- the identity holds only at exactly one Witt level

These scopes are enforced by the conformance validator `validate_forall_scope_alignment`,
which checks that the `universallyValid` boolean is consistent with the `validityKind`
and that `validKMin`/`validKMax` bounds are present when required.

## Tower Chains

Amendments 41 and beyond introduced tower chain vocabulary to formalize the W-n scaling
of inductive proofs. A {@class https://uor.foundation/type/LiftChain} records a sequence
of type lifts from level k to level k+1, and a
{@class https://uor.foundation/type/LiftObstruction} records where the lifting fails.

The {@class https://uor.foundation/cert/LiftChainCertificate} certifies that a lift chain
is valid all the way to the target level. The
{@class https://uor.foundation/type/ObstructionChain} captures the failure mode -- where
and why a lift cannot proceed.

Inductive proofs ({@class https://uor.foundation/proof/InductiveProof}) have three key
properties (see {@concept proof-system} for details):
- `baseCase` -- proof at the minimal Witt level
- `inductiveStep` -- proof that validity at level k implies validity at level k+1
- `validForKAtLeast` -- the certified minimum Witt level

## Connection to Content Addressing

The Witt level determines the granularity of {@concept content-addressing}. At W32
(byte width), a content address is a sequence of bytes -- the standard interpretation
for file systems, network protocols, and cryptographic hash functions.

At W8 (bit width), a content address is a single bit -- useful for Boolean domains.
The {@concept ring} Z/(2)Z has only two elements (0 and 1), making W8 addresses the
coarsest possible content-addressable identifiers.

The {@class https://uor.foundation/schema/WittLevel} individuals are referenced by the
{@class https://uor.foundation/op/WittLevelBinding} class, which links operations to the
Witt level at which they are defined. This binding ensures that no operation is applied
at a ring scale it was not designed for.
