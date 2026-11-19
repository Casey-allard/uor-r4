fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = uor_value_rank_calibration::fixtures::witness()
        .map_err(|error| format!("invalid authored fixture: {error:?}"))?;
    serde_json::to_writer(std::io::stdout().lock(), &result)?;
    println!();
    Ok(())
}
