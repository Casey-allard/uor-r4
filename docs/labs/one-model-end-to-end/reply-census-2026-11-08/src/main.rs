//! `reply-census split=DIR name=NAME [against=DIR2 against_name=NAME2] [top=10]`; see the README.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match reply_census::run(&args) {
        Ok(summary) => print!("{summary}"),
        Err(e) => {
            eprintln!("reply-census: refused: {e}");
            std::process::exit(1);
        }
    }
}
