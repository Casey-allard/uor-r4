// Archival CLI adapter for the repository's unchanged normative report module.
#[path = "../../../../../crates/uor-r4-integer/src/report_output.rs"]
mod report_output;
fn main() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "usage: report-tool claim|seal|verify DIR",
        ));
    }
    let path = std::path::Path::new(&args[2]);
    match args[1].as_str() {
        "claim" => report_output::claim(path),
        "seal" => report_output::seal(path).map(|_| ()),
        "verify" => report_output::verify(path).map(|_| ()),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "unknown operation",
        )),
    }
}
