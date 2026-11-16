//! Throughput probe of the exact CUDA n-gram counter (claude-412). Not an
//! evaluation: it counts orders on a UORT token stream, times each, and checks
//! that two partition budgets give one SHA. A separate binary, so the training
//! driver's source identity is unchanged.
//! Usage: ngram-count-probe TOKENS.u16 MAX_ORDER BUDGET_A BUDGET_B   (needs `--features cuda`)

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        eprintln!("usage: ngram-count-probe TOKENS.u16 MAX_ORDER BUDGET_A BUDGET_B");
        return std::process::ExitCode::from(2);
    }
    #[cfg(feature = "cuda")]
    {
        match run(&args) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                std::process::ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        eprintln!("ngram-count-probe requires --features cuda");
        std::process::ExitCode::FAILURE
    }
}

#[cfg(feature = "cuda")]
fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    use uor_r4_discrete::ngram_count::{read_uort_tokens, NgramAssembler};
    use uor_r4_discrete::ngram_count_device::{compiled_ngram_count_image, NgramCounter};
    let tokens = read_uort_tokens(&std::fs::read(&args[0])?)?;
    let max_order: u32 = args[1].parse()?;
    let budgets: [u64; 2] = [args[2].parse()?, args[3].parse()?];
    let started = std::time::Instant::now();
    let counter = NgramCounter::new(0, &tokens)?;
    println!(
        "{{\"tokens\":{},\"upload_ms\":{},\"image_sha256\":{:?}}}",
        tokens.len(),
        started.elapsed().as_millis(),
        compiled_ngram_count_image()
    );
    for order in 0..=max_order {
        let mut digests = Vec::new();
        for budget in budgets {
            let mut assembler = NgramAssembler::new(order, false)?;
            let started = std::time::Instant::now();
            let plan = counter.count_order(order, budget, &mut assembler)?;
            let micros = started.elapsed().as_micros().max(1);
            let (digest, _, _) = assembler.finish();
            println!(
                "{{\"order\":{order},\"budget\":{budget},\"partitions\":{},\"targets\":{},\"grams\":{},\"contexts\":{},\"count_of_counts\":{:?},\"micros\":{micros},\"targets_per_s\":{},\"triple_sha256\":\"{}\",\"context_sha256\":\"{}\"}}",
                plan.len(),
                digest.targets,
                digest.grams,
                digest.contexts,
                digest.count_of_counts,
                u128::from(digest.targets) * 1_000_000 / micros,
                digest.triple_sha256,
                digest.context_sha256
            );
            digests.push(digest);
        }
        if digests[0] != digests[1] {
            return Err(format!("order {order}: the two budgets gave different digests").into());
        }
    }
    Ok(())
}
