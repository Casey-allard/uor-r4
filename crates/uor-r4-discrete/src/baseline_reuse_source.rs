//! Separate opt-in CUDA entry derived from unchanged ordinary helpers/writer.
//! Transformations fail closed when the admitted ordinary control flow changes.

fn replace_once(source: &mut String, needle: &str, replacement: &str) -> Result<(), String> {
    if source.matches(needle).count() != 1 {
        return Err(format!("baseline reuse source anchor changed: {needle}"));
    }
    *source = source.replacen(needle, replacement, 1);
    Ok(())
}

pub fn generate(source: &str) -> Result<String, String> {
    let writer_start = source
        .find("__device__ W writer(")
        .ok_or("writer missing")?;
    let writer_end = source[writer_start..]
        .find("// Statuses added")
        .ok_or("writer end missing")?
        + writer_start;
    let kernel_start = source
        .find("extern \"C\" __global__ void discrete_counterfactual(")
        .ok_or("counterfactual missing")?;
    let kernel_end = source[kernel_start..]
        .find("__device__ U resulting_port")
        .ok_or("counterfactual end missing")?
        + kernel_start;
    let mut kernel = source[kernel_start..kernel_end].to_owned();
    replace_once(
        &mut kernel,
        "void discrete_counterfactual(",
        "void discrete_counterfactual_baseline_reuse(",
    )?;
    replace_once(
        &mut kernel,
        "W* losses,U* status)",
        "W* losses,U* status,W* audit)",
    )?;
    replace_once(&mut kernel, HEADER, PRODUCERS)?;
    replace_once(&mut kernel, BASELINE_LOOP, CONSUMER_PREFIX)?;
    // The virtual read/writer, sentinel, checked sum and final loss store are
    // retained verbatim; each ordinary error also records its terminal code.
    for code in ["2u", "arithmetic_status", "12u", "8u"] {
        let needle = format!("atomicExch(status,{code});");
        replace_once(
            &mut kernel,
            &needle,
            &format!("lane.words[5]={code};{needle}"),
        )?;
    }
    let helpers_and_writer = &source[..writer_end];
    Ok(format!("{helpers_and_writer}\n{AUDIT}\n{kernel}"))
}

const HEADER: &str = r#"    U candidate=blockIdx.x*blockDim.x+threadIdx.x;if(candidate>=16)return;
    W sum=0;U eligible=0;losses[candidate]=~(W)0;
    bool gated=s[8]==MODE_FLOCK || s[8]==MODE_RELATION;
    if((kind==GATE_KIND && !gated) || (kind==0 && gated)){atomicExch(status,9u);return;}
    if(kind==GATE_KIND && index>=gate_count(s)){atomicExch(status,4u);return;}
    if(kind==GATE_KIND && candidate>=3)return;
"#;

const PRODUCERS: &str = r#"    // Host admission binds exactly one block of sixteen physical lanes,
    // Route read, Route/VectorBlock parameter and 1..64 validated rows.
    U candidate=threadIdx.x;
    ReuseLane lane(audit,candidate);
    __shared__ U baseline_status[64];
    __shared__ U baseline_eligible[64];
    for(U row=candidate;row<s[4];row+=16){
        int baseline[MAX_WIDTH];U prediction,tr,ar,arithmetic_status=0;
        U row_status=0,row_eligible=0;
        lane.words[0]|=(W)1<<row;++lane.words[1];
        if(!read_vector(input,ids,routes,gates,plan,g,s,row,2,0,0,baseline,0)){
            row_status=2u;
        }else{
            W base=writer(baseline,table,g,s,targets[row],2,0,0,&prediction,&tr,&ar,0,0,0,0,&arithmetic_status);
            if(base==~(W)0)row_status=arithmetic_status?arithmetic_status:8u;
            else row_eligible=tr!=0;
        }
        // Every produced row writes both words even when its read/writer fails.
        baseline_status[row]=row_status;baseline_eligible[row]=row_eligible;
        audit[row]=(W)row_status|((W)row_eligible<<32);
    }
    // Producer loop lengths may differ; no lane returns or publishes a global
    // status before every lane reaches this unconditional barrier.
    __syncthreads();
    W sum=0;U eligible=0;losses[candidate]=~(W)0;
"#;

const BASELINE_LOOP: &str = r#"    for(U row=0;row<s[4];++row){int q[MAX_WIDTH],baseline[MAX_WIDTH];U prediction,tr,ar;
        if(!read_vector(input,ids,routes,gates,plan,g,s,row,2,0,0,baseline,0)){atomicExch(status,2u);return;}
        U arithmetic_status=0;
        W base=writer(baseline,table,g,s,targets[row],2,0,0,&prediction,&tr,&ar,0,0,0,0,&arithmetic_status);
        if(base==~(W)0){atomicExch(status,arithmetic_status?arithmetic_status:8u);return;}
        // The baseline fixes eligible rows. A virtual move cannot improve by
        // removing its supervised competitor; absent baseline comparisons skip.
        if(tr==0)continue;++eligible;
"#;

const CONSUMER_PREFIX: &str = r#"    for(U row=0;row<s[4];++row){int q[MAX_WIDTH];U prediction,tr,ar;
        lane.words[2]|=(W)1<<row;
        // A speculative producer error is visible only if this lane reaches it.
        U row_status=baseline_status[row];
        if(row_status){lane.words[5]=row_status;atomicExch(status,row_status);return;}
        if(baseline_eligible[row]==0)continue;++eligible;
        lane.words[3]|=(W)1<<row;lane.words[4]|=(W)1<<row;
        U arithmetic_status=0;
"#;

const AUDIT: &str = r#"
// 64 packed row records followed by sixteen complete six-word lane records.
// Audit stores are part of the same R image used for controls and timing.
struct ReuseLane {
    W* output;W words[6];
    __device__ ReuseLane(W* audit,U candidate):output(audit+64+candidate*6){
        for(U i=0;i<6;++i)words[i]=0;
    }
    __device__ ~ReuseLane(){for(U i=0;i<6;++i)output[i]=words[i];}
};
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn separate_entry_preserves_helpers_and_virtual_body() {
        let source = include_str!("discrete.cu");
        let generated = generate(source).expect("admitted source anchors");
        let prefix_end = source.find("// Statuses added").expect("writer end");
        assert!(generated.starts_with(&source[..prefix_end]));
        assert_eq!(generated.matches("__global__").count(), 1);
        assert_eq!(generated.matches("__syncthreads();").count(), 1);
        let (before, after) = generated.split_once("__syncthreads();").expect("barrier");
        let producer = before
            .split_once("void discrete_counterfactual_baseline_reuse(")
            .expect("entry")
            .1;
        assert!(!producer.contains("return;"));
        assert!(!producer.contains("atomicExch"));
        assert!(after.contains("if(cost==~(W)0 || tr==0){sum=~(W)0;break;}"));
        assert!(after.contains("sum+=cost;"));
        assert!(!generated.contains("clock64"));
        assert!(!generated.contains("__maxnreg__"));
        assert!(!generated.contains("discrete_apply("));
        assert!(generate(&source.replace(
            "if(tr==0)continue;++eligible;",
            "if (tr == 0) continue; ++eligible;"
        ))
        .is_err());
        assert!(
            generate(&source.replace("atomicExch(status,12u);", "atomicExch(status,13u);"))
                .is_err()
        );
    }
}
