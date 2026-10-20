//! Observation only: arithmetic and solver control flow remain in the upstream code.
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};
#[derive(Default)]
struct State {
    root: Option<PathBuf>,
    phase: String,
    lp_phase: String,
    iteration: u64,
    sequence: u64,
    failures: u64,
    captured_bound_bytes: u64,
    context: String,
    mapping: Vec<usize>,
    errors: Vec<String>,
}
fn state() -> &'static Mutex<State> {
    static S: OnceLock<Mutex<State>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(State::default()))
}
fn change(f: impl FnOnce(&mut State)) {
    if let Ok(mut s) = state().lock() {
        f(&mut s);
    }
}
pub fn configure(root: &Path) -> io::Result<()> {
    fs::create_dir(root)?;
    let mut s = state()
        .lock()
        .map_err(|_| io::Error::other("observer lock poisoned"))?;
    *s = State {
        root: Some(root.to_owned()),
        ..State::default()
    };
    Ok(())
}
pub fn finish() -> io::Result<(u64, u64)> {
    let s = state()
        .lock()
        .map_err(|_| io::Error::other("observer lock poisoned"))?;
    if !s.errors.is_empty() {
        return Err(io::Error::other(s.errors.join("; ")));
    }
    Ok((s.sequence, s.failures))
}
pub(crate) fn phase(x: &str) {
    change(|s| s.phase = x.to_owned());
}
pub(crate) fn lp_phase(x: &str) {
    change(|s| s.lp_phase = x.to_owned());
}
pub(crate) fn iteration(x: u64) {
    change(|s| s.iteration = x);
}
pub(crate) fn context(x: String) {
    change(|s| s.context = x);
}
pub(crate) fn mapping(x: &[usize]) {
    change(|s| s.mapping = x.to_vec());
}
pub(crate) fn factor_begin() {
    change(|s| s.sequence += 1);
}
pub(crate) fn bits(x: &[f64]) -> Vec<u64> {
    x.iter().map(|x| x.to_bits()).collect()
}
fn write_new(p: &Path, body: &str) -> io::Result<()> {
    let mut f = OpenOptions::new().create_new(true).write(true).open(p)?;
    f.write_all(body.as_bytes())
}
pub(crate) fn singular<'a>(
    kind: &str,
    size: usize,
    get_col: impl Fn(usize) -> (&'a [usize], &'a [f64]),
    detail: String,
) {
    change(|s| {
        let Some(root) = &s.root else {
            return;
        };
        s.failures += 1;
        let estimated = (0..size)
            .map(|c| get_col(c).0.len() as u64 * 80 + 64)
            .sum::<u64>()
            + s.context.len() as u64
            + detail.len() as u64
            + s.mapping.len() as u64 * 24
            + 4096;
        if s.captured_bound_bytes.saturating_add(estimated) > 480 * 1024 * 1024 {
            s.errors
                .push("observer capture exceeds 480MiB bound; arithmetic was not changed".into());
            return;
        }
        s.captured_bound_bytes += estimated;
        let d = root.join(format!("singular-{:04}", s.failures));
        let result = (|| -> io::Result<()> {
            fs::create_dir(&d)?;
            write_new(&d.join("context.txt"),&format!("kind={kind}\nfactor_sequence={}\nphase={}\nlp_phase={}\nlp_iterations={}\nbasic_vars={:?}\n{}\n{}\n",s.sequence,s.phase,s.lp_phase,s.iteration,s.mapping,s.context,detail))?;
            let mut f = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(d.join("basis-csc.tsv"))?;
            writeln!(f,"# full pre-elimination basis; dimension={size}; columns map through basic_vars; f64 values exact hexadecimal bits")?;
            for c in 0..size {
                let (rows, vals) = get_col(c);
                writeln!(f, "COLUMN\t{c}\t{}", rows.len())?;
                for (&r, &v) in rows.iter().zip(vals) {
                    writeln!(f, "ENTRY\t{c}\t{r}\t{:016x}", v.to_bits())?;
                }
            }
            Ok(())
        })();
        if let Err(e) = result {
            s.errors.push(e.to_string());
        }
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observer_capture_preserves_bits_and_empty_column() -> io::Result<()> {
        let d = PathBuf::from(std::env::var("UOR_DIAGNOSTIC_TEST_ROOT").map_err(io::Error::other)?)
            .join(format!("uor-basis-observer-{}", std::process::id()));
        configure(&d)?;
        mapping(&[3, 4]);
        phase("fixture");
        factor_begin();
        let rows = [0usize];
        let vals = [-0f64];
        singular(
            "symbolic",
            2,
            |c| if c == 0 { (&rows, &vals) } else { (&[], &[]) },
            "empty=1".into(),
        );
        assert_eq!(finish()?, (1, 1));
        let text = fs::read_to_string(d.join("singular-0001/basis-csc.tsv"))?;
        assert!(text.contains("8000000000000000"));
        assert!(text.contains("COLUMN\t1\t0"));
        fs::remove_dir_all(d)?;
        Ok(())
    }
}

/// Exercise existing LU failure paths on tiny synthetic matrices, for observation tests only.
#[doc(hidden)]
pub fn factorization_fixture(symbolic: bool) -> String {
    let rows = vec![vec![0usize, 1], if symbolic { vec![] } else { vec![0, 1] }];
    let vals = vec![vec![1f64, 2.], if symbolic { vec![] } else { vec![2., 4.] }];
    mapping(&[0, 1]);
    phase("synthetic_factor_fixture");
    context("factor_callsite=synthetic_fixture".into());
    let mut scratch = crate::lu::ScratchSpace::with_capacity(2);
    match crate::lu::lu_factorize(2, |c| (&rows[c], &vals[c]), 0.1, &mut scratch) {
        Ok(_) => "unexpected_success".into(),
        Err(e) => e.to_string(),
    }
}
