use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
};

const NIL: u32 = u32::MAX;
const WIDTH: usize = 192;
const MAX_DISTANCE: u64 = 32 * (2 * 65535 + 2 * 4294967295);

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn word(input: &mut impl Read) -> io::Result<u32> {
    let mut bytes = [0; 4];
    input.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn distance(input: &mut impl Read) -> io::Result<u64> {
    let mut bytes = [0; 8];
    input.read_exact(&mut bytes)?;
    let distance = u64::from_le_bytes(bytes);
    if distance > MAX_DISTANCE {
        return Err(invalid("distance exceeds native coordinate bounds"));
    }
    Ok(distance)
}

fn forced_winner(spans: &[(u32, u32)], pair: &[Vec<(u64, u64)>], reversed: bool) -> u32 {
    for (i, &(start, _)) in spans.iter().enumerate() {
        if pair[i].iter().enumerate().all(|(j, &(forward, reverse))| {
            i == j
                || if reversed {
                    reverse < forward
                } else {
                    forward < reverse
                }
        }) {
            return start;
        }
    }
    NIL
}

fn option(value: u32) -> String {
    if value == NIL {
        "null".into()
    } else {
        value.to_string()
    }
}

fn decode(input: &mut impl Read, output: &mut impl Write) -> io::Result<()> {
    let mut magic = [0; 8];
    input.read_exact(&mut magic)?;
    let rows = word(input)? as usize;
    if &magic != b"D35RANK1" || !(1..=4096).contains(&rows) {
        return Err(invalid("invalid score header"));
    }
    writeln!(output, "[")?;
    let mut total_pairs = 0usize;
    for row in 0..rows {
        let n = word(input)? as usize;
        let winner_forward = word(input)?;
        let winner_reversed = word(input)?;
        if n > WIDTH {
            return Err(invalid("invalid candidate count"));
        }
        total_pairs += n * n.saturating_sub(1);
        if total_pairs > 1_000_000 {
            return Err(invalid("too many scored pairs"));
        }
        let mut spans: Vec<(u32, u32)> = Vec::with_capacity(n);
        let mut pairs = vec![vec![(0, 0); n]; n];
        for (i, rivals) in pairs.iter_mut().enumerate() {
            let start = word(input)?;
            let end = word(input)?;
            if start == 0
                || end <= start
                || end > WIDTH as u32
                || spans.last().is_some_and(|previous| previous.0 >= start)
            {
                return Err(invalid("invalid or unordered original candidate span"));
            }
            spans.push((start, end));
            for (j, pair) in rivals.iter_mut().enumerate() {
                if i != j {
                    *pair = (distance(input)?, distance(input)?);
                }
            }
        }
        for (i, rivals) in pairs.iter().enumerate() {
            for (j, &(forward, reverse)) in rivals.iter().enumerate() {
                if pairs[j][i] != (reverse, forward) {
                    return Err(invalid(
                        "ordered pair distances violate orientation symmetry",
                    ));
                }
            }
        }
        if match spans.as_slice() {
            [] => winner_forward != NIL || winner_reversed != NIL,
            [(start, _)] => {
                winner_forward != winner_reversed
                    || (winner_forward != NIL && winner_forward != *start)
            }
            _ => {
                winner_forward != forced_winner(&spans, &pairs, false)
                    || winner_reversed != forced_winner(&spans, &pairs, true)
            }
        } {
            return Err(invalid("recorded winner disagrees with candidate traces"));
        }

        if row > 0 {
            writeln!(output, ",")?;
        }
        write!(
            output,
            "{{\"packet_row\":{row},\"winner_forward\":{},\"winner_reversed\":{},\"candidates\":[",
            option(winner_forward),
            option(winner_reversed)
        )?;
        for (i, &(start, end)) in spans.iter().enumerate() {
            if i > 0 {
                write!(output, ",")?;
            }
            write!(output, "{{\"start\":{start},\"end\":{end}}}")?;
        }
        write!(output, "],\"pairs\":[")?;
        let mut delimiter = "";
        for (i, &(candidate, _)) in spans.iter().enumerate() {
            for (j, &(rival, _)) in spans.iter().enumerate() {
                if i != j {
                    let (forward, reverse) = pairs[i][j];
                    let margin = reverse as i64 - forward as i64;
                    write!(output, "{delimiter}{{\"candidate\":{candidate},\"rival\":{rival},\"nearest_forward\":{forward},\"nearest_reverse\":{reverse},\"margin\":{margin}}}")?;
                    delimiter = ",";
                }
            }
        }
        write!(output, "]}}")?;
    }
    if input.read(&mut [0])? != 0 {
        return Err(invalid("trailing score data"));
    }
    writeln!(output, "\n]")
}

fn main() -> io::Result<()> {
    let file = std::env::args_os()
        .nth(1)
        .ok_or_else(|| invalid("expected one D35 score result file"))?;
    let mut output = BufWriter::new(io::stdout().lock());
    decode(&mut BufReader::new(File::open(file)?), &mut output)?;
    output.flush()
}
