//! Opt-in K-coordinate row-parallel scoring for BVG-K (W3, Stage C), derived
//! from the generated row-parallel module. One launch scores K coordinates of
//! one kind: the shared per-row baseline is the row-parallel baseline kernel,
//! the eval kernel adds a coordinate axis to the row-parallel eval, and the
//! reducer replays the ordinary lane loop per (coordinate, lane) with one
//! status word per coordinate. Each coordinate's losses and status are those of
//! an independent row-parallel proposal on the same state (control V2), which
//! is itself exact to the ordinary kernel (#2630). The row-parallel image is not
//! changed: this is a separate module that appends two entries to its text.

pub const ENTRIES: [&str; 5] = [
    "discrete_counterfactual_rows_baseline",
    "discrete_counterfactual_rows_eval",
    "discrete_counterfactual_rows_reduce",
    "discrete_counterfactual_rowsk_eval",
    "discrete_counterfactual_rowsk_reduce",
];

/// Append the K-coordinate entries to the generated row-parallel source.
pub fn generate(rows_source: &str) -> Result<String, String> {
    if !rows_source.contains("extern \"C\" __global__ void discrete_counterfactual_rows_reduce(") {
        return Err("rows-K anchor changed: row-parallel reducer missing".into());
    }
    // The reducer replays this exact row-parallel lane loop per coordinate. If
    // its text changes, the K replay must be re-derived and re-reviewed.
    if rows_source.matches(ROWS_LANE_LOOP).count() != 1 {
        return Err("rows-K anchor changed: row-parallel reducer loop differs".into());
    }
    Ok(format!("{rows_source}\n{ROWSK}"))
}

/// Exact text of the row-parallel reducer's row loop (`rows_source.rs`).
pub const ROWS_LANE_LOOP: &str = r#"    for(U row=0;row<s[4];++row){
        U row_status=base[row*2];
        if(row_status){atomicExch(status,row_status);return;}
        if(base[row*2+1]==0)continue;++eligible;
        U slot=row*16+candidate,f=flag[slot],virtual_status=f&ROWS_STATUS_MASK;
        if(virtual_status){atomicExch(status,virtual_status);return;}
        W c=cost[slot];
        if(c==~(W)0 || (f&ROWS_TR_BIT)==0){sum=~(W)0;break;}
        if(~(W)0-sum<c){atomicExch(status,12u);return;}sum+=c;
    }"#;

const ROWSK: &str = r#"
// ---- K-coordinate row-parallel scoring (opt-in, BVG-K Stage C) ----
// Records, per coordinate c in 0..k (all one kind), at offset c*rows*16:
//   cost[c*rows*16 + row*16 + lane], flag[...] as in the row-parallel eval.
// losses[c*16 + lane]; status[c] is coordinate c's own status word.
extern "C" __global__ void discrete_counterfactual_rowsk_eval(const int* table,const U* routes,const int* gates,const U* ids,const int* input,const U* targets,const U* plan,const U* g,const U* s,U kind,const U* indices,U k,const U* base,W* cost,U* flag) {
    U per=s[4]*16;U pair=blockIdx.x*blockDim.x+threadIdx.x;
    if(per==0)return;
    U c=pair/per,rest=pair%per,row=rest/16,candidate=rest%16;
    if(c>=k || row>=s[4])return;
    U index=indices[c];U slot=c*per+row*16+candidate;cost[slot]=~(W)0;flag[slot]=0;
    if(rows_refused(s,kind,index) || (kind==GATE_KIND && candidate>=3))return;
    if(base[row*2]!=0 || base[row*2+1]==0)return;
    int q[MAX_WIDTH];U prediction,tr,ar,arithmetic_status=0;
    if(!read_vector(input,ids,routes,gates,plan,g,s,row,kind,index,candidate,q,0)){flag[slot]=2u;return;}
    W loss=writer(q,table,g,s,targets[row],kind,index,candidate,&prediction,&tr,&ar,0,0,0,0,&arithmetic_status);
    cost[slot]=loss;flag[slot]=(arithmetic_status&ROWS_STATUS_MASK)|(loss!=~(W)0 && tr!=0?ROWS_TR_BIT:0u);
}
extern "C" __global__ void discrete_counterfactual_rowsk_reduce(const U* s,U kind,const U* indices,U k,const U* base,const W* all_cost,const U* all_flag,W* all_losses,U* all_status) {
    U t=blockIdx.x*blockDim.x+threadIdx.x;U c=t/16,candidate=t%16;if(c>=k)return;
    U index=indices[c];U per=s[4]*16;
    const W* cost=all_cost+c*per;const U* flag=all_flag+c*per;W* losses=all_losses+c*16;U* status=all_status+c;
    W sum=0;U eligible=0;losses[candidate]=~(W)0;
    bool gated=s[8]==MODE_FLOCK || s[8]==MODE_RELATION;
    if((kind==GATE_KIND && !gated) || (kind==0 && gated)){atomicExch(status,9u);return;}
    if(kind==GATE_KIND && index>=gate_count(s)){atomicExch(status,4u);return;}
    if(kind==GATE_KIND && candidate>=3)return;
    for(U row=0;row<s[4];++row){
        U row_status=base[row*2];
        if(row_status){atomicExch(status,row_status);return;}
        if(base[row*2+1]==0)continue;++eligible;
        U slot=row*16+candidate,f=flag[slot],virtual_status=f&ROWS_STATUS_MASK;
        if(virtual_status){atomicExch(status,virtual_status);return;}
        W c=cost[slot];
        if(c==~(W)0 || (f&ROWS_TR_BIT)==0){sum=~(W)0;break;}
        if(~(W)0-sum<c){atomicExch(status,12u);return;}sum+=c;
    }
    if(eligible==0){atomicExch(status,8u);return;}
    losses[candidate]=sum;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> String {
        crate::rows_source::generate(include_str!("discrete.cu")).expect("rows module")
    }

    #[test]
    fn rowsk_module_appends_exactly_two_entries_to_the_rows_module() {
        let rows = rows();
        let rowsk = generate(&rows).expect("rows-K module");
        assert!(
            rowsk.starts_with(&rows),
            "the row-parallel text is a prefix, unchanged"
        );
        let entries: Vec<&str> = rowsk
            .match_indices("extern \"C\" __global__ void ")
            .map(|(at, prefix)| {
                let name = &rowsk[at + prefix.len()..];
                &name[..name.find('(').expect("entry signature")]
            })
            .collect();
        assert_eq!(entries, ENTRIES);
    }

    #[test]
    fn rowsk_reducer_replays_the_row_parallel_loop_with_a_coordinate_offset() {
        // The K reducer's loop is the row-parallel loop verbatim; only the
        // pointers it reads are offset per coordinate.
        assert_eq!(ROWSK.matches(ROWS_LANE_LOOP).count(), 1);
    }

    #[test]
    fn changed_anchors_are_refused() {
        let rows = rows();
        assert!(generate(&rows.replace(ROWS_LANE_LOOP, "    for(;;){}")).is_err());
        assert!(generate("no reducer here").is_err());
    }
}
