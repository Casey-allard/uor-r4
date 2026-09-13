//! Offline exact finite loss contrasts through the actual deterministic session.
//! This is not a derivative, unbiased gradient estimator, or fitted optimizer.
use super::{
    artifact::Artifact,
    policy::{Environment, Intervention, Policy},
    session::{Observation, Prediction, Session},
};
use crate::native_geometric::{
    addressed_attention::{artifact::BoundGeometry, objects::Symbol, pilot_data::Example},
    hamming_refinement::metric::Metric,
};
use serde_json::{json, Value};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub fn environment(g: &BoundGeometry) -> Environment {
    Environment {
        last_observed: None,
        zeta_bins: [0; 4],
        position: 0,
        initial_roots: [g.identity(); 4],
    }
}
pub fn ce(scores: &[u8; 257], target: Symbol) -> f64 {
    let maximum = f64::from(*scores.iter().max().unwrap_or(&0));
    let target = match target {
        Symbol::Byte(b) => usize::from(b),
        Symbol::Eos => 256,
    };
    maximum
        + scores
            .iter()
            .map(|&x| (f64::from(x) - maximum).exp())
            .sum::<f64>()
            .ln()
        - f64::from(scores[target])
}
pub fn ce_reference(scores: &[u8; 257], target: Symbol) -> f64 {
    let target = match target {
        Symbol::Byte(b) => usize::from(b),
        Symbol::Eos => 256,
    };
    scores.iter().map(|&x| f64::from(x).exp()).sum::<f64>().ln() - f64::from(scores[target])
}
fn refinement(r: &crate::native_geometric::hamming_refinement::runtime::Refinement) -> Value {
    json!({"initial":r.initial_roots,"roots":r.roots,"frontier":r.frontier,"turn":r.turn,"candidate_scores":r.candidate_scores,
        "hops":r.hops[..2].iter().flatten().map(|h|json!({"query":h.query.roots,"null":h.query.null,"before":h.before,"delta":h.delta,"after":h.after,
        "selected":h.selected.map(|l|json!({"reference":format!("{:?}",l.reference()),"roots":l.roots(),"keys":l.keys(),"payload":l.payload()})),"distance":h.distance})).collect::<Vec<_>>()})
}
fn observed(o: &Observation) -> Value {
    json!({"actual":symbol(o.actual),"refinement":refinement(&o.refinement),"roots":o.roots,"keys":o.keys,"result_frontier":o.result_frontier})
}
fn predicted(p: &Prediction) -> Value {
    json!({"offered":symbol(p.offer.symbol),"action":format!("{:?}",p.offer.action),"offer":p.offer.id,"refinement":refinement(&p.refinement),"scores":p.scores.as_slice()})
}
pub fn symbol(s: Symbol) -> u16 {
    match s {
        Symbol::Byte(b) => u16::from(b),
        Symbol::Eos => 256,
    }
}
#[derive(Debug)]
pub struct Trajectory {
    pub loss: f64,
    pub reference_loss: f64,
    pub correct: usize,
    pub positions: usize,
    pub trace: Vec<Value>,
    pub calls: u64,
    pub cells: Vec<u32>,
    pub snapshots: u64,
    pub snapshot_bytes: u64,
}
/// Every invocation starts from an empty bound session and ingests the document.
/// Targets are handed to the scorer only after prediction, then observed once.
pub fn trajectory(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    e: &Example,
    control: Intervention,
) -> Result<Trajectory> {
    if e.prompt.len() + e.answer.len() + 1 > 96 {
        return Err("96-position conformance limit".into());
    }
    let mut s = Session::new(a, g, 1)?;
    let mut p = Policy::new(a.params().clone(), g, environment(g), control)?;
    let mut trace = Vec::new();
    for &b in &e.prompt {
        trace.push(json!({"input":observed(&s.observe_input(b,g,m,&mut p)?)}));
    }
    let mut loss = 0.;
    let mut reference_loss = 0.;
    let mut correct = 0;
    for target in e
        .answer
        .iter()
        .copied()
        .map(Symbol::Byte)
        .chain(std::iter::once(Symbol::Eos))
    {
        let offer = s.predict(g, m, &mut p)?;
        let before_calls = p.audit().calls;
        if s.predict(g, m, &mut p)? != offer || p.audit().calls != before_calls {
            return Err("offer repeat reevaluated".into());
        }
        loss += ce(&offer.scores, target);
        reference_loss += ce_reference(&offer.scores, target);
        correct += usize::from(offer.offer.symbol == target);
        let observation = s.observe(target, offer.offer.id, g, &mut p)?;
        trace.push(json!({"prediction":predicted(&offer),"observation":observed(&observation)}));
    }
    Ok(Trajectory {
        loss,
        reference_loss,
        correct,
        positions: e.answer.len() + 1,
        trace,
        calls: p.audit().calls,
        cells: p.audit().cells.clone(),
        snapshots: s.snapshot_imports,
        snapshot_bytes: s.snapshot_bytes,
    })
}
pub fn generate(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    e: &Example,
    control: Intervention,
) -> Result<Value> {
    if e.prompt.len() > 64 {
        return Err("prompt limit".into());
    }
    let mut s = Session::new(a, g, 1)?;
    let mut p = Policy::new(a.params().clone(), g, environment(g), control)?;
    for &b in &e.prompt {
        s.observe_input(b, g, m, &mut p)?;
    }
    let mut tokens = Vec::new();
    let mut trace = Vec::new();
    for _ in 0..64 {
        let offer = s.predict(g, m, &mut p)?;
        tokens.push(symbol(offer.offer.symbol));
        let o = s.observe(offer.offer.symbol, offer.offer.id, g, &mut p)?;
        trace.push(json!({"prediction":predicted(&offer),"observation":observed(&o)}));
        if s.ended() {
            break;
        }
    }
    let expected: Vec<_> = e
        .answer
        .iter()
        .map(|&b| u16::from(b))
        .chain(std::iter::once(256))
        .collect();
    Ok(
        json!({"id":e.id,"family":e.family,"control":format!("{control:?}"),"prompt":e.prompt,"expected_tokens":expected,"tokens":tokens,"exact":tokens==expected,"eos":s.ended(),"calls":p.audit().calls,"snapshots":s.snapshot_imports,"snapshot_bytes":s.snapshot_bytes,"trace":trace}),
    )
}
