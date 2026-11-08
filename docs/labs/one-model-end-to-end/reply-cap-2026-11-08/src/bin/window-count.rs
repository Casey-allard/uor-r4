//! `window-count split=DIR context=N`; see the README.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match reply_cap::run_window_count(&args) {
        Ok(line) => println!("{line}"),
        Err(e) => {
            eprintln!("window-count: refused: {e}");
            std::process::exit(1);
        }
    }
}
