use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, io::Write, path::Path};
use uor_r4_integer::{
    discrete_d11::IntegerDiscreteModel,
    paired_role::{validate_role_ids, RequestedContext},
};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const MODEL: &str = "b17352679c64a96bf657fad7640d4bb05ea3de0ecd08dadd13e9d2756e979bc4";
const NIL: u32 = u32::MAX;
const LOW: u32 = i32::MIN as u32;

struct Row {
    id: u32,
    ids: Vec<u32>,
    content: [u32; 2],
}

fn write_words(file: &mut fs::File, words: impl IntoIterator<Item = u32>) -> Result<()> {
    for word in words {
        file.write_all(&word.to_le_bytes())?;
    }
    Ok(())
}

fn emit(root: &Path, sites: &[[u32; 4]], cells: &[[u32; 4]], rows: &[Row]) -> Result<()> {
    fs::create_dir(root)?;
    let mut index = fs::File::create_new(root.join("index.tsv"))?;
    for row in rows {
        validate_role_ids(&row.ids)?;
        let mut hash = Sha256::new();
        for id in &row.ids {
            hash.update(id.to_le_bytes());
        }
        writeln!(index, "{}\t{:x}", row.id, hash.finalize())?;
    }
    for mode in 4..8 {
        let mut f = fs::File::create_new(root.join(format!("mode-{mode}.bin")))?;
        write_words(
            &mut f,
            [
                0x3141524f,
                mode,
                rows.len().try_into()?,
                cells.len().try_into()?,
            ],
        )?;
        for site in sites {
            write_words(&mut f, *site)?;
        }
        for cell in cells {
            write_words(&mut f, *cell)?;
        }
        for row in rows {
            write_words(
                &mut f,
                row.ids
                    .iter()
                    .copied()
                    .chain(std::iter::repeat(0))
                    .take(192),
            )?;
            write_words(
                &mut f,
                [row.ids.len().try_into()?, row.content[0], row.content[1]],
            )?;
        }
    }
    Ok(())
}

fn pack(carrier: &Path, tokenizer: &Path, blind: &Path, out: &Path) -> Result<()> {
    let model = IntegerDiscreteModel::load(carrier)?;
    if model.artifact_sha256() != MODEL {
        return Err("wrong D26 carrier".into());
    }
    let bytes = fs::read(tokenizer)?;
    if model.tokenizer_sha256() != Some(format!("{:x}", Sha256::digest(&bytes)).as_str()) {
        return Err("wrong tokenizer".into());
    }
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).ok_or("invalid tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)?;
    let encoder = protocol.bind(&tokenizer)?;
    let state = model.frozen_role_replay()?;
    if state.requested_context != Some(RequestedContext::FullPrefix) {
        return Err("wrong role policy".into());
    }
    let frame = state.requested_framing.ok_or("missing framing")?;
    let mut rows = Vec::new();
    for line in fs::read_to_string(blind)?.lines() {
        let (key, text) = line.split_once('\t').ok_or("missing row ID")?;
        if text.contains('\t') {
            return Err("unexpected extra column".into());
        }
        let id = key.parse()?;
        if rows.iter().any(|r: &Row| r.id == id) {
            return Err("duplicate row ID".into());
        }
        let ids = encoder
            .encode_assistant_prefix(&[Message {
                role: "user",
                content: text,
            }])
            .tokens;
        let content = frame.content(&ids)?;
        rows.push(Row {
            id,
            ids,
            content: [content.start.try_into()?, content.end.try_into()?],
        });
    }
    if rows.len() != 22 {
        return Err("expected exposed 22-row panel".into());
    }
    let sites: Vec<_> = state
        .roles
        .sites()
        .iter()
        .enumerate()
        .map(|(id, &[root, class])| {
            [
                root,
                class,
                state.assertion_classes[id],
                u32::from(state.delimiters.contains(&(id as u32))),
            ]
        })
        .collect();
    emit(out, &sites, state.roles.cells(), &rows)
}

fn fixture(out: &Path) -> Result<()> {
    let mut sites = vec![[0, 0, 0, 0]; 4096];
    sites[17][3] = 1;
    for id in [30, 31, 32, 80, 82, 90] {
        sites[id][1] = 1;
    }
    for id in [80, 82] {
        sites[id][2] = 1;
    }
    for id in [81, 83, 84] {
        sites[id][2] = 2;
    }
    sites[83][1] = 2;
    sites[92][0] = 3;
    let prompts = [
        vec![0, 17, 80, 81, 90, 17, 30, 80, 81, 17],
        vec![0, 17, 80, 81, 90, 17, 30, 80, 81, 17, 31, 32, 17],
        vec![0, 17, 82, 83, 90, 17, 30, 82, 83, 17, 31, 32, 17],
        vec![0, 17, 80, 81, 90, 17, 30, 80, 81, 84, 17],
        vec![0, 17, 82, 83, 90, 17, 30, 82, 83, 30, 82, 83, 17],
        vec![0, 17, 80, 81, 90, 17, 30, 80, 81, 92, 17],
        vec![0, 17, 80, 81, 90, 17, 30, 80, 81, 84, 17],
    ];
    let rows: Vec<_> = prompts
        .into_iter()
        .enumerate()
        .map(|(i, ids)| Row {
            id: i as u32,
            // Row six puts the continuing ID just outside the framed user text.
            content: [1, if i == 6 { 9 } else { ids.len() as u32 }],
            ids,
        })
        .collect();
    emit(out, &sites, &[[3 * 65537 + 3, 0, 0, 1]], &rows)?;
    for mode in 4..8 {
        let mut expected = fs::File::create_new(out.join(format!("mode-{mode}.expected")))?;
        for row in 0..rows.len() {
            if row == 5 && mode >= 6 {
                write_words(&mut expected, [7, 9, 7, 9, (-20i32) as u32, 16, LOW, LOW])?;
                continue;
            }
            let recovered = match row {
                0 => mode >= 6,
                1 => mode == 7,
                2 => mode == 5 || mode == 7,
                _ => false,
            };
            let best = if recovered || row == 4 { 0 } else { LOW };
            let runner = if row == 4 { 0 } else { LOW };
            let span = if recovered { [7, 9] } else { [NIL, NIL] };
            write_words(
                &mut expected,
                [span[0], span[1], NIL, NIL, best, LOW, runner, LOW],
            )?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match args.as_slice() {
        [flag, out] if flag == "finite" => fixture(Path::new(out)),
        [flag, model, tokenizer, blind, out] if flag == "pack" => pack(
            Path::new(model),
            Path::new(tokenizer),
            Path::new(blind),
            Path::new(out),
        ),
        _ => Err("use finite OUT or pack D26 TOKENIZER BLIND_TSV OUT".into()),
    }
}
