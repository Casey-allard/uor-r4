//! One-off exact historical H4 payload compiler; no training or model loading.
//! Usage: export-h4-tables NEW_REPORT_ROOT (build with UOR_BUILD_SOURCE_COMMIT).

use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::Path;

use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_core::canonical_lexical_ingestion::{
    validate_h4_binary_icosahedral_closure,
    H4_BINARY_ICOSAHEDRAL_MULTIPLICATION_TABLE_KAPPA_REFERENCE,
    H4_BINARY_ICOSAHEDRAL_ROOT_TABLE_KAPPA_REFERENCE,
};
use uor_r4_core::native_geometric::learner::prefix_artifact::{historical_roots, ExactGroupTable};
use uor_r4_core::report_output;
use uor_r4_integer::h4_classifier::H4_ROOT_COEFFICIENTS;
use uor_r4_integer::h4_tables::{
    coefficients_sha256, mathematical_sha256, IDENTITY_FRAMING, IDENTITY_OFFSET, INVERSE_OFFSET,
    MAPPING_OFFSET, PAYLOAD_BYTES, PRODUCT_BYTES, ROOT_COUNT, ROW_STRIDE, SCHEMA,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    fs::File::create_new(path)?.write_all(bytes)?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err("usage: export-h4-tables NEW_REPORT_ROOT".into());
    }
    let source_commit = option_env!("UOR_BUILD_SOURCE_COMMIT")
        .filter(|value| value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or("build requires exact UOR_BUILD_SOURCE_COMMIT")?;
    let out = Path::new(&args[1]);
    report_output::claim(out)?;
    let result = compile(out, source_commit);
    if let Err(error) = &result {
        write_new(
            &out.join("failed-attempt.json"),
            &serde_json::to_vec_pretty(&json!({"error":error.to_string()}))?,
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result
}

fn compile(out: &Path, source_commit: &str) -> Result<()> {
    let donor = historical_roots();
    if donor.len() != ROOT_COUNT {
        return Err("historical donor root count differs".into());
    }
    for (expected, actual) in donor.iter().zip(H4_ROOT_COEFFICIENTS.iter()) {
        for axis in 0..4 {
            for coefficient in 0..2 {
                if expected[axis][coefficient] != i64::from(actual[axis][coefficient]) {
                    return Err(
                        "classifier coefficient enumeration differs from exact donor".into(),
                    );
                }
            }
        }
    }
    // Reuse the existing exact compiler, including independent canonical closure
    // comparison; do not reproduce its Z[phi] or quaternion algebra here.
    let table = ExactGroupTable::build()?;
    let closure = validate_h4_binary_icosahedral_closure()?;
    if table.max_mismatches_vs_floating != 0
        || table.identity != 1
        || table.product.len() != PRODUCT_BYTES
        || table.inverse.len() != ROOT_COUNT
        || table.historical_to_sorted.len() != ROOT_COUNT
        || closure.h4_root_table_kappa != H4_BINARY_ICOSAHEDRAL_ROOT_TABLE_KAPPA_REFERENCE
        || closure.reproduce_multiplication_table_kappa()?
            != H4_BINARY_ICOSAHEDRAL_MULTIPLICATION_TABLE_KAPPA_REFERENCE
    {
        return Err("exact historical/canonical table or floating compatibility differs".into());
    }
    let mut payload = Vec::with_capacity(PAYLOAD_BYTES);
    payload.extend_from_slice(&table.product);
    payload.extend_from_slice(&table.inverse);
    payload.extend_from_slice(&table.historical_to_sorted);
    payload.push(table.identity);
    let mathematical_sha256 = mathematical_sha256(&payload)?;
    write_new(&out.join("h4-tables.bin"), &payload)?;
    let report = json!({
        "schema":SCHEMA,
        "status":"EXACT_PAYLOAD_COMPILED_NOT_RUNTIME_ADMISSION",
        "payload":{"file":"h4-tables.bin","bytes":PAYLOAD_BYTES,"sha256":hex::encode(Sha256::digest(&payload))},
        "mathematical_sha256":mathematical_sha256,
        "identity_framing_utf8":std::str::from_utf8(IDENTITY_FRAMING)?,
        "coefficient_sha256":coefficients_sha256(),
        "coefficient_bytes":960,
        "root_count":ROOT_COUNT,"row_stride":ROW_STRIDE,"identity_index":1,
        "layout":{"product_offset":0,"inverse_offset":INVERSE_OFFSET,"historical_to_sorted_offset":MAPPING_OFFSET,"identity_offset":IDENTITY_OFFSET},
        "order":"learner.embedding.historical/1;signed roots;no antipode folding",
        "relative":"inverse(query)*key",
        "canonical_root_kappa":closure.h4_root_table_kappa,
        "canonical_product_kappa":closure.multiplication_table_kappa,
        "exact_products_compared":ROOT_COUNT*ROOT_COUNT,
        "floating_product_mismatches":table.max_mismatches_vs_floating,
        "source_commit":source_commit,
        "executable_sha256":uor_r4_integer::sha256_file(&std::env::current_exe()?)?,
        "source_sha256":{
            "exporter":hex::encode(Sha256::digest(include_bytes!("export-h4-tables.rs"))),
            "exact_donor":hex::encode(Sha256::digest(include_bytes!("../src/native_geometric/learner/prefix_artifact.rs"))),
            "canonical_closure":hex::encode(Sha256::digest(include_bytes!("../src/canonical_lexical_ingestion.rs"))),
            "classifier":hex::encode(Sha256::digest(include_bytes!("../../uor-r4-integer/src/h4_classifier.rs"))),
            "integer_boundary":hex::encode(Sha256::digest(include_bytes!("../../uor-r4-integer/src/h4_tables.rs")))
        },
        "scope":"Exact table compilation and order binding only; no learned scores, quantized Q/K transfer, model execution, language or energy claim. Pin this mathematical digest and payload fixture before integer admission tests; no public bypass loader."
    });
    write_new(
        &out.join("h4-tables.json"),
        &serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}
