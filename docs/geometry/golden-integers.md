# Golden integers

The coordinates of the 120 icosian rotations contain the golden ratio, an irrational number, so a computer would normally round them. There is a way to avoid rounding altogether: keep every coordinate as a pair of whole numbers $(a, b)$ that stands for $a + b\varphi$. Adding and multiplying such pairs gives another pair, exactly. This chapter explains that arithmetic, called $\mathbb{Z}[\varphi]$, and says where the project uses it. Terms are also defined in the [glossary](../start/glossary.md).

![Exact arithmetic in Z[phi]: pairs of integers add and multiply without rounding, and the coordinates of unit icosians are such pairs scaled by one half](../figures/geometry/golden-integers.svg)

*Exact arithmetic in $\mathbb{Z}[\varphi]$, with the coordinates of unit icosians scaled by one half.*

## The arithmetic

The golden ratio $\varphi = (1+\sqrt5)/2$ satisfies

$$\varphi^2 = \varphi + 1 .$$

The set $\mathbb{Z}[\varphi] = \{a + b\varphi : a, b \text{ integers}\}$ is closed under addition and multiplication, so it supports exact pair arithmetic. Addition adds the pairs. Multiplication uses the rule above to remove $\varphi^2$:

$$(a+b\varphi)(c+d\varphi) = (ac+bd) + (ad+bc+bd)\,\varphi .$$

This is a **proof** by expanding and substituting $\varphi^2=\varphi+1$. Two worked examples:

- $\varphi\cdot\varphi$ is $(0,1)\cdot(0,1) = (1,1)$, that is $1+\varphi$.
- $\varphi^{-1} = \varphi - 1$ is the pair $(-1,1)$, and $\varphi\cdot(\varphi-1)=\varphi^2-\varphi=1$.

The other root of $x^2=x+1$ is $1-\varphi=-\varphi^{-1}$. Swapping $\varphi$ for it is the **Galois conjugate**, $a+b\varphi \mapsto (a+b)-b\varphi$. Multiplying a number by its conjugate gives the integer norm $N(a+b\varphi)=a^2+ab-b^2$.

## Icosian coordinates need the factor one half

The unit icosian coordinates are $(a+b\varphi)/2$, which lie in $\tfrac12\mathbb{Z}[\varphi]$. The factor one half must be kept. Scaled integer pairs represent them without rounding. For example, the icosian $\tfrac12(\varphi,\,1,\,\varphi^{-1},\,0)$ has doubled coordinates $(0,1)$, $(1,0)$, $(-1,1)$ and $(0,0)$, and its length checks out exactly: $\varphi^2+1+\varphi^{-2}=4$.

[`anchors.rs`](../../crates/uor-r4-core/src/native_geometric/anchors.rs) documents this scale explicitly. Its norm numerators have denominator 4. The same file stores one H4 root as $[a_0,\dots,a_3,b_0,\dots,b_3]$ plus a Galois companion, and warns that this invertible coefficient representation does not by itself establish an orthogonal Euclidean E8 embedding or independent extra state.

Fixed [`ZPhi` arithmetic](../../crates/uor-r4-core/src/prime_route_attention.rs) and exact anchor orientation tables support the research. **Chirality and polarity** are exact signs on $\mathbb{Z}[\varphi]$, held as fixed integer tables in `anchors.rs`. They are used by the retained native geometric learner, not by the current M1 path.

A caution about addresses: equality addresses for complete tuples are not semantic distances between class numbers. Two entries being exactly equal tells you they are the same entry. It does not tell you how alike two different entries are.

## Where the model uses this

| Piece | Status | Role |
|---|---|---|
| Fixed `ZPhi` arithmetic and anchor orientation tables, `uor-r4-core` | in `uor-r4-core`; no step of the M1 read is recorded as using it | Exact arithmetic for the icosian and E8 checks and for the retained native learner |
| The [retained native geometric engine](../mechanisms/native-geometric.md) | PARKED | Chirality and polarity signs; version chains and prime addresses |
| [Spin-native weights](../mechanisms/spin-weights.md) | PARKED | Quaternion blocks snapped to the 120 icosians; their coordinates live in $\tfrac12\mathbb{Z}[\varphi]$ |
| Current M1 carrier ([discrete learning](../mechanisms/discrete-learning.md)) | ACTIVE | Plain integer coordinates such as signed permutations of $(2,2,0,0,0,0,0,0)$, not $\mathbb{Z}[\varphi]$ pairs |

The two ends of the table matter. The exact golden arithmetic is real, tested code, but the current model's own carrier is built from ordinary integers. Whether exact golden coordinates help a language model has not been measured.

## Where it stands

| Claim | Kind | Source |
|---|---|---|
| $\mathbb{Z}[\varphi]$ is closed under $+$ and $\times$, with the product rule above | proof | derivation above |
| The 120 unit icosians have coordinates in $\tfrac12\mathbb{Z}[\varphi]$ | proof / finite enumeration | the list in the previous chapter and [`verify_connections.py`](../figures/geometry/verify_connections.py) |
| Exact golden arithmetic is a step of the current M1 read | not recorded | the M1 carrier is built from plain integer coordinates ([`build.rs`](../../crates/uor-r4-discrete/build.rs)) |
| Modular coefficients such as $\mathbb{Z}/2^{256}$ replace exact $\mathbb{Z}[\varphi]$ pairs | no | they are neither the real normed division algebra nor a substitute (see the [SpiralCore chapter](./spiralcore.md)) |

## Go deeper

- Code: [`anchors.rs`](../../crates/uor-r4-core/src/native_geometric/anchors.rs) and [`prime_route_attention.rs`](../../crates/uor-r4-core/src/prime_route_attention.rs).
- The forced link from the icosians to $\mathbb{Z}[\varphi]$ is checked in [`verify_connections.py`](../figures/geometry/verify_connections.py).
- Next: [The Hopf map and the kept fibre](./hopf-observation.md). Back to the [overview](./README.md).
