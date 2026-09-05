//! Construction and first-use cases. This module never enters serving.
use super::*;

fn case(split: &str, world: usize, task: usize, style: usize) -> Case {
    let (names, places) = if split == "fit" {
        (
            ["ada", "cyra", "ivy", "mina"],
            ["Rome", "Cairo", "Perth", "Dover"],
        )
    } else {
        (
            ["velra", "tovin", "neril", "sovek"],
            ["Lodov", "Merok", "Vesul", "Talven"],
        )
    };
    let a = names[world % 4];
    let b = names[(world + 1) % 4];
    let x = places[(world + style) % 4];
    let y = places[(world + style + 1) % 4];
    if task == 5 {
        let mut c = word_copy_case(split, style, world * 4 + style, names[(world + style) % 4]);
        c.id = format!("role-read/{split}/{world}/{task}/{style}");
        c.pair_id = format!("role-read/{split}/{world}/{task}/{}", style / 2);
        c.world = if split == "fit" {
            1_000_000 + world
        } else {
            2_000_000 + world
        };
        c.variant = style % 2;
        return c;
    }
    let (fact, answer) = match task {
        0 => (format!("{a} in {x}."), x),
        1 => (format!("{x} holds {a}."), x),
        2 if style % 2 == 0 => (format!("{a} in {x}. {b} in {y}."), x),
        2 => (format!("{b} in {y}. {a} in {x}."), x),
        3 => (format!("{a} in {x}. {a} now in {y}."), y),
        _ => (format!("{b} in {x}."), "Unknown"),
    };
    let query = match style {
        1 => format!("Which city is {a} in?"),
        2 => format!("What city does {a} live in?"),
        _ => format!("Where is {a}?"),
    };
    Case {
        id: format!("role-read/{split}/{world}/{task}/{style}"),
        pair_id: format!("role-read/{split}/{world}/{task}/{}", style / 2),
        family: "prose".into(),
        task: format!("role_{task}_style_{style}"),
        world: if split == "fit" {
            1_000_000 + world
        } else {
            2_000_000 + world
        },
        variant: style % 2,
        prompt: format!("Record: {fact} {query} Answer:"),
        response: format!(" {answer}.\n"),
    }
}

pub(super) fn prepare() -> ProbeResult<()> {
    let args: Vec<_> = std::env::args().skip(2).collect();
    if args.len() != 2 {
        return Err("prepare-role-read SOURCE_V3 NEW_DIRECTORY".into());
    }
    let mut source: Source = serde_json::from_slice(&fs::read(&args[0])?)?;
    validate_source(&source)?;
    let prior = source.clone();
    for world in 0..4 {
        for task in 0..6 {
            for style in 0..4 {
                source.fit.push(case("fit", world, task, style));
            }
        }
    }
    // The source copies prior development only for preservation evaluation;
    // fitting reads only `fit`. First-use cases are a separate saved source.
    validate_source(&source)?;
    let mut first = source.clone();
    first.development.clear();
    for task in 0..6 {
        for style in 0..4 {
            first.development.push(case("first-use", 0, task, style));
        }
    }
    validate_source(&first)?;
    let words = |text: &str| {
        text.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .filter(|w| !w.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let old_words: BTreeSet<_> = prior
        .fit
        .iter()
        .chain(&prior.development)
        .chain(&source.fit)
        .flat_map(|c| words(&c.prompt).into_iter().chain(words(&c.response)))
        .collect();
    for novel in [
        "velra", "tovin", "neril", "sovek", "Lodov", "Merok", "Vesul", "Talven",
    ] {
        if old_words.contains(novel) {
            return Err(format!("first-use word already in supplied material: {novel}").into());
        }
    }
    for c in source
        .fit
        .iter()
        .filter(|c| c.id.starts_with("role-read/"))
        .chain(&first.development)
    {
        // Prose examples deliberately fit the whole capture. Rust tests the
        // actual last16 words, preserving the required declared argument.
        let retained = words(&c.prompt);
        if c.family == "prose" && retained.len() > 16 {
            return Err(format!("retention exceeds16: {}", c.id).into());
        }
        if c.family == "rust" {
            let target = c.response.lines().next().ok_or("empty Rust response")?;
            if !retained.iter().rev().take(16).any(|w| w == target) {
                return Err("Rust argument not retained".into());
            }
        }
    }
    let root = Path::new(&args[1]);
    fs::create_dir(root)?;
    source.scope="OPEN role-read construction plus unchanged prior preservation. Position-shared source/query local contexts, role reversal, source order, updates, absence and Rust names. No semantic labels enter serving.".into();
    first.scope="24 first-use cases prepared before fitting/design selection:20 prose (4 unsupported) and4 Rust; new entity/value words relative to the supplied construction and prior development. Known grammar families, no unrestricted-language or final sealed capability claim. Acceptance:24/24 complete responses, all62 preservation responses and8binding outputs, causal/snapshot and generated-Rust semantic checks. A negative retains #1137 open.".into();
    write_json(&root.join("development.json"), &source)?;
    write_json(&root.join("first-use.json"), &first)?;
    write_json(
        &root.join("acceptance.json"),
        &json!({"fit_cases":source.fit.len(),"first_use":24,"required_exact":24,"preservation":62,"binding":8,"selection":"construction quantized objective only; open development may diagnose, but no changes after first-use evaluation","retention_and_split_assertions":"PASS before any evaluation","model_execution":"NOT_RUN"}),
    )?;
    println!(
        "{}",
        json!({"fit":source.fit.len(),"preservation":source.development.len(),"first_use":24,"assertions":"PASS"})
    );
    Ok(())
}
