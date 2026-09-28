# Exact parameter count and FLOP accounting for the D8 joint recurrent learner
# (shapes from crates/uor-r4-integer/src/config.rs:59-85; ops from joint_model.rs core_step/output_distribution)
V,d,r,T=4096,256,64,256
shapes={
 "embedding.weight":(V,d),"recurrent.input.weight":(3*d,d),"recurrent.state.weight":(3*d,d),
 "recurrent.bias":(3*d,),"read.query.weight":(r,d),"read.query.bias":(r,),"read.key.weight":(r,d),
 "read.key.bias":(r,),"read.value.weight":(d,d),"read.value.bias":(d,),"read.age":(T-1,),
 "read.no_read.weight":(1,d),"read.no_read.bias":(1,),"update.weight":(d,2*d),"update.bias":(d,),
 "update.gate.weight":(1,2*d),"update.gate.bias":(1,),"copy.gate.weight":(1,2*d),"copy.gate.bias":(1,),
 "output.norm.weight":(d,),"output.bias":(V,)}
import math
n={k:math.prod(v) for k,v in shapes.items()}
tot=sum(n.values()); emb=n["embedding.weight"]
print("total params",tot)
print("embedding (tied in/out)",emb, f"{emb/tot:.1%}")
print("non-embedding incl output.bias",tot-emb)
print("non-embedding excl output.bias/norm",tot-emb-n["output.bias"]-n["output.norm.weight"])
grp={"recurrent (W_in,W_s,b)":n["recurrent.input.weight"]+n["recurrent.state.weight"]+n["recurrent.bias"],
     "read (Q,K,V,age,null)":sum(v for k,v in n.items() if k.startswith("read.")),
     "update+gates":sum(v for k,v in n.items() if k.startswith("update")or k.startswith("copy")),
     "output norm+bias":n["output.norm.weight"]+n["output.bias"]}
for k,v in grp.items(): print(f"  {k:28s} {v:9d}")
# ---- MACs per token (forward) ----
mac={}
mac["token affine E[x]W_in^T"]=3*d*d
mac["recurrent W_s rms(h)"]=3*d*d
mac["transport (64 lanes x 16 MAC)"]=(d//4)*16
mac["query"]=r*d; mac["null"]=d
mac["update W_u[p;read]"]=d*2*d; mac["update gate"]=2*d; mac["copy gate"]=2*d
mac["key"]=r*d; mac["value"]=d*d
mac["logits (tied E)"]=V*d
dense=sum(mac.values())
avg_prev=(T-1)/2          # mean number of previous events over a 256 window
read_avg=avg_prev*(r+d)   # q.k over r dims + sum a_i v_i over d dims
read_max=(T-1)*(r+d)
print("\nforward MACs/token (dense, excl read):",dense)
for k,v in mac.items(): print(f"  {k:32s} {v:9d}  {v/dense:.1%}")
print("read MACs/token: mean over window",read_avg,"at t=255",read_max)
fwd_flops=2*(dense+read_avg)
print(f"forward FLOPs/token (mean) {fwd_flops/1e6:.3f} M ; train (x3) {3*fwd_flops/1e6:.2f} M ; 6N rule {6*tot/1e6:.2f} M")
# ---- serving (per generated token at full context) ----
serve=dense-mac["token affine E[x]W_in^T"]+read_max   # W_in*E can be a 4096x768 table
print(f"serving MACs/token @255 prev events: {serve} (token affine tabulated) ; with W_in computed: {serve+3*d*d}")
print(f"  share of logits in serving: {mac['logits (tied E)']/serve:.1%}")
# ---- achieved training throughput ----
for label,tps in [("sustained per arm",1362),("profile 2 workers/arm (best)",1575),("Metal quaternion profile",823)]:
    print(f"{label:30s} {tps} tok/s -> {tps*3*fwd_flops/1e9:.1f} GFLOP/s")
print("machine total (2 arms x 1362):", 2*1362*3*fwd_flops/1e9, "GFLOP/s")
# transformer #1014 on the same M1 (MPS): 29,999,104 tokens in 4220.42 s, 7,155,360 params, 6L d=288 ctx256
tok_s=29999104/4220.42; N=7155360; L,dm,ctx=6,288,256
attn_fwd=L*2*2*(ctx/2)*dm   # QK^T and AV, mean causal length
tf_flops=6*N+3*attn_fwd
print(f"\n#1014 transformer: {tok_s:.0f} tok/s, {tf_flops/1e6:.1f} MFLOP/token -> {tok_s*tf_flops/1e12:.3f} TFLOP/s = {tok_s*tf_flops/2.6e12:.1%} of 2.6 TFLOPS M1 GPU")
print(f"recurrent learner machine-level: {2*1362*3*fwd_flops/1e12:.4f} TFLOP/s ; per-token speed ratio transformer/recurrent-arm = {tok_s/1362:.2f}x at {N/tot:.2f}x params")
