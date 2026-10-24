//! Fail-closed source transformation for a separate diagnostic CUDA module.
//! The ordinary CUDA source is never edited. Every insertion is delimited so
//! the ordinary model operations remain visible at their original anchors.

fn insert_once(source: &mut String, needle: &str, replacement: &str) -> Result<(), String> {
    if source.matches(needle).count() != 1 {
        return Err(format!("decomposition source anchor changed: {needle}"));
    }
    *source = source.replacen(needle, replacement, 1);
    Ok(())
}

fn probe(code: &str) -> String {
    format!("/*PROBE_BEGIN*/{code}/*PROBE_END*/")
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
    let helpers = &source[..writer_start];
    let mut writer = source[writer_start..writer_end].to_owned();
    insert_once(
        &mut writer,
        "U* arithmetic_status=0)",
        &format!(
            "U* arithmetic_status=0{}{}",
            probe(",volatile W* probe_words=0,U probe_base=0"),
            ")"
        ),
    )?;
    insert_once(
        &mut writer,
        "    bool occupied",
        &format!(
            "{}    bool occupied",
            probe("ProbeSpan probe_occupancy(probe_words+probe_base);")
        ),
    )?;
    insert_once(
        &mut writer,
        "    U best_token",
        &format!(
            "{}    U best_token",
            probe("probe_occupancy.finish();ProbeSpan probe_members(probe_words+probe_base+1);")
        ),
    )?;
    insert_once(
        &mut writer,
        "    *prediction=best_token",
        &format!(
            "{}    *prediction=best_token",
            probe("probe_members.finish();ProbeSpan probe_final(probe_words+probe_base+2);")
        ),
    )?;
    // Count function calls using the executed branches, without per-token clocks.
    writer = writer.replace(
        "token_cell(",
        &format!("{}token_cell(", probe("(++probe_words[probe_base+4],")),
    );
    // Close the comma expressions at the three exact call sites.
    for anchor in [
        "candidate);if(c==ABSENT)",
        "candidate);\n    *token_rivals",
        "candidate)==cell)",
    ] {
        let replacement = anchor.replacen(')', &format!("){}", probe(")")), 1);
        insert_once(&mut writer, anchor, &replacement)?;
    }
    insert_once(
        &mut writer,
        "used[cell]=true;",
        &format!("used[cell]=true;{}", probe("++probe_words[probe_base+3];")),
    )?;
    insert_once(
        &mut writer,
        "U d=token_distance(q,table,g,s,token,kind,index,candidate);",
        &format!(
            "{}U d=token_distance(q,table,g,s,token,kind,index,candidate);",
            probe("++probe_words[probe_base+5];")
        ),
    )?;

    let mut kernel = source[kernel_start..kernel_end].to_owned();
    insert_once(
        &mut kernel,
        "W* losses,U* status)",
        &format!("W* losses,U* status{})", probe(",W* records")),
    )?;
    insert_once(
        &mut kernel,
        "if(candidate>=16)return;",
        &format!(
            "if(candidate>=16)return;{}",
            probe("ProbeLane lane(records,candidate);volatile W* t=lane.words;")
        ),
    )?;
    insert_once(
        &mut kernel,
        "for(U row=0;row<s[4];++row){",
        &format!(
            "for(U row=0;row<s[4];++row){{{}",
            probe("++t[7];t[8]|=(W)1<<row;")
        ),
    )?;
    let base_read = "if(!read_vector(input,ids,routes,gates,plan,g,s,position_steps,row,2,0,0,baseline,0)){atomicExch(status,2u);return;}";
    insert_once(
        &mut kernel,
        base_read,
        &format!(
            "{}{base_read}{}",
            probe("{++t[11];ProbeSpan span(t+15);"),
            probe("}")
        ),
    )?;
    let base_write = "W base=writer(baseline,table,g,s,targets[row],2,0,0,&prediction,&tr,&ar,0,0,0,0,&arithmetic_status);";
    let modified = base_write
        .replacen("W base=", "base=", 1)
        .replace("&arithmetic_status)", "&arithmetic_status,t,21)");
    // Keep the original declaration shape outside the timing scope.
    insert_once(
        &mut kernel,
        base_write,
        &format!(
            "W base;{}{}{}",
            probe("{++t[12];ProbeSpan span(t+16);"),
            modified,
            probe("}")
        ),
    )?;
    insert_once(
        &mut kernel,
        "if(tr==0)continue;++eligible;",
        &format!(
            "{}if(tr==0)continue;++eligible;{}",
            probe("{ProbeSpan span(t+17);"),
            probe("++t[9];t[10]|=(W)1<<row;}")
        ),
    )?;
    let virtual_read = "if(!read_vector(input,ids,routes,gates,plan,g,s,position_steps,row,kind,index,candidate,q,0)){atomicExch(status,2u);return;}";
    insert_once(
        &mut kernel,
        virtual_read,
        &format!(
            "{}{virtual_read}{}",
            probe("{++t[13];ProbeSpan span(t+18);"),
            probe("}")
        ),
    )?;
    let virtual_write = "W cost=writer(q,table,g,s,targets[row],kind,index,candidate,&prediction,&tr,&ar,0,0,0,0,&arithmetic_status);";
    let modified = virtual_write
        .replacen("W cost=", "cost=", 1)
        .replace("&arithmetic_status)", "&arithmetic_status,t,27)");
    insert_once(
        &mut kernel,
        virtual_write,
        &format!(
            "W cost;{}{}{}",
            probe("{++t[14];ProbeSpan span(t+19);"),
            modified,
            probe("}")
        ),
    )?;
    insert_once(
        &mut kernel,
        "        if(arithmetic_status){",
        &format!(
            "{}        if(arithmetic_status){{",
            probe("{ProbeSpan span(t+20);")
        ),
    )?;
    insert_once(
        &mut kernel,
        "sum+=cost;",
        &format!("sum+=cost;{}", probe("}")),
    )?;
    // Preserve every original atomic/error/return; record its lane-local code too.
    for code in [
        "9u",
        "4u",
        "2u",
        "arithmetic_status?arithmetic_status:8u",
        "arithmetic_status",
        "12u",
        "8u",
    ] {
        let needle = format!("atomicExch(status,{code});");
        kernel = kernel.replace(
            &needle,
            &format!("{}{needle}", probe(&format!("t[3]={code};"))),
        );
    }
    // Names are the only non-probe changes; the diagnostic module has one entry.
    writer = writer.replacen("W writer(", "W writer_decomposition(", 1);
    kernel = kernel
        .replace(
            "void discrete_counterfactual(",
            "void discrete_counterfactual_decomposition(",
        )
        .replace("=writer(", "=writer_decomposition(");
    Ok(format!("{helpers}\n{PROBES}\n{writer}\n{kernel}"))
}

const PROBES: &str = r#"
// Clocks are integer observation metadata, with compiler memory barriers.
__device__ __forceinline__ W probe_clock() { W now;asm volatile("mov.u64 %0, %%clock64;" : "=l"(now) :: "memory");return now; }
struct ProbeSpan {
    volatile W* volatile destination;volatile W begin;volatile bool active;
    __device__ ProbeSpan(volatile W* out):destination(out),begin(probe_clock()),active(true){}
    __device__ void finish(){if(active){*destination+=probe_clock()-begin;active=false;}}
    __device__ ~ProbeSpan(){finish();}
};
struct ProbeLane {
    W* output;volatile W words[40];
    __device__ ProbeLane(W* records,U candidate):output(records+candidate*40){
        W begin=probe_clock();for(U i=0;i<40;++i)words[i]=0;
        words[0]=0x434631;words[1]=candidate;words[2]=1;words[5]=begin;
    }
    __device__ ~ProbeLane(){
        words[6]=probe_clock();words[4]=words[6]-words[5];
        // One terminal record per lane, including all ordinary early returns.
        // Output stores are outside lane cycles and inside the enclosing event.
        for(U i=0;i<40;++i)output[i]=words[i];
    }
};
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changed_production_anchor_is_an_instrument_blocker() {
        assert!(generate("missing source").is_err());
        let source = include_str!("discrete.cu");
        let generated = generate(source).expect("admitted source anchors");
        assert_eq!(generated.matches("__global__").count(), 1);
        assert!(generated.contains("void discrete_counterfactual_decomposition("));
        assert!(!generated.contains("__maxnreg__"));
        assert!(generated.contains("writer_decomposition(q,"));
        assert!(!generated.contains("discrete_apply("));
        assert!(generate(&source.replace("sum+=cost;", "sum = sum + cost;")).is_err());
    }
}
