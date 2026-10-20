# Free Rank

## Definition

The **free rank** formalizes the completeness criterion for type resolution
in the UOR framework. The ring R_n = Z/(2^n)Z admits an iterated Z/2Z fibration
with exactly n binary sites. Each constraint applied during resolution **pins**
one or more of these sites. When all n sites are pinned, the type is fully
resolved and the partition is complete.

## Site Indices

A {@class https://uor.foundation/partition/SiteIndex} represents a single
binary degree of freedom in the ring's structure. Each site has a position
within the fibration and a state indicating whether it has been determined:

| Property | Range | Description |
|----------|-------|-------------|
| {@prop https://uor.foundation/partition/sitePosition} | xsd:nonNegativeInteger | Zero-based position in the fibration (0 = LSB) |
| {@prop https://uor.foundation/partition/siteState} | xsd:string | Either `"pinned"` or `"free"` |

## The Budget

A {@class https://uor.foundation/partition/FreeRank} tracks how many sites
have been pinned versus how many remain free:

| Property | Range | Description |
|----------|-------|-------------|
| {@prop https://uor.foundation/partition/totalSites} | xsd:nonNegativeInteger | Total sites (= Witt level n) |
| {@prop https://uor.foundation/partition/pinnedCount} | xsd:nonNegativeInteger | Sites pinned by constraints |
| {@prop https://uor.foundation/partition/freeRank} | xsd:nonNegativeInteger | Sites still free (= total - pinned) |
| {@prop https://uor.foundation/partition/isClosed} | xsd:boolean | True when all sites are pinned |
| {@prop https://uor.foundation/partition/hasSite} | SiteIndex | A site in this budget |
| {@prop https://uor.foundation/partition/hasBinding} | SiteBinding | A pinning record |

A {@class https://uor.foundation/partition/Partition} links to its budget via
{@prop https://uor.foundation/partition/siteBudget}.

## Site Binding

A {@class https://uor.foundation/partition/SiteBinding} records which
constraint determined a specific site:

| Property | Range | Description |
|----------|-------|-------------|
| {@prop https://uor.foundation/partition/pinnedBy} | Constraint | The constraint that pinned this site |
| {@prop https://uor.foundation/partition/pinsCoordinate} | SiteIndex | The site that was pinned |

## Example: R_4 Budget

For R_4 (n=4), there are 4 sites. A residue constraint `x ≡ 1 (mod 2)` pins
site 0 (the parity bit), leaving 3 free:

```turtle
<https://uor.foundation/instance/budget-R4>
    a                       partition:FreeRank ;
    partition:totalSites    "4"^^xsd:nonNegativeInteger ;
    partition:pinnedCount   "1"^^xsd:nonNegativeInteger ;
    partition:freeRank      "3"^^xsd:nonNegativeInteger ;
    partition:isClosed      false .
```

## Completeness

When {@prop https://uor.foundation/partition/isClosed} is `true`, the free
rank budget is closed and the resolution is complete. This corresponds to the
resolver's {@prop https://uor.foundation/resolver/isComplete} flag and a
{@prop https://uor.foundation/resolver/siteDeficit} of zero.
