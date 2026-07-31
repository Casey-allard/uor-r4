//! Issue #246 quality probe: new sparse-QR routing versus the #245
//! content-reconnection cosine baseline.

use uor_r4_router::UorR4Router;

const ID: &str = "user:zeta-quality";
const CORPUS: [&str; 6] = [
    "galaxy rotation reveals a supermassive black hole",
    "bread dough rises when yeast produces carbon dioxide",
    "planets orbit the sun in elliptical paths",
    "glaciers carve valleys as they advance slowly",
    "the chef seasoned soup with fresh basil",
    "telescopes gather light from distant ancient stars",
];
const QUERIES: [(&str, usize); 6] = [
    ("galaxy black hole rotation", 0),
    ("yeast makes dough rise", 1),
    ("elliptical orbit planets", 2),
    ("glaciers carve valleys", 3),
    ("soup seasoned basil", 4),
    ("distant stars telescope light", 5),
];

fn cosine(left: &[f64], right: &[f64]) -> f64 {
    let dot = left
        .iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum::<f64>();
    let left_norm = left.iter().map(|value| value * value).sum::<f64>().sqrt();
    let right_norm = right.iter().map(|value| value * value).sum::<f64>().sqrt();
    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm * right_norm)
    }
}

fn rank_metrics(router: &mut UorR4Router, stored: &[Vec<f64>]) -> (f64, f64) {
    let mut top1 = 0.0;
    let mut reciprocal_rank = 0.0;
    for (query_index, &(query, target)) in QUERIES.iter().enumerate() {
        let scratch = format!("user:zeta-baseline-{query_index}");
        router.index_sentence(query, &scratch);
        let query_vector = router.corpus_items_for(&scratch)[0].state_vector.clone();
        let mut ranked: Vec<(usize, f64)> = stored
            .iter()
            .enumerate()
            .map(|(index, vector)| (index, cosine(&query_vector, vector)))
            .collect();
        ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
        let rank = ranked
            .iter()
            .position(|(index, _)| *index == target)
            .expect("every target is indexed")
            + 1;
        if rank == 1 {
            top1 += 1.0;
        }
        reciprocal_rank += 1.0 / rank as f64;
    }
    (
        top1 / QUERIES.len() as f64,
        reciprocal_rank / QUERIES.len() as f64,
    )
}

#[test]
fn sparse_qr_quality_delta_against_content_reconnection() {
    let mut router = UorR4Router::new(0.5);
    for sentence in CORPUS {
        router.index_sentence(sentence, ID);
    }
    let stored: Vec<Vec<f64>> = router
        .corpus_items_for(ID)
        .iter()
        .map(|item| item.state_vector.clone())
        .collect();

    // #245 baseline: rank the content-derived stored vectors directly.
    let (baseline_top1, baseline_mrr) = rank_metrics(&mut router, &stored);

    // #246 path: route and retrieve through the sparse QR windows.
    let mut routed_top1 = 0.0;
    let mut routed_mrr = 0.0;
    for &(query, target) in &QUERIES {
        let results = router.get_top_resonances_native(query, ID, CORPUS.len());
        let rank = results
            .iter()
            .position(|result| result.sentence == CORPUS[target])
            .map(|rank| rank + 1)
            .unwrap_or(CORPUS.len() + 1);
        if rank == 1 {
            routed_top1 += 1.0;
        }
        routed_mrr += 1.0 / rank as f64;
    }
    routed_top1 /= QUERIES.len() as f64;
    routed_mrr /= QUERIES.len() as f64;

    println!("issue #246 quality delta: baseline top1={baseline_top1:.3} MRR={baseline_mrr:.3}; ");
    println!(
        "  sparse-QR routed top1={routed_top1:.3} MRR={routed_mrr:.3}; delta top1={:.3} MRR={:.3}",
        routed_top1 - baseline_top1,
        routed_mrr - baseline_mrr
    );

    assert!((0.0..=1.0).contains(&baseline_top1));
    assert!((0.0..=1.0).contains(&baseline_mrr));
    assert!((0.0..=1.0).contains(&routed_top1));
    assert!((0.0..=1.0).contains(&routed_mrr));
}
