//! `uor-website` — Generates the UOR Foundation static website.
//!
//! **Outputs (`public/`):**
//! - `index.html` — Homepage
//! - `search.html` — Search page
//! - `search-index.json` — Full-text search index
//! - `sitemap.xml` — Sitemap for crawlers
//! - `namespaces/<prefix>/index.html` — Namespace landing pages (14 total, auto-generated)
//! - `css/style.css` — Complete stylesheet (no CDN dependencies)
//! - `js/search.js` — Lightweight client-side search
//!
//! **Usage:**
//! ```
//! uor-website [--out <path>]
//! ```

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    missing_docs,
    clippy::missing_errors_doc
)]

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use uor_website::generate;

/// Generate the UOR Foundation static website.
#[derive(Parser)]
#[command(
    name = "uor-website",
    about = "Generate the UOR Foundation static website"
)]
struct Args {
    /// Output directory for the generated website.
    #[arg(long, default_value = "public")]
    out: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    generate(&args.out)?;

    println!("Website generated successfully.");
    println!("  Output: {}", args.out.display());

    Ok(())
}
