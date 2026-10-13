//! Reject known retired numerical dependencies in the selected native graph.
//! Input is `cargo tree -p uor-r4-training --features cuda --edges features`.
//! This is a dependency admission check, not semantic proof of arbitrary code.
pub fn verify(text: &str) -> Result<(), String> {
    if !text.contains("uor-r4-training")
        || !text.contains("candle-core")
        || !text.contains("cudarc")
    {
        return Err("dependency evidence lacks selected native CUDA packages".into());
    }
    for token in text.split(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_') {
        if token == "uor-r4-model-source"
            || token == "matrixmultiply"
            || token == "blas"
            || token == "blas-sys"
            || token == "cblas-sys"
            || token == "intel-mkl-src"
            || token == "uor-prism-tensor"
            || token == "uor-prism-numerics"
            || token.starts_with("uor-matmul")
            || token == "gemm"
            || token.starts_with("gemm-")
        {
            return Err(format!(
                "D29: selected native dependency graph activates excluded package {token}"
            ));
        }
    }
    Ok(())
}

pub fn self_test() {
    let good = "uor-r4-training v0.1.0\n├── candle-core v0.9.2\n└── cudarc feature \"driver\"\n";
    assert!(verify(good).is_ok());
    assert!(verify("").is_err());
    for bad in [
        "uor-r4-model-source",
        "uor-matmul",
        "uor-matmul-gemm",
        "gemm-f32",
        "matrixmultiply",
        "cblas-sys",
        "uor-prism-tensor",
        "uor-prism-numerics",
    ] {
        assert!(
            verify(&format!("{good}    └── {bad} v0.1.0 (*)\n")).is_err(),
            "missed {bad}"
        );
    }
}

#[test]
fn excluded_dependency_fixtures() {
    self_test();
}
