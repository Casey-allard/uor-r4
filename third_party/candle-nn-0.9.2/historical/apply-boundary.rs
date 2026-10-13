//! One-time text edit recipe. Not referenced by the crate's build or module tree.
use std::{fs, path::Path};
fn replace(text: &mut String, marker: &str, body: &str, expected: usize) {
    let mut count = 0;
    let mut from = 0;
    while let Some(offset) = text[from..].find(marker) {
        let start = from + offset;
        let open = start + text[start..].find('{').expect("function body");
        let bytes = text.as_bytes();
        let mut i = open + 1;
        let mut depth = 1;
        while depth > 0 {
            assert!(i < bytes.len(), "unclosed function {marker}");
            if bytes[i] == b'"' {
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' { i += 2; }
                    else if bytes[i] == b'"' { i += 1; break; }
                    else { i += 1; }
                }
            } else if i+1 < bytes.len() && &bytes[i..i+2] == b"//" {
                while i < bytes.len() && bytes[i] != b'\n' { i += 1; }
            } else if i+1 < bytes.len() && &bytes[i..i+2] == b"/*" {
                i += 2;
                while i+1 < bytes.len() && &bytes[i..i+2] != b"*/" { i += 1; }
                i += 2;
            } else {
                match bytes[i] { b'{' => depth += 1, b'}' => depth -= 1, _ => {} }
                i += 1;
            }
        }
        text.replace_range(open+1..i-1, body);
        from = open + body.len() + 2;
        count += 1;
    }
    assert_eq!(count, expected, "function count for {marker}");
}
fn main() {
    let root = Path::new("third_party/candle-nn-0.9.2/src");
    for (file, marker, count, refusal) in [
        ("ops.rs", "\n    fn cpu_fwd(", 2, "D24: CPU model normalization is inactive"),
        ("rotary_emb.rs", "\n    fn cpu_fwd(", 3, "D24: CPU model rotation is inactive"),
        ("linear.rs", "\n    fn forward(", 1, "D29: historical dense linear operation is inactive"),
        ("conv.rs", "\n    fn forward(", 4, "D29: historical convolution operation is inactive"),
        ("rnn.rs", "\n    fn step(&self, input: &Tensor, in_state: &Self::State)", 2, "D29: historical dense recurrent operation is inactive"),
        ("loss.rs", "\npub fn nll(", 1, "D29: historical probability objective is inactive"),
        ("loss.rs", "\npub fn cross_entropy(", 1, "D29: historical probability objective is inactive"),
        ("loss.rs", "\npub fn binary_cross_entropy_with_logit(", 1, "D29: historical probability objective is inactive"),
    ] {
        let path = root.join(file);
        let mut text = fs::read_to_string(&path).expect("read owned source");
        replace(&mut text, marker, &format!("\n        candle::bail!({refusal:?})\n    "), count);
        fs::write(path, text).expect("write owned source");
    }
}
