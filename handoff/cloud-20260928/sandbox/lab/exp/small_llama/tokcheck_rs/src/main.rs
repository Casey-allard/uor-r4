//! usage: small-llama-tokcheck TOKENIZER.json TEXT_FILE...
use uor_r4_tokenizer::ByteBpeTokenizer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let json = std::fs::read(&args[1]).expect("read tokenizer.json");
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&json).expect("tokenizer.json must parse");
    println!("vocab_size={} model_vocab_size={}", tok.vocab_size(), tok.model_vocab_size());
    let specials = ["<|endoftext|>", "<|im_start|>", "<|im_end|>"];
    // Every id decodes to exactly its byte (or its special's literal text).
    let mut bad = 0;
    for id in 0..tok.vocab_size() as u32 {
        let want: Vec<u8> = if id < 3 { specials[id as usize].as_bytes().to_vec() } else { vec![(id - 3) as u8] };
        if tok.decode_bytes(&[id]) != want { bad += 1; }
    }
    let lens = tok.token_byte_lengths();
    println!("per-id decode mismatches={bad} (of {}) byte_lengths[3..259] all 1: {}", tok.vocab_size(),
             lens[3..259].iter().all(|&l| l == 1));
    for (i, s) in specials.iter().enumerate() {
        println!("encode({s:?}) = {:?} (want [{i}])", tok.encode(s));
    }
    // Encode is byte+3 (specials atomic) and decode inverts it, on each text file.
    for path in &args[2..] {
        let text = std::fs::read_to_string(path).expect("utf-8 text");
        let ids = tok.encode(&text);
        let byte_ids = !specials.iter().any(|s| text.contains(s))
            && ids.len() == text.len()
            && ids.iter().zip(text.as_bytes()).all(|(&i, &b)| i == u32::from(b) + 3);
        let round = tok.decode(&ids) == text && tok.decode_bytes(&ids) == text.as_bytes();
        println!("{path}: bytes={} tokens={} ids_are_byte_plus_3={byte_ids} decode(encode(x))==x: {round}",
                 text.len(), ids.len());
    }
    let a = tok.adapter();
    println!("adapter family={} version={} tokenizer_cid={} pre_tokenizers={:?} digest={}",
             a.family, a.version, a.tokenizer_cid, a.policy.pre_tokenizers, a.adapter_digest);
}
