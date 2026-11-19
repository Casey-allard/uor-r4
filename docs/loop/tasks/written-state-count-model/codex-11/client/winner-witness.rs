//! Extract a device-observed gold-target mass when the target equals the GPU's greedy ID.
use std::{collections::BTreeMap, env, error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let tape = fs::read(args.next().ok_or("CPU-readable device output required")?)?;
    let tokens = fs::read(args.next().ok_or("query input required")?)?;
    let mut requested = BTreeMap::new();
    for arg in args {
        let previous: u16 = arg.parse()?;
        if previous >= 4096 {
            return Err("previous ID out of range".into());
        }
        requested.insert(previous, None);
    }
    let n = tokens.len() / 2;
    if tokens.len() != 2 * n || tape.len() != 8192 + 12 * n {
        return Err("invalid native tape shape".into());
    }
    let mut last = 0;
    for (i, word) in tokens.chunks_exact(2).enumerate() {
        let target = u16::from_le_bytes([word[0], word[1]]);
        if let Some(witness) = requested.get_mut(&last) {
            let at = usize::from(last) * 2;
            let winner = u16::from_le_bytes([tape[at], tape[at + 1]]);
            if target != 0 && target == winner {
                let begin = 8192 + i * 8;
                let mass = u64::from_le_bytes(tape[begin..begin + 8].try_into()?);
                if let Some((_, _, seen)) = *witness {
                    if mass != seen {
                        return Err("same context and winner have different masses".into());
                    }
                } else {
                    *witness = Some((i, winner, mass));
                }
            }
        }
        last = target;
    }
    for (previous, witness) in requested {
        if let Some((position, winner, mass)) = witness {
            println!("{previous}\t{position}\t{winner}\t{mass}");
        } else {
            eprintln!("No observed winning target for context {previous}");
        }
    }
    Ok(())
}
