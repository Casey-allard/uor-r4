use warm_cache_proto::manifest::SeedManifest;
use warm_cache_proto::*;

fn main() {
    let mut cell = KuramotoCell::new(71, PHASE_BITS);
    for _ in 0..2000 { cell.step(0.001); }
    let h = TimeHeader::from_snapshot(0, 0, 0, &cell.snapshot());
    let sizes = [4usize * 1024, 64 * 1024, 256 * 1024, 1 << 20, 4 << 20, 16 << 20];
    println!("size,cold_bytes,pages,manifest_bytes,warm_identity_bytes,warm_payload,repair_bytes,repair_payload");
    for &size in &sizes {
        let bytes: Vec<u8> = (0..size).map(|i| ((i * 17 + 3) & 0xFF) as u8).collect();
        let seed = Seed::new(bytes.clone(), 1);
        let identical = make_warm(&seed, Plan::Identity, h.clone());
        let mut target = bytes.clone();
        let page_id = 7.min(seed.page_count().saturating_sub(1));
        target[page_id * PAGE + 13] ^= 0xAA;
        let new_page = target[page_id * PAGE..(page_id + 1) * PAGE].to_vec();
        let repair = make_warm(&seed, Plan::PageRepair(vec![(page_id as u32, new_page)]), h.clone());
        let manifest = SeedManifest::from_tree(&seed.tree);
        assert_eq!(receive_warm(&seed, &h, &identical), WarmResult::Committed);
        assert_eq!(receive_warm(&seed, &h, &repair), WarmResult::Committed);
        println!("{},{},{},{},{},{},{},{}",
            size, seed.cold_wire_len(), seed.page_count(), manifest.wire_len(),
            identical.wire_len(), identical.plan.payload_len(),
            repair.wire_len(), repair.plan.payload_len());
    }
}
