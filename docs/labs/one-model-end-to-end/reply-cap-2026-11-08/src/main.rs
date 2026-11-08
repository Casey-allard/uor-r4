//! `reply-cap split=DIR out=DIR k=K`; see the README.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match reply_cap::run(&args) {
        Ok(line) => println!("{line}"),
        Err(e) => {
            eprintln!("reply-cap: refused: {e}");
            std::process::exit(1);
        }
    }
}
