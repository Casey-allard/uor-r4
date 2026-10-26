//! Separate ahead-of-time `nvcc`-compiled CUDA module for the Stage-1 wider code
//! carrier (`m1-wider-code-carrier`, #2847). Reuses the unchanged `discrete.cu`
//! helpers (`read_vector`, `address_distance`, `token_distance`, `token_cell`,
//! `cell_precedes_w`) so `discrete.cu` and `driver_source_sha256` stay untouched.

pub const ENTRIES: [&str; 2] = [
    "discrete_wider_carrier_cell_distances",
    "discrete_wider_carrier_token_distances",
];

pub fn generate(source: &str) -> Result<String, String> {
    let writer_end = source
        .find("// Statuses added")
        .ok_or("wider carrier source anchor changed: writer end missing")?;
    if source.matches("W cell_distance(").count() != 1 {
        return Err("wider carrier source anchor changed: cell_distance missing".into());
    }
    let helpers_and_writer = &source[..writer_end];
    Ok(format!("{helpers_and_writer}\n{WIDER_CARRIER}"))
}

const WIDER_CARRIER: &str = r#"
// ---- Wider code carrier qualification & inspection kernels (#2847) ----
// Supports B in 2..=16 blocks of 8 integers (width 16..=128).
// For s[15] == 2 (two-scale full-vector cell metric), B <= 8 uses (env << 8) + sum
// (identical to discrete.cu cell_distance; strictly lexicographical at quota <= 16
// since 16*(d+4) - (d + 15*8*B) + 1024 = 1088 - 120*B + 15*d >= 128 > 0 for B <= 8),
// and B in 9..=16 uses (env << 11) + sum.
__device__ W wider_cell_distance(const int* q,const U* g,const U* s,U cell,const U* env,const W* sum) {
    if(s[15]==0u)return address_distance(q,g,s,cell);
    if(s[15]==1u)return (W)env[cell];
    U shift=s[1]<=8u?8u:11u;
    return ((W)env[cell]<<shift)+sum[cell];
}
extern "C" __global__ void discrete_wider_carrier_cell_distances(
    const int* table,const U* routes,const int* gates,const U* ids,const int* input,
    const U* plan,const U* g,const U* s,const U* position_steps,
    int* out_q,U* out_env,W* out_sum,W* out_cell_dist,U* out_admitted,U* status
) {
    U row=blockIdx.x*blockDim.x+threadIdx.x;if(row>=s[4])return;
    if(s[1]<2u || s[1]>16u || s[3]==0u || s[3]>16u || s[15]>2u){atomicExch(status,13u);return;}
    int q[MAX_WIDTH];
    if(!read_vector(input,ids,routes,gates,plan,g,s,position_steps,row,2,0,0,q,0)){atomicExch(status,2u);return;}
    U width=s[1]*8u;
    for(U d=0;d<width;++d)out_q[row*width+d]=q[d];
    bool occupied[256],used[256];U env[256];W sum[256];
    for(U c=0;c<256;++c){occupied[c]=false;used[c]=false;env[c]=ABSENT;sum[c]=0;}
    for(U token=0;token<s[0];++token){
        U c=token_cell(table,g,s,token,2,0,0);
        if(c==ABSENT || c>=256u){atomicExch(status,8u);return;}
        occupied[c]=true;
        U e=token_distance(q,table,g,s,token,2,0,0);
        if(e<env[c])env[c]=e;
        sum[c]+=e;
    }
    for(U c=0;c<256;++c){
        out_env[row*256+c]=env[c];
        out_sum[row*256+c]=sum[c];
        out_cell_dist[row*256+c]=occupied[c]?wider_cell_distance(q,g,s,c,env,sum):~(W)0;
    }
    for(U k=0;k<16;++k)out_admitted[row*16+k]=ABSENT;
    for(U k=0;k<s[3];++k){
        U cell=ABSENT;W cost=~(W)0;
        for(U c=0;c<256;++c)if(occupied[c] && !used[c]){
            W d=wider_cell_distance(q,g,s,c,env,sum);
            if(cell_precedes_w(d,c,cost,cell)){cell=c;cost=d;}
        }
        if(cell==ABSENT)break;
        used[cell]=true;
        out_admitted[row*16+k]=cell;
    }
}
extern "C" __global__ void discrete_wider_carrier_token_distances(
    const int* table,const U* g,const U* s,const int* queries,U query_count,
    U* out_full_l1,U* out_outer_l1,U* out_interior_l1,U* status
) {
    U idx=blockIdx.x*blockDim.x+threadIdx.x;
    U total=query_count*s[0];
    if(idx>=total)return;
    if(s[1]<2u || s[1]>16u){atomicExch(status,13u);return;}
    U q_idx=idx/s[0],token=idx%s[0];
    const int* q=queries+q_idx*(s[1]*8u);
    U c=token_cell(table,g,s,token,2,0,0);
    if(c==ABSENT || c>=256u){atomicExch(status,8u);return;}
    U full=token_distance(q,table,g,s,token,2,0,0);
    U outer=address_distance(q,g,s,c);
    if(full<outer){atomicExch(status,12u);return;}
    out_full_l1[idx]=full;
    out_outer_l1[idx]=outer;
    out_interior_l1[idx]=full-outer;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wider_carrier_module_generates_two_entries_from_admitted_helpers() {
        let source = include_str!("discrete.cu");
        let generated = generate(source).expect("wider carrier generation");
        let prefix_end = source.find("// Statuses added").expect("writer end");
        assert!(generated.starts_with(&source[..prefix_end]));
        assert_eq!(generated.matches("__global__").count(), 2);
        for entry in ENTRIES {
            assert_eq!(
                generated.matches(&format!("void {entry}(")).count(),
                1,
                "{entry}"
            );
        }
        assert!(!generated.contains("void discrete_counterfactual("));
        assert!(!generated.contains("discrete_apply("));
    }
}
