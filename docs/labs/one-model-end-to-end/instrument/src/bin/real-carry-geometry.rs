fn main() {
    if let Err(error) = one_model_end_to_end_panel::carry_geometry::run(std::env::args().skip(1)) {
        eprintln!("real-carry-geometry: {error}");
        std::process::exit(1);
    }
}
