"""Derived sizing for the Geometric Chat Model (GCM) plan. All numbers are arithmetic from the stated shapes."""
import json
T = 2048          # context for KV/state figures
ADMIT = 256 + 64  # exact read candidates per exact layer with admission (recent window + routed)

def llama_params(d, L, H, Hkv, ffn, V, r=64):
    attn = d * H * r * 2 + d * Hkv * r * 2
    mlp = 3 * d * ffn
    return {"emb": V * d, "attn": L * attn, "mlp": L * mlp, "total": V * d + L * (attn + mlp)}

out = {}
# ---- teacher and converted student (GCM-L): SmolLM2-135M-Instruct
t = llama_params(576, 30, 9, 3, 1536, 49152)
out["smollm2_135m"] = t
L, d, H, Hkv, r, ffn, V = 30, 576, 9, 3, 64, 1536, 49152
n_exact = 8                      # exact kappa-Lorentz read layers (~27%)
n_conv = L - n_exact             # window(64) kappa-Lorentz + recurrent-state layers
feat = 128                       # recurrent key-feature width per kv head (LoLCATs-like, effective 128)
body = t["attn"] + t["mlp"]
new_params = n_conv * (2 * H * r * feat // 2) + L * H * 4          # feature maps + (kappa, beta, gamma, bias)
kv_exact_B_per_tok_layer = Hkv * (r + r + 4)                          # int8 K, int8 V, |k|^2 & k0 scalars
kv = {
  "teacher_bf16_full_KV_MB_at_2K": L * Hkv * 2 * r * 2 * T / 1e6,
  "exact_layers_KV_MB_at_2K": n_exact * kv_exact_B_per_tok_layer * T / 1e6,
  "window_layers_KV_MB": n_conv * kv_exact_B_per_tok_layer * 64 / 1e6,
  "recurrent_state_MB": n_conv * Hkv * feat * r * 2 / 1e6,
}
kv["student_total_MB_at_2K"] = kv["exact_layers_KV_MB_at_2K"] + kv["window_layers_KV_MB"] + kv["recurrent_state_MB"]
kv["student_KV_read_MB_per_token_with_admission"] = (n_exact * kv_exact_B_per_tok_layer * ADMIT + n_conv * kv_exact_B_per_tok_layer * 64) / 1e6 + kv["recurrent_state_MB"]
kv["teacher_KV_read_MB_per_token_at_2K"] = kv["teacher_bf16_full_KV_MB_at_2K"]
weights = {
  "teacher_bf16_MB_per_token": t["total"] * 2 / 1e6,
  "student_body_4bit_MB": body * 0.5 / 1e6,
  "student_body_ternary_2bitpack_MB": body * 0.25 / 1e6,
  "student_head_4bit_MB": t["emb"] * 0.5 / 1e6,
}
# multiplier-free activation products per token (served by table multiplies)
prods = {
  "swiglu_gate": L * ffn,
  "exact_read_scores(per-query multiples tables: reads)": n_exact * H * ADMIT * r,
  "exact_read_value_mix": n_exact * H * ADMIT * r,
  "window_read_scores+mix": n_conv * H * 64 * r * 2,
  "recurrent_update(outer+decay)": n_conv * Hkv * feat * r * 2,
  "recurrent_readout": n_conv * H * feat * r,
  "rope_and_norms(approx)": L * (4 * H * r + 3 * d),
}
prods["total_products"] = sum(v for k, v in prods.items())
lut_ops_weights = body * 0.5 + t["emb"] * 2.0     # ~0.5 op/weight ternary-LUT body; ~2 ops/weight 4-bit bit-serial head
out["gcm_L_converted_135m"] = {"new_params": new_params, "kv": kv, "weights": weights, "act_products": prods,
                               "weight_lut_ops_approx": lut_ops_weights,
                               "energy_proxy_mJ_per_token_at_80mJ_per_GB": {
                                   "teacher_bf16": (weights["teacher_bf16_MB_per_token"] + kv["teacher_KV_read_MB_per_token_at_2K"]) * 0.08,
                                   "student_4bit": (weights["student_body_4bit_MB"] + weights["student_head_4bit_MB"] + kv["student_KV_read_MB_per_token_with_admission"]) * 0.08,
                                   "student_ternary_body": (weights["student_body_ternary_2bitpack_MB"] + weights["student_head_4bit_MB"] + kv["student_KV_read_MB_per_token_with_admission"]) * 0.08}}
# ---- conversion compute
teacher_fwd = 2 * body + L * 4 * (T // 2) * d          # GFLOP-ish per token incl. attention at mean position
stage1 = teacher_fwd + 3 * (n_conv * (4 * H * 64 * r + 4 * H * feat * r) + n_exact * 4 * H * (T // 2) * r)
stage2 = 6 * t["total"] + teacher_fwd + 3 * L * 4 * (T // 2) * d
rates = {"M1_assumed_0.3TF": 0.3e12, "M1_assumed_0.6TF": 0.6e12, "container_1thread_measured_29GF": 29e9}
conv = {"stage1_flop_per_token": stage1, "stage2_flop_per_token": stage2}
for tok, lab in [(20e6, "stage1_20M_tokens"), (40e6, "lolcats_budget_40M"), (0.2e9, "stage2_0.2B"), (1e9, "stage2_1B")]:
    f = (stage1 if lab.startswith("stage1") else stage2) * tok
    conv[lab] = {k: round(f / v / 3600, 1) for k, v in rates.items()}   # hours
    conv[lab]["PFLOP"] = round(f / 1e15, 2)
conv["stage1_per_layer_hours"] = {k: round(v / 30, 2) for k, v in conv["stage1_20M_tokens"].items() if k != "PFLOP"}
out["conversion_hours"] = conv
# ---- native sizes
def gcm(d, L, n_read, V, e=4):
    rec = 4 * d * d + 3 * d * d // 16          # c,f,g,o + quaternion raw (1/4 lanes)
    read = 2 * d * d + 2 * d * (d // 4)         # Q,O full; K,V at Hkv = H/4
    mlp = 2 * e * d * d                          # ReLU^2
    body = (L - n_read) * (rec + mlp) + n_read * (read + mlp)
    return {"d": d, "L": L, "reads": n_read, "V": V, "body": body, "emb": V * d, "total": body + V * d,
            "body_ternary_MB": round(body * 0.25 / 1e6, 2), "head_4bit_MB": round(V * d * 0.5 / 1e6, 2),
            "rec_state_KB": round((L - n_read) * d * 2 * 1.5 / 1e3, 1),
            "read_KV_MB_2K": round(n_read * (d // 4) * 2 * T / 1e6, 2),
            "train_tok_per_week_M1_0.3TF_B": round(0.3e12 * 604800 / (6 * (body + V * d) * 1.1) / 1e9, 2)}
out["gcm_S"] = gcm(320, 12, 3, 4096)
out["gcm_M"] = gcm(512, 18, 4, 8192)
out["d8_reference"] = {"params": 1678466, "nonemb": 629890}
print(json.dumps(out, indent=1, default=lambda x: round(x, 3)))
