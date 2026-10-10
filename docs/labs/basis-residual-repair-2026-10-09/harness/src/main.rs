//! Saved constructor with an explicitly patched, residual-qualified local LU backend.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs,
    io::BufReader,
    path::{Path, PathBuf},
    time::Instant,
};
#[path = "../../../../../crates/uor-r4-training/examples/geometric_frozen_map_fit/protected_legal_set.rs"]
mod constructor;
#[path = "../../../../../crates/uor-r4-integer/src/report_output.rs"]
mod report_output;
const SOURCE: &[u8] = include_bytes!("../../../../../crates/uor-r4-training/examples/geometric_frozen_map_fit/protected_legal_set.rs");
const SOURCE_SHA: &str = "f0242836d08dbadbb5659cd8f158d79c8de9420c9de2fb213de9ef149510c24b";
const REPAIR: &[u8] = include_bytes!("../../repair.rs");
const INSTALLER: &[u8] = include_bytes!("../../apply-repair.py");
const PATCH: &[u8] = include_bytes!("../../../legal-basis-diagnosis-2026-10-09/observation.patch");
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    schema: String,
    master_bits: Vec<u32>,
    gradient_bits: Vec<u32>,
    jacobian_bits: Vec<Vec<u32>>,
    provenance: serde_json::Value,
}
#[derive(Serialize)]
struct VariableMapping {
    backend_variable: usize,
    coordinate: usize,
    family: &'static str,
    family_index: usize,
    component: &'static str,
    lower: i32,
    upper: i32,
}
fn hash(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn write(p: &Path, v: &impl Serialize) -> Result<(), Box<dyn Error>> {
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(p)?;
    serde_json::to_writer_pretty(&mut f, v)?;
    f.write_all(b"\n")?;
    Ok(())
}
fn unit_rows(j: &[Vec<f32>]) -> Result<Vec<Vec<f64>>, Box<dyn Error>> {
    j.iter()
        .map(|v| {
            if v.len() != 1920 || v.iter().any(|x| !x.is_finite()) {
                return Err("malformed J row".into());
            }
            let mut r = v.iter().map(|x| f64::from(*x)).collect::<Vec<_>>();
            let n = r.iter().map(|x| x * x).sum::<f64>().sqrt();
            if !n.is_finite() {
                return Err("J norm overflow".into());
            }
            if n > 0. {
                for x in &mut r {
                    *x /= n;
                }
            }
            Ok(r)
        })
        .collect()
}
fn mapping(m: &[f32]) -> (Vec<VariableMapping>, Vec<serde_json::Value>) {
    let mut vars = Vec::new();
    let mut rows = Vec::new();
    for (i, &m) in m.iter().enumerate() {
        let q0 = (m * 4.).round() as i32;
        let family = if i < 960 { "prefix" } else { "generate.unary" };
        let fi = i % 960;
        vars.push(VariableMapping {
            backend_variable: vars.len(),
            coordinate: i,
            family,
            family_index: fi,
            component: "k=q-q0",
            lower: -7 - q0,
            upper: 7 - q0,
        });
        if m != (q0 as f32) * 0.25 {
            for component in ["lower_branch_l", "upper_branch_r"] {
                vars.push(VariableMapping {
                    backend_variable: vars.len(),
                    coordinate: i,
                    family,
                    family_index: fi,
                    component,
                    lower: 0,
                    upper: 1,
                });
            }
            for kind in ["side_sum_le1", "centered_lower_ge0", "centered_upper_le0"] {
                rows.push(
                    serde_json::json!({"model_constraint":rows.len(),"kind":kind,"coordinate":i}),
                );
            }
        }
    }
    for i in 0..380 {
        rows.push(serde_json::json!({"model_constraint":rows.len(),"kind":"unit_guard_ge0","guard_index":i}));
    }
    (vars, rows)
}
fn execute(input: &Path, expected: &str, out: &Path) -> Result<serde_json::Value, Box<dyn Error>> {
    let started = Instant::now();
    if hash(SOURCE) != SOURCE_SHA {
        return Err("constructor source identity differs".into());
    }
    let meta = fs::metadata(input)?;
    if !meta.is_file() || meta.len() > 16 * 1024 * 1024 {
        return Err("input must be regular file <=16MiB".into());
    }
    let bytes = fs::read(input)?;
    if hash(&bytes) != expected {
        return Err("saved input SHA mismatch".into());
    }
    let x: Input = serde_json::from_reader(BufReader::new(bytes.as_slice()))?;
    if x.schema != "uor-r4.legal-basis-input/1"
        || x.master_bits.len() != 1920
        || x.gradient_bits.len() != 1920
        || x.jacobian_bits.len() != 380
    {
        return Err("input shape/schema differs".into());
    }
    let m = x
        .master_bits
        .iter()
        .map(|x| f32::from_bits(*x))
        .collect::<Vec<_>>();
    let g = x
        .gradient_bits
        .iter()
        .map(|x| f32::from_bits(*x))
        .collect::<Vec<_>>();
    let j = x
        .jacobian_bits
        .iter()
        .map(|r| r.iter().map(|x| f32::from_bits(*x)).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let rows = unit_rows(&j)?;
    let (variables, constraints) = mapping(&m);
    write(
        &out.join("model-mapping.json"),
        &serde_json::json!({"variables":variables,"constraints":constraints,"source_sha256":SOURCE_SHA,"internal_row_mapping":"model order is authoritative; backend context contains row scales and full internal basic/nonbasic/slack mapping; no assumption of identity without inspection"}),
    )?;
    microlp::diagnostics::configure(&out.join("backend-observations"))?;
    let receipt = constructor::run(&m, &g, &rows)?;
    constructor::authenticate(&m, &g, &rows, &receipt)?;
    let (factorizations, singular_events) = microlp::diagnostics::finish()?;
    write(&out.join("constructor-receipt.json"), &receipt)?;
    Ok(
        serde_json::json!({"status":"COMPLETED_SAVED_PATCHED_BACKEND","schema":"uor-r4.basis-residual-repair/1","input_sha256":expected,"input_bytes":meta.len(),"constructor_source_sha256":SOURCE_SHA,"observation_patch_sha256":hash(PATCH),"repair_source_sha256":hash(REPAIR),"repair_installer_sha256":hash(INSTALLER),"actual_backend":"microlp=0.6.0 + corrected observation + scale-aware residual-qualified LU patch; nested receipt base-version field is not unmodified upstream","producer_provenance":x.provenance,"factorizations_observed":factorizations,"singular_events":singular_events,"backend_status":receipt.status,"backend_termination":receipt.termination,"elapsed_seconds":started.elapsed().as_secs_f64(),"scope":"Exact saved LP constructor replay only; zero training/backward/native scoring/native candidate acceptance; singular capture is numerical execution evidence, not rank/condition-number/infeasibility proof"}),
    )
}

fn verification_json(v: &microlp::repair::Verification) -> serde_json::Value {
    serde_json::json!({
        "dimension":v.dimension, "rhs_checked_per_direction":v.rhs_checked_per_direction,
        "max_normal_error":v.max_normal_error, "max_transpose_error":v.max_transpose_error,
        "max_sparse_normal_error":v.max_sparse_normal_error,
        "max_sparse_transpose_error":v.max_sparse_transpose_error,
        "worst_normal_rhs":v.worst_normal_rhs,"worst_transpose_rhs":v.worst_transpose_rhs,
        "worst_sparse_normal_rhs":v.worst_sparse_normal_rhs,"worst_sparse_transpose_rhs":v.worst_sparse_transpose_rhs,
        "tolerance":v.tolerance
    })
}
fn execute_basis(
    input: &Path,
    expected: &str,
    out: &Path,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let started = Instant::now();
    let bytes = fs::read(input)?;
    if hash(&bytes) != expected || bytes.len() > 16 * 1024 * 1024 {
        return Err("basis identity/size differs".into());
    }
    let text = std::str::from_utf8(&bytes)?;
    let mut columns: Vec<Vec<(usize, f64)>> = Vec::new();
    let mut sizes = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        match fields.as_slice() {
            ["COLUMN", c, count] => {
                if c.parse::<usize>()? != columns.len() {
                    return Err("column order differs".into());
                }
                columns.push(Vec::new());
                sizes.push(count.parse::<usize>()?);
            }
            ["ENTRY", c, r, bits] => {
                let c = c.parse::<usize>()?;
                let r = r.parse::<usize>()?;
                if columns.is_empty() || c != columns.len() - 1 || r >= 440 {
                    return Err("entry coordinate differs".into());
                }
                let x = f64::from_bits(u64::from_str_radix(bits, 16)?);
                if !x.is_finite() || columns[c].iter().any(|(row, _)| *row == r) {
                    return Err("invalid or duplicate entry".into());
                }
                columns[c].push((r, x));
            }
            _ => return Err("unknown basis line".into()),
        }
    }
    if columns.len() != 440 || columns.iter().zip(sizes).any(|(c, n)| c.len() != n) {
        return Err("basis dimensions/counts differ".into());
    }
    microlp::diagnostics::configure(&out.join("backend-observations"))?;
    let result = microlp::repair::qualify_saved_basis(&columns);
    let (factors, singular) = microlp::diagnostics::finish()?;
    Ok(serde_json::json!({
        "schema":"uor-r4.saved-basis-qualification/1",
        "status":if result.is_ok() {"QUALIFIED"} else {"NUMERICALLY_REJECTED"},
        "basis_sha256":expected, "stored_entries":columns.iter().map(Vec::len).sum::<usize>(),
        "verification":result.as_ref().ok().map(verification_json),
        "error":result.as_ref().err(),
        "factorizations_observed":factors,"singular_events":singular,
        "repair_source_sha256":hash(REPAIR),"repair_installer_sha256":hash(INSTALLER),
        "actual_backend":"microlp=0.6.0 + corrected observation + scale-aware residual-qualified LU",
        "elapsed_seconds":started.elapsed().as_secs_f64(),
        "scope":"Saved original matrix; all unit RHS normal and transpose, dense and sparse fresh-factor solves. No eta update, rank, condition number, forward-error, global feasibility or model quality claim."
    }))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() == 2 && args[0] == "--verify-report" {
        report_output::verify(Path::new(&args[1]))?;
        return Ok(());
    }
    if args.len() != 6
        || !["--input", "--basis"].contains(&args[0].as_str())
        || args[2] != "--expected-sha256"
        || args[4] != "--output"
        || args[3].len() != 64
        || !args[3].bytes().all(|x| x.is_ascii_hexdigit())
    {
        return Err(
            "usage: --input/--basis FILE --expected-sha256 HEX --output EXCLUSIVE_DIR".into(),
        );
    }
    let input = PathBuf::from(&args[1]);
    let out = PathBuf::from(&args[5]);
    if out.exists() {
        return Err("output already exists".into());
    }
    report_output::claim(&out)?;
    let result = if args[0] == "--basis" {
        execute_basis(&input, &args[3], &out)
    } else {
        execute(&input, &args[3], &out)
    };
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => serde_json::json!({"status":"FAILED","error":e.to_string()}),
    };
    write(&out.join("report.json"), &report)?;
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_zero_rows_and_mapping() {
        let m = vec![-0f32; 1920];
        let (v, c) = mapping(&m);
        assert_eq!(v.len(), 1920);
        assert_eq!(c.len(), 380);
        assert_eq!(v[0].lower, -7);
        assert_eq!(v[1919].family, "generate.unary");
        assert_eq!(unit_rows(&vec![vec![0.; 1920]; 380]).unwrap().len(), 380);
        assert_eq!(hash(SOURCE), SOURCE_SHA);
    }
    #[test]
    fn fractional_mapping_preserves_insertion_order() {
        let mut m = vec![0.; 1920];
        m[2] = 0.13;
        let (v, c) = mapping(&m);
        assert_eq!(v.len(), 1922);
        assert_eq!(v[2].component, "k=q-q0");
        assert_eq!(v[3].component, "lower_branch_l");
        assert_eq!(v[4].component, "upper_branch_r");
        assert_eq!(c.len(), 383);
        assert_eq!(c[3]["guard_index"], 0);
    }
}
