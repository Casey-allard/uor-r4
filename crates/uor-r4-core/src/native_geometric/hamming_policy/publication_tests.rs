//! Authored constant truth tables exercise the actual offer/acknowledgment path.
//! The fixture is not fitted arithmetic or language behavior.
use super::{
    artifact::Artifact,
    policy::{Environment, Intervention, Params, Policy, GATES},
    session::Session,
};
use crate::native_geometric::{
    addressed_attention::{
        artifact::BoundGeometry,
        objects::{Action, Reference, Symbol},
    },
    hamming_refinement::metric::Metric,
};

fn authored_add_two(g: &BoundGeometry) -> Params {
    let q = g.identity();
    let n = if q == 0 { 1 } else { 0 };
    let mut params = Params {
        seed: 973,
        tables: vec![0; GATES],
    };
    for (lane, root) in [q, q, n, n].into_iter().enumerate() {
        let signature = g.signature(root).unwrap();
        for bit in 0..120 {
            if (signature[bit >> 6] >> (bit & 63)) & 1 != 0 {
                params.tables[1280 + lane * 120 + bit] = u16::MAX;
            }
        }
    }
    // Length one, AddAB and the byte '2' have the only positive scores.
    for output in [480, 736 + (3 << 2), 768 + (usize::from(b'2') << 2)] {
        for bit in 0..4 {
            params.tables[1280 + output + bit] = u16::MAX;
        }
    }
    params
}

#[test]
fn hamming_policy_actual_session_publishes_only_matching_complete_result() {
    let g = BoundGeometry::canonical().unwrap();
    let metric = Metric::new(&g).unwrap();
    let params = authored_add_two(&g);
    let artifact = Artifact::new(params.clone(), &g).unwrap();
    let environment = Environment {
        initial_roots: [g.identity(); 4],
        last_observed: None,
        zeta_bins: [0; 4],
        position: 0,
    };
    for actual in [b'2', b'3'] {
        let epoch = 31;
        let mut policy = Policy::new(params.clone(), &g, environment, Intervention::Full).unwrap();
        let mut session = Session::new(&artifact, &g, epoch).unwrap();
        let input = session
            .observe_input(b'1', &g, &metric, &mut policy)
            .unwrap();
        assert_eq!(input.keys, Some([g.identity(); 2]));
        assert_eq!(session.objects().result_frontier(), 1);
        let prediction = session.predict(&g, &metric, &mut policy).unwrap();
        assert_eq!(prediction.offer.symbol, Symbol::Byte(b'2'));
        assert_eq!(prediction.offer.action, Action::AddAB);
        let source = Reference::Occurrence {
            epoch,
            turn: 0,
            start: 0,
            end: 1,
        };
        for hop in prediction.refinement.hops[..2].iter().flatten() {
            let selected = hop.selected.unwrap();
            assert_eq!(selected.reference(), source);
            assert_eq!(selected.payload(), b"1");
        }
        assert_eq!(prediction.refinement.candidate_scores, 4);
        // A proposal alone is not a committed canonical arithmetic result.
        assert_eq!(session.objects().result_frontier(), 1);
        let calls = policy.audit().calls;
        let imports = session.snapshot_imports;
        assert_eq!(
            session.predict(&g, &metric, &mut policy).unwrap(),
            prediction
        );
        assert_eq!(policy.audit().calls, calls);
        assert_eq!(session.snapshot_imports, imports);
        let observed = session
            .observe(Symbol::Byte(actual), prediction.offer.id, &g, &mut policy)
            .unwrap();
        assert_eq!(observed.keys, Some([g.identity(); 2]));
        assert_eq!(observed.roots, session.roots());
        assert_eq!(session.objects().frontier(), 2);
        assert!(!session.objects().key_pending());
        assert!(session.objects().pending().is_none());
        assert_eq!(session.objects().occurrence(epoch, 1).unwrap().byte, actual);
        if actual == b'2' {
            assert_eq!(observed.result_frontier, 2);
            let result = session.objects().result(epoch, 1).unwrap();
            assert_eq!(result.lease.payload(), b"2");
            assert_eq!(result.derivation.action, Action::AddAB);
            assert_eq!(result.derivation.operands, [source; 2]);
            assert_eq!(result.derivation.operand_values, [1; 2]);
            assert_eq!(result.derivation.value, 2);
            assert_eq!(Some(result.lease.keys()), observed.keys);
            assert_eq!(result.lease.roots(), observed.roots);
            // Both ordinary occurrences, the published result and Null are now
            // admitted on each hop. The oldest equal-key occurrence still wins.
            let next = session.predict(&g, &metric, &mut policy).unwrap();
            assert_eq!(next.refinement.candidate_scores, 8);
        } else {
            assert_eq!(observed.result_frontier, 1);
            assert!(session.objects().result(epoch, 1).is_err());
            assert!(session.objects().active().is_none());
            let next = session.predict(&g, &metric, &mut policy).unwrap();
            assert_eq!(next.refinement.candidate_scores, 6);
        }
    }
}
