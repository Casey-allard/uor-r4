#![forbid(unsafe_code)]

use serde_json::{json, Value};
use std::error::Error;
use std::fs;
use uor_r4_count_runtime::CompactCountLookup;

const TRAIN: &str = "8d94e88ac31305ad6973734181a9727f520da46a5ecc263b944247ceedd07030";
const COUNTS: &str = "10bf54111de5c0cb471ed3c96365710431f1ebae13aef85900134d5871e30955";
const HIGHEST: usize = 10;
const RETAIN_ALL_THROUGH: usize = 3;
const FILE_HEADERS: u64 = 44 + 72 * (HIGHEST as u64 + 1);
const THRESHOLDS: usize = 13;

#[derive(Copy, Clone, Default)]
struct Mass {
    contexts: u64,
    followers: u64,
    raw: u64,
    adjusted: u64,
}

impl Mass {
    fn add(&mut self, other: Self) {
        self.contexts += other.contexts;
        self.followers += other.followers;
        self.raw += other.raw;
        self.adjusted += other.adjusted;
    }

    fn bytes(self, order: usize) -> u64 {
        let index_stride = (12 * order).div_ceil(8) as u64 + 4;
        let follower_stride = if order == HIGHEST { 6 } else { 10 };
        self.contexts * index_stride + self.followers * follower_stride
    }

    fn row(self, order: usize) -> Value {
        json!({"order": order, "contexts": self.contexts, "followers": self.followers,
            "raw_support": self.raw, "adjusted_support": self.adjusted,
            "section_data_bytes": self.bytes(order)})
    }
}

#[derive(Default)]
struct Order {
    last: Option<u128>,
    current: Mass,
    bins: [Mass; THRESHOLDS],
}

impl Order {
    fn finish_context(&mut self) {
        if self.current.contexts != 0 {
            let bin = (self.current.raw.ilog2() as usize).min(THRESHOLDS - 1);
            self.bins[bin].add(self.current);
            self.current = Mass::default();
        }
    }

    fn observe(&mut self, key: u128, raw: u32, adjusted: u32) {
        if self.last != Some(key) {
            self.finish_context();
            self.last = Some(key);
            self.current.contexts = 1;
        }
        self.current.followers += 1;
        self.current.raw += u64::from(raw);
        self.current.adjusted += u64::from(adjusted);
    }

    fn retained(&self, minimum_bin: usize) -> Mass {
        self.bins[minimum_bin..]
            .iter()
            .fold(Mass::default(), |mut sum, &bin| {
                sum.add(bin);
                sum
            })
    }
}

fn digest(hex_digest: &str) -> Result<[u8; 32], hex::FromHexError> {
    let mut result = [0; 32];
    hex::decode_to_slice(hex_digest, &mut result)?;
    Ok(result)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let [_, compact] = args.as_slice() else {
        return Err("usage: d33-retention-inventory C1.compact.counts".into());
    };
    let bytes = fs::read(compact)?;
    let counts = CompactCountLookup::from_bytes(&bytes, digest(TRAIN)?, digest(COUNTS)?)?;
    if counts.order_count() != HIGHEST + 1 {
        return Err("inventory expects all eleven orders".into());
    }
    let mut orders = (0..counts.order_count())
        .map(|_| Order::default())
        .collect::<Vec<_>>();
    counts.visit_counts(|order, key, _, raw, adjusted| {
        orders[order as usize].observe(key, raw, adjusted);
    })?;
    for order in &mut orders {
        order.finish_context();
    }

    let mut sweep = Vec::with_capacity(THRESHOLDS);
    for minimum_bin in 0..THRESHOLDS {
        let mut encoded_bytes = FILE_HEADERS;
        let rows = orders
            .iter()
            .enumerate()
            .map(|(order, histogram)| {
                let retained = histogram.retained(if order <= RETAIN_ALL_THROUGH {
                    0
                } else {
                    minimum_bin
                });
                encoded_bytes += retained.bytes(order);
                retained.row(order)
            })
            .collect::<Vec<_>>();
        sweep.push(json!({"minimum_raw_context_support": 1u64 << minimum_bin,
            "compact_count_bytes": encoded_bytes, "orders": rows}));
    }
    if sweep[0]["compact_count_bytes"].as_u64() != Some(bytes.len() as u64) {
        return Err("retention inventory failed to reproduce physical compact size".into());
    }
    let report = json!({
        "schema": "uor-r4.d33.retention-inventory/1", "train_sha256": TRAIN,
        "compact_sha256": COUNTS, "physical_count_bytes": bytes.len(),
        "fixed_count_header_bytes": FILE_HEADERS, "always_keep_through_order": RETAIN_ALL_THROUGH,
        "rows": sweep
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
