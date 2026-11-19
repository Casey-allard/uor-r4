fn main() -> uor_value_rank_capture::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.split_first() {
        Some((command, rest)) if command == "synthetic" && rest.is_empty() => {
            serde_json::to_writer(
                std::io::stdout().lock(),
                &uor_value_rank_capture::synthetic_fixture()?,
            )?;
            println!();
            Ok(())
        }
        Some((command, rest)) if command == "prepare" => {
            uor_value_rank_capture::requested::prepare(rest)
        }
        Some((command, rest)) if command == "requested" => {
            uor_value_rank_capture::requested::requested(rest)
        }
        Some((command, rest)) if command == "capture" => uor_value_rank_capture::capture(rest),
        _ => uor_value_rank_capture::capture(&args),
    }
}
