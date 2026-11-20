//! Select the first 128 complete records from the independently pinned D29 C1 UORT.
use std::{env, fs, io::Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [source, out] = args.as_slice() else {
        return Err("usage: cut-c1 SOURCE DESTINATION".into());
    };
    let bytes = fs::read(source)?;
    if bytes.len() != 41_508_666
        || !bytes.starts_with(b"UORT")
        || u64::from_le_bytes(bytes[8..16].try_into()?) != 20_754_301
    {
        return Err("invalid D29 C1 source shape".into());
    }
    let mut previous = 1;
    let mut records = 0;
    let mut end = 0;
    for (i, pair) in bytes[64..].chunks_exact(2).enumerate() {
        let token = u16::from_le_bytes(pair.try_into()?);
        if token > 4095 || (token == 0) != (previous == 1) {
            return Err("invalid BOS/EOS or token in source".into());
        }
        if token == 1 {
            records += 1;
            if records == 128 {
                end = i + 1;
                break;
            }
        }
        previous = token;
    }
    if end == 0 {
        return Err("missing complete record 128".into());
    }
    let mut header: [u8; 64] = bytes[..64].try_into()?;
    header[8..16].copy_from_slice(&(end as u64).to_le_bytes());
    let mut output = fs::File::options().write(true).create_new(true).open(out)?;
    output.write_all(&header)?;
    output.write_all(&bytes[64..64 + end * 2])?;
    output.sync_all()?;
    println!("records={records} tokens={end} bytes={}", 64 + end * 2);
    Ok(())
}
