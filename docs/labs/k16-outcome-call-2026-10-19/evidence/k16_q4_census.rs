// K16 Q4 (cursor 16,404) coverage census, Amendment 7 / D39 item 2.
// Host grouping and counting of stored codebook tables only: no model
// execution and no cell reconstruction. Built at the frozen-lineage commit so
// the driver source hash equals the run's.
use uor_r4_discrete::corpus::{self, CorpusSource, FilePin, PreparedCorpus};
use uor_r4_discrete::training::{driver_source_sha256, DriverCheckpoint};

fn fail(m: &str) -> ! {
    eprintln!("FAIL: {m}");
    std::process::exit(1)
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() < 5 || (a.len() - 3) % 2 != 0 {
        fail("usage: k16_q4_census DIR DEV_TOKENS DEV_MASK (ROLE RAW_SHA256)...");
    }
    let (dir, dev_tokens, dev_mask) = (&a[0], &a[1], &a[2]);
    let pinned = "9a1b121b0d8896bdba85baed40d95834f01bdb9b22d48167c374e34de8fa069e";
    if driver_source_sha256() != pinned {
        fail("census build driver hash differs from the frozen pin");
    }
    let mut out = Vec::new();
    for c in a[3..].chunks(2) {
        let (role, raw_pin) = (&c[0], &c[1]);
        let path = format!("{dir}/report-{role}/checkpoint-16404.json");
        let bytes = std::fs::read(&path).unwrap_or_else(|_| fail(&format!("{role}: read {path}")));
        if corpus::sha256(&bytes) != *raw_pin {
            fail(&format!("{role}: raw checkpoint SHA differs from the sealed pin"));
        }
        let report: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or_else(|e| fail(&format!("{role}: {e}")));
        let ck: DriverCheckpoint = serde_json::from_value(report["checkpoint"].clone())
            .unwrap_or_else(|e| fail(&format!("{role}: checkpoint: {e}")));
        let model_bytes = serde_json::to_vec(&ck.model).unwrap_or_else(|e| fail(&format!("{e}")));
        if corpus::sha256(&model_bytes) != ck.model_sha256 {
            fail(&format!("{role}: model storage hash"));
        }
        if ck.lineage.driver_source_sha256 != pinned {
            fail(&format!("{role}: lineage driver hash"));
        }
        if ck.cursor.completed_batches != 16_404 {
            fail(&format!("{role}: not the 16,404 checkpoint"));
        }
        let cfg = &ck.lineage.config;
        let src = CorpusSource {
            tokens: FilePin { path: dev_tokens.into(), sha256: cfg.dev.tokens.sha256.clone() },
            mask: FilePin { path: dev_mask.into(), sha256: cfg.dev.mask.sha256.clone() },
            ..cfg.dev.clone()
        };
        let dev = PreparedCorpus::load(&src, cfg.spec.vocabulary, &cfg.bos)
            .unwrap_or_else(|e| fail(&format!("{role} dev: {e}")));
        let b = dev.binding();
        let l = &ck.lineage.dev;
        if (b.payload_tokens, b.mask_selected_positions, b.eligible_targets, b.documents)
            != (l.payload_tokens, l.mask_selected_positions, l.eligible_targets, l.documents)
        {
            fail(&format!("{role}: DEV binding counts differ from lineage"));
        }
        let spec = serde_json::from_value(ck.model["spec"].clone())
            .unwrap_or_else(|e| fail(&format!("{role} spec: {e}")));
        let table: Vec<i32> = serde_json::from_value(ck.model["table"].clone())
            .unwrap_or_else(|e| fail(&format!("{role} table: {e}")));
        let r = uor_r4_discrete::coverage_census::census(
            &table,
            spec,
            &dev.eligible_targets(),
            None,
            Some(1941),
        )
        .unwrap_or_else(|e| fail(&format!("{role}: {e}")));
        out.push(serde_json::json!({
            "role": role, "checkpoint_report_raw_sha256": raw_pin,
            "model_sha256": ck.model_sha256,
            "cursor": {"completed_batches": ck.cursor.completed_batches, "accepted_updates": ck.cursor.accepted_updates},
            "distinct_vectors": r["distinct_vectors"],
            "alias_group_size_histogram": r["alias_group_size_histogram"],
            "dev_rows": r["dev_rows"],
            "dev_alias_ceiling_rows_raw": r["dev_alias_ceiling_rows"],
            "dev_targets_alias_representative": r["dev_targets_alias_representative"],
            "dev_distinct_targets": r["dev_distinct_targets"],
            "within_cell_tie_mass": r["within_cell_tie_mass"],
            "g_cap_raw_verdict": r["g_cap"]["verdict"],
            "cells": r["cells"]["status"]
        }));
    }
    let doc = serde_json::json!({
        "schema": "uor-r4.k16-q4-coverage-census/1",
        "driver_source_sha256": pinned, "bar_rows": 1941, "roles": out,
        "model_execution": false, "host_cell_reconstruction": false
    });
    match serde_json::to_string_pretty(&doc) {
        Ok(s) => println!("{s}"),
        Err(e) => fail(&format!("{e}")),
    }
}
