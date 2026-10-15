use super::*;
use std::{fs::OpenOptions, io::Write};

pub(super) fn new_output(path: &Path) -> Result<()> {
    fs::create_dir(path)?;
    Ok(())
}

pub(super) fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

pub(super) fn seal(output: &Path) -> Result<()> {
    let mut paths = fs::read_dir(output)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.sort();
    let mut sums = String::new();
    for path in paths {
        require(
            fs::symlink_metadata(&path)?.file_type().is_file(),
            "unexpected non-file in exclusive output",
        )?;
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or("non-UTF8 receipt name")?;
        sums.push_str(&format!("{}  {name}\n", corpus::sha256(&fs::read(&path)?)));
    }
    write_new(&output.join("SHA256SUMS"), sums.as_bytes())?;
    fs::File::open(output)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_publication_never_overwrites_or_discards_a_failure() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let output = directory.path().join("attempt");
        new_output(&output)?;
        assert!(new_output(&output).is_err());
        write_new(&output.join("batches.jsonl"), b"partial\n")?;
        write_new(
            &output.join("failure.json"),
            b"{\"status\":\"INSTRUMENT_FAULT\"}",
        )?;
        assert!(write_new(&output.join("batches.jsonl"), b"replacement").is_err());
        seal(&output)?;
        let sums = fs::read_to_string(output.join("SHA256SUMS"))?;
        assert!(sums.contains(&format!(
            "{}  batches.jsonl\n",
            corpus::sha256(b"partial\n")
        )));
        assert!(sums.contains("  failure.json\n"));
        assert!(!sums.contains("  report.json\n"));
        assert!(seal(&output).is_err());
        assert_eq!(fs::read(output.join("batches.jsonl"))?, b"partial\n");
        Ok(())
    }
}
