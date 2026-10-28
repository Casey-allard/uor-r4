//! `train-in-space --data-root DIR --out DIR [--objective sheet|voronoi|nearest|cell] [--steps N]
//! [--train-split DIR --train-sha256 HEX --train-mask-sha256 HEX]
//! [--batch B] [--eval-every K] [--seed S] [--lr X] [--lr-end X] [--decay-mult X]
//! [--retain-rates 0|1] [--fold-scores 0|1] [--reflect-floor 0|1] [--fix-input-table 0|1] [--flock-read K] [--content-read K] [--hamming-read H] [--opening-read K] [--opening-anchor turn|last] [--prompt-register Q] [--register-picks K] [--register-select learned|all] [--width N] [--type-spin 0|1] [--samples N] [--reply-tokens N]`
//!
//! `train-in-space --data-root DIR --inspect RUN [--retain-rates 0|1] [--fold-scores 0|1] [--reflect-floor 0|1] [--seed S]`
//!
//! `train-in-space --data-root DIR --reach CHECKPOINT [--retain-rates 0|1] [--fold-scores 0|1] [--reflect-floor 0|1] [--seed S]`
//!
//! `train-in-space --data-root DIR --grad-norms 1 [--objective ...] [--retain-rates 0|1] [--fold-scores 0|1] [--reflect-floor 0|1] [--batch B] [--seed S]`
//!
//! `train-in-space --data-root DIR --decode-speed CHECKPOINT [--objective nearest] [--retain-rates 0|1] [--reply-tokens N]`
//!
//! `train-in-space --recall PAIRS --out DIR [--steps N] [--batch B] [--eval-every K] [--seed S] [--lr X] [--lr-end X] [--retain-rates 0|1] [--flock-read K] [--content-read K] [--hamming-read H] [--opening-read K] [--opening-anchor turn|last] [--prompt-register Q] [--register-picks K] [--register-select learned|all]`:
//! the multi-query associative recall task (`in_space_long::run_recall`), `PAIRS` key-value pairs
//! per distance bucket in each 192-token window, fresh every step; held-out recall per bucket goes
//! to `DIR/recall.jsonl`, and the final model to `DIR/models/`. Needs no `--data-root`, and
//! ignores it if given.
//!
//! `train-in-space --data-root DIR --opening-bound 1`: how well the prompt words a window shows
//! could name each reply opening, as TRAIN's commonest opening per key scored on DEV. A count
//! over the corpus that needs no GPU; one JSON line per key, then one line per split
//! (`dev`, `train`) with how many openings the opening read's turn anchor keeps the first-8 key at.
//!
//! Long training of the SpiralCore/FBS geometric trunk in its own space, with text at every
//! checkpoint (`uor_r4_float_train::in_space_long`). GPU only. `--data-root` holds `tokenizer.json`,
//! `mw-balp/train/` and `ft/dev/`, each split with `tokens.u16` and `response_mask.u8`. `--out`
//! must not exist; it receives `config.json`, `progress.jsonl`, `samples.md`, `samples.jsonl` and
//! `models/step-*/`. `--inspect` instead reads every checkpoint a run saved and prints one JSON
//! line each, after one line with the address roots' separation.
use std::env;
use std::path::PathBuf;

use candle_core::Device;
use uor_r4_float_train::flock_read_opening::OpeningAnchor;
use uor_r4_float_train::flock_read_register::RegisterSelect;
use uor_r4_float_train::in_space_long::{
    context_reach, decode_speed, gradient_norms, inspect_run, opening_bound, root_separation,
    run_long, run_recall, LongConfig, PointObjective, TrainSplit, DEFAULT_REGISTER_PICKS,
};
use uor_r4_float_train::{FloatTrainError, Result};

fn main() -> Result<()> {
    let mut data_root = None;
    let mut out = None;
    let mut inspect = None;
    let mut reach = None;
    let mut grad_norms = false;
    let mut decode = None;
    let mut bound = false;
    let (mut split_dir, mut split_tokens, mut split_mask) = (None, None, None);
    let mut recall = 0usize;
    let (mut opening_read, mut opening_anchor) = (None, None);
    let (mut prompt_register, mut register_picks, mut register_select) = (None, None, None);
    let mut config = LongConfig {
        objective: PointObjective::Sheet,
        steps: 32_768,
        batch: 16,
        eval_every: 2_048,
        seed: 1,
        lr: 0.002,
        lr_end: f64::NAN,
        decay_mult: 20.0,
        retain_rates: false,
        fold_scores: false,
        reflect_floor: false,
        fix_input_table: false,
        train_split: None,
        flock_read: 0,
        content_read: 0,
        hamming_read: 0,
        opening_read: 0,
        opening_anchor: None,
        prompt_register: 0,
        register_picks: None,
        register_select: None,
        width: uor_r4_float_train::hamilton_sheet::SHEET_WIDTH,
        type_spin: false,
        samples: 6,
        reply_tokens: 48,
    };
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| FloatTrainError::Invalid(format!("missing {arg} value")))?;
        let number = |v: &str| {
            v.parse::<usize>()
                .map_err(|_| FloatTrainError::Invalid(format!("{arg} wants a whole number")))
        };
        let switch = |v: &str| match v {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(FloatTrainError::Invalid(format!("{arg} wants 0 or 1"))),
        };
        match arg.as_str() {
            "--data-root" => data_root = Some(PathBuf::from(&value)),
            "--out" => out = Some(PathBuf::from(&value)),
            "--inspect" => inspect = Some(PathBuf::from(&value)),
            "--reach" => reach = Some(PathBuf::from(&value)),
            "--grad-norms" => grad_norms = switch(&value)?,
            "--decode-speed" => decode = Some(PathBuf::from(&value)),
            "--opening-bound" => bound = switch(&value)?,
            "--recall" => recall = number(&value)?,
            "--objective" => {
                config.objective = match value.as_str() {
                    "sheet" => PointObjective::Sheet,
                    "voronoi" => PointObjective::Voronoi,
                    "nearest" => PointObjective::Nearest,
                    "cell" => PointObjective::Cell,
                    other => {
                        return Err(FloatTrainError::Invalid(format!(
                            "--objective wants sheet, voronoi, nearest or cell, not {other}"
                        )))
                    }
                }
            }
            "--steps" => config.steps = number(&value)?,
            "--batch" => config.batch = number(&value)?,
            "--eval-every" => config.eval_every = number(&value)?,
            "--seed" => config.seed = number(&value)? as u64,
            "--samples" => config.samples = number(&value)?,
            "--reply-tokens" => config.reply_tokens = number(&value)?,
            "--retain-rates" => config.retain_rates = switch(&value)?,
            "--fold-scores" => config.fold_scores = switch(&value)?,
            "--reflect-floor" => config.reflect_floor = switch(&value)?,
            "--fix-input-table" => config.fix_input_table = switch(&value)?,
            "--flock-read" => config.flock_read = number(&value)?,
            "--content-read" => config.content_read = number(&value)?,
            "--hamming-read" => config.hamming_read = number(&value)?,
            "--opening-read" => match number(&value)? {
                k @ 1..=16 => opening_read = Some(k),
                _ => {
                    return Err(FloatTrainError::Invalid(
                        "--opening-read wants 1 to 16".to_owned(),
                    ))
                }
            },
            "--opening-anchor" => {
                opening_anchor = Some(match value.as_str() {
                    "turn" => OpeningAnchor::Turn,
                    "last" => OpeningAnchor::Last,
                    other => {
                        return Err(FloatTrainError::Invalid(format!(
                            "--opening-anchor wants turn or last, not {other}"
                        )))
                    }
                })
            }
            "--prompt-register" => match number(&value)? {
                q @ 1..=16 => prompt_register = Some(q),
                _ => {
                    return Err(FloatTrainError::Invalid(
                        "--prompt-register wants 1 to 16".to_owned(),
                    ))
                }
            },
            "--register-picks" => match number(&value)? {
                k @ 1..=32 => register_picks = Some(k),
                _ => {
                    return Err(FloatTrainError::Invalid(
                        "--register-picks wants 1 to 32".to_owned(),
                    ))
                }
            },
            "--register-select" => {
                register_select = Some(match value.as_str() {
                    "learned" => RegisterSelect::Learned,
                    "all" => RegisterSelect::All,
                    other => {
                        return Err(FloatTrainError::Invalid(format!(
                            "--register-select wants learned or all, not {other}"
                        )))
                    }
                })
            }
            "--width" => config.width = number(&value)?,
            "--type-spin" => config.type_spin = switch(&value)?,
            "--train-split" => split_dir = Some(PathBuf::from(&value)),
            "--train-sha256" => split_tokens = Some(value.clone()),
            "--train-mask-sha256" => split_mask = Some(value.clone()),
            "--lr" | "--lr-end" | "--decay-mult" => {
                let v: f64 = value
                    .parse()
                    .map_err(|_| FloatTrainError::Invalid(format!("{arg} wants a number")))?;
                match arg.as_str() {
                    "--lr" => config.lr = v,
                    "--lr-end" => config.lr_end = v,
                    _ => config.decay_mult = v,
                }
            }
            other => {
                return Err(FloatTrainError::Invalid(format!(
                    "unrecognized argument: {other}"
                )))
            }
        }
    }
    // The opening read (claude-154): its anchor is written exactly when the read is on.
    match (opening_read, opening_anchor) {
        (Some(k), anchor) => {
            config.opening_read = k;
            config.opening_anchor = Some(anchor.unwrap_or_default());
        }
        (None, Some(_)) => {
            return Err(FloatTrainError::Invalid(
                "--opening-anchor needs --opening-read".to_owned(),
            ))
        }
        (None, None) => {}
    }
    // The prompt register (claude-155): its picks and select are written exactly when it is on.
    match (prompt_register, register_picks, register_select) {
        (Some(q), _, Some(RegisterSelect::All)) if q != 1 => {
            return Err(FloatTrainError::Invalid(
                "--register-select all wants --prompt-register 1".to_owned(),
            ))
        }
        (Some(q), picks, select) => {
            config.prompt_register = q;
            config.register_picks = Some(picks.unwrap_or(DEFAULT_REGISTER_PICKS));
            config.register_select = Some(select.unwrap_or_default());
        }
        (None, None, None) => {}
        (None, _, _) => {
            return Err(FloatTrainError::Invalid(
                "--register-picks and --register-select need --prompt-register".to_owned(),
            ))
        }
    }
    if config.lr_end.is_nan() {
        config.lr_end = config.lr;
    }
    config.train_split = match (split_dir, split_tokens, split_mask) {
        (None, None, None) => None,
        (Some(dir), Some(tokens_sha256), Some(mask_sha256)) => Some(TrainSplit {
            dir,
            tokens_sha256,
            mask_sha256,
        }),
        _ => {
            return Err(FloatTrainError::Invalid(
                "--train-split needs --train-sha256 and --train-mask-sha256, and they need it"
                    .to_owned(),
            ))
        }
    };
    let modes = [
        inspect.is_some(),
        reach.is_some(),
        grad_norms,
        decode.is_some(),
        bound,
        out.is_some(),
    ];
    if modes.iter().filter(|&&m| m).count() > 1 {
        return Err(FloatTrainError::Invalid(
            "give one of --out, --inspect, --reach, --grad-norms 1, --decode-speed or --opening-bound 1"
                .to_owned(),
        ));
    }
    if recall > 0 {
        let out = out.ok_or_else(|| FloatTrainError::Invalid("--recall needs --out".to_owned()))?;
        if modes[..5].iter().any(|&m| m) {
            return Err(FloatTrainError::Invalid(
                "--recall trains; it takes --out and no other mode".to_owned(),
            ));
        }
        let device = Device::new_cuda(0)?;
        return run_recall(&out, &config, recall, &device);
    }
    let data_root =
        data_root.ok_or_else(|| FloatTrainError::Invalid("--data-root is required".to_owned()))?;
    if bound {
        let (rows, checks) = opening_bound(&data_root)?;
        for row in rows {
            println!("{}", serde_json::to_string(&row)?);
        }
        for check in checks {
            println!("{}", serde_json::to_string(&check)?);
        }
        return Ok(());
    }
    let device = Device::new_cuda(0)?;
    if let Some(checkpoint) = decode {
        let nearest = config.objective.decodes_nearest();
        for speed in decode_speed(
            &data_root,
            &checkpoint,
            config.retain_rates,
            config.fold_scores,
            config.reflect_floor,
            nearest,
            config.reply_tokens,
            config.seed,
            &device,
        )? {
            println!("{}", serde_json::to_string(&speed)?);
        }
        return Ok(());
    }
    if grad_norms {
        for norm in gradient_norms(&data_root, &config, &device)? {
            println!("{}", serde_json::to_string(&norm)?);
        }
        return Ok(());
    }
    if let Some(checkpoint) = reach {
        let (reach, bins) = context_reach(
            &data_root,
            &checkpoint,
            config.retain_rates,
            config.fold_scores,
            config.reflect_floor,
            config.seed,
            &device,
        )?;
        for point in reach {
            println!("{}", serde_json::to_string(&point)?);
        }
        for layer in bins {
            println!("{}", serde_json::to_string(&layer)?);
        }
        return Ok(());
    }
    if let Some(run) = inspect {
        let (nearest, farthest) = root_separation()?;
        println!(
            "{}",
            serde_json::json!({"address_root_separation": {"nearest": nearest, "farthest": farthest}})
        );
        for reading in inspect_run(
            &data_root,
            &run,
            config.retain_rates,
            config.fold_scores,
            config.reflect_floor,
            config.seed,
            &device,
        )? {
            println!("{}", serde_json::to_string(&reading)?);
        }
        return Ok(());
    }
    let out = out.ok_or_else(|| FloatTrainError::Invalid("--out is required".to_owned()))?;
    run_long(&data_root, &out, &config, &device)
}
