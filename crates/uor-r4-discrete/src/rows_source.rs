//! Separate opt-in row-parallel exact counterfactual, derived from the unchanged
//! ordinary helpers and writer. The ordinary `discrete_counterfactual` runs one
//! block of sixteen lanes, each looping over every row serially and recomputing
//! the baseline per lane. This module computes each row's baseline once (one
//! thread per row), each (row, lane) virtual move once (one thread per pair), and
//! then replays the ordinary lane loop over those records in row order (one
//! thread per lane). Losses, chooser input and published status codes are those
//! of the ordinary lane loop; speculative records past a lane's first terminal
//! row are never read, so they can never publish a status.
//!
//! The writer sets `arithmetic_status` only on the path that also returns the
//! MAX sentinel (`discrete.cu` writer), so a baseline that returned a loss left
//! it zero and a fresh zero before the virtual writer is the ordinary value.

pub const ENTRIES: [&str; 3] = [
    "discrete_counterfactual_rows_baseline",
    "discrete_counterfactual_rows_eval",
    "discrete_counterfactual_rows_reduce",
];

pub fn generate(source: &str) -> Result<String, String> {
    let writer_end = source
        .find("// Statuses added")
        .ok_or("rows source anchor changed: writer end missing")?;
    // The ordinary lane loop these kernels replay. If its text changes, the
    // replay must be re-derived and re-reviewed rather than silently diverge.
    if source.matches(ORDINARY_LANE_LOOP).count() != 1 {
        return Err("rows source anchor changed: ordinary lane loop differs".into());
    }
    let helpers_and_writer = &source[..writer_end];
    Ok(format!("{helpers_and_writer}\n{ROWS}"))
}

/// Exact text of the ordinary kernel body from the refusals through the final
/// loss store (`discrete.cu`, `discrete_counterfactual`).
pub const ORDINARY_LANE_LOOP: &str = r#"    W sum=0;U eligible=0;losses[candidate]=~(W)0;
    bool gated=s[8]==MODE_FLOCK || s[8]==MODE_RELATION;
    if((kind==GATE_KIND && !gated) || (kind==0 && gated)){atomicExch(status,9u);return;}
    if(kind==GATE_KIND && index>=gate_count(s)){atomicExch(status,4u);return;}
    if(kind==GATE_KIND && candidate>=3)return;
    for(U row=0;row<s[4];++row){int q[MAX_WIDTH],baseline[MAX_WIDTH];U prediction,tr,ar;
        if(!read_vector(input,ids,routes,gates,plan,g,s,row,2,0,0,baseline,0)){atomicExch(status,2u);return;}
        U arithmetic_status=0;
        W base=writer(baseline,table,g,s,targets[row],2,0,0,&prediction,&tr,&ar,0,0,0,0,&arithmetic_status);
        if(base==~(W)0){atomicExch(status,arithmetic_status?arithmetic_status:8u);return;}
        // The baseline fixes eligible rows. A virtual move cannot improve by
        // removing its supervised competitor; absent baseline comparisons skip.
        if(tr==0)continue;++eligible;
        if(!read_vector(input,ids,routes,gates,plan,g,s,row,kind,index,candidate,q,0)){atomicExch(status,2u);return;}
        W cost=writer(q,table,g,s,targets[row],kind,index,candidate,&prediction,&tr,&ar,0,0,0,0,&arithmetic_status);
        if(arithmetic_status){atomicExch(status,arithmetic_status);return;}
        if(cost==~(W)0 || tr==0){sum=~(W)0;break;}
        if(~(W)0-sum<cost){atomicExch(status,12u);return;}sum+=cost;
    }
    if(eligible==0){atomicExch(status,8u);return;}
    losses[candidate]=sum;
"#;

const ROWS: &str = r#"
// ---- Row-parallel exact counterfactual (opt-in) ----
// Records, written for every admitted row/pair and read only in ordinary order:
//   base[row*2]   baseline terminal status (0 = baseline loss returned)
//   base[row*2+1] baseline eligibility (token_rivals != 0)
//   cost[row*16+lane]  virtual writer loss (MAX when not evaluated)
//   flag[row*16+lane]  low 16 bits: virtual terminal status (2 read, else the
//                      writer's arithmetic_status); bit 16: virtual tr != 0
#define ROWS_TR_BIT 0x10000u
#define ROWS_STATUS_MASK 0xffffu
__device__ bool rows_refused(const U* s,U kind,U index) {
    bool gated=s[8]==MODE_FLOCK || s[8]==MODE_RELATION;
    return (kind==GATE_KIND && !gated) || (kind==0 && gated) || (kind==GATE_KIND && index>=gate_count(s));
}
extern "C" __global__ void discrete_counterfactual_rows_baseline(const int* table,const U* routes,const int* gates,const U* ids,const int* input,const U* targets,const U* plan,const U* g,const U* s,U* base) {
    U row=blockIdx.x*blockDim.x+threadIdx.x;if(row>=s[4])return;
    int baseline[MAX_WIDTH];U prediction,tr,ar,arithmetic_status=0;
    U row_status=0,row_eligible=0;
    if(!read_vector(input,ids,routes,gates,plan,g,s,row,2,0,0,baseline,0))row_status=2u;
    else {
        W loss=writer(baseline,table,g,s,targets[row],2,0,0,&prediction,&tr,&ar,0,0,0,0,&arithmetic_status);
        if(loss==~(W)0)row_status=arithmetic_status?arithmetic_status:8u;
        else row_eligible=tr!=0;
    }
    base[row*2]=row_status;base[row*2+1]=row_eligible;
}
extern "C" __global__ void discrete_counterfactual_rows_eval(const int* table,const U* routes,const int* gates,const U* ids,const int* input,const U* targets,const U* plan,const U* g,const U* s,U kind,U index,const U* base,W* cost,U* flag) {
    U pair=blockIdx.x*blockDim.x+threadIdx.x;U row=pair/16,candidate=pair%16;
    if(row>=s[4])return;
    U slot=row*16+candidate;cost[slot]=~(W)0;flag[slot]=0;
    // Refused calls and inactive Gate lanes never read; the reducer publishes
    // the ordinary refusal before any row is consulted.
    if(rows_refused(s,kind,index) || (kind==GATE_KIND && candidate>=3))return;
    // The ordinary lane evaluates a virtual move only on an eligible row whose
    // baseline returned a loss; other rows are never consumed.
    if(base[row*2]!=0 || base[row*2+1]==0)return;
    int q[MAX_WIDTH];U prediction,tr,ar,arithmetic_status=0;
    if(!read_vector(input,ids,routes,gates,plan,g,s,row,kind,index,candidate,q,0)){flag[slot]=2u;return;}
    W loss=writer(q,table,g,s,targets[row],kind,index,candidate,&prediction,&tr,&ar,0,0,0,0,&arithmetic_status);
    // The writer can return MAX before it stores token_rivals; tr is read only
    // after a returned loss (the reducer tests MAX first, as the ordinary does).
    cost[slot]=loss;flag[slot]=(arithmetic_status&ROWS_STATUS_MASK)|(loss!=~(W)0 && tr!=0?ROWS_TR_BIT:0u);
}
extern "C" __global__ void discrete_counterfactual_rows_reduce(const U* s,U kind,U index,const U* base,const W* cost,const U* flag,W* losses,U* status) {
    U candidate=blockIdx.x*blockDim.x+threadIdx.x;if(candidate>=16)return;
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

    #[test]
    fn ordinary_lane_loop_anchor_matches_the_admitted_kernel() {
        let source = include_str!("discrete.cu");
        assert_eq!(source.matches(ORDINARY_LANE_LOOP).count(), 1);
    }

    #[test]
    fn generated_module_keeps_helpers_and_adds_exactly_three_entries() {
        let source = include_str!("discrete.cu");
        let generated = generate(source).expect("admitted anchors");
        let prefix_end = source.find("// Statuses added").expect("writer end");
        assert!(generated.starts_with(&source[..prefix_end]));
        assert_eq!(generated.matches("__global__").count(), 3);
        for entry in ENTRIES {
            assert_eq!(
                generated.matches(&format!("void {entry}(")).count(),
                1,
                "{entry}"
            );
        }
        // The ordinary entry and the learning-state writers are not part of it.
        assert!(!generated.contains("void discrete_counterfactual("));
        assert!(!generated.contains("discrete_apply("));
        assert!(!generated.contains("__syncthreads"));
        assert!(!generated.contains("clock64"));
    }

    #[test]
    fn only_the_reducer_publishes_status() {
        let generated = generate(include_str!("discrete.cu")).expect("anchors");
        let rows = generated
            .split_once("// ---- Row-parallel")
            .expect("rows")
            .1;
        let (producers, reducer) = rows
            .split_once("void discrete_counterfactual_rows_reduce(")
            .expect("reducer");
        assert!(!producers.contains("atomicExch"));
        assert!(!producers.contains("losses["));
        // The reducer replays the ordinary terminal order exactly.
        let order = [
            "atomicExch(status,9u)",
            "atomicExch(status,4u)",
            "candidate>=3)return;",
            "atomicExch(status,row_status)",
            "if(base[row*2+1]==0)continue;++eligible;",
            "atomicExch(status,virtual_status)",
            "{sum=~(W)0;break;}",
            "atomicExch(status,12u)",
            "atomicExch(status,8u)",
            "losses[candidate]=sum;",
        ];
        let mut at = 0;
        for needle in order {
            let found = reducer[at..]
                .find(needle)
                .unwrap_or_else(|| panic!("{needle}"));
            at += found + needle.len();
        }
    }

    #[test]
    fn a_changed_ordinary_loop_refuses_generation() {
        let source = include_str!("discrete.cu");
        assert!(generate(&source.replace(
            "if(cost==~(W)0 || tr==0){sum=~(W)0;break;}",
            "if(cost==~(W)0){sum=~(W)0;break;}"
        ))
        .is_err());
        assert!(generate(&source.replace("// Statuses added", "// statuses")).is_err());
    }
}
