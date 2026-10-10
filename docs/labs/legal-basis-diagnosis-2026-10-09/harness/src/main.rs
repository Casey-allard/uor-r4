//! Saved LP diagnosis only. The exact production constructor is compiled unchanged.
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
const PATCH: &[u8] = include_bytes!("../../observation.patch");
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
        serde_json::json!({"status":"COMPLETED_SAVED_BACKEND_DIAGNOSIS","schema":"uor-r4.legal-basis-diagnosis/1","input_sha256":expected,"input_bytes":meta.len(),"constructor_source_sha256":SOURCE_SHA,"observation_patch_sha256":hash(PATCH),"producer_provenance":x.provenance,"factorizations_observed":factorizations,"singular_events":singular_events,"backend_status":receipt.status,"backend_termination":receipt.termination,"elapsed_seconds":started.elapsed().as_secs_f64(),"scope":"Exact saved LP constructor replay only; zero training/backward/native scoring/native candidate acceptance; singular capture is numerical execution evidence, not rank/condition-number/infeasibility proof"}),
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 6
        || args[0] != "--input"
        || args[2] != "--expected-sha256"
        || args[4] != "--output"
        || args[3].len() != 64
        || !args[3].bytes().all(|x| x.is_ascii_hexdigit())
    {
        return Err("usage: --input FILE --expected-sha256 HEX --output EXCLUSIVE_DIR".into());
    }
    let input = PathBuf::from(&args[1]);
    let out = PathBuf::from(&args[5]);
    if out.exists() {
        return Err("output already exists".into());
    }
    report_output::claim(&out)?;
    let result = execute(&input, &args[3], &out);
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

#[cfg(test)]
mod observation_tests {
    use super::*;
    #[test]
    fn real_symbolic_and_numeric_lu_capture() -> Result<(), Box<dyn Error>> {
        let root = PathBuf::from(std::env::var("UOR_DIAGNOSTIC_TEST_ROOT")?)
            .join(format!("real-lu-{}", std::process::id()));
        microlp::diagnostics::configure(&root)?;
        assert_eq!(
            microlp::diagnostics::factorization_fixture(true),
            "Singular matrix"
        );
        assert_eq!(
            microlp::diagnostics::factorization_fixture(false),
            "Singular matrix"
        );
        assert_eq!(microlp::diagnostics::finish()?, (2, 2));
        let symbolic = fs::read_to_string(root.join("singular-0001/context.txt"))?;
        let numeric = fs::read_to_string(root.join("singular-0002/context.txt"))?;
        assert!(symbolic.contains("kind=symbolic_order_simple"));
        assert!(numeric.contains("kind=numeric_lu_pivot"));
        assert!(numeric.contains("max_abs_bits=0000000000000000"));
        assert!(numeric.contains("residual_stored_nonzero_original_rows="));
        assert!(numeric.contains("prior_upper_diagonal_bits="));
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
