//! Probe: exact spin-1 frame invariance for the `sh-spin` read.
//!
//! A token's pose is a unit quaternion `u ∈ S³`; a local feature is a vector `a ∈ R³` in the
//! spin-1 representation; the pair score is `⟨a_i, q_ij · a_j · q_ij⁻¹⟩` with `q_ij = u_i⁻¹ u_j`.
//! Under a change of local frame `u → u g`, the feature transforms as `a → D(g)⁻¹ a` and the score
//! must be **exactly invariant**. Everything below is exact: `Z[φ]` with `φ² = φ + 1`, the 120
//! binary-icosahedral quaternions, and cross-multiplied rationals. No float, no `exp`.
//!
//! This is the spin-1 rung only. Spins `l ≥ 2` (the spherical-harmonic ladder) and the map from
//! tokens to coefficients are the next unit.
//!
//! `cargo test -p uor-r4-integer --test sh_spin_probe`

use std::collections::BTreeSet;

/// `a + b·φ`, with `φ² = φ + 1`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Phi {
    a: i128,
    b: i128,
}

const PHI_ZERO: Phi = Phi { a: 0, b: 0 };
const PHI_ONE: Phi = Phi { a: 1, b: 0 };

fn phi() -> Phi {
    Phi { a: 0, b: 1 }
}

impl Phi {
    fn add(self, other: Self) -> Self {
        Phi {
            a: self.a + other.a,
            b: self.b + other.b,
        }
    }

    fn sub(self, other: Self) -> Self {
        Phi {
            a: self.a - other.a,
            b: self.b - other.b,
        }
    }

    /// `(a + bφ)(c + dφ) = ac + (ad + bc)φ + bdφ² = (ac + bd) + (ad + bc + bd)φ`.
    fn mul(self, other: Self) -> Self {
        Phi {
            a: self.a * other.a + self.b * other.b,
            b: self.a * other.b + self.b * other.a + self.b * other.b,
        }
    }

    fn neg(self) -> Self {
        Phi {
            a: -self.a,
            b: -self.b,
        }
    }

    fn is_zero(self) -> bool {
        self.a == 0 && self.b == 0
    }

    fn scale(self, k: i128) -> Self {
        Phi {
            a: self.a * k,
            b: self.b * k,
        }
    }
}

/// An exact rational with a positive denominator; features only ever need powers of two.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Rat {
    num: Phi,
    den: i128,
}

impl Rat {
    fn integer(value: i128) -> Self {
        Rat {
            num: Phi { a: value, b: 0 },
            den: 1,
        }
    }

    fn zero() -> Self {
        Rat {
            num: PHI_ZERO,
            den: 1,
        }
    }

    fn add(self, other: Self) -> Self {
        Rat {
            num: self
                .num
                .mul(Phi { a: other.den, b: 0 })
                .add(other.num.mul(Phi { a: self.den, b: 0 })),
            den: self.den * other.den,
        }
    }

    fn mul(self, other: Self) -> Self {
        Rat {
            num: self.num.mul(other.num),
            den: self.den * other.den,
        }
    }

    /// Cross-multiplied equality, exact for positive denominators.
    fn same(self, other: Self) -> bool {
        self.num.mul(Phi { a: other.den, b: 0 }) == other.num.mul(Phi { a: self.den, b: 0 })
    }
}

type Quat = [Phi; 4];

fn qmul(a: &Quat, b: &Quat) -> Quat {
    let [aw, ax, ay, az] = *a;
    let [bw, bx, by, bz] = *b;
    [
        aw.mul(bw).sub(ax.mul(bx)).sub(ay.mul(by)).sub(az.mul(bz)),
        aw.mul(bx).add(ax.mul(bw)).add(ay.mul(bz)).sub(az.mul(by)),
        aw.mul(by).sub(ax.mul(bz)).add(ay.mul(bw)).add(az.mul(bx)),
        aw.mul(bz).add(ax.mul(by)).sub(ay.mul(bx)).add(az.mul(bw)),
    ]
}

fn qconj(q: &Quat) -> Quat {
    [q[0], q[1].neg(), q[2].neg(), q[3].neg()]
}

fn qnorm2(q: &Quat) -> Phi {
    q.iter().fold(PHI_ZERO, |acc, c| acc.add(c.mul(*c)))
}

/// A unit quaternion carried as an integer numerator over `den`; `|n|² = den²`.
#[derive(Clone, Copy, Debug)]
struct Unit {
    n: Quat,
    den: i128,
}

impl Unit {
    fn mul(self, other: Self) -> Self {
        Unit {
            n: qmul(&self.n, &other.n),
            den: self.den * other.den,
        }
    }

    fn conj(self) -> Self {
        Unit {
            n: qconj(&self.n),
            den: self.den,
        }
    }
}

/// `q · v · q⁻¹` for a unit quaternion `q = n/den` and `v` a spin-1 vector.
fn rotate(q: Unit, v: [Rat; 3]) -> [Rat; 3] {
    debug_assert_eq!(
        qnorm2(&q.n),
        Phi {
            a: q.den * q.den,
            b: 0
        }
    );
    let common = v.iter().map(|r| r.den).max().expect("three coordinates");
    let right = qconj(&q.n);
    let mut out = [PHI_ZERO; 3];
    for (coordinate, value) in v.iter().enumerate() {
        let scaled = value.num.mul(Phi {
            a: common / value.den,
            b: 0,
        });
        let mut basis = [PHI_ZERO; 4];
        basis[coordinate + 1] = PHI_ONE;
        let term = qmul(&qmul(&q.n, &basis), &right);
        for (slot, out_slot) in out.iter_mut().enumerate() {
            *out_slot = out_slot.add(term[slot + 1].mul(scaled));
        }
    }
    let den = q.den * q.den * common;
    [
        Rat { num: out[0], den },
        Rat { num: out[1], den },
        Rat { num: out[2], den },
    ]
}

fn dot(a: [Rat; 3], b: [Rat; 3]) -> Rat {
    a[0].mul(b[0]).add(a[1].mul(b[1])).add(a[2].mul(b[2]))
}

/// The pair score `⟨a_i, q_ij · a_j · q_ij⁻¹⟩`, with `q_ij = q_i⁻¹ q_j`.
fn score(qi: Unit, qj: Unit, ai: [Rat; 3], aj: [Rat; 3]) -> Rat {
    let relative = qi.conj().mul(qj);
    dot(ai, rotate(relative, aj))
}

fn key(n: &Quat) -> [(i128, i128); 4] {
    [
        (n[0].a, n[0].b),
        (n[1].a, n[1].b),
        (n[2].a, n[2].b),
        (n[3].a, n[3].b),
    ]
}

/// The 120 binary-icosahedral quaternions, as integer numerators over 2.
fn group_2i() -> Vec<Unit> {
    let mut set: BTreeSet<[(i128, i128); 4]> = BTreeSet::new();
    let mut push = |n: Quat| {
        set.insert(key(&n));
    };

    // 8: (±1, 0, 0, 0) and its permutations, as numerators ±2 over 2.
    for position in 0..4 {
        for sign in [-2, 2] {
            let mut n = [PHI_ZERO; 4];
            n[position] = Phi { a: sign, b: 0 };
            push(n);
        }
    }

    // 16: (±1, ±1, ±1, ±1).
    for bits in 0..16u32 {
        let sign = |index: usize| Phi {
            a: if bits & (1 << index) == 0 { 1 } else { -1 },
            b: 0,
        };
        push([sign(0), sign(1), sign(2), sign(3)]);
    }

    // 96: the even permutations of (±1, ±φ, ±1/φ, 0), each with the eight sign patterns of its
    // three nonzero coordinates. (1 + φ² + 1/φ²)/4 = 1, so these are unit.
    let inv_phi = Phi { a: -1, b: 1 }; // φ - 1 = 1/φ
    let values = [PHI_ONE, phi(), inv_phi, PHI_ZERO];
    let even = [
        [0, 1, 2, 3],
        [0, 2, 3, 1],
        [0, 3, 1, 2],
        [1, 0, 3, 2],
        [1, 2, 0, 3],
        [1, 3, 2, 0],
        [2, 0, 1, 3],
        [2, 1, 3, 0],
        [2, 3, 0, 1],
        [3, 0, 2, 1],
        [3, 1, 0, 2],
        [3, 2, 1, 0],
    ];
    for perm in even {
        let base = [
            values[perm[0]],
            values[perm[1]],
            values[perm[2]],
            values[perm[3]],
        ];
        for bits in 0..8u32 {
            let sign = |index: usize| Phi {
                a: if bits & (1 << index) == 0 { 1 } else { -1 },
                b: 0,
            };
            // The zero coordinate has no sign; apply the three signs to the nonzero entries in
            // order.
            let mut nonzero = 0;
            let mut signed = [PHI_ZERO; 4];
            for (slot, value) in base.iter().enumerate() {
                if value.is_zero() {
                    signed[slot] = PHI_ZERO;
                } else {
                    signed[slot] = value.mul(sign(nonzero));
                    nonzero += 1;
                }
            }
            push(signed);
        }
    }

    set.into_iter()
        .map(|n| Unit {
            n: [
                Phi {
                    a: n[0].0,
                    b: n[0].1,
                },
                Phi {
                    a: n[1].0,
                    b: n[1].1,
                },
                Phi {
                    a: n[2].0,
                    b: n[2].1,
                },
                Phi {
                    a: n[3].0,
                    b: n[3].1,
                },
            ],
            den: 2,
        })
        .collect()
}

#[test]
fn the_120_icosians_are_unit_with_a_closed_sample() {
    let group = group_2i();
    assert_eq!(
        group.len(),
        120,
        "the binary icosahedral group has 120 elements"
    );
    for q in &group {
        assert_eq!(
            qnorm2(&q.n),
            Phi { a: 4, b: 0 },
            "a 2I numerator must satisfy |n|² = 4"
        );
    }

    // Closure: q · g = (n·m)/4. Doubling a member numerator gives 2·m, and n·m = 2·m' for the
    // member m' the product equals.
    let doubled: BTreeSet<[(i128, i128); 4]> = group
        .iter()
        .map(|q| key(&q.n.map(|c| c.scale(2))))
        .collect();
    let mut checked = 0;
    for q in group.iter().take(12) {
        for g in group.iter().take(12) {
            let product = qmul(&q.n, &g.n);
            assert_eq!(qnorm2(&product), Phi { a: 16, b: 0 });
            assert!(
                doubled.contains(&key(&product)),
                "a product of two 2I elements left the group"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 144);
}

#[test]
fn the_spin_one_score_is_exactly_frame_invariant() {
    let group = group_2i();
    let features = [
        (
            [Rat::integer(3), Rat::integer(-5), Rat::integer(7)],
            [Rat::integer(-2), Rat::integer(11), Rat::integer(4)],
        ),
        (
            [Rat::integer(1), Rat::integer(0), Rat::integer(0)],
            [Rat::integer(0), Rat::integer(1), Rat::integer(0)],
        ),
        (
            [Rat::integer(-9), Rat::integer(2), Rat::integer(5)],
            [Rat::integer(6), Rat::integer(-3), Rat::integer(8)],
        ),
    ];
    let pairs = [(0usize, 1usize), (7, 42), (17, 113)];
    let mut comparisons = 0;
    for &(i, j) in &pairs {
        for (ai, aj) in features {
            let base = score(group[i], group[j], ai, aj);
            for g in &group {
                let qi = group[i].mul(*g);
                let qj = group[j].mul(*g);
                let gi = g.conj();
                let ai_frame = rotate(gi, ai);
                let aj_frame = rotate(gi, aj);
                let moved = score(qi, qj, ai_frame, aj_frame);
                assert!(
                    moved.same(base),
                    "the score changed under the frame change g = {:?}",
                    g.n
                );
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 3 * 3 * 120);
}

#[test]
fn the_score_separates_pairs_and_the_rank_weights_use_no_exponent() {
    let group = group_2i();
    let ai = [Rat::integer(3), Rat::integer(-5), Rat::integer(7)];
    let aj = [Rat::integer(-2), Rat::integer(11), Rat::integer(4)];
    let same = score(group[0], group[1], ai, aj);
    let other = score(group[0], group[2], ai, aj);
    let zero = Rat::zero();
    assert!(!same.same(other), "the score does not separate pairs");
    assert!(!same.same(zero), "the score is zero on a nontrivial pair");

    // The aggregation is the rank rule 1/(i+1), normalised to Q31: no exponent anywhere.
    let mut scores = [
        score(group[0], group[1], ai, aj),
        score(group[0], group[2], ai, aj),
        score(group[0], group[3], ai, aj),
    ];
    scores.sort_by(|a, b| {
        let left = a.num.mul(Phi { a: b.den, b: 0 });
        let right = b.num.mul(Phi { a: a.den, b: 0 });
        right.cmp(&left)
    });
    let weights: Vec<i128> = (0..scores.len())
        .map(|rank| (1i128 << 31) / (rank as i128 + 1))
        .collect();
    let total: i128 = weights.iter().sum();
    assert_eq!(total, (1 << 31) + (1 << 30) + ((1 << 31) / 3));
    let normalized: Vec<i128> = weights.iter().map(|w| w * (1 << 31) / total).collect();
    assert!(normalized.iter().all(|w| *w >= 0));
    assert!(normalized.iter().sum::<i128>() <= 1 << 31);
}
