mod grade;

use std::{env, error::Error, fs, io::Write};
use uor_r4_integer::discrete_d11::spiralcore_orbit_roots;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "grade") {
        return grade::grade(&args[1..]);
    }

    if let [cmd, out] = args.as_slice() {
        if cmd == "roots" {
            let mut file = fs::File::create_new(out)?;
            for root in spiralcore_orbit_roots() {
                let words: Vec<_> = root.iter().map(ToString::to_string).collect();
                writeln!(file, "{}", words.join(" "))?;
            }
            return Ok(());
        }
    }
    Err("expected roots NEW_FILE".into())
}
