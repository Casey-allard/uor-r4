//! `scale-probe mix|seen|grade|pair key=value...`; see the README.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match scale_probe_instrument::run(&args) {
        Ok(summary) => print!("{summary}"),
        Err(e) => {
            eprintln!("scale-probe: refused: {e}");
            std::process::exit(1);
        }
    }
}
