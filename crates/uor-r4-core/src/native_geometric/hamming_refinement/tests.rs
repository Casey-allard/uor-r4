use super::{metric::Metric, runtime::*};
use crate::native_geometric::addressed_attention::{
    artifact::BoundGeometry,
    objects::{Action, ObjectSession, Symbol},
};

pub(super) struct SharedPolicy {
    pub null: [u16; 2],
    pub suppress_update: bool,
    pub first_root_only: bool,
}
impl Policy for SharedPolicy {
    fn identity(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"authored-hamming-causal-fixture-policy/1");
        h.update(&self.null[0].to_le_bytes());
        h.update(&self.null[1].to_le_bytes());
        h.update(&[
            u8::from(self.suppress_update),
            u8::from(self.first_root_only),
        ]);
        *h.finalize().as_bytes()
    }
    fn query(&mut self, c: &Context<'_>) -> Result<Query> {
        Ok(Query {
            roots: [c.roots[0], c.roots[1]],
            null: self.null,
        })
    }
    fn extent(&mut self, _: &SelectedContext<'_>, _: &[bool; 64]) -> Result<u8> {
        Ok(1)
    }
    fn delta(&mut self, c: &SelectedContext<'_>) -> Result<[u16; 4]> {
        // Authored transport for a finite source-dependence fixture, not a fitted
        // policy or a token-family rule. Every round uses this same function.
        let r = c.selected.roots();
        if self.suppress_update {
            Ok(c.context.initial_roots)
        } else if self.first_root_only {
            Ok([r[0]; 4])
        } else {
            Ok([r[3], r[2], r[1], r[0]])
        }
    }
}
pub(super) fn fixture(
    g: &BoundGeometry,
    changed: bool,
    reordered: bool,
) -> (Memory, SharedPolicy, [u16; 4], [u16; 2]) {
    let metric = Metric::new(g).unwrap();
    let i = g.identity();
    let mut roots = vec![i];
    for r in 0..120 {
        if roots
            .iter()
            .all(|&old| metric.root_distance(old, r).unwrap() > 0)
        {
            roots.push(r);
        }
        if roots.len() == 4 {
            break;
        }
    }
    assert_eq!(roots.len(), 4);
    let (r, s, n) = (roots[1], roots[2], roots[3]);
    let mut m = Memory::new([19; 32], g, 7);
    let records = [
        (b'L', [i, i], [i, i, i, if changed { s } else { r }]),
        (b'R', [r, i], [i; 4]),
        (b'S', [s, i], [i; 4]),
    ];
    for index in if reordered { [2, 0, 1] } else { [0, 1, 2] } {
        let (byte, keys, state) = records[index];
        m.observe_input(byte, keys, state).unwrap();
    }
    (
        m,
        SharedPolicy {
            null: [n, n],
            suppress_update: false,
            first_root_only: false,
        },
        [i; 4],
        [r, s],
    )
}
pub(super) fn selected_bytes(r: &Refinement) -> Vec<Option<Vec<u8>>> {
    r.hops[..usize::from(r.hop_count)]
        .iter()
        .map(|h| {
            h.as_ref()
                .and_then(|h| h.selected)
                .map(|l| l.payload().to_vec())
        })
        .collect()
}
#[test]
fn hamming_refinement_reads_update_then_read_again_with_fourth_context_root() {
    let g = BoundGeometry::canonical().unwrap();
    let metric = Metric::new(&g).unwrap();
    let (m, mut p, initial, roots) = fixture(&g, false, false);
    let before = m.objects().snapshot().unwrap();
    let full = refine(&m, &g, &metric, initial, 2, &mut p).unwrap();
    assert_eq!(
        selected_bytes(&full),
        vec![Some(b"L".to_vec()), Some(b"R".to_vec())]
    );
    assert_eq!(full.hops[1].unwrap().query.roots, [roots[0], initial[1]]);
    assert_eq!(full.hops[1].unwrap().before, full.hops[0].unwrap().after);
    assert_eq!(
        full.hops[1].unwrap().parent_digest,
        full.hops[0].unwrap().digest
    );
    assert_eq!(m.objects().snapshot().unwrap(), before);
    p.suppress_update = true;
    let frozen = refine(&m, &g, &metric, initial, 2, &mut p).unwrap();
    assert_eq!(
        selected_bytes(&frozen),
        vec![Some(b"L".to_vec()), Some(b"L".to_vec())]
    );
    p.suppress_update = false;
    p.first_root_only = true;
    let thin = refine(&m, &g, &metric, initial, 2, &mut p).unwrap();
    assert_eq!(selected_bytes(&thin), selected_bytes(&frozen));
    let (changed, mut p, initial, _) = fixture(&g, true, false);
    let changed = refine(&changed, &g, &metric, initial, 2, &mut p).unwrap();
    assert_eq!(
        selected_bytes(&changed),
        vec![Some(b"L".to_vec()), Some(b"S".to_vec())]
    );
    assert_eq!(
        full.hops[0].unwrap().selected.unwrap().payload(),
        changed.hops[0].unwrap().selected.unwrap().payload()
    );
    assert_ne!(full.digest, changed.digest);
    let (relocated, mut p, initial, _) = fixture(&g, false, true);
    let relocated = refine(&relocated, &g, &metric, initial, 2, &mut p).unwrap();
    assert_eq!(selected_bytes(&relocated), selected_bytes(&full));
    assert_ne!(
        relocated.hops[0].unwrap().selected.unwrap().reference(),
        full.hops[0].unwrap().selected.unwrap().reference()
    );
    assert_ne!(relocated.digest, full.digest);
}
struct PayloadPolicy {
    null: [u16; 2],
    length: u8,
    initial: [u16; 4],
    saw: [u8; 2],
}
impl Policy for PayloadPolicy {
    fn identity(&self) -> [u8; 32] {
        [23; 32]
    }
    fn query(&mut self, c: &Context<'_>) -> Result<Query> {
        Ok(Query {
            roots: [c.roots[0], c.roots[1]],
            null: self.null,
        })
    }
    fn extent(&mut self, _: &SelectedContext<'_>, _: &[bool; 64]) -> Result<u8> {
        Ok(self.length)
    }
    fn delta(&mut self, c: &SelectedContext<'_>) -> Result<[u16; 4]> {
        assert_eq!(c.selected.payload().len(), 2);
        self.saw.copy_from_slice(c.selected.payload());
        Ok(self.initial)
    }
}
#[test]
fn hamming_refinement_exposes_owned_full_payload_and_legal_extent() {
    let g = BoundGeometry::canonical().unwrap();
    let metric = Metric::new(&g).unwrap();
    let (mut m, p, initial, _) = fixture(&g, false, false);
    let mut policy = PayloadPolicy {
        null: p.null,
        length: 2,
        initial,
        saw: [0; 2],
    };
    let r = refine(&m, &g, &metric, initial, 1, &mut policy).unwrap();
    assert_eq!(policy.saw, *b"LR");
    let lease = r.hops[0].unwrap().selected.unwrap();
    for _ in 0..256 {
        m.observe_input(b'x', p.null, initial).unwrap();
    }
    assert_eq!(lease.payload(), b"LR");
    let mut m = Memory::new([19; 32], &g, 7);
    m.observe_input(b'L', [initial[0]; 2], initial).unwrap();
    m.begin_turn().unwrap();
    m.observe_input(b'R', p.null, initial).unwrap();
    assert!(matches!(
        refine(&m, &g, &metric, initial, 1, &mut policy),
        Err(Error::Invalid("extent"))
    ));
}
#[test]
fn hamming_refinement_bounds_null_and_no_mutation_on_policy_errors() {
    let g = BoundGeometry::canonical().unwrap();
    let metric = Metric::new(&g).unwrap();
    let (mut m, mut p, initial, _) = fixture(&g, false, false);
    let before = m.objects().snapshot().unwrap();
    for hops in [0, 5, 255] {
        assert!(refine(&m, &g, &metric, initial, hops, &mut p).is_err());
    }
    assert!(refine(&m, &g, &metric, [120; 4], 2, &mut p).is_err());
    p.null = [120; 2];
    assert!(refine(&m, &g, &metric, initial, 2, &mut p).is_err());
    assert_eq!(m.objects().snapshot().unwrap(), before);
    p.null = [initial[0]; 2];
    let null = refine(&m, &g, &metric, initial, 4, &mut p).unwrap();
    assert_eq!(selected_bytes(&null), vec![None; 4]);
    assert_eq!(null.roots, initial);
    for _ in 0..260 {
        m.observe_input(b'x', [initial[0]; 2], initial).unwrap();
    }
    let max = refine(&m, &g, &metric, initial, 4, &mut p).unwrap();
    assert_eq!(max.candidate_scores, 4 * 257);
    assert!(max.candidate_scores <= 4 * 265);
    let empty = Memory::new([19; 32], &g, 8);
    let empty = refine(&empty, &g, &metric, initial, 2, &mut p).unwrap();
    assert_eq!(empty.candidate_scores, 2);
}
#[test]
fn hamming_refinement_import_rejects_wrong_binding_and_pending_frontier() {
    let g = BoundGeometry::canonical().unwrap();
    let metric = Metric::new(&g).unwrap();
    let (_, mut p, initial, _) = fixture(&g, false, false);
    let mut objects = ObjectSession::new([19; 32], g.identity_digest(), 7);
    objects
        .observe_input(b'L', [initial[0]; 2], initial)
        .unwrap();
    let snapshot = objects.snapshot().unwrap();
    assert!(Memory::import_object_snapshot(&snapshot, [20; 32], &g).is_err());
    let m = Memory::import_object_snapshot(&snapshot, [19; 32], &g).unwrap();
    assert!(refine(&m, &g, &metric, initial, 1, &mut p).is_ok());
    objects
        .cache_offer(Symbol::Byte(b'x'), Action::Hold, None, None)
        .unwrap();
    let pending =
        Memory::import_object_snapshot(&objects.snapshot().unwrap(), [19; 32], &g).unwrap();
    assert!(matches!(
        refine(&pending, &g, &metric, initial, 1, &mut p),
        Err(Error::Invalid("uncommitted frontier"))
    ));
}
