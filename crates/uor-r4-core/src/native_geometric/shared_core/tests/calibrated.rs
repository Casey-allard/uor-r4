use super::*;

#[test]
fn calibrated_caps_constants_and_collapsed_pair_separation() {
    let m = model().upgrade_calibrated().unwrap();
    let id = m.artifact.geometry.identity;
    for root in 0..120 {
        for union in [false, true] {
            for (threshold, right) in [(-5, true), (5, false)] {
                let b = CapBranch {
                    landmarks: [id; 2],
                    thresholds: [threshold; 2],
                    union,
                };
                assert_eq!(
                    m.cap_score([root; 4], b, 0, &mut Work::default()) > 0,
                    right
                );
            }
        }
    }
    let other = (0..120)
        .find(|&r| m.relative_score(r, id, &mut Work::default()) < 4)
        .unwrap();
    let a = [id; 4];
    let b = [
        other,
        m.artifact.geometry.inverses[usize::from(other)],
        id,
        id,
    ];
    assert_eq!(
        m.product(a[0], a[1], &mut Work::default()),
        m.product(b[0], b[1], &mut Work::default())
    );
    let classifier = CapBranch {
        landmarks: [id; 2],
        thresholds: [3, -5],
        union: false,
    };
    assert!(m.cap_score(a, classifier, 0, &mut Work::default()) > 0);
    assert!(m.cap_score(b, classifier, 0, &mut Work::default()) <= 0);
}

#[test]
fn calibrated_covers_every_leaf_and_preserves_recurrence_and_snapshots() {
    let legacy = model();
    let mut upgraded = legacy.upgrade_calibrated().unwrap();
    let (mut a, mut b) = (
        legacy.session(Intervention::Full),
        upgraded.session(Intervention::Full),
    );
    for byte in 0..=255 {
        a.observe(byte).unwrap();
        b.observe(byte).unwrap();
        assert_eq!(a.state, b.state);
        let before = b.checkpoint().unwrap();
        let predicted = b.predict();
        assert_eq!(before, b.checkpoint().unwrap());
        assert_eq!(predicted, upgraded.restore(&before).unwrap().predict());
    }
    drop(b);
    for target in 0..=256 {
        let (mut lo, mut hi, mut node) = (0, 257, 0);
        while hi - lo > 1 {
            let mid = (lo + hi) >> 1;
            let right = target >= mid;
            upgraded.artifact.calibrated.as_mut().unwrap().branches[node].thresholds =
                if right { [-5; 2] } else { [5; 2] };
            if right {
                lo = mid;
            } else {
                hi = mid;
            }
            node = (node << 1) + 1 + usize::from(right);
        }
        assert_eq!(upgraded.session(Intervention::Full).predict(), target);
    }
}

#[test]
fn calibrated_wire_rejects_malformed_domains_and_dispatch() {
    let m = model().upgrade_calibrated().unwrap();
    let bytes = m.to_bytes().unwrap();
    assert_eq!(
        SharedCore::from_bytes(&bytes).unwrap().to_bytes().unwrap(),
        bytes
    );
    for (key, value) in [
        ("thresholds", serde_json::json!([6, 0])),
        ("landmarks", serde_json::json!([120, 0])),
    ] {
        let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        wire["calibrated"]["branches"][0][key] = value;
        assert!(SharedCore::from_bytes(&serde_json::to_vec(&wire).unwrap()).is_err());
    }
    let mut wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    wire["schema"] = serde_json::json!(SCHEMA);
    assert!(SharedCore::from_bytes(&serde_json::to_vec(&wire).unwrap()).is_err());
}

#[test]
fn calibrated_training_records_separate_provenance_and_preserves_parent() {
    let m = model().upgrade_calibrated().unwrap();
    let original = m.to_bytes().unwrap();
    let docs = vec![b"red blue red".to_vec(), b"blue red blue".to_vec()];
    let (cal, report) = m.calibrate_emission(&docs).unwrap();
    assert_eq!(m.to_bytes().unwrap(), original);
    assert!(report.after.mean_nll <= report.before.mean_nll + 1e-12);
    assert_eq!(report.passes, 3);
    assert_eq!(cal.artifact.calibrated.as_ref().unwrap().parent, m.cid);
    let before = cal.to_bytes().unwrap();
    let (trained, r) = cal
        .fit(
            &docs,
            FitConfig {
                seed: 11,
                max_proposals: 90,
                max_seconds: 10,
            },
        )
        .unwrap();
    assert_eq!(r.proposals_by_family, [10; 9]);
    assert_eq!(cal.to_bytes().unwrap(), before);
    assert_eq!(trained.artifact.training_parent.as_ref(), Some(&cal.cid));
    assert_eq!(
        SharedCore::from_bytes(&trained.to_bytes().unwrap())
            .unwrap()
            .cid,
        trained.cid
    );
}
