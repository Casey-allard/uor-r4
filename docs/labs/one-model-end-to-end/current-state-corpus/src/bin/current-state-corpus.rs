//! `current-state-corpus out=DIR rows=N seed=S [show=K]`: write the
//! current-state TRAIN store into the new directory DIR, then print the first
//! K documents decoded.

use current_state_corpus::{episodes, load_tokenizer, write_store};
use std::path::PathBuf;
use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;

fn run(args: &[String]) -> Result<(), String> {
    let arg = |key: &str| -> Option<String> {
        args.iter()
            .find_map(|a| a.strip_prefix(&format!("{key}=")))
            .map(str::to_string)
    };
    let need = |key: &str| arg(key).ok_or_else(|| format!("missing {key}="));
    let out = PathBuf::from(need("out")?);
    let rows: usize = need("rows")?.parse().map_err(|_| "rows= is not a count")?;
    let seed: u64 = need("seed")?.parse().map_err(|_| "seed= is not a u64")?;
    let show: usize = match arg("show") {
        Some(v) => v.parse().map_err(|_| "show= is not a count")?,
        None => 0,
    };
    let pinned = load_tokenizer()?;
    let manifest = write_store(&pinned, &out, rows, seed)?;
    println!(
        "wrote {} rows, {} tokens, {} response tokens, max document {} tokens to {}",
        manifest["rows"],
        manifest["tokens"],
        manifest["response_tokens"],
        manifest["max_document_tokens"],
        out.display()
    );
    if show > 0 {
        let reader = MmapCorpusReader::open(out.join("tokens.u16")).map_err(|e| e.to_string())?;
        let ids: Vec<u32> = reader.as_slice().iter().map(|&t| u32::from(t)).collect();
        let shown = episodes(show.min(rows), seed);
        let mut start = 0;
        for (row, episode) in shown.iter().enumerate() {
            let end = start
                + ids[start..]
                    .iter()
                    .position(|&t| t == pinned.protocol.eos_id)
                    .ok_or("no EOS")?
                + 1;
            println!(
                "--- row {row} ({}) ---\n{}\n--- end row {row} ---",
                episode.shape,
                pinned.tokenizer.decode(&ids[start..end])
            );
            start = end;
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&args) {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
