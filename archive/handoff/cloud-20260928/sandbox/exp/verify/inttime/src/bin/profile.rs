//! Scratch per-section profile of IntegerModel::step (software vs hardware mul/div).
//! Reuses a model directory produced by the main harness (argv[1]=model dir, argv[2]=tables dir).
use std::path::PathBuf;
macro_rules! profile {
    ($krate:ident, $label:expr, $model_dir:expr, $tables:expr, $tokens:expr, $reps:expr) => {{
        let model = $krate::IntegerModel::load_with_tables(&$model_dir, &$tables).unwrap();
        let mut best: Option<(u128, [u128; 10])> = None;
        for _ in 0..$reps {
            $krate::model::PROFILE.with(|p| *p.borrow_mut() = [0; 10]);
            let mut session = model.new_session();
            let start = std::time::Instant::now();
            for &t in $tokens.iter() {
                model.step(&mut session, t, $krate::ReadMode::Enabled).unwrap();
            }
            let total = start.elapsed().as_nanos();
            let sections = $krate::model::PROFILE.with(|p| *p.borrow());
            if best.map_or(true, |(b, _)| total < b) {
                best = Some((total, sections));
            }
        }
        let (total, sections) = best.unwrap();
        println!("{} (Read, 256 steps, best of {}): total {:.1} ms = {:.3} ms/step", $label, $reps, total as f64 / 1e6, total as f64 / 256e6);
        for (name, ns) in $krate::model::SECTIONS.iter().zip(sections) {
            println!("   {:<55} {:>8.3} ms/step  {:>5.1}%", name, ns as f64 / 256e6, 100.0 * ns as f64 / total as f64);
        }
    }};
}
fn main() {
    let model_dir = PathBuf::from(std::env::args().nth(1).unwrap());
    let tables = PathBuf::from(std::env::args().nth(2).unwrap());
    let mut state = 0x1234_5678_9abc_def1u64;
    let tokens: Vec<u32> = (0..256)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state % 4096) as u32
        })
        .collect();
    profile!(uor_r4_integer_swprof, "software mul/div (repo arithmetic)", model_dir, tables, tokens, 3);
    profile!(uor_r4_integer_hwprof, "hardware mul/div (scratch)", model_dir, tables, tokens, 3);
}
