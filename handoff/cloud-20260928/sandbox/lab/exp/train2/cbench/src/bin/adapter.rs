//! train2 scratch check (not project code): run the repository's own teacher
//! adapter declarations (uor_r4_model_source::conformance::AdapterFeatures)
//! against retrieved Hugging Face config.json files, before any weights exist.
use uor_r4_model_source::conformance::AdapterFeatures;

fn main() {
    for path in std::env::args().skip(1) {
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                println!("{path}: unreadable: {e}");
                continue;
            }
        };
        let json: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                println!("{path}: not JSON: {e}");
                continue;
            }
        };
        for (name, feats) in [
            ("huggingface-llama", AdapterFeatures::huggingface_llama()),
            ("huggingface-gpt2", AdapterFeatures::huggingface_gpt2()),
        ] {
            match feats.validate_config(&json) {
                Ok(()) => println!("{path}: {name}: ACCEPTED"),
                Err(e) => println!("{path}: {name}: REJECTED ({:?}): {}", e.kind, e.reason),
            }
        }
    }
}
