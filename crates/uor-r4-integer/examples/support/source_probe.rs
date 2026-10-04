//! Resolve an output through existing ancestors without creating directories.
use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

pub fn prospective_output(path: &Path) -> io::Result<PathBuf> {
    if path
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "output cannot contain parent traversal",
        ));
    }
    let mut ancestor = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut suffix = Vec::new();
    loop {
        match fs::symlink_metadata(&ancestor) {
            Ok(_) => break,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                suffix.push(
                    ancestor
                        .file_name()
                        .ok_or_else(|| {
                            io::Error::new(io::ErrorKind::InvalidInput, "output ancestor absent")
                        })?
                        .to_os_string(),
                );
                ancestor = ancestor
                    .parent()
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "output parent absent")
                    })?
                    .to_path_buf();
            }
            Err(error) => return Err(error),
        }
    }
    let mut output = fs::canonicalize(ancestor)?;
    for part in suffix.into_iter().rev() {
        output.push(part);
    }
    for parent in output.ancestors() {
        if parent.is_dir()
            && parent
                .join(uor_r4_integer::report_output::MANIFEST_FILE)
                .exists()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "output beneath a sealed report ancestor is forbidden",
            ));
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn sealed_output_ancestor_rejects_before_creating_missing_parents() -> io::Result<()> {
        let root = std::env::temp_dir().join(format!(
            "native-probe-sealed-output-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root)?;
        let result = (|| {
            fs::write(
                root.join(uor_r4_integer::report_output::MANIFEST_FILE),
                b"{}",
            )?;
            let error = prospective_output(&root.join("missing/attempt"))
                .err()
                .ok_or_else(|| io::Error::other("sealed ancestor accepted"))?;
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
            assert!(!root.join("missing").exists());
            Ok(())
        })();
        fs::remove_dir_all(&root)?;
        result
    }

    #[test]
    fn unsealed_output_resolution_creates_nothing() -> io::Result<()> {
        let root = std::env::temp_dir().join(format!(
            "native-probe-unsealed-output-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root)?;
        let result = prospective_output(&root.join("missing/attempt"));
        assert!(!root.join("missing").exists());
        fs::remove_dir_all(&root)?;
        result.map(|_| ())
    }
}
