//! SCRATCH before/after probe for the read-geometry change. NOT COMMITTED.
//! Prints SHA-256 digests of Dot-model outputs, gradients and artifacts so the
//! same file can be run on the base commit and on the changed tree.

use std::fs;
use std::path::PathBuf;

use candle_core::{Device, Tensor};
use sha2::{Digest, Sha256};
use uor_r4_training::joint_admission::AdmissionPolicy;
use uor_r4_training::joint_model::{
    numerical_contract, quantized_numerical_contract, JointConfig, JointModel, ReadMode,
    Transport,
};

fn add(hasher: &mut Sha256, tensor: &Tensor) {
    let values = tensor.flatten_all().unwrap().to_vec1::<f32>().unwrap();
    hasher.update((values.len() as u64).to_le_bytes());
    for value in values {
        hasher.update(value.to_bits().to_le_bytes());
    }
}

fn hex_of(hasher: Sha256) -> String {
    hex::encode(hasher.finalize())
}

fn file_hash(path: &std::path::Path) -> String {
    hex::encode(Sha256::digest(fs::read(path).unwrap()))
}

fn root(label: &str) -> PathBuf {
    let base = PathBuf::from(std::env::var("PROBE_ROOT").unwrap());
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = base.join(format!("{label}-{nonce}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn small(transport: Transport, context: usize) -> JointConfig {
    JointConfig {
        width: 128,
        context,
        transport,
        seed: 7,
        ..JointConfig::default()
    }
}

fn outputs(model: &JointModel, ids: &[u32], batch: usize, time: usize, mode: ReadMode) -> String {
    let output = model.forward(ids, batch, time, mode, false).unwrap();
    let mut h = Sha256::new();
    for t in [
        &output.probabilities,
        &output.states,
        &output.read_masses,
        &output.no_read_mass,
        &output.copy_gate,
    ] {
        add(&mut h, t);
    }
    hex_of(h)
}

fn gradients(model: &JointModel, ids: &[u32], batch: usize, time: usize) -> String {
    let output = model.forward(ids, batch, time, ReadMode::Enabled, true).unwrap();
    let targets: Vec<u32> = ids.iter().map(|x| (x + 1) % 4096).collect();
    let loss = output.loss(&targets).unwrap();
    let grads = loss.backward().unwrap();
    let mut h = Sha256::new();
    add(&mut h, &loss);
    for (name, variable) in model.variables() {
        h.update(name.as_bytes());
        match grads.get(variable.as_tensor()) {
            Some(g) => add(&mut h, g),
            None => h.update(b"missing"),
        }
    }
    hex_of(h)
}

fn session(model: &JointModel, ids: &[u32], batch: usize, time: usize) -> String {
    let mut s = model.new_session(batch).unwrap();
    let mut h = Sha256::new();
    for position in 0..time {
        let tokens: Vec<u32> = (0..batch).map(|lane| ids[lane * time + position]).collect();
        let step = model.step(&mut s, &tokens, ReadMode::Enabled).unwrap();
        for t in [
            &step.probabilities,
            &step.no_read_mass,
            &step.read_masses,
            &step.copy_gate,
            &step.state,
        ] {
            add(&mut h, t);
        }
        h.update(format!("{:?}", step.read_occurrences_by_lane).as_bytes());
    }
    hex_of(h)
}

fn parameters(model: &JointModel) -> String {
    let mut h = Sha256::new();
    for (name, variable) in model.variables() {
        h.update(name.as_bytes());
        add(&mut h, variable.as_tensor());
    }
    hex_of(h)
}

#[test]
fn diagnostic_row_sums() {
    use uor_r4_training::joint_model::ReadGeometry;
    let ids = [3u32, 17, 4, 9, 2, 10, 6, 11, 7, 22];
    for transport in [Transport::Quaternion, Transport::HouseholderPair] {
        for geometry in [ReadGeometry::Dot, ReadGeometry::Lorentz] {
            let config = JointConfig {
                read_geometry: geometry,
                ..small(transport, 8)
            };
            let model = JointModel::new(config, &Device::Cpu).unwrap();
            let output = model.forward(&ids, 2, 5, ReadMode::Enabled, false).unwrap();
            let sums = output.probabilities.sum(2).unwrap().flatten_all().unwrap();
            let sums = sums.to_vec1::<f32>().unwrap();
            let worst = sums.iter().map(|s| (s - 1.0).abs()).fold(0f32, f32::max);
            let f64_sums: Vec<f64> = output
                .probabilities
                .to_vec3::<f32>()
                .unwrap()
                .iter()
                .flatten()
                .map(|row| row.iter().map(|&p| f64::from(p)).sum::<f64>())
                .collect();
            let worst64 = f64_sums.iter().map(|s| (s - 1.0).abs()).fold(0f64, f64::max);
            let no_read = output.no_read_mass.flatten_all().unwrap().to_vec1::<f32>().unwrap();
            println!(
                "rowsum {transport:?} {geometry:?} f32_sum_dev={worst:e} f64_sum_dev={worst64:e} no_read={no_read:?}"
            );
        }
    }
}

#[test]
fn probe() {
    let mut lines = Vec::new();
    for config in [
        JointConfig::default(),
        small(Transport::Quaternion, 8),
        small(Transport::HouseholderPair, 80),
    ] {
        lines.push(format!("config_json {}", serde_json::to_string(&config).unwrap()));
    }
    lines.push(format!(
        "contract {}",
        hex::encode(Sha256::digest(
            serde_json::to_vec(&numerical_contract()).unwrap()
        ))
    ));
    lines.push(format!(
        "quantized_contract {}",
        hex::encode(Sha256::digest(
            serde_json::to_vec(&quantized_numerical_contract()).unwrap()
        ))
    ));
    let ids = [3u32, 17, 4, 9, 2, 10, 6, 11, 7, 22];
    for transport in [Transport::Quaternion, Transport::HouseholderPair] {
        let model = JointModel::new(small(transport, 8), &Device::Cpu).unwrap();
        let names: Vec<_> = model.variables().keys().cloned().collect();
        lines.push(format!("{transport:?} names {}", names.join(",")));
        lines.push(format!("{transport:?} parameters {}", parameters(&model)));
        for mode in [ReadMode::Enabled, ReadMode::NoRead] {
            lines.push(format!(
                "{transport:?} full {mode:?} {}",
                outputs(&model, &ids, 2, 5, mode)
            ));
        }
        lines.push(format!(
            "{transport:?} full gradients {}",
            gradients(&model, &ids, 2, 5)
        ));
        lines.push(format!(
            "{transport:?} full session {}",
            session(&model, &ids, 2, 5)
        ));
        lines.push(format!(
            "{transport:?} contract {}",
            hex::encode(Sha256::digest(
                serde_json::to_vec(&model.numerical_contract()).unwrap()
            ))
        ));
        let dir = root("checkpoint");
        model.save(&dir).unwrap();
        lines.push(format!(
            "{transport:?} checkpoint config {} weights {}",
            file_hash(&dir.join("config.json")),
            file_hash(&dir.join("model.safetensors"))
        ));
        let reloaded = JointModel::load(&dir, &Device::Cpu).unwrap();
        lines.push(format!(
            "{transport:?} reloaded full Enabled {}",
            outputs(&reloaded, &ids, 2, 5, ReadMode::Enabled)
        ));

        let mut quantized = JointModel::new(small(transport, 8), &Device::Cpu).unwrap();
        quantized.configure_quantization(7, 4).unwrap();
        quantized.set_completed_step(11).unwrap();
        lines.push(format!(
            "{transport:?} quantized Enabled {}",
            outputs(&quantized, &ids, 2, 5, ReadMode::Enabled)
        ));
        let dir = root("quantized");
        quantized.save(&dir).unwrap();
        let packed = root("packed");
        quantized.save_hard(&packed).unwrap();
        lines.push(format!(
            "{transport:?} quantized config {} hard-model {} hard-parameters {}",
            file_hash(&dir.join("config.json")),
            file_hash(&packed.join("hard-model.json")),
            file_hash(&packed.join("hard-parameters.json"))
        ));
        let hard = JointModel::load_hard(&packed, &Device::Cpu).unwrap();
        lines.push(format!(
            "{transport:?} hard Enabled {}",
            outputs(&hard, &ids, 2, 5, ReadMode::Enabled)
        ));
    }
    let bounded_ids: Vec<u32> = (0..160)
        .map(|i| ((i * 17 + i / 7) % 4095 + 1) as u32)
        .collect();
    for policy in [
        AdmissionPolicy::Full,
        AdmissionPolicy::Recent64,
        AdmissionPolicy::Orthant64,
        AdmissionPolicy::ExactCache64,
    ] {
        let mut model =
            JointModel::new(small(Transport::HouseholderPair, 80), &Device::Cpu).unwrap();
        model.set_admission_policy(policy).unwrap();
        lines.push(format!(
            "bounded {policy:?} Enabled {}",
            outputs(&model, &bounded_ids, 2, 80, ReadMode::Enabled)
        ));
        lines.push(format!(
            "bounded {policy:?} gradients {}",
            gradients(&model, &bounded_ids, 2, 80)
        ));
        lines.push(format!(
            "bounded {policy:?} session {}",
            session(&model, &bounded_ids, 2, 80)
        ));
    }
    let text = lines.join("\n");
    println!("{text}");
    fs::write(
        PathBuf::from(std::env::var("PROBE_ROOT").unwrap())
            .join(std::env::var("PROBE_LABEL").unwrap()),
        text + "\n",
    )
    .unwrap();
}
