use super::{
    artifact::Artifact,
    credit::{environment, trajectory},
    policy::{Intervention, Params, Policy},
    session::{argmax, Session},
};
use crate::native_geometric::{
    addressed_attention::{artifact::BoundGeometry, objects::Symbol, pilot_data},
    hamming_refinement::metric::Metric,
};
#[test]
fn offers_observation_turn_and_parameter_binding() -> Result<(), Box<dyn std::error::Error>> {
    let g = BoundGeometry::canonical()?;
    let m = Metric::new(&g)?;
    let a = Artifact::new(Params::seeded(973), &g)?;
    let mut p = Policy::new(a.params().clone(), &g, environment(&g), Intervention::Full)?;
    let mut s = Session::new(&a, &g, 1)?;
    for b in b"a=7" {
        let before = s.objects().frontier();
        let o = s.observe_input(*b, &g, &m, &mut p)?;
        assert_eq!(o.refinement.frontier, before);
        assert_eq!(s.objects().frontier(), before + 1);
    }
    let offer = s.predict(&g, &m, &mut p)?;
    let calls = p.audit().calls;
    let snapshot = s.objects().snapshot()?;
    assert_eq!(s.predict(&g, &m, &mut p)?, offer);
    assert_eq!(p.audit().calls, calls);
    assert!(s
        .observe(Symbol::Byte(0), offer.offer.id + 1, &g, &mut p)
        .is_err());
    assert_eq!(s.objects().snapshot()?, snapshot);
    assert_eq!(p.audit().calls, calls);
    p.intervention = Intervention::UpdateDisabled;
    assert!(s
        .observe(Symbol::Byte(0), offer.offer.id, &g, &mut p)
        .is_err());
    p.intervention = Intervention::Full;
    let actual = if offer.offer.symbol == Symbol::Byte(0) {
        Symbol::Byte(1)
    } else {
        Symbol::Byte(0)
    };
    let o = s.observe(actual, offer.offer.id, &g, &mut p)?;
    assert_eq!(o.refinement, offer.refinement);
    assert!(s.objects().active().is_none());
    assert_eq!(s.objects().result_frontier(), 1);
    assert!(s.observe(actual, offer.offer.id, &g, &mut p).is_err());
    let old = s.predict(&g, &m, &mut p)?;
    s.begin_turn(&g)?;
    assert!(s.observe(actual, old.offer.id, &g, &mut p).is_err());
    let eos = s.predict(&g, &m, &mut p)?;
    s.observe(Symbol::Eos, eos.offer.id, &g, &mut p)?;
    assert!(s.ended());
    assert!(s.predict(&g, &m, &mut p).is_err());
    s.begin_turn(&g)?;
    let mut wrong = a.params().clone();
    wrong.flip(0, 0)?;
    let mut wrong = Policy::new(wrong, &g, environment(&g), Intervention::Full)?;
    assert!(s.predict(&g, &m, &mut wrong).is_err());
    assert_eq!(argmax(&[3, 7, 7], &[true, true, true])?, 1);
    assert!(argmax(&[1], &[false]).is_err());
    Ok(())
}
#[test]
fn full_document_replay_and_source_bound_reload() -> Result<(), Box<dyn std::error::Error>> {
    let g = BoundGeometry::canonical()?;
    let m = Metric::new(&g)?;
    let a = Artifact::new(Params::seeded(973), &g)?;
    let b = Artifact::decode(&a.encode()?, &g)?;
    let e = &pilot_data::training()[0];
    let first = trajectory(&a, &g, &m, e, Intervention::Full)?;
    let again = trajectory(&b, &g, &m, e, Intervention::Full)?;
    assert_eq!(first.trace, again.trace);
    assert_eq!(first.cells, again.cells);
    assert_eq!(first.loss, again.loss);
    assert!((first.loss - first.reference_loss).abs() < 1e-10);
    assert_eq!(
        first.cells.iter().map(|&x| u64::from(x)).sum::<u64>(),
        first.calls * 3076
    );
    assert!(first.cells.iter().any(|&x| x > 1));
    assert!(first.snapshots > e.prompt.len() as u64);
    Ok(())
}
