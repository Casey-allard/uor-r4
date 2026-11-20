#![forbid(unsafe_code)]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};
use uor_r4_count_runtime::FrozenPickMixture;
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const PANEL_SHA: &str = "e50d7812f4a4dcc3016793e07cf2efb55fa7524d880bf1253324ec255b4b8b11";
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const TRUNK: &str = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
fn sha(input: &[u8]) -> String {
    hex::encode(Sha256::digest(input))
}
fn json_file(path: &Path, v: &Value) -> Result<()> {
    let mut f = File::create_new(path)?;
    serde_json::to_writer_pretty(&mut f, v)?;
    f.sync_all()?;
    Ok(())
}

fn pack(fit: &[u8], weights: &[u8], wanted: &str, count: &Path, output: &Path) -> Result<()> {
    if sha(weights) != wanted {
        return Err("weight hash differs from producer receipt".into());
    }
    FrozenPickMixture::from_bytes(weights, Sha256::digest(fit).into())?;
    let manifest: Value = serde_json::from_slice(fit)?;
    if manifest["schema"] != "uor-r4.d31.a1-fit/1"
        || manifest["prepared"]["format"] != "uor-r4.d31.a1-preparation/1"
    {
        return Err("unexpected producer format".into());
    }
    let expected = manifest["prepared"]["compact_sha256"]
        .as_str()
        .ok_or("missing compact count digest")?;
    let mut counts = BufReader::with_capacity(1024 * 1024, File::open(count)?);
    let f_len = u32::try_from(fit.len())?;
    let w_len = u32::try_from(weights.len())?;
    let tmp = output.with_extension(format!("{}.part", std::process::id()));
    let file = File::create_new(&tmp)?;
    let result = (|| -> Result<u64> {
        let mut writer = BufWriter::new(file);
        writer.write_all(b"UORP1A01")?;
        writer.write_all(&f_len.to_le_bytes())?;
        writer.write_all(&w_len.to_le_bytes())?;
        writer.write_all(&Sha256::digest(weights))?;
        writer.write_all(fit)?;
        writer.write_all(weights)?;
        let mut count_bytes = 0u64;
        let mut digest = Sha256::new();
        loop {
            let chunk = counts.fill_buf()?;
            if chunk.is_empty() {
                break;
            }
            let n = chunk.len();
            writer.write_all(chunk)?;
            digest.update(chunk);
            count_bytes += n as u64;
            counts.consume(n);
        }
        if hex::encode(digest.finalize()) != expected {
            return Err("copied count bytes differ from fit manifest".into());
        }
        writer.flush()?;
        writer.get_ref().sync_all()?;
        // Seal without overwriting a previously published artifact.
        fs::hard_link(&tmp, output)?;
        Ok(count_bytes)
    })();
    fs::remove_file(tmp)?;
    let count_bytes = result?;
    println!(
        "{}",
        json!({"package_bytes":fs::metadata(output)?.len(),"count_bytes":count_bytes,"weight_bytes":w_len,"fit_bytes":f_len,"header_bytes":48})
    );
    Ok(())
}

fn panel(tok: &Path, inputs: &Path, output: &Path) -> Result<()> {
    let tok = fs::read(tok)?;
    let inputs = fs::read(inputs)?;
    if sha(&tok) != TOKENIZER || sha(&inputs) != PANEL_SHA {
        return Err("frozen tokenizer or prompts changed".into());
    }
    let t = ByteBpeTokenizer::from_tokenizer_json_bytes(&tok).ok_or("invalid tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_version(&t, 2)?;
    let framed = protocol.bind(&t)?;
    let mut rows = Vec::new();
    let mut grading = Vec::new();
    for line in std::str::from_utf8(&inputs)?.lines() {
        let mut fields = line.splitn(3, '\t');
        let id = fields.next().ok_or("id")?.parse::<u32>()?;
        let gold = fields.next().ok_or("gold")?;
        let text = fields.next().ok_or("text")?;
        let prefix = framed
            .encode_assistant_prefix(&[Message {
                role: "user",
                content: text,
            }])
            .tokens;
        if prefix.is_empty() || prefix.len() > 192 {
            return Err("unexpected protocol prefix".into());
        }
        rows.push(json!({"id":id,"prefix":prefix}));
        let tape = (gold != "-").then(|| {
            let mut v = t.encode(&format!(" {gold}"));
            v.push(protocol.eos_id);
            v
        });
        grading.push(json!({"id":id,"text":text,"gold":tape}));
    }
    fs::create_dir(output)?;
    json_file(
        &output.join("blind.json"),
        &json!({"panel_sha256":PANEL_SHA,"protocol":2,"max_tokens":12,"rows":rows}),
    )?;
    json_file(&output.join("key.json"), &json!(grading))
}

fn fixture(count: &Path, output: &Path) -> Result<()> {
    // C128 is a measured count table. These planted weights exercise the CLI only.
    let counts = fs::read(count)?;
    let count_sha = sha(&counts);
    if count_sha != "eea859b5d7272ef8d4be3a4ee623c92c2b11da866478b05765a7b20557b3788e" {
        return Err("invalid finite C128 table".into());
    }
    let fit = serde_json::to_vec(
        &json!({"schema":"uor-r4.d31.a1-fit/1","fixture":"planted tie and exact-copy switch, no training","prepared":{"format":"uor-r4.d31.a1-preparation/1","c_sha256":hex::encode(&counts[12..44]),"trunk_sha256":TRUNK,"compact_sha256":count_sha}}),
    )?;
    drop(counts);
    let mut words = vec![0; 528 * 14];
    for (b, row) in words.chunks_exact_mut(14).enumerate() {
        row[if b < 176 { 12 } else { 13 }] = 1u64 << 32;
    }
    let weights = FrozenPickMixture::new(10, Sha256::digest(&fit).into(), words)?.to_bytes();
    pack(&fit, &weights, &sha(&weights), count, output)
}

fn main() -> Result<()> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    match args.as_slice() {
        [mode,fit,weights,want,counts,out] if mode==Path::new("pack")=>pack(&fs::read(fit)?,&fs::read(weights)?,want.to_str().ok_or("weight digest")?,counts,out),
        [mode,tokenizer,input,out] if mode==Path::new("panel")=>panel(tokenizer,input,out),
        [mode,counts,out] if mode==Path::new("fixture")=>fixture(counts,out),
        _=>Err("expected pack FIT WEIGHTS EXPECTED_WEIGHT_SHA COUNTS NEW_FILE | panel TOKENIZER INPUTS NEW_DIR | fixture C128 NEW_FILE".into())
    }
}
