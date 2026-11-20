use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, io::Write, path::Path};
use uor_r4_integer::{
    discrete_d11::IntegerDiscreteModel,
    paired_role::{validate_role_ids, DetachedWordSites, RequestedContext},
};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const MODEL: &str = "b17352679c64a96bf657fad7640d4bb05ea3de0ecd08dadd13e9d2756e979bc4";
const BLIND: &str = "176082f66f2e73ba4080567fb2b5e77cc7c4478c14da939b47fad72fa2660a30";
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

fn emit(root: &Path, sites: &[[u32; 5]], cells: &[[u32; 4]], rows: &[Row]) -> Result<()> {
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
    for mode in [7, 8] {
        let mut f = fs::File::create_new(root.join(format!("mode-{mode}.bin")))?;
        write_words(
            &mut f,
            [
                0x31443452,
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
    let blind = fs::read(blind)?;
    if format!("{:x}", Sha256::digest(&blind)) != BLIND {
        return Err("wrong D32 exposed text tape".into());
    }
    let mut rows = Vec::new();
    for line in std::str::from_utf8(&blind)?.lines() {
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
    let lexical = DetachedWordSites::from_tokenizer(&tokenizer)?;
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
                lexical.classes()[id],
            ]
        })
        .collect();
    emit(out, &sites, state.roles.cells(), &rows)
}

const EMPTY: [u32; 5] = [NIL, NIL, NIL, NIL, 0];
const NO_RANK: [u32; 8] = [NIL, NIL, NIL, NIL, LOW, LOW, LOW, LOW];

struct Finite {
    ids: &'static [u32],
    content_end: Option<u32>,
    ranks: [u32; 8],
    records: &'static [(usize, [u32; 5])],
}

fn fixture(out: &Path) -> Result<()> {
    let mut sites = vec![[0, 0, 0, 0, 0]; 4096];
    sites[17][3] = 1;
    sites[51][4] = 1;
    for id in [80, 81, 82, 91] {
        sites[id][2] = 2;
        sites[id][4] = 2;
    }
    // A token can carry an old assertion class without proving lexical bytes.
    sites[84][2] = 2;
    sites[90][2] = 1;
    sites[92][0] = 3;
    sites[93][0] = 4;
    let fixtures = [
        // Earliest factual head and full query. A strong exclusion can coexist.
        Finite {
            ids: &[0, 17, 51, 80, 81, 17, 30, 51, 80, 81, 92, 17],
            content_end: None,
            ranks: [7, 10, 7, 10, (-20i32) as u32, 16, LOW, LOW],
            records: &[(2, [5, NIL, NIL, 5, 0]), (7, [10, 2, 5, NIL, 3])],
        },
        // A legal candidate and strong request score cannot replace a witness.
        Finite {
            ids: &[0, 17, 30, 51, 80, 81, 93, 17],
            content_end: None,
            ranks: NO_RANK,
            records: &[(3, [6, NIL, NIL, NIL, 0])],
        },
        Finite {
            // The short factual prefix cannot witness a longer query name.
            ids: &[0, 17, 51, 80, 81, 17, 30, 51, 80, 81, 82, 93, 17],
            content_end: None,
            ranks: NO_RANK,
            records: &[(2, [5, NIL, NIL, 5, 0]), (7, [11, NIL, NIL, NIL, 0])],
        },
        Finite {
            // The short query prefix cannot match a longer factual head.
            ids: &[0, 17, 51, 80, 81, 82, 17, 30, 51, 80, 81, 93, 17],
            content_end: None,
            ranks: NO_RANK,
            records: &[(2, [6, NIL, NIL, 6, 0]), (8, [11, NIL, NIL, NIL, 0])],
        },
        Finite {
            // The suffix of the factual name lacks an exact witness.
            ids: &[0, 17, 51, 80, 81, 17, 30, 51, 81, 93, 17],
            content_end: None,
            ranks: NO_RANK,
            records: &[(2, [5, NIL, NIL, 5, 0]), (7, [9, NIL, NIL, NIL, 0])],
        },
        Finite {
            // Punctuation after a blank cannot start a detached word.
            ids: &[0, 17, 51, 83, 80, 81, 17, 30, 51, 83, 80, 81, 93, 17],
            content_end: None,
            ranks: NO_RANK,
            records: &[],
        },
        Finite {
            // A continuation after the framed end rules out a prefix candidate.
            ids: &[0, 17, 51, 80, 81, 17, 30, 51, 80, 81, 92, 17],
            content_end: Some(9),
            ranks: NO_RANK,
            records: &[(2, [5, NIL, NIL, 5, 0])],
        },
        Finite {
            // Two blanks: the lexical edge exists but is not a factual head.
            ids: &[0, 17, 51, 51, 80, 81, 17, 30, 51, 80, 81, 93, 17],
            content_end: None,
            ranks: NO_RANK,
            records: &[(3, [6, NIL, NIL, NIL, 0]), (8, [11, NIL, NIL, NIL, 0])],
        },
        Finite {
            // Deduplicate witnesses while preserving the earliest exact one.
            ids: &[
                0, 17, 51, 80, 81, 17, 51, 80, 81, 17, 30, 51, 80, 81, 93, 17,
            ],
            content_end: None,
            ranks: [11, 14, NIL, NIL, 20, LOW, 0, LOW],
            records: &[
                (2, [5, NIL, NIL, 5, 0]),
                (6, [9, 2, 5, 9, 1]),
                (11, [14, 2, 5, NIL, 1]),
            ],
        },
        Finite {
            // Two equal, eligible query occurrences tie and withhold.
            ids: &[
                0, 17, 51, 80, 81, 17, 30, 51, 80, 81, 93, 30, 51, 80, 81, 93, 17,
            ],
            content_end: None,
            ranks: [NIL, NIL, NIL, NIL, 20, LOW, 20, LOW],
            records: &[
                (2, [5, NIL, NIL, 5, 0]),
                (7, [10, 2, 5, NIL, 1]),
                (12, [15, 2, 5, NIL, 1]),
            ],
        },
        Finite {
            // A punctuation-bearing continuation class does not prove a word.
            ids: &[0, 17, 51, 84, 81, 17, 30, 51, 84, 81, 93, 17],
            content_end: None,
            ranks: NO_RANK,
            records: &[],
        },
        Finite {
            // Original assertion repetition is active in both arms.
            ids: &[0, 17, 90, 91, 17, 30, 90, 91, 93, 17],
            content_end: None,
            ranks: [6, 8, NIL, NIL, 20, LOW, LOW, LOW],
            records: &[],
        },
        Finite {
            // A genuine punctuation boundary can end a complete detached word.
            ids: &[0, 17, 51, 80, 81, 17, 30, 51, 80, 81, 83, 17],
            content_end: None,
            ranks: [7, 10, NIL, NIL, 0, LOW, LOW, LOW],
            records: &[(2, [5, NIL, NIL, 5, 0]), (7, [10, 2, 5, NIL, 1])],
        },
        Finite {
            // A word after the user frame is never an eligible candidate.
            ids: &[0, 17, 51, 80, 81, 17, 30, 51, 80, 81, 93, 17],
            content_end: Some(7),
            ranks: NO_RANK,
            records: &[(2, [5, NIL, NIL, 5, 0])],
        },
        Finite {
            // The same lexical tape can carry opposite local role evidence.
            ids: &[
                0, 17, 51, 80, 81, 17, 30, 51, 80, 81, 93, 30, 51, 80, 81, 92, 17,
            ],
            content_end: None,
            ranks: [7, 10, 12, 15, 20, 16, (-20i32) as u32, LOW],
            records: &[
                (2, [5, NIL, NIL, 5, 0]),
                (7, [10, 2, 5, NIL, 1]),
                (12, [15, 2, 5, NIL, 3]),
            ],
        },
        Finite {
            // Token 86 is a multi-space piece, classified as other.
            ids: &[0, 17, 86, 80, 81, 17, 30, 86, 80, 81, 93, 17],
            content_end: None,
            ranks: NO_RANK,
            records: &[],
        },
    ];
    let rows = fixtures
        .iter()
        .enumerate()
        .map(|(id, fixture)| {
            Ok(Row {
                id: id.try_into()?,
                ids: fixture.ids.to_vec(),
                content: [
                    1,
                    fixture.content_end.unwrap_or(fixture.ids.len().try_into()?),
                ],
            })
        })
        .collect::<Result<Vec<_>>>()?;
    emit(
        out,
        &sites,
        &[[3 * 65537 + 3, 0, 0, 1], [3 * 65537 + 4, 0, 1, 0]],
        &rows,
    )?;
    for mode in [7, 8] {
        let mut expected = fs::File::create_new(out.join(format!("mode-{mode}.expected")))?;
        for (index, fixture) in fixtures.iter().enumerate() {
            let ranks = if mode == 8 || index == 11 {
                fixture.ranks
            } else {
                NO_RANK
            };
            write_words(&mut expected, ranks)?;
            let mut endpoints = [EMPTY; 192];
            if mode == 8 {
                for &(start, entry) in fixture.records {
                    endpoints[start] = entry;
                }
            }
            for entry in endpoints {
                write_words(&mut expected, entry)?;
            }
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
