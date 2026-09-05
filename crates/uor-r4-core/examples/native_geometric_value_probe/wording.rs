//! Authored raw-text interventions. This module never enters inference.
use super::*;

/// Fresh complete-answer check authored after selecting the single-row ablation.
/// No fitting consumes these cases. A later reuse must label them development.
pub(super) fn prepare_entry_check() -> ProbeResult<()> {
    let args: Vec<_> = std::env::args().skip(2).collect();
    if args.len() != 2 {
        return Err("prepare-entry-check SOURCE_V3 NEW_OUTPUT_SOURCE".into());
    }
    let mut source: Source = serde_json::from_slice(&fs::read(&args[0])?)?;
    validate_source(&source)?;
    if source.schema != WORD_COPY_SOURCE_SCHEMA {
        return Err("entry check requires word-copy source".into());
    }
    source.development.clear();
    for task in 0..4 {
        for style in 0..4 {
            let names = ["elva", "brin", "sena", "kael"];
            let cities = ["Metz", "Cork", "Graz", "Pune"];
            let a = names[style];
            let b = names[(style + 1) % 4];
            let x = cities[style];
            let y = cities[(style + 2) % 4];
            let (facts, answer) = match task {
                0 => (format!("{a} lives in {x}."), x),
                1 if style % 2 == 0 => (format!("{a} in {x}. {b} in {y}."), x),
                1 => (format!("{b} in {y}. {a} in {x}."), x),
                2 => (format!("{a} in {x}. {a} now in {y}."), y),
                _ => (format!("{b} lives in {x}."), "Unknown"),
            };
            let query = match style {
                1 => format!("What city does {a} live in?"),
                3 => format!("Which city is {a} in?"),
                _ => format!("Where is {a}?"),
            };
            let prompt = if style == 2 {
                format!("User: {facts}\nUser: {query}\nAssistant:")
            } else {
                format!("{facts} {query} Answer:")
            };
            let words = prompt
                .split(|c: char| !c.is_ascii_alphanumeric())
                .filter(|w| !w.is_empty())
                .count();
            if words > 16 {
                return Err("entry check exceeds retained-word capacity".into());
            }
            source.development.push(Case {
                id: format!("source-entry/final/{task}/{style}"),
                // Evaluator bookkeeping only: these vary wording and names,
                // so pair metrics are not a matched counterfactual claim.
                pair_id: format!("source-entry/final/{task}/{}", style / 2),
                family: "prose".into(),
                task: format!("fact_{task}_style_{style}"),
                world: 900000 + style,
                variant: style % 2,
                prompt,
                response: format!(" {answer}.\n"),
            });
        }
    }
    source.scope = "Final16 after fixed zero-binding ablation selection: four each simple/distractor/update/unsupported, new entity/value combinations, four question/wrapper forms; all source words retained. Pune occurred in earlier material; this is not fully unseen vocabulary. Construction copied for evaluator compatibility only; no refit. First use follows saved preparation and design selection; later reuse is open development.".into();
    validate_source(&source)?;
    let bytes = serde_json::to_vec_pretty(&source)?;
    write_new(Path::new(&args[1]), &bytes)?;
    println!(
        "{}",
        json!({"source":args[1],"blake3":blake3::hash(&bytes).to_hex().to_string(),"evaluation":16})
    );
    Ok(())
}

fn case(split: &str, world: usize, task: usize, style: usize, variant: usize) -> Case {
    let (names, cities) = match split {
        "fit" => (
            ["ivy", "jade", "lyra", "mina"],
            ["Rome", "Dover", "Cairo", "Perth"],
        ),
        _ => (
            ["zora", "leif", "rhea", "noam"],
            ["Kyoto", "Turin", "Basel", "Dakar"],
        ),
    };
    let a = names[world % 4];
    let b = names[(world + 1) % 4];
    let x = cities[(world + variant) % 4];
    let y = cities[(world + variant + 2) % 4];
    let (facts, answer) = match task {
        0 => (format!("{a} lives in {x}."), x),
        1 if variant == 0 => (format!("{a} in {x}. {b} in {y}."), x),
        1 => (format!("{b} in {y}. {a} in {x}."), x),
        2 => (format!("{a} in {x}. {a} now in {y}."), y),
        _ => (format!("{b} lives in {x}."), "Unknown"),
    };
    let query = match style {
        1 | 3 => format!("What city does {a} live in?"),
        4 => format!("Which city is {a} in?"),
        _ => format!("Where is {a}?"),
    };
    let prompt = if matches!(style, 2 | 3) {
        format!("User: {facts}\nUser: {query}\nAssistant:")
    } else {
        format!("{facts} {query} Answer:")
    };
    Case {
        id: format!("wording/{split}/{world}/{task}/{style}/{variant}"),
        pair_id: format!("wording/{split}/{world}/{task}/{style}"),
        family: "prose".into(),
        task: format!("wording_{task}_style_{style}"),
        world: if split == "fit" {
            600000 + world
        } else {
            700000 + world
        },
        variant,
        prompt,
        response: format!(" {answer}.\n"),
    }
}

pub(super) fn prepare() -> ProbeResult<()> {
    let args: Vec<_> = std::env::args().skip(2).collect();
    if args.len() != 2 {
        return Err("prepare-wording SOURCE_V3 NEW_DIRECTORY".into());
    }
    let mut source: Source = serde_json::from_slice(&fs::read(&args[0])?)?;
    validate_source(&source)?;
    if source.schema != WORD_COPY_SOURCE_SCHEMA || source.fit.len() != 288 {
        return Err("wording preparation requires the unchanged 288-case fact source".into());
    }
    let root = Path::new(&args[1]);
    fs::create_dir(root)?;
    // Diagnostic factorial: same entity/value and query meaning; question,
    // source distractor and speaker wrapper each vary independently.
    let mut diagnostic = source.clone();
    diagnostic.development.clear();
    for distractor in 0..2 {
        for long in 0..2 {
            for wrapper in 0..2 {
                for variant in 0..2 {
                    let city = ["Oslo", "Lima"][variant];
                    let facts = format!(
                        "{}orin lives in {city}.",
                        if distractor == 1 {
                            "suri has 73 coins. tavi has 301 coins. "
                        } else {
                            ""
                        }
                    );
                    let query = if long == 1 {
                        "What city does orin live in?"
                    } else {
                        "Where is orin?"
                    };
                    let prompt = if wrapper == 1 {
                        format!("User: {facts}\nUser: {query}\nAssistant:")
                    } else {
                        format!("{facts} {query} Answer:")
                    };
                    let pair = format!("wording/diagnostic/{distractor}/{long}/{wrapper}");
                    diagnostic.development.push(Case {
                        id: format!("{pair}/{variant}"),
                        pair_id: pair,
                        family: "prose".into(),
                        task: format!("diagnostic_d{distractor}_q{long}_w{wrapper}"),
                        world: 800000,
                        variant,
                        prompt,
                        response: format!(" {city}.\n"),
                    });
                }
            }
        }
    }
    for world in 0..4 {
        for task in 0..4 {
            for style in 0..3 {
                for variant in 0..2 {
                    source.fit.push(case("fit", world, task, style, variant));
                }
            }
        }
    }
    let mut reserved = source.clone();
    reserved.development.clear();
    for world in [0, 2] {
        for task in 0..4 {
            for style in [3, 4] {
                for variant in 0..2 {
                    reserved
                        .development
                        .push(case("reserved", world, task, style, variant));
                }
            }
        }
    }
    source.scope.push_str(" Wording successor:96new construction cases cross short/long questions and short-question speaker wrappers. Existing62development remain open; separately saved32reserved cases cross long-question speaker wrappers and a new question form with new names/value combinations. No reserved response is passed to fitting.");
    for (name, data) in [
        ("source", &source),
        ("diagnostic", &diagnostic),
        ("reserved", &reserved),
    ] {
        validate_source(data)?;
        let bytes = serde_json::to_vec_pretty(data)?;
        write_new(&root.join(format!("{name}.json")), &bytes)?;
        println!(
            "{}",
            json!({"source":name,"fit":data.fit.len(),"evaluation":data.development.len(),"blake3":blake3::hash(&bytes).to_hex().to_string()})
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_wording_sources_retain_all_fact_words() {
        for split in ["fit", "reserved"] {
            for task in 0..4 {
                for style in 0..5 {
                    let c = case(split, 0, task, style, 0);
                    let words = c
                        .prompt
                        .split(|c: char| !c.is_ascii_alphanumeric())
                        .filter(|s| !s.is_empty())
                        .count();
                    assert!(words <= 16, "{}: {words}", c.prompt);
                }
            }
        }
    }
}
