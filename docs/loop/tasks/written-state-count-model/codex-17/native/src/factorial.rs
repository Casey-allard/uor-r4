use super::*;

const POLICIES: [OnsetMode; 4] = [
    OnsetMode::Ordered,
    OnsetMode::Ranked,
    OnsetMode::Veto,
    OnsetMode::Both,
];
const FIRST: [u32; 2] = [0, 4];
const SECOND: [u32; 2] = [0, 5];
const UNSEEN: [u32; 2] = [2, NIL];
const TIE: [u32; 2] = [3, NIL];
const NONPOSITIVE: [u32; 2] = [4, NIL];

#[derive(Clone, Copy)]
struct Exemplar {
    positive: u16,
    negative: Option<u16>,
    radius: u32,
}

const FIRST_SUPPORTED: Exemplar = Exemplar {
    positive: 0,
    negative: Some(2),
    radius: 0,
};
const SECOND_SUPPORTED: Exemplar = Exemplar {
    positive: 2,
    negative: Some(0x11),
    radius: 4,
};
const OUTSIDE: Exemplar = Exemplar {
    positive: 2,
    negative: Some(0x11),
    radius: 0,
};

struct Fixture {
    name: &'static str,
    exemplars: &'static [Exemplar],
    expected: [[u32; 2]; 4],
}

const FIXTURES: [Fixture; 7] = [
    // Candidate 4 has distances 0,4 and candidate 5 has distances 4,16.
    // Slot denominators prefer 12/3 over 4/2. Distance sums prefer 4/4 over 12/20.
    Fixture {
        name: "ranking",
        exemplars: &[FIRST_SUPPORTED, SECOND_SUPPORTED],
        expected: [SECOND, FIRST, SECOND, FIRST],
    },
    Fixture {
        name: "known-outside",
        exemplars: &[FIRST_SUPPORTED, OUTSIDE],
        expected: [UNSEEN, UNSEEN, FIRST, FIRST],
    },
    Fixture {
        name: "combined",
        exemplars: &[FIRST_SUPPORTED, SECOND_SUPPORTED, OUTSIDE],
        expected: [UNSEEN, UNSEEN, SECOND, FIRST],
    },
    Fixture {
        name: "missing-radius",
        exemplars: &[
            FIRST_SUPPORTED,
            Exemplar {
                radius: NO_RADIUS,
                ..OUTSIDE
            },
        ],
        expected: [UNSEEN; 4],
    },
    Fixture {
        name: "missing-class",
        exemplars: &[
            FIRST_SUPPORTED,
            Exemplar {
                radius: 4,
                negative: None,
                ..OUTSIDE
            },
        ],
        expected: [UNSEEN; 4],
    },
    // Negative distances 8 and 12 make two unequal cross-products equal in each policy.
    Fixture {
        name: "tie",
        exemplars: &[
            Exemplar {
                negative: Some(1),
                ..FIRST_SUPPORTED
            },
            Exemplar {
                negative: Some(0x21),
                ..FIRST_SUPPORTED
            },
        ],
        expected: [TIE; 4],
    },
    Fixture {
        name: "nonpositive",
        exemplars: &[
            Exemplar {
                positive: 2,
                negative: Some(15),
                radius: 4,
            },
            OUTSIDE,
        ],
        expected: [UNSEEN, UNSEEN, NONPOSITIVE, NONPOSITIVE],
    },
];

pub(super) fn run(device: &Path, out: &Path) -> Result<()> {
    fs::create_dir(out)?;
    for fixture in FIXTURES {
        let dir = out.join(fixture.name);
        fs::create_dir(&dir)?;
        let mut rows = Rows {
            provenance: synthetic_hashes(),
            rows: Vec::new(),
        };
        for (ordinal, fact) in [(0, 2), (1, 174)] {
            let count = fixture.exemplars.len();
            let mask = (1 << count) - 1;
            let mut probe = Row::new(ordinal, 0, 192, fact, fact + 10, fact + 7, [mask; 2]);
            for i in 0..count {
                let source = fact + 2 + i as u32;
                let end = source + if i == 2 { 2 } else { 1 };
                let classes = geometry(i + 1);
                probe.insert(i, source, end, true, &classes, &[0; 5][..classes.len()])?;
            }
            rows.rows.push(probe);
        }
        rows.rows.push(Row::new(2, NIL, 192, 0, 0, NIL, [0, 0]));
        let packet = rows.encode()?;
        let input = dir.join("probes.packet");
        save(&input, &packet)?;

        let mut raw = vec![RAW, 0, 0, CAP as u32];
        for (i, exemplar) in fixture.exemplars.iter().enumerate() {
            let candidate = rows.rows[0].candidate(i);
            let shape = (i as u32 + 1) | ((candidate[1] - candidate[0]) << 8);
            let signature = candidate[3];
            for (label, root) in [Some(exemplar.positive), exemplar.negative]
                .into_iter()
                .enumerate()
            {
                if let Some(root) = root {
                    raw.extend([
                        shape | ((label as u32) << 16),
                        signature,
                        exemplar.radius,
                        u32::from(root),
                        0,
                        0,
                        0,
                        0,
                    ]);
                    raw[2] += 1;
                }
            }
        }
        raw.resize(4 + CAP * RECORD, 0);
        let bank = dir.join("ordered.bank");
        save(
            &bank,
            &seal(&rows, &packet, fixture.name.as_bytes(), &le(raw))?,
        )?;

        let legacy = dir.join("legacy.trace");
        score(device, &input, &bank, None, &legacy)?;
        for (policy, goal) in POLICIES.into_iter().zip(fixture.expected) {
            let trace = dir.join(format!("{}.trace", policy.name()));
            score(device, &input, &bank, Some(policy), &trace)?;
            let bytes = fs::read(&trace)?;
            if policy == OnsetMode::Ordered && bytes != fs::read(&legacy)? {
                return Err("explicit ordered and legacy entry points differ".into());
            }
            let words = trace_rows(&rows, policy as u32, &bytes)?;
            for (i, row) in words[4..].chunks_exact(OUT).enumerate() {
                let mut expected = goal;
                if i == 1 && expected[0] == 0 {
                    expected[1] += 172;
                } else if i == 2 {
                    expected = [1, NIL];
                }
                if row[2..4] != expected {
                    return Err(format!(
                        "{} {} row {i}: expected {expected:?}, got {:?}",
                        fixture.name,
                        policy.name(),
                        &row[2..4]
                    )
                    .into());
                }
            }
            let first = &words[4..4 + OUT];
            let emitted = match first[3] {
                4 => {
                    let ids = finite_writer(4)?;
                    assert_eq!(ids, [11, 13, 15, 14, 45, 1]);
                    ids
                }
                5 => {
                    let ids = finite_writer(5)?;
                    assert_eq!(ids, [13, 15, 14, 45, 1]);
                    ids
                }
                NIL => Vec::new(),
                _ => return Err("unexpected finite writer address".into()),
            };
            println!(
                "{} {} status={} source={} ratios={:?} writes={emitted:?}",
                fixture.name,
                policy.name(),
                first[2],
                first[3],
                &first[4..8]
            );
        }
    }
    Ok(())
}
