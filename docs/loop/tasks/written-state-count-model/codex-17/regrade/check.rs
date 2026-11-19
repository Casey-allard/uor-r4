mod observed;
// Independent grading witnesses over original unmarked text and immutable token tapes.
use serde_json::Value;
use std::{fs, io::Write, path::Path};
use uor_r4_tokenizer::ByteBpeTokenizer;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn read(p: &str) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn save(p: &str, v: &Value) -> Result<()> {
    let mut file = fs::File::create_new(Path::new(p))?;
    file.write_all(&serde_json::to_vec(v)?)?;
    file.sync_all()?;
    Ok(())
}
fn ids(v: &Value) -> Result<Vec<u32>> {
    Ok(serde_json::from_value(v.clone())?)
}
fn valid(actual: &[u32], expected: &[u32], answer: &str, t: &ByteBpeTokenizer) -> bool {
    let Some((&1, body)) = actual.split_last() else {
        return false;
    };
    !body.contains(&1)
        && actual == expected
        && t.decode_bytes(body) == format!(" {answer}").as_bytes()
}
fn main() -> Result<()> {
    observed::score(&std::env::args().skip(1).collect::<Vec<_>>())
}
