use m2_donor_corpus::{check_extra_body, check_record, parse_reply, request_body, Api, Request};
use serde_json::json;

#[test]
fn extra_body_refuses_score_keys_at_any_depth() {
    assert!(
        check_extra_body(&json!({"thinking": {"type": "enabled"}, "reasoning_effort": "low"}))
            .is_ok()
    );
    for bad in [
        json!({"logprobs": true}),
        json!({"top_logprobs": 20}),
        json!({"prompt_logprobs": 1}),
        json!({"x": {"LogProbs": true}}),
        json!({"x": [{"return_score": true}]}),
        json!({"rank": 1}),
    ] {
        assert!(check_extra_body(&bad).is_err(), "{bad} passed");
    }
}

fn single() -> serde_json::Value {
    json!({"id": "p0", "kind": "single", "source": "s",
        "messages": [{"role": "user", "content": "Hi"}],
        "turns": [{"reasoning": "think", "answer": "Hello!"}]})
}

fn two_turn() -> serde_json::Value {
    json!({"id": "p1", "kind": "two-turn", "source": "s",
        "messages": [{"role": "user", "content": "A"}, {"role": "user", "content": "B"}],
        "turns": [{"reasoning": "", "answer": "a"}, {"reasoning": "r", "answer": "b"}]})
}

#[test]
fn check_record_accepts_good_and_rejects_bad() {
    assert!(check_record(&single()).is_ok());
    assert!(check_record(&two_turn()).is_ok());
    for extra in ["logprobs", "score"] {
        let mut bad = single();
        bad[extra] = json!([5]);
        assert!(check_record(&bad).is_err(), "{extra} passed");
    }
    let mut bad = single();
    bad["turns"][0]["logprob"] = json!(-1);
    assert!(check_record(&bad).is_err(), "extra turn key passed");
    let mut bad = two_turn();
    bad["turns"] = json!([{"reasoning": "", "answer": "a"}]);
    assert!(check_record(&bad).is_err(), "turn count passed");
    let mut bad = single();
    bad["turns"][0]["reasoning"] = json!("<think>x");
    assert!(check_record(&bad).is_err(), "markup passed");
    let mut bad = single();
    bad["turns"][0]["answer"] = json!("<\u{ff5c}end\u{ff5c}>");
    assert!(check_record(&bad).is_err(), "fullwidth markup passed");
    let mut bad = single();
    bad["turns"][0]["answer"] = json!("  \n");
    assert!(check_record(&bad).is_err(), "empty answer passed");
}

#[test]
fn replies_parse_both_reasoning_fields() {
    let r = parse_reply(&json!({"choices": [{"finish_reason": "stop",
        "message": {"content": "ans", "reasoning_content": "rc", "reasoning": "r"}}],
        "usage": {"completion_tokens": 7, "prompt_tokens": 3}}));
    assert_eq!((r.answer.as_str(), r.reasoning.as_str()), ("ans", "rc"));
    assert_eq!((r.completion_tokens, r.prompt_tokens), (7, 3));
    assert_eq!((r.model.as_str(), r.cache_hit_tokens), ("", 0));
    assert_eq!(r.drop_reason(), None);
    let r = parse_reply(
        &json!({"model": "deepseek-v4-flash", "choices": [{"finish_reason": "stop",
        "message": {"content": "a", "reasoning_content": "r"}}],
        "usage": {"completion_tokens": 7, "prompt_tokens": 3, "prompt_cache_hit_tokens": 2}}),
    );
    assert_eq!(
        (r.model.as_str(), r.cache_hit_tokens),
        ("deepseek-v4-flash", 2)
    );
    let r = parse_reply(&json!({"choices": [{"finish_reason": "stop",
        "message": {"content": " ans ", "reasoning": "r"}}]}));
    assert_eq!((r.answer.as_str(), r.reasoning.as_str()), (" ans ", "r"));
    let r = parse_reply(&json!({"choices": [{"finish_reason": "stop",
        "message": {"content": "ans"}}]}));
    assert_eq!(r.reasoning, "");
    assert_eq!(r.drop_reason(), None);
    let r = parse_reply(&json!({"choices": [{"finish_reason": "length",
        "message": {"content": "ans"}}]}));
    assert_eq!(r.drop_reason(), Some("length"));
}

#[test]
fn request_body_uses_integers_and_extra_overrides() {
    let req = Request {
        model_name: "donor".into(),
        max_tokens: 16384,
        effort: 50,
        base_seed: 1,
        api: Api::Vllm,
        extra: Some(json!({"top_p": 1, "chat_template_kwargs": {"thinking": false}})),
    };
    let b = request_body(&req, &json!([]), 9);
    assert!(b["temperature"].is_u64() && b["seed"] == 9 && b["stream"] == false);
    assert_eq!(b["chat_template_kwargs"], json!({"thinking": false}));
}
