use candle_core::Device;
use std::sync::Arc;
use uor_r4_integer::stack::IntegerStackModel;
use uor_r4_lut::format::StackArtifact;
use uor_r4_training::d4_codecs::{
    codec_by_name, D4Grouped4BitAdapter, E8MatchedBitMapCodec, HeadCompensatedMapCodec,
    RecurrenceOutMinMseMapCodec,
};
use uor_r4_training::geometric_stack::{
    D11Interim, MapCodec, ReadScore, StackAdamW, StackArch, StackConfig, StackModel,
};
use uor_r4_training::stack_export::{
    check_export_representation, export_stack, stack_grid_reference,
};
use uor_r4_training::Result;

#[test]
fn native_grouped_4bit_codec_matches_d11_interim_bit_for_bit() -> Result<()> {
    let values: Vec<f32> = (0..64 * 32)
        .map(|i| (((i as f32) * 0.17) % 7.0 - 3.5) * 0.1)
        .collect();
    let d11 = D11Interim;
    let native_rtn = D4Grouped4BitAdapter::rtn();

    let rt_d11 = d11.round_trip(&values, 64, 32)?;
    let rt_native = native_rtn.round_trip(&values, 64, 32)?;
    assert_eq!(
        rt_d11, rt_native,
        "Native Grouped4BitCodec via D4Grouped4BitAdapter must match D11Interim round-trip bit for bit"
    );

    let dyn_codec: Arc<dyn MapCodec> = Arc::new(native_rtn);
    assert_eq!(dyn_codec.name(), "native-d11-grouped-4bit-g32-rtn");
    let rt_dyn = dyn_codec.round_trip(&values, 64, 32)?;
    assert_eq!(rt_dyn, rt_d11);

    // Test with MinimumMseScale mode as well
    let native_opt = D4Grouped4BitAdapter::min_mse();
    let dyn_opt: Arc<dyn MapCodec> = Arc::new(native_opt);
    assert_eq!(dyn_opt.name(), "native-d11-grouped-4bit-g32-min-mse");
    let rt_opt = dyn_opt.round_trip(&values, 64, 32)?;
    assert_eq!(rt_opt.len(), 64 * 32);

    Ok(())
}

#[test]
fn head_compensated_codec_produces_finite_values() -> Result<()> {
    let values: Vec<f32> = (0..64 * 32)
        .map(|i| (((i as f32) * 0.31) % 5.0 - 2.5) * 0.1)
        .collect();
    let codec_all = HeadCompensatedMapCodec::all_maps();
    assert_eq!(codec_all.name(), "native-d4-head-compensated-all-maps");
    let rt_all = codec_all.round_trip(&values, 64, 32)?;
    assert_eq!(rt_all.len(), 64 * 32);
    assert!(rt_all.iter().all(|v| v.is_finite()));

    // Test explicit target shape with head_only_for:
    let codec_targeted = HeadCompensatedMapCodec::head_only_for(64, 32);
    assert_eq!(
        codec_targeted.name(),
        "native-d4-head-compensated-head-only-64x32"
    );
    let rt_targeted_match = codec_targeted.round_trip(&values, 64, 32)?;
    let mat = uor_r4_integer::codec::quantize_matrix_compensated(&values, 64, 32)?;
    let rt_comp = uor_r4_integer::codec::Grouped4BitCodec::default().dequantize(&mat)?;
    assert_eq!(rt_targeted_match, rt_comp);

    // Non-matching shape falls back to D11Interim:
    let rt_d11 = D11Interim.round_trip(&values, 32, 64)?;
    let rt_targeted_nonmatch = codec_targeted.round_trip(&values, 32, 64)?;
    assert_eq!(rt_targeted_nonmatch, rt_d11);

    Ok(())
}

#[test]
fn qat_adapters_integrate_with_stack_model_and_export_contract() -> Result<()> {
    let config = StackConfig {
        arch: StackArch::Geometric,
        vocab_size: 96,
        width: 64,
        heads: 2,
        mlp_hidden: 40,
        context: 16,
        pattern: "rar".into(),
        read: ReadScore::Lorentz,
        rotation: true,
        seed: 29,
        memory: None,
    };
    let device = Device::Cpu;
    let mut model = StackModel::new(config, &device)?;

    // 1. Set served representation with D4Grouped4BitAdapter
    model.set_served_representation(Some(Arc::new(D4Grouped4BitAdapter::rtn())))?;
    assert_eq!(
        model.served_codec().unwrap().name(),
        "native-d11-grouped-4bit-g32-rtn"
    );

    let tmp_root = std::env::temp_dir().join(format!("d4-qat-test-{}", std::process::id()));
    let save_dir = tmp_root.join("saved_model");
    model.save(&save_dir)?;

    let record = StackModel::saved_served_representation(&save_dir)?;
    assert_eq!(
        record.as_ref().unwrap().codec,
        "native-d11-grouped-4bit-g32-rtn"
    );

    // Verify export representation check passes for interim-compatible QAT model
    check_export_representation(record.as_ref(), false)?;
    // Calibrated export should be rejected for a QAT model
    assert!(check_export_representation(record.as_ref(), true).is_err());

    // 2. Set served representation with HeadCompensatedMapCodec
    model.set_served_representation(Some(Arc::new(HeadCompensatedMapCodec::head_only_for(
        96, 64,
    ))))?;
    assert_eq!(
        model.served_codec().unwrap().name(),
        "native-d4-head-compensated-head-only-96x64"
    );

    let save_dir_hc = tmp_root.join("saved_model_hc");
    model.save(&save_dir_hc)?;

    let record_hc = StackModel::saved_served_representation(&save_dir_hc)?;
    assert_eq!(
        record_hc.as_ref().unwrap().codec,
        "native-d4-head-compensated-head-only-96x64"
    );

    // Verify export representation check passes for HeadCompensated QAT model
    check_export_representation(record_hc.as_ref(), false)?;
    // Calibrated export should be rejected for a HeadCompensated QAT model
    assert!(check_export_representation(record_hc.as_ref(), true).is_err());

    // Clean up
    let _ = std::fs::remove_dir_all(&tmp_root);
    Ok(())
}

#[test]
fn qat_head_compensated_end_to_end_export_and_exactness() -> Result<()> {
    let config = StackConfig {
        arch: StackArch::Geometric,
        vocab_size: 96,
        width: 32,
        heads: 1,
        mlp_hidden: 32,
        context: 8,
        pattern: "r".into(),
        read: ReadScore::Dot,
        rotation: false,
        seed: 77,
        memory: None,
    };
    let device = Device::Cpu;
    let mut model = StackModel::new(config, &device)?;

    // 1. Enable QAT with HeadCompensatedMapCodec
    let codec = Arc::new(HeadCompensatedMapCodec::head_only_for(96, 32));
    model.set_served_representation(Some(codec))?;
    assert_eq!(
        model.served_codec().unwrap().name(),
        "native-d4-head-compensated-head-only-96x32"
    );

    // 2. Compute loss on dummy inputs
    let ids: Vec<u32> = vec![2, 7, 15, 25];
    let targets: Vec<u32> = vec![7, 15, 25, 30];
    let loss = model.loss(&ids, &targets, 1, 4)?;
    let initial_loss = loss.to_scalar::<f32>()?;
    assert!(
        initial_loss.is_finite(),
        "initial head-compensated QAT loss must be finite"
    );

    // 3. Backward pass: gradients flow through StraightThrough op to master variables
    let grads = loss.backward()?;
    let embed_var = model.variables().get("embedding.weight").unwrap();
    let embed_grad = grads.get(embed_var.as_tensor());
    assert!(
        embed_grad.is_some(),
        "gradient must reach embedding.weight under head-compensated QAT"
    );
    assert!(
        embed_grad.unwrap().sqr()?.sum_all()?.to_scalar::<f32>()? > 0.0,
        "gradient on embedding.weight must be non-zero"
    );

    // 4. Optimizer update modifies master parameters
    let mut optimizer = StackAdamW::new(&model, 0.01, 1.0)?;
    let norm = optimizer.update(&model, &grads, 0.05)?;
    assert!(norm > 0.0, "gradient norm must be positive");

    // Served float forward logits
    let served_logits = model.forward(&ids, 1, ids.len())?.to_vec2::<f32>()?;

    // 5. Save model and verify metadata
    let tmp_root = std::env::temp_dir().join(format!("d4-hc-qat-e2e-{}", std::process::id()));
    let save_dir = tmp_root.join("saved_model");
    model.save(&save_dir)?;

    let record = StackModel::saved_served_representation(&save_dir)?;
    assert_eq!(
        record.as_ref().unwrap().codec,
        "native-d4-head-compensated-head-only-96x32"
    );

    // 6. Export stack artifact without calibration (using QAT weights)
    check_export_representation(record.as_ref(), false)?;
    let (lut_bytes, summary) = export_stack(&model, "head-compensated-qat-test".into(), None)?;
    assert!(!lut_bytes.is_empty());
    assert_eq!(
        summary["method"]["quantizer"],
        "native-d4-head-compensated-head-only-96x32"
    );

    // 7. Verify S1.1 invariant: served float forward matches exported grid reference bit for bit
    let artifact = StackArtifact::parse(lut_bytes.clone())
        .map_err(|e| uor_r4_training::TrainingError::Invalid(e.to_string()))?;
    let grid_ref = stack_grid_reference(&model, &artifact)?;
    let ref_logits = grid_ref.logits(&ids, 1, ids.len())?.to_vec2::<f32>()?;
    assert_eq!(
        served_logits, ref_logits,
        "Served float forward logits must match exported grid reference exactly"
    );

    // 8. Load into IntegerStackModel and verify native integer serving execution and logit parity
    let lut_path = tmp_root.join("model.lut");
    std::fs::write(&lut_path, &lut_bytes)?;
    let integer_model = IntegerStackModel::load(&lut_path)
        .map_err(|e| uor_r4_training::TrainingError::Invalid(e.to_string()))?;
    assert_eq!(integer_model.shape().vocab, 96);
    assert_eq!(integer_model.shape().width, 32);

    let mut session = integer_model.session();
    for (t, &id) in ids.iter().enumerate() {
        let int_logits = session
            .step(id)
            .map_err(|e| uor_r4_training::TrainingError::Invalid(e.to_string()))?;
        assert_eq!(
            int_logits.len(),
            96,
            "logits length must match vocabulary size"
        );
        let int_top = uor_r4_integer::stack::stack_argmax(int_logits);
        let float_top = ref_logits[t]
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;
        assert_eq!(
            int_top, float_top,
            "position {t}: integer top-1 argmax must equal served float argmax"
        );

        let int_f64: Vec<f64> = int_logits.iter().map(|&v| f64::from(v) / 65536.0).collect();
        let ref_f64: Vec<f64> = ref_logits[t].iter().map(|&v| f64::from(v)).collect();
        let max_gap = int_f64
            .iter()
            .zip(&ref_f64)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f64, f64::max);
        assert!(
            max_gap < 0.1,
            "position {t}: gap between integer and served float logits must be bounded, got {max_gap}"
        );
    }

    // 9. Verify reload and codec recovery via codec_by_name
    let mut reloaded = StackModel::load(&save_dir, &device)?;
    assert!(reloaded.served_codec().is_none());
    let saved_rec = StackModel::saved_served_representation(&save_dir)?;
    let recovered_codec = codec_by_name(&saved_rec.unwrap().codec)?;
    reloaded.set_served_representation(Some(recovered_codec))?;
    assert_eq!(
        reloaded.served_codec().unwrap().name(),
        "native-d4-head-compensated-head-only-96x32"
    );

    // Clean up
    let _ = std::fs::remove_dir_all(&tmp_root);
    Ok(())
}

#[test]
fn qat_min_mse_end_to_end_export_and_exactness() -> Result<()> {
    let config = StackConfig {
        arch: StackArch::Geometric,
        vocab_size: 64,
        width: 32,
        heads: 1,
        mlp_hidden: 32,
        context: 8,
        pattern: "r".into(),
        read: ReadScore::Dot,
        rotation: false,
        seed: 42,
        memory: None,
    };
    let device = Device::Cpu;
    let mut model = StackModel::new(config, &device)?;

    // 1. Enable QAT with D4Grouped4BitAdapter min_mse
    let adapter = Arc::new(D4Grouped4BitAdapter::min_mse());
    model.set_served_representation(Some(adapter))?;
    assert_eq!(
        model.served_codec().unwrap().name(),
        "native-d11-grouped-4bit-g32-min-mse"
    );

    // 2. Compute loss on dummy inputs
    let ids: Vec<u32> = vec![1, 5, 12, 18];
    let targets: Vec<u32> = vec![5, 12, 18, 22];
    let loss = model.loss(&ids, &targets, 1, 4)?;
    let initial_loss = loss.to_scalar::<f32>()?;
    assert!(initial_loss.is_finite(), "initial QAT loss must be finite");

    // 3. Backward pass: gradients flow through StraightThrough op to master variables
    let grads = loss.backward()?;
    let embed_var = model.variables().get("embedding.weight").unwrap();
    let embed_grad = grads.get(embed_var.as_tensor());
    assert!(embed_grad.is_some(), "gradient must reach embedding.weight");
    assert!(
        embed_grad.unwrap().sqr()?.sum_all()?.to_scalar::<f32>()? > 0.0,
        "gradient on embedding.weight must be non-zero"
    );

    // 4. Optimizer update modifies master parameters
    let mut optimizer = StackAdamW::new(&model, 0.01, 1.0)?;
    let norm = optimizer.update(&model, &grads, 0.05)?;
    assert!(norm > 0.0, "gradient norm must be positive");

    // Served float forward logits
    let served_logits = model.forward(&ids, 1, ids.len())?.to_vec2::<f32>()?;

    // 5. Save model and verify metadata
    let tmp_root = std::env::temp_dir().join(format!("d4-min-mse-qat-e2e-{}", std::process::id()));
    let save_dir = tmp_root.join("saved_model");
    model.save(&save_dir)?;

    let record = StackModel::saved_served_representation(&save_dir)?;
    assert_eq!(
        record.as_ref().unwrap().codec,
        "native-d11-grouped-4bit-g32-min-mse"
    );

    // 6. Export stack artifact without calibration (using QAT weights)
    check_export_representation(record.as_ref(), false)?;
    let (lut_bytes, summary) = export_stack(&model, "qat-min-mse-test".into(), None)?;
    assert!(!lut_bytes.is_empty());
    assert_eq!(
        summary["method"]["quantizer"],
        "native-d11-grouped-4bit-g32-min-mse"
    );

    // 7. Verify S1.1 invariant: served float forward matches exported grid reference bit for bit
    let artifact = StackArtifact::parse(lut_bytes.clone())
        .map_err(|e| uor_r4_training::TrainingError::Invalid(e.to_string()))?;
    let grid_ref = stack_grid_reference(&model, &artifact)?;
    let ref_logits = grid_ref.logits(&ids, 1, ids.len())?.to_vec2::<f32>()?;
    assert_eq!(
        served_logits, ref_logits,
        "Served float forward logits must match exported grid reference exactly"
    );

    // 8. Load into IntegerStackModel and verify native integer serving execution and logit parity
    let lut_path = tmp_root.join("model.lut");
    std::fs::write(&lut_path, &lut_bytes)?;
    let integer_model = IntegerStackModel::load(&lut_path)
        .map_err(|e| uor_r4_training::TrainingError::Invalid(e.to_string()))?;
    assert_eq!(integer_model.shape().vocab, 64);
    assert_eq!(integer_model.shape().width, 32);

    let mut session = integer_model.session();
    for (t, &id) in ids.iter().enumerate() {
        let int_logits = session
            .step(id)
            .map_err(|e| uor_r4_training::TrainingError::Invalid(e.to_string()))?;
        assert_eq!(
            int_logits.len(),
            64,
            "logits length must match vocabulary size"
        );
        let int_top = uor_r4_integer::stack::stack_argmax(int_logits);
        let float_top = ref_logits[t]
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;
        assert_eq!(
            int_top, float_top,
            "position {t}: integer top-1 argmax must equal served float argmax"
        );

        let int_f64: Vec<f64> = int_logits.iter().map(|&v| f64::from(v) / 65536.0).collect();
        let ref_f64: Vec<f64> = ref_logits[t].iter().map(|&v| f64::from(v)).collect();
        let max_gap = int_f64
            .iter()
            .zip(&ref_f64)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f64, f64::max);
        assert!(
            max_gap < 0.1,
            "position {t}: gap between integer and served float logits must be bounded, got {max_gap}"
        );
    }

    // Clean up
    let _ = std::fs::remove_dir_all(&tmp_root);
    Ok(())
}

#[test]
fn head_quantization_mse_comparison() -> Result<()> {
    let r = 256;
    let c = 64;
    let head_weights: Vec<f32> = (0..r * c)
        .map(|i| (((i as f32) * 0.037) % 5.0 - 2.5) * 0.05)
        .collect();

    let codec_rtn = D4Grouped4BitAdapter::rtn();
    let codec_mse = D4Grouped4BitAdapter::min_mse();
    let codec_comp = HeadCompensatedMapCodec::head_only_for(r, c);

    let deq_rtn = codec_rtn.round_trip(&head_weights, r, c)?;
    let deq_mse = codec_mse.round_trip(&head_weights, r, c)?;
    let deq_comp = codec_comp.round_trip(&head_weights, r, c)?;

    let mse_rtn: f64 = head_weights
        .iter()
        .zip(&deq_rtn)
        .map(|(&w, &q)| (f64::from(w) - f64::from(q)).powi(2))
        .sum::<f64>()
        / (r * c) as f64;
    let mse_min: f64 = head_weights
        .iter()
        .zip(&deq_mse)
        .map(|(&w, &q)| (f64::from(w) - f64::from(q)).powi(2))
        .sum::<f64>()
        / (r * c) as f64;
    let mse_comp: f64 = head_weights
        .iter()
        .zip(&deq_comp)
        .map(|(&w, &q)| (f64::from(w) - f64::from(q)).powi(2))
        .sum::<f64>()
        / (r * c) as f64;

    assert!(mse_rtn > 0.0 && mse_rtn.is_finite());
    assert!(mse_min > 0.0 && mse_min.is_finite());
    assert!(mse_comp > 0.0 && mse_comp.is_finite());
    // Minimum-MSE scale search is guaranteed to achieve <= MSE than RTN:
    assert!(
        mse_min <= mse_rtn + 1e-9,
        "Minimum-MSE scale search must achieve <= MSE than RTN: min_mse={mse_min:.8}, rtn={mse_rtn:.8}"
    );

    Ok(())
}

#[test]
fn test_e8_matched_bit_map_codec_qat_and_export_refusal() -> Result<()> {
    let codec = E8MatchedBitMapCodec::default();
    assert_eq!(codec.name(), "native-d4-e8-matched-bit");

    // Verify round_trip on test weights
    let values: Vec<f32> = (0..64 * 32)
        .map(|i| (((i as f32) * 0.13) % 4.0 - 2.0) * 0.05)
        .collect();
    let deq = codec.round_trip(&values, 64, 32)?;
    assert_eq!(deq.len(), 64 * 32);
    assert!(deq.iter().all(|v| v.is_finite()));

    // Instantiate StackModel with E8MatchedBitMapCodec
    let config = StackConfig {
        arch: StackArch::Geometric,
        vocab_size: 64,
        width: 32,
        heads: 1,
        mlp_hidden: 32,
        context: 8,
        pattern: "r".into(),
        read: ReadScore::Dot,
        rotation: false,
        seed: 88,
        memory: None,
    };
    let device = Device::Cpu;
    let mut model = StackModel::new(config, &device)?;
    model.set_served_representation(Some(Arc::new(codec)))?;
    assert_eq!(
        model.served_codec().unwrap().name(),
        "native-d4-e8-matched-bit"
    );

    // Compute loss and backward gradients through STE
    let ids: Vec<u32> = vec![3, 11, 22, 33];
    let targets: Vec<u32> = vec![11, 22, 33, 44];
    let loss = model.loss(&ids, &targets, 1, 4)?;
    let initial_loss = loss.to_scalar::<f32>()?;
    assert!(initial_loss.is_finite());

    let grads = loss.backward()?;
    let embed_var = model.variables().get("embedding.weight").unwrap();
    let embed_grad = grads.get(embed_var.as_tensor());
    assert!(
        embed_grad.is_some(),
        "gradient must reach embedding.weight under E8 matched-bit QAT"
    );
    assert!(
        embed_grad.unwrap().sqr()?.sum_all()?.to_scalar::<f32>()? > 0.0,
        "gradient on embedding.weight must be non-zero"
    );

    // Update master parameters via optimizer
    let mut optimizer = StackAdamW::new(&model, 0.01, 1.0)?;
    let norm = optimizer.update(&model, &grads, 0.05)?;
    assert!(norm > 0.0);

    // Save and check metadata
    let tmp_root = std::env::temp_dir().join(format!("d4-e8-qat-test-{}", std::process::id()));
    let save_dir = tmp_root.join("saved_model_e8");
    model.save(&save_dir)?;

    let record = StackModel::saved_served_representation(&save_dir)?;
    assert_eq!(record.as_ref().unwrap().codec, "native-d4-e8-matched-bit");

    // Export stack artifact must be REFUSED: D11 container cannot hold E8 codes!
    let export_err = check_export_representation(record.as_ref(), false).unwrap_err();
    assert!(
        export_err.to_string().contains("native-d4-e8-matched-bit")
            && export_err.to_string().contains("export does not write"),
        "E8 export must be refused: {export_err}"
    );

    // Clean up
    let _ = std::fs::remove_dir_all(&tmp_root);
    Ok(())
}

fn fold_columns(values: &mut [f32], cols: usize, gain: &[f32]) {
    for row in values.chunks_exact_mut(cols) {
        for (v, &g) in row.iter_mut().zip(gain) {
            *v *= g;
        }
    }
}

fn pad(values: &[f32], rows: usize, cols: usize, to_rows: usize, to_cols: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; to_rows * to_cols];
    for r in 0..rows {
        out[r * to_cols..r * to_cols + cols].copy_from_slice(&values[r * cols..(r + 1) * cols]);
    }
    out
}

#[test]
fn test_exported_artifact_dequantized_weights_equal_served_view_element_by_element() -> Result<()> {
    let codecs: Vec<Arc<dyn MapCodec>> = vec![
        Arc::new(D4Grouped4BitAdapter::min_mse()),
        Arc::new(HeadCompensatedMapCodec::all_maps()),
        Arc::new(HeadCompensatedMapCodec::head_only_for(96, 32)),
    ];

    for codec in codecs {
        let codec_name = codec.name().to_string();
        let config = StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 96,
            width: 32,
            heads: 2,
            mlp_hidden: 40,
            context: 8,
            pattern: "rar".into(),
            read: ReadScore::Lorentz,
            rotation: true,
            seed: 42,
            memory: None,
        };
        let device = Device::Cpu;
        let mut model = StackModel::new(config.clone(), &device)?;
        model.set_served_representation(Some(codec.clone()))?;

        let (lut_bytes, _summary) =
            export_stack(&model, serde_json::json!({"test": "parity"}), None)?;
        let artifact = StackArtifact::parse(lut_bytes)
            .map_err(|e| uor_r4_training::TrainingError::Invalid(e.to_string()))?;

        // Check every single matrix in the exported artifact against the served view element by element
        let vars = model.variables();
        for spec in &artifact.header.matrices {
            let dequantized = uor_r4_training::lut_export::dequantize_matrix(
                spec.rows,
                spec.cols,
                spec.exp_base,
                artifact.section(spec.nibbles),
                artifact.section(spec.scales),
            )?;

            let var_name = |name: &str| -> Result<Vec<f32>> {
                Ok(vars
                    .get(name)
                    .ok_or_else(|| uor_r4_training::TrainingError::Invalid(name.to_string()))?
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?)
            };

            let expected_served = match spec.name.as_str() {
                "embed" => {
                    let src = var_name("embedding.weight")?;
                    codec.round_trip(&src, spec.rows, spec.cols)?
                }
                "head" => {
                    let mut src = var_name("embedding.weight")?;
                    let gain = var_name("final_norm.weight")?;
                    fold_columns(&mut src, spec.cols, &gain);
                    codec.round_trip(&src, spec.rows, spec.cols)?
                }
                name if name.starts_with("l0.")
                    || name.starts_with("l1.")
                    || name.starts_with("l2.") =>
                {
                    let layer = name[1..2].parse::<usize>().unwrap();
                    let part = &name[3..];
                    match part {
                        "rec_in" => {
                            let mut src = var_name(&format!("layers.{layer:02}.rec.in.weight"))?;
                            let gain = var_name(&format!("layers.{layer:02}.rec_norm.weight"))?;
                            fold_columns(&mut src, spec.cols, &gain);
                            codec.round_trip(&src, spec.rows, spec.cols)?
                        }
                        "rec_gate" => {
                            let mut src = var_name(&format!("layers.{layer:02}.rec.gate.weight"))?;
                            let gain = var_name(&format!("layers.{layer:02}.rec_norm.weight"))?;
                            fold_columns(&mut src, spec.cols, &gain);
                            codec.round_trip(&src, spec.rows, spec.cols)?
                        }
                        "rec_out" => {
                            let src = var_name(&format!("layers.{layer:02}.rec.out.weight"))?;
                            codec.round_trip(&src, spec.rows, spec.cols)?
                        }
                        "query" | "key" | "value" | "null" => {
                            let mut src =
                                var_name(&format!("layers.{layer:02}.read.{part}.weight"))?;
                            let gain = var_name(&format!("layers.{layer:02}.read_norm.weight"))?;
                            fold_columns(&mut src, spec.cols, &gain);
                            codec.round_trip(&src, spec.rows, spec.cols)?
                        }
                        "out" => {
                            let src = var_name(&format!("layers.{layer:02}.read.out.weight"))?;
                            codec.round_trip(&src, spec.rows, spec.cols)?
                        }
                        "gate" | "up" => {
                            let mut src =
                                var_name(&format!("layers.{layer:02}.mlp.{part}.weight"))?;
                            let gain = var_name(&format!("layers.{layer:02}.mlp_norm.weight"))?;
                            fold_columns(&mut src, config.width, &gain);
                            let padded =
                                pad(&src, config.mlp_hidden, config.width, spec.rows, spec.cols);
                            codec.round_trip(&padded, spec.rows, spec.cols)?
                        }
                        "down" => {
                            let src = var_name(&format!("layers.{layer:02}.mlp.down.weight"))?;
                            let padded =
                                pad(&src, config.width, config.mlp_hidden, spec.rows, spec.cols);
                            codec.round_trip(&padded, spec.rows, spec.cols)?
                        }
                        other => panic!("unknown matrix in artifact: {other}"),
                    }
                }
                other => panic!("unknown matrix name: {other}"),
            };

            assert_eq!(
                dequantized.len(),
                expected_served.len(),
                "codec {codec_name}, matrix {}: length mismatch",
                spec.name
            );
            assert_eq!(
                dequantized,
                expected_served,
                "codec {codec_name}, matrix {}: exported artifact's dequantized weights must equal served view element by element bit for bit",
                spec.name
            );
        }

        // Also check forward logits bit-for-bit against stack_grid_reference
        let ids: Vec<u32> = vec![3, 15, 29, 44];
        let served_logits = model.forward(&ids, 1, ids.len())?.to_vec2::<f32>()?;
        let grid_ref = stack_grid_reference(&model, &artifact)?;
        let ref_logits = grid_ref.logits(&ids, 1, ids.len())?.to_vec2::<f32>()?;
        assert_eq!(
            served_logits,
            ref_logits,
            "codec {codec_name}: served forward logits must equal exported grid reference logits bit for bit"
        );
    }

    Ok(())
}

#[test]
fn qat_rec_out_min_mse_end_to_end_export_and_exactness() -> Result<()> {
    let config = StackConfig {
        arch: StackArch::Geometric,
        vocab_size: 96,
        width: 32,
        heads: 1,
        mlp_hidden: 40,
        context: 8,
        pattern: "r".into(),
        read: ReadScore::Dot,
        rotation: false,
        seed: 88,
        memory: None,
    };
    let device = Device::Cpu;
    let mut model = StackModel::new(config, &device)?;

    // 1. Enable QAT with RecurrenceOutMinMseMapCodec
    let codec = Arc::new(RecurrenceOutMinMseMapCodec::for_shape(32, 32));
    model.set_served_representation(Some(codec))?;
    assert_eq!(
        model.served_codec().unwrap().name(),
        "native-d4-rec-out-min-mse-32x32"
    );

    // 2. Compute loss on dummy inputs
    let ids: Vec<u32> = vec![3, 9, 17, 27];
    let targets: Vec<u32> = vec![9, 17, 27, 33];
    let loss = model.loss(&ids, &targets, 1, 4)?;
    let initial_loss = loss.to_scalar::<f32>()?;
    assert!(
        initial_loss.is_finite(),
        "initial rec_out Min-MSE QAT loss must be finite"
    );

    // 3. Backward pass: gradients flow through StraightThrough op to master variables
    let grads = loss.backward()?;
    let rec_out_var = model.variables().get("layers.00.rec.out.weight").unwrap();
    let rec_out_grad = grads.get(rec_out_var.as_tensor());
    assert!(
        rec_out_grad.is_some(),
        "gradient must reach layers.00.rec.out.weight under rec_out Min-MSE QAT"
    );
    assert!(
        rec_out_grad.unwrap().sqr()?.sum_all()?.to_scalar::<f32>()? > 0.0,
        "gradient on layers.00.rec.out.weight must be non-zero"
    );

    // 4. Optimizer update modifies master parameters
    let mut optimizer = StackAdamW::new(&model, 0.01, 1.0)?;
    let norm = optimizer.update(&model, &grads, 0.05)?;
    assert!(norm > 0.0, "gradient norm must be positive");

    // Served float forward logits
    let served_logits = model.forward(&ids, 1, ids.len())?.to_vec2::<f32>()?;

    // 5. Save model and verify metadata
    let tmp_root = std::env::temp_dir().join(format!("d4-rec-out-qat-e2e-{}", std::process::id()));
    let save_dir = tmp_root.join("saved_model");
    model.save(&save_dir)?;

    let record = StackModel::saved_served_representation(&save_dir)?;
    assert_eq!(
        record.as_ref().unwrap().codec,
        "native-d4-rec-out-min-mse-32x32"
    );

    // 6. Export stack artifact without calibration (using QAT weights)
    check_export_representation(record.as_ref(), false)?;
    assert!(check_export_representation(record.as_ref(), true).is_err());
    let (lut_bytes, summary) = export_stack(&model, "rec-out-min-mse-qat-test".into(), None)?;
    assert!(!lut_bytes.is_empty());
    assert_eq!(
        summary["method"]["quantizer"],
        "native-d4-rec-out-min-mse-32x32"
    );

    // 7. Verify S1.1 invariant: served float forward matches exported grid reference bit for bit
    let artifact = StackArtifact::parse(lut_bytes.clone())
        .map_err(|e| uor_r4_training::TrainingError::Invalid(e.to_string()))?;

    // Element-by-element exactness across all matrices in artifact
    let vars = model.variables();
    let codec_ref = RecurrenceOutMinMseMapCodec::for_shape(32, 32);
    for spec in &artifact.header.matrices {
        let dequantized = uor_r4_training::lut_export::dequantize_matrix(
            spec.rows,
            spec.cols,
            spec.exp_base,
            artifact.section(spec.nibbles),
            artifact.section(spec.scales),
        )?;
        let expected_served = match spec.name.as_str() {
            "l0.rec_out" => {
                let src = vars["layers.00.rec.out.weight"]
                    .as_tensor()
                    .flatten_all()?
                    .to_vec1::<f32>()?;
                codec_ref.round_trip(&src, spec.rows, spec.cols)?
            }
            _ => {
                let d11 = D11Interim;
                let src = match spec.name.as_str() {
                    "embed" => vars["embedding.weight"]
                        .as_tensor()
                        .flatten_all()?
                        .to_vec1::<f32>()?,
                    "head" => {
                        let mut w = vars["embedding.weight"]
                            .as_tensor()
                            .flatten_all()?
                            .to_vec1::<f32>()?;
                        let gain = vars["final_norm.weight"]
                            .as_tensor()
                            .flatten_all()?
                            .to_vec1::<f32>()?;
                        fold_columns(&mut w, spec.cols, &gain);
                        w
                    }
                    "l0.rec_in" => {
                        let mut w = vars["layers.00.rec.in.weight"]
                            .as_tensor()
                            .flatten_all()?
                            .to_vec1::<f32>()?;
                        let gain = vars["layers.00.rec_norm.weight"]
                            .as_tensor()
                            .flatten_all()?
                            .to_vec1::<f32>()?;
                        fold_columns(&mut w, spec.cols, &gain);
                        w
                    }
                    "l0.rec_gate" => {
                        let mut w = vars["layers.00.rec.gate.weight"]
                            .as_tensor()
                            .flatten_all()?
                            .to_vec1::<f32>()?;
                        let gain = vars["layers.00.rec_norm.weight"]
                            .as_tensor()
                            .flatten_all()?
                            .to_vec1::<f32>()?;
                        fold_columns(&mut w, spec.cols, &gain);
                        w
                    }
                    "l0.gate" | "l0.up" => {
                        let part = if spec.name == "l0.gate" { "gate" } else { "up" };
                        let mut w = vars[&format!("layers.00.mlp.{part}.weight")]
                            .as_tensor()
                            .flatten_all()?
                            .to_vec1::<f32>()?;
                        let gain = vars["layers.00.mlp_norm.weight"]
                            .as_tensor()
                            .flatten_all()?
                            .to_vec1::<f32>()?;
                        fold_columns(&mut w, 32, &gain);
                        pad(&w, 40, 32, spec.rows, spec.cols)
                    }
                    "l0.down" => {
                        let w = vars["layers.00.mlp.down.weight"]
                            .as_tensor()
                            .flatten_all()?
                            .to_vec1::<f32>()?;
                        pad(&w, 32, 40, spec.rows, spec.cols)
                    }
                    other => panic!("unknown matrix: {other}"),
                };
                d11.round_trip(&src, spec.rows, spec.cols)?
            }
        };
        assert_eq!(
            dequantized, expected_served,
            "matrix {}: exported artifact dequantized weights must match served view element by element",
            spec.name
        );
    }

    let grid_ref = stack_grid_reference(&model, &artifact)?;
    let ref_logits = grid_ref.logits(&ids, 1, ids.len())?.to_vec2::<f32>()?;
    assert_eq!(
        served_logits, ref_logits,
        "Served float forward logits must match exported grid reference exactly"
    );

    // 8. Load into IntegerStackModel and verify native integer serving execution and logit parity
    let lut_path = tmp_root.join("model.lut");
    std::fs::write(&lut_path, &lut_bytes)?;
    let integer_model = IntegerStackModel::load(&lut_path)
        .map_err(|e| uor_r4_training::TrainingError::Invalid(e.to_string()))?;
    assert_eq!(integer_model.shape().vocab, 96);
    assert_eq!(integer_model.shape().width, 32);

    let mut session = integer_model.session();
    for (t, &id) in ids.iter().enumerate() {
        let int_logits = session
            .step(id)
            .map_err(|e| uor_r4_training::TrainingError::Invalid(e.to_string()))?;
        assert_eq!(
            int_logits.len(),
            96,
            "logits length must match vocabulary size"
        );
        let int_top = uor_r4_integer::stack::stack_argmax(int_logits);
        let float_top = ref_logits[t]
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap()
            .0;
        assert_eq!(
            int_top, float_top,
            "position {t}: integer top-1 argmax must equal served float argmax"
        );

        let int_f64: Vec<f64> = int_logits.iter().map(|&v| f64::from(v) / 65536.0).collect();
        let ref_f64: Vec<f64> = ref_logits[t].iter().map(|&v| f64::from(v)).collect();
        let max_gap = int_f64
            .iter()
            .zip(&ref_f64)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f64, f64::max);
        assert!(
            max_gap < 0.1,
            "position {t}: gap between integer and served float logits must be bounded, got {max_gap}"
        );
    }

    // 9. Verify reload and codec recovery via codec_by_name
    let mut reloaded = StackModel::load(&save_dir, &device)?;
    assert!(reloaded.served_codec().is_none());
    let saved_rec = StackModel::saved_served_representation(&save_dir)?;
    let recovered_codec = codec_by_name(&saved_rec.unwrap().codec)?;
    reloaded.set_served_representation(Some(recovered_codec))?;
    assert_eq!(
        reloaded.served_codec().unwrap().name(),
        "native-d4-rec-out-min-mse-32x32"
    );

    // Clean up
    let _ = std::fs::remove_dir_all(&tmp_root);
    Ok(())
}

#[test]
fn test_codec_by_name_refuses_bare_shape_dependent_names_and_accepts_qualified() -> Result<()> {
    match codec_by_name("native-d4-head-compensated-head-only") {
        Err(err) => assert!(
            err.to_string().contains("requires explicit target shape"),
            "unexpected error message: {err}"
        ),
        Ok(_) => panic!("expected refusal for bare head-compensated name"),
    }

    match codec_by_name("native-d4-rec-out-min-mse") {
        Err(err) => assert!(
            err.to_string().contains("requires explicit target shape"),
            "unexpected error message: {err}"
        ),
        Ok(_) => panic!("expected refusal for bare rec-out-min-mse name"),
    }

    match codec_by_name("native-d4-s2-rec-out-min-mse") {
        Err(err) => assert!(
            err.to_string().contains("requires explicit target shape"),
            "unexpected error message: {err}"
        ),
        Ok(_) => panic!("expected refusal for bare s2-rec-out-min-mse name"),
    }

    // Qualified names parse successfully:
    let hc = codec_by_name("native-d4-head-compensated-head-only-4096x288")?;
    assert_eq!(hc.name(), "native-d4-head-compensated-head-only-4096x288");

    let rec = codec_by_name("native-d4-rec-out-min-mse-288x288")?;
    assert_eq!(rec.name(), "native-d4-rec-out-min-mse-288x288");

    let s2_rec = codec_by_name("native-d4-s2-rec-out-min-mse-288x288")?;
    assert_eq!(s2_rec.name(), "native-d4-rec-out-min-mse-288x288");

    Ok(())
}

#[test]
fn test_export_stack_directly_refuses_e8_matched_bit() -> Result<()> {
    let config = StackConfig {
        arch: StackArch::Geometric,
        vocab_size: 96,
        width: 32,
        heads: 1,
        mlp_hidden: 32,
        context: 8,
        pattern: "r".into(),
        read: ReadScore::Dot,
        rotation: false,
        seed: 42,
        memory: None,
    };
    let device = Device::Cpu;
    let mut model = StackModel::new(config, &device)?;
    model.set_served_representation(Some(Arc::new(E8MatchedBitMapCodec::default())))?;

    let err = export_stack(&model, "e8-direct-refusal-test".into(), None).unwrap_err();
    assert!(
        err.to_string().contains("stack export does not write"),
        "expected export refusal for E8 served codec, got: {err}"
    );
    Ok(())
}

#[test]
fn test_s2_real_proportions_exported_artifact_dequantized_weights_equal_served_view() -> Result<()>
{
    let config = StackConfig {
        arch: StackArch::Geometric,
        vocab_size: 4096,
        width: 288,
        heads: 4,
        mlp_hidden: 768,
        context: 16,
        pattern: "rrarra".into(),
        read: ReadScore::Lorentz,
        rotation: true,
        seed: 42,
        memory: None,
    };
    let device = Device::Cpu;

    let codecs: Vec<Arc<dyn MapCodec>> = vec![
        Arc::new(HeadCompensatedMapCodec::head_only_for(4096, 288)),
        Arc::new(RecurrenceOutMinMseMapCodec::for_shape(288, 288)),
    ];

    for codec in codecs {
        let mut model = StackModel::new(config.clone(), &device)?;
        model.set_served_representation(Some(codec.clone()))?;

        let (lut_bytes, summary) =
            export_stack(&model, serde_json::json!({"test": "s2_parity"}), None)?;
        assert_eq!(summary["method"]["quantizer"], codec.name());

        let artifact = StackArtifact::parse(lut_bytes.clone())
            .map_err(|e| uor_r4_training::TrainingError::Invalid(e.to_string()))?;

        // Verify element-for-element bitwise equality across all matrices
        let vars = model.variables();
        let var_name = |name: &str| -> Result<Vec<f32>> {
            Ok(vars
                .get(name)
                .ok_or_else(|| uor_r4_training::TrainingError::Invalid(name.to_string()))?
                .as_tensor()
                .flatten_all()?
                .to_vec1::<f32>()?)
        };

        for spec in &artifact.header.matrices {
            let dequantized = uor_r4_training::lut_export::dequantize_matrix(
                spec.rows,
                spec.cols,
                spec.exp_base,
                artifact.section(spec.nibbles),
                artifact.section(spec.scales),
            )?;

            let expected_served = match spec.name.as_str() {
                "embed" => {
                    let src = var_name("embedding.weight")?;
                    codec.round_trip(&src, spec.rows, spec.cols)?
                }
                "head" => {
                    let mut src = var_name("embedding.weight")?;
                    let gain = var_name("final_norm.weight")?;
                    fold_columns(&mut src, spec.cols, &gain);
                    codec.round_trip(&src, spec.rows, spec.cols)?
                }
                name => {
                    let layer = name[1..2].parse::<usize>().unwrap();
                    let part = &name[3..];
                    match part {
                        "rec_in" => {
                            let mut src = var_name(&format!("layers.{layer:02}.rec.in.weight"))?;
                            let gain = var_name(&format!("layers.{layer:02}.rec_norm.weight"))?;
                            fold_columns(&mut src, spec.cols, &gain);
                            codec.round_trip(&src, spec.rows, spec.cols)?
                        }
                        "rec_gate" => {
                            let mut src = var_name(&format!("layers.{layer:02}.rec.gate.weight"))?;
                            let gain = var_name(&format!("layers.{layer:02}.rec_norm.weight"))?;
                            fold_columns(&mut src, spec.cols, &gain);
                            codec.round_trip(&src, spec.rows, spec.cols)?
                        }
                        "rec_out" => {
                            let src = var_name(&format!("layers.{layer:02}.rec.out.weight"))?;
                            codec.round_trip(&src, spec.rows, spec.cols)?
                        }
                        "query" | "key" | "value" => {
                            let mut src =
                                var_name(&format!("layers.{layer:02}.read.{part}.weight"))?;
                            let gain = var_name(&format!("layers.{layer:02}.read_norm.weight"))?;
                            fold_columns(&mut src, spec.cols, &gain);
                            codec.round_trip(&src, spec.rows, spec.cols)?
                        }
                        "null" => {
                            let mut src = var_name(&format!("layers.{layer:02}.read.null.weight"))?;
                            let gain = var_name(&format!("layers.{layer:02}.read_norm.weight"))?;
                            fold_columns(&mut src, spec.cols, &gain);
                            codec.round_trip(&src, spec.rows, spec.cols)?
                        }
                        "out" => {
                            let src = var_name(&format!("layers.{layer:02}.read.out.weight"))?;
                            codec.round_trip(&src, spec.rows, spec.cols)?
                        }
                        "gate" | "up" => {
                            let mut src =
                                var_name(&format!("layers.{layer:02}.mlp.{part}.weight"))?;
                            let gain = var_name(&format!("layers.{layer:02}.mlp_norm.weight"))?;
                            fold_columns(&mut src, config.width, &gain);
                            let padded =
                                pad(&src, config.mlp_hidden, config.width, spec.rows, spec.cols);
                            codec.round_trip(&padded, spec.rows, spec.cols)?
                        }
                        "down" => {
                            let src = var_name(&format!("layers.{layer:02}.mlp.down.weight"))?;
                            let padded =
                                pad(&src, config.width, config.mlp_hidden, spec.rows, spec.cols);
                            codec.round_trip(&padded, spec.rows, spec.cols)?
                        }
                        other => panic!("unknown matrix in artifact: {other}"),
                    }
                }
            };

            assert_eq!(
                dequantized,
                expected_served,
                "codec {}, matrix {}: exported artifact dequantized weights must match served view element by element bit for bit",
                codec.name(),
                spec.name
            );
        }

        // Verify save and reload with codec_by_name recovery
        let tmp_root = std::env::temp_dir().join(format!("d4-s2-reload-{}", std::process::id()));
        let save_dir = tmp_root.join("saved");
        model.save(&save_dir)?;

        let saved_rec = StackModel::saved_served_representation(&save_dir)?;
        let recovered_codec = codec_by_name(&saved_rec.unwrap().codec)?;
        assert_eq!(recovered_codec.name(), codec.name());

        let mut reloaded = StackModel::load(&save_dir, &device)?;
        assert!(reloaded.served_codec().is_none());
        reloaded.set_served_representation(Some(recovered_codec.clone()))?;
        assert_eq!(reloaded.served_codec().unwrap().name(), codec.name());

        let (reloaded_bytes, reloaded_summary) =
            export_stack(&reloaded, serde_json::json!({"test": "s2_reloaded"}), None)?;
        assert_eq!(reloaded_summary["method"]["quantizer"], codec.name());
        assert_eq!(
            reloaded_bytes,
            lut_bytes,
            "codec {}: reloaded model export must be bit-identical to original export",
            codec.name()
        );

        let _ = std::fs::remove_dir_all(&tmp_root);
    }

    Ok(())
}
