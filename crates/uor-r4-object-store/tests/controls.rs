use sha2::{Digest, Sha256};
use std::{
    ops::Range,
    sync::atomic::{AtomicUsize, Ordering},
};
use uor_r4_object_store::{Error, Limits, Manifest, NodeId, Record, RecordBackend, Store};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn wires(bytes: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut p = 0;
    while p < bytes.len() {
        let n = if bytes[p] == 0 { 45 } else { 49 };
        out.push(&bytes[p..p + n]);
        p += n;
    }
    out
}

#[test]
fn every_small_range_and_carry_boundary_is_exact() -> Result {
    for len in [0, 1, 31, 32, 33, 63, 64, 65, 95, 96, 97, 287, 288, 289] {
        let bytes: Vec<_> = (0..len).map(|i| (i % 251) as u8).collect();
        let mut s = Store::new(Limits::default());
        let f = s.fold(&bytes)?;
        assert_eq!(s.rebuild(&f.manifest)?.bytes(), bytes);
        assert_eq!(
            Manifest::decode(&f.manifest.encode(), s.limits())?,
            f.manifest
        );
        for start in 0..=len {
            for end in start..=len {
                let r = s.range(&f.manifest, start as u64..end as u64)?;
                assert_eq!(r.bytes(), &bytes[start..end]);
                assert!(!r.whole_object_digest_verified());
                let mut expected: Vec<Range<u64>> = Vec::new();
                if start > 0 {
                    expected.push(0..start as u64);
                }
                if end < len {
                    expected.push(end as u64..len as u64);
                }
                assert_eq!(r.unverified_remainder(), expected);
            }
        }
    }
    Ok(())
}
#[test]
fn partial_integrity_is_not_whole_integrity_or_utf8() -> Result {
    let mut s = Store::new(Limits::default());
    let f = s.fold("a🌀b".as_bytes())?;
    assert_eq!(s.range(&f.manifest, 2..3)?.text(), Err(Error::Utf8));
    let mut changed = f.manifest.encode();
    changed[16] ^= 1;
    let forged = Manifest::decode(&changed, s.limits())?;
    let r = s.range(&forged, 0..forged.byte_len())?;
    assert!(!r.whole_object_digest_verified());
    assert!(r.unverified_remainder().is_empty());
    assert!(matches!(s.rebuild(&forged), Err(Error::IntegrityFailure)));
    assert_ne!(forged.version(), f.manifest.version());
    Ok(())
}
#[test]
fn missing_frontier_and_unrelated_absence_are_explicit() -> Result {
    let bytes: Vec<_> = (0..96).map(|i| i as u8).collect();
    let mut s = Store::new(Limits::default());
    let f = s.fold(&bytes)?;
    let exported = s.export_records()?;
    let parsed = wires(&exported);
    let mut t = Store::new(Limits::default());
    assert_eq!(t.missing(&f.manifest)?, vec![f.manifest.roots()[0].0]);
    let parent = parsed.iter().find(|r| r[0] == 1).ok_or("parent")?;
    t.import_records(parent)?;
    assert_eq!(t.missing(&f.manifest)?.len(), 3);
    let leaf = parsed
        .iter()
        .find(|r| r[0] == 0 && r[13] == 0)
        .ok_or("leaf")?;
    t.import_records(leaf)?;
    assert_eq!(t.range(&f.manifest, 0..1)?.bytes(), &bytes[..1]);
    assert_eq!(t.missing(&f.manifest)?.len(), 2);
    assert!(matches!(t.rebuild(&f.manifest), Err(Error::Missing(_))));
    t.import_records(&exported)?;
    assert!(t.missing(&f.manifest)?.is_empty());
    assert_eq!(t.rebuild(&f.manifest)?.bytes(), bytes);
    Ok(())
}
#[test]
fn malformed_imports_are_atomic_and_export_order_is_independent() -> Result {
    let mut s = Store::new(Limits::default());
    s.fold(&[4; 97])?;
    let wire = s.export_records()?;
    for cut in 1..45 {
        let mut t = Store::new(Limits::default());
        assert!(t.import_records(&wire[..cut]).is_err());
        assert_eq!(t.record_count(), 0);
    }
    let mut bad = wire.clone();
    bad.push(255);
    let mut t = Store::new(Limits::default());
    assert!(t.import_records(&bad).is_err());
    assert_eq!(t.record_count(), 0);
    let mut changed = wire.clone();
    let last = changed.len() - 1;
    changed[last] ^= 1;
    assert!(t.import_records(&changed).is_err());
    assert_eq!(t.record_count(), 0);
    let reverse: Vec<_> = wires(&wire).into_iter().rev().flatten().copied().collect();
    t.import_records(&reverse)?;
    assert_eq!(t.export_records()?, wire);
    assert_eq!(t.import_records(&wire)?.new_records, 0);
    Ok(())
}
#[test]
fn shape_padding_scale_and_manifest_tampering_refuse() -> Result {
    let mut s = Store::new(Limits::default());
    let f = s.fold(&[7; 32])?;
    let wrong_padding = Manifest::from_parts(
        31,
        f.manifest.roots().to_vec(),
        Sha256::digest([7; 31]).into(),
        s.limits(),
    )?;
    assert!(matches!(
        s.rebuild(&wrong_padding),
        Err(Error::IntegrityFailure)
    ));
    let mut wire = f.manifest.encode();
    wire.push(0);
    assert!(Manifest::decode(&wire, s.limits()).is_err());
    let mut wire = f.manifest.encode();
    wire[61] = 1;
    assert!(Manifest::decode(&wire, s.limits()).is_err());
    assert!(Manifest::from_parts(33, f.manifest.roots().to_vec(), [0; 32], s.limits()).is_err());
    let mut record = s.export_records()?;
    record[0] = 255;
    assert!(Record::decode(&record).is_err());
    // A valid leaf used as a parent fails expected scale before any traversal.
    let m = Manifest::from_parts(96, vec![(f.manifest.roots()[0].0, 1)], [0; 32], s.limits())?;
    assert!(matches!(s.range(&m, 0..1), Err(Error::IntegrityFailure)));
    Ok(())
}
#[test]
fn limits_refuse_before_fetch_or_publication() -> Result {
    let mut source = Store::new(Limits::default());
    let f = source.fold(&[2; 97])?;
    let wire = source.export_records()?;
    for limit in [
        Limits {
            max_object_bytes: 96,
            ..Limits::default()
        },
        Limits {
            max_records: 1,
            ..Limits::default()
        },
        Limits {
            max_record_bytes: 44,
            ..Limits::default()
        },
        Limits {
            max_visits: 1,
            ..Limits::default()
        },
    ] {
        let mut s = Store::new(limit);
        assert!(matches!(s.fold(&[2; 97]), Err(Error::LimitExceeded(_))));
        assert_eq!(s.record_count(), 0);
    }
    let mut s = Store::new(Limits {
        max_records: 1,
        ..Limits::default()
    });
    assert!(s.import_records(&wire).is_err());
    assert_eq!(s.record_count(), 0);
    struct Never(AtomicUsize);
    impl RecordBackend for Never {
        fn fetch(&self, _: NodeId) -> uor_r4_object_store::Result<Option<Record>> {
            self.0.fetch_add(1, Ordering::Relaxed);
            Ok(None)
        }
    }
    let b = Never(AtomicUsize::new(0));
    let l = Limits {
        max_returned_bytes: 0,
        ..Limits::default()
    };
    assert!(matches!(
        Store::range_from(&b, &f.manifest, 0..1, &l),
        Err(Error::LimitExceeded(_))
    ));
    assert_eq!(b.0.load(Ordering::Relaxed), 0);
    let l = Limits {
        max_visits: 0,
        ..Limits::default()
    };
    assert!(matches!(
        Store::range_from(&b, &f.manifest, 0..1, &l),
        Err(Error::LimitExceeded(_))
    ));
    assert_eq!(b.0.load(Ordering::Relaxed), 0);
    assert!(matches!(
        source.range(&f.manifest, 98..98),
        Err(Error::InvalidRange)
    ));
    Ok(())
}
#[test]
fn backend_cannot_substitute_another_valid_record() -> Result {
    let mut s = Store::new(Limits::default());
    let f = s.fold(&[1; 32])?;
    let mut other = Store::new(Limits::default());
    other.fold(&[2; 32])?;
    struct Wrong(Record);
    impl RecordBackend for Wrong {
        fn fetch(&self, _: NodeId) -> uor_r4_object_store::Result<Option<Record>> {
            Ok(Some(self.0.clone()))
        }
    }
    let b = Wrong(Record::decode(&other.export_records()?)?);
    assert!(matches!(
        Store::range_from(&b, &f.manifest, 0..1, s.limits()),
        Err(Error::IntegrityFailure)
    ));
    Ok(())
}
#[test]
fn repeated_content_keeps_occurrence_multiplicity() -> Result {
    let mut s = Store::new(Limits::default());
    let bytes = [5; 32 * 81];
    let f = s.fold(&bytes)?;
    assert_eq!(s.record_count(), 5);
    let r = s.rebuild(&f.manifest)?;
    assert_eq!(r.bytes(), bytes);
    assert_eq!(r.stats().distinct_records, 5);
    assert_eq!(r.stats().visits, 121);
    let again = s.fold(&bytes)?;
    assert_eq!(again.stats.new_record_bytes, 0);
    assert_eq!(again.manifest, f.manifest);
    Ok(())
}
#[test]
fn bounded_restart_and_failed_load_preserve_existing_store() -> Result {
    let dir = std::env::temp_dir().join(format!("uor-object-store-control-{}", std::process::id()));
    std::fs::create_dir(&dir)?;
    let result = (|| -> Result {
        let path = dir.join("records");
        let mut s = Store::new(Limits::default());
        let f = s.fold(b"durable exact bytes\0\0")?;
        s.save(&path)?;
        let mut t = Store::new(Limits::default());
        t.load(&path)?;
        assert_eq!(t.rebuild(&f.manifest)?.bytes(), b"durable exact bytes\0\0");
        let before = t.export_records()?;
        std::fs::write(&path, [0u8; 46])?;
        assert!(t.load(&path).is_err());
        assert_eq!(t.export_records()?, before);
        let mut bounded = Store::new(Limits {
            max_record_bytes: 1,
            ..Limits::default()
        });
        assert!(matches!(bounded.load(&path), Err(Error::LimitExceeded(_))));
        // Failed rename cannot remove the destination directory or strand our temp file.
        let blocked = dir.join("directory");
        std::fs::create_dir(&blocked)?;
        assert!(s.save(&blocked).is_err());
        assert!(blocked.is_dir());
        assert_eq!(std::fs::read_dir(&dir)?.count(), 2);
        Ok(())
    })();
    std::fs::remove_dir_all(&dir)?;
    result
}

#[test]
fn io_failure_retains_machine_readable_kind_and_os_code() {
    let missing = Error::from(std::io::Error::from(std::io::ErrorKind::NotFound));
    assert!(matches!(
        missing,
        Error::Io {
            kind: std::io::ErrorKind::NotFound,
            raw_os_error: None
        }
    ));
    let original = std::io::Error::from_raw_os_error(13);
    let expected_kind = original.kind();
    let converted = Error::from(original);
    assert_eq!(
        converted,
        Error::Io {
            kind: expected_kind,
            raw_os_error: Some(13)
        }
    );
}
