//! Throughput probe of the exact CUDA n-gram counter (claude-412). Not an
//! evaluation: it counts orders on a UORT token stream, times each, and checks
//! that two partition budgets give one SHA. A separate binary, so the training
//! driver's source identity is unchanged.
//! Usage: ngram-count-probe TOKENS.u16 MAX_ORDER BUDGET_A BUDGET_B [adjusted]   (needs `--features cuda`)
//! With `adjusted` it counts the modified Kneser-Ney adjusted counts of orders
//! 0..=MAX_ORDER under N = MAX_ORDER (claude-418) instead of the raw counts.

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !(args.len() == 4 || (args.len() == 5 && args[4] == "adjusted")) {
        eprintln!("usage: ngram-count-probe TOKENS.u16 MAX_ORDER BUDGET_A BUDGET_B [adjusted]");
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
    let adjusted = args.len() == 5;
    let started = std::time::Instant::now();
    let counter = NgramCounter::new(0, &tokens)?;
    println!(
        "{{\"tokens\":{},\"upload_ms\":{},\"adjusted\":{adjusted},\"image_sha256\":{:?}}}",
        tokens.len(),
        started.elapsed().as_millis(),
        compiled_ngram_count_image()
    );
    for order in 0..=max_order {
        let mut digests = Vec::new();
        for budget in budgets {
            let started = std::time::Instant::now();
            let (plan, assembler) = if adjusted {
                let mut assembler = NgramAssembler::new_adjusted(order, false)?;
                (counter.count_order_adjusted(order, max_order, budget, &mut assembler)?, assembler)
            } else {
                let mut assembler = NgramAssembler::new(order, false)?;
                (counter.count_order(order, budget, &mut assembler)?, assembler)
            };
            let micros = started.elapsed().as_micros().max(1);
            let (digest, _, _) = assembler.finish();
            println!(
                "{{\"order\":{order},\"highest\":{},\"budget\":{budget},\"plan_order\":{},\"partitions\":{},\"targets\":{},\"grams\":{},\"contexts\":{},\"count_of_counts\":{:?},\"micros\":{micros},\"targets_per_s\":{},\"triple_sha256\":\"{}\",\"context_sha256\":\"{}\"}}",
                if adjusted { max_order.to_string() } else { "null".to_string() },
                // Adjusted order 0 under N >= 1 returns order 1's plan, whose
                // partitions feed the unigram continuation.
                if adjusted && order == 0 && max_order >= 1 { 1 } else { order },
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
