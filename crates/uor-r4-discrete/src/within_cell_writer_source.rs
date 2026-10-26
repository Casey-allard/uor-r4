//! Separate ahead-of-time `nvcc`-compiled CUDA module for `m1-within-cell-writer`
//! (`#3267`, referencing `#3128` and `#3129`). Reuses the unchanged `discrete.cu`
//! geometry and writer helpers so `discrete.cu`, `compiled_identity`, and
//! `driver_source_sha256` remain untouched.

pub const ENTRIES: [&str; 2] = [
    "discrete_within_cell_snap_orbit_check",
    "discrete_within_cell_writer_eval",
];

pub fn generate(source: &str) -> Result<String, String> {
    let writer_end = source
        .find("// Statuses added")
        .ok_or("within-cell writer source anchor changed: writer end missing")?;
    if source.matches("W writer(").count() != 1 {
        return Err("within-cell writer source anchor changed: writer missing".into());
    }
    let helpers_and_writer = &source[..writer_end];
    Ok(format!("{helpers_and_writer}\n{WITHIN_CELL_WRITER}"))
}

const WITHIN_CELL_WRITER: &str = r#"
// ---- Versioned ReadLawV2 (#3128) & Two-Pass / Framing B0 Evaluator (#3129, #3267) ----
// wc_cfg[0]: read_law_version (0 = ReadLawV1 legacy, 1 = ReadLawV2 weighted snap + capped rel)
// wc_cfg[1]: frame_mode (
//   0 = CurrentSinglePassV1: f(0)=3, f(1)=0, f(2)=0, f(3)=0, fq from cursor
//   1 = NearestEndpointSinglePassV1: f(0)=3, f(1)=0, f(2)=3, f(3)=0, fq from cursor
//   2 = TwoPassEndpointOutputV1: Pass 1 runs b in {0,3}; Pass 2 uses out_0 for b=1 (f=0) and out_3 for b=2 (f=3)
//   3 = FrozenConstantFrameV1: interior b in {1,2} use constant frame root (wc_cfg[4] & 15)
//   4 = PermutedFrameV1: Pass 1 runs b in {0,3}; Pass 2 uses frame_remap[out_0] and frame_remap[16 + out_3]
//   5 = TrainRemap16V1: single-pass interior b in {1,2} use frame_remap[cursor_r0] and frame_remap[16 + cursor_r3]
// )
// wc_cfg[2]: transport_mode (0 = RelativeFrameTransportV1, 1 = GlobalWriterFrameV1 on leaf carry)
// wc_cfg[3]: max_rel_bound (positive integer cap checked on |rel[i]| in ReadLawV2; status 14 on overflow)
// wc_cfg[4]: frozen_frame_root (0..15 for frame_mode == 3)
// wc_cfg[5]: bypass_refusal_mask (0 in all normal runs; bitmask for negative-gate unit tests only:
//   bit 1 = bypass s[1] != 4 check, bit 2 = bypass s[14] >= 3 check,
//   bit 4 = bypass s[13] != 0 check, bit 8 = bypass max_rel_bound check)

#define RECENT_MATCH_CAP 8u

extern "C" __global__ void discrete_within_cell_snap_orbit_check(
    const U* g,U* out_v1,U* out_v2_pos,U* out_v2_neg,U* status
) {
    U idx=blockIdx.x*blockDim.x+threadIdx.x;
    if(idx>=256u)return;
    U rq=idx/16u,rv=idx%16u;
    int u_v1[8],u_v2_pos[8],u_v2_neg[8];
    for(U d=0;d<8;++d){
        int qd=(int)g[ROOT+rq*8+d];
        int vd=(int)g[ROOT+rv*8+d];
        u_v1[d]=qd+vd;
        u_v2_pos[d]=qd+(vd<<1);
        u_v2_neg[d]=qd-(vd<<1);
    }
    U s1=snap(u_v1,2u,g);
    U s2p=snap(u_v2_pos,3u,g);
    U s2n=snap(u_v2_neg,3u,g);
    if(s1>=16u || s2p>=16u || s2n>=16u){atomicExch(status,8u);return;}
    out_v1[idx]=s1;
    out_v2_pos[idx]=s2p;
    out_v2_neg[idx]=s2n;
}

__device__ bool flock_vector_versioned(
    const int* input,const U* ids,const int* gates,const U* plan,const U* g,const U* s,
    const U* position_steps,const U* wc_cfg,const U* frame_remap,
    U row,int* output,U* trace,U* refusal_status
) {
    U read_law=wc_cfg[0],frame_mode=wc_cfg[1],transport_mode=wc_cfg[2];
    U max_rel_bound=wc_cfg[3],frozen_root=wc_cfg[4]&15u,bypass_mask=wc_cfg[5];
    U eff_s13=(wc_cfg[6]!=0u && wc_cfg[6]!=99u)?wc_cfg[6]:s[13];
    U eff_s14=wc_cfg[7]!=0u?wc_cfg[7]:s[14];
    if(read_law>1u || frame_mode>5u || transport_mode>1u){*refusal_status=13u;return false;}
    if((bypass_mask&1u)==0u && s[1]!=4u){*refusal_status=15u;return false;}
    if((bypass_mask&2u)==0u && eff_s14>=3u){*refusal_status=16u;return false;}
    if((bypass_mask&4u)==0u && eff_s13!=0u){*refusal_status=17u;return false;}
    if((bypass_mask&16u)==0u && wc_cfg[6]==99u){*refusal_status=10u;return false;}
    plan+=row*(3+s[6]*4+16);
    U length=plan[0],nodes=plan[1];plan+=3;
    U kept=s[9],sweeps=s[10];
    if(length==0 || length>s[2] || nodes>s[6] || nodes>MAX_NODES || kept==0 || kept>MAX_KEPT || s[1]<2){*refusal_status=2u;return false;}
    if(s[8]==MODE_RELATION && (sweeps==0 || sweeps>MAX_SWEEPS)){*refusal_status=10u;return false;}
    if(s[12]>2u){*refusal_status=10u;return false;}
    U level[MAX_NODES],first[MAX_NODES],last[MAX_NODES],frame[MAX_NODES];
    for(U node=0;node<nodes;++node) {
        U atom=plan[node*4+3];
        if(atom!=ABSENT){if(atom>=length){*refusal_status=2u;return false;}level[node]=0;first[node]=atom;last[node]=atom;}
        else {
            U c0=plan[node*4],c2=plan[node*4+2];
            if(c0>=node || plan[node*4+1]>=node || c2>=node){*refusal_status=2u;return false;}
            level[node]=level[c0]+1;first[node]=first[c0];last[node]=last[c2];
        }
    }
    // Endpoint blocks execute first (0, s[1]-1), followed by interior blocks (1..s[1]-2),
    // so Pass-1 predicted endpoints are available for TwoPass and PermutedFrame modes.
    U pass1_out_first=0u,pass1_out_last=0u;
    U total_blocks=s[1];
    for(U step=0;step<total_blocks;++step) {
        U b;
        if(step==0u)b=0u;
        else if(step==1u)b=total_blocks-1u;
        else b=step-1u;
        bool is_interior=(b>0u && b+1u<total_blocks);
        U f=frame_block(b,total_blocks);
        if(is_interior && frame_mode!=0u) {
            f=(b==1u)?0u:(total_blocks-1u);
        }
        int query_frame[8],u[8];
        input_block(input,ids,g,row,length-1,f,s,2,0,0,query_frame);
        input_block(input,ids,g,row,length-1,b,s,2,0,0,u);
        U fq=root_id(query_frame,g);
        if(fq==ABSENT){*refusal_status=2u;return false;}
        if(is_interior) {
            if(frame_mode==2u) {
                fq=(b==1u)?pass1_out_first:pass1_out_last;
            } else if(frame_mode==3u) {
                fq=frozen_root;
            } else if(frame_mode==4u) {
                U src=(b==1u)?pass1_out_first:pass1_out_last;
                U off=(b==1u)?0u:16u;
                fq=frame_remap[off+src]&15u;
            } else if(frame_mode==5u) {
                U off=(b==1u)?0u:16u;
                fq=frame_remap[off+fq]&15u;
            }
            for(U d=0;d<8;++d)query_frame[d]=(int)g[ROOT+fq*8+d];
        }
        int query_payload[8];for(U d=0;d<8;++d)query_payload[d]=u[d];
        int* query_position=s[14]==0?query_frame:query_payload;
        U action_query=fq;
        if(s[14]==2){action_query=root_id(u,g);if(action_query==ABSENT){*refusal_status=2u;return false;}}
        unsigned char prefix[MAX_POSITIONS];
        if(s[14]==1) {
            U acc=g[IDENTITY];
            for(U p=0;p<length;++p) {
                int fv[8];input_block(input,ids,g,row,p,b,s,2,0,0,fv);
                U root=root_id(fv,g);if(root==ABSENT){*refusal_status=2u;return false;}
                acc=compose(g,position_step(position_steps,b,root,g),acc);
                prefix[p]=(unsigned char)acc;
            }
        }
        int payload[MAX_NODES*8];
        for(U node=0;node<nodes;++node) {
            U atom=plan[node*4+3];
            if(atom!=ABSENT) {
                int fv[8];input_block(input,ids,g,row,atom,f,s,2,0,0,fv);
                frame[node]=root_id(fv,g);if(frame[node]==ABSENT){*refusal_status=2u;return false;}
                input_block(input,ids,g,row,atom,b,s,2,0,0,payload+node*8);
            } else {
                U c0=plan[node*4];frame[node]=frame[c0];
                int sum[8]={0,0,0,0,0,0,0,0};
                for(U c=0;c<3;++c){U child=plan[node*4+c];int carried[8];
                    action(g,g[BFS+frame[child]*16+frame[c0]],payload+child*8,carried);
                    for(U d=0;d<8;++d)sum[d]+=carried[d];}
                U r=snap(sum,s[12]==1?1u:3u,g);for(U d=0;d<8;++d)payload[node*8+d]=(int)g[ROOT+r*8+d];
            }
        }
        unsigned char transport_root[MAX_NODES],transport[MAX_NODES];
        for(U node=0;node<nodes;++node) {
            U tr=frame[node];
            if(s[14]!=0) {
                int rv[8];input_block(input,ids,g,row,last[node],b,s,2,0,0,rv);
                tr=root_id(rv,g);if(tr==ABSENT){*refusal_status=2u;return false;}
            }
            transport_root[node]=(unsigned char)tr;
            transport[node]=(unsigned char)position_action(position_steps,prefix,g,s,b,tr,action_query,last[node],length-1);
        }
        int rel[MAX_NODES];
        for(U node=0;node<nodes;++node)rel[node]=0;
        if(s[8]==MODE_RELATION) {
            for(U rg=0;rg<EDGE_TYPES*s[5];++rg) {
                int gt=gates[s[1]*s[5]+rg];
                if(gt<-1 || gt>1){*refusal_status=10u;return false;}
            }
            int prev[MAX_NODES];
            for(U node=0;node<nodes;++node) {
                U distance,hamming;
                if(!positioned_root_cost(s,query_position,g,transport[node],transport_root[node],&distance,&hamming)){*refusal_status=2u;return false;}
                prev[node]=8-(int)distance;
            }
            U query_tid=ids[row*s[2]+(length-1)];
            for(U sweep=0;sweep<sweeps;++sweep) {
                if(read_law==1u) {
                    for(U i=0;i<nodes;++i)rel[i]=0;
                }
                for(U i=0;i<nodes;++i) {
                    if(last[i]>=length)continue;
                    U li=level[i];
                    for(U e=0;e<EDGE_TYPES;++e) {
                        int gt=gates[s[1]*s[5]+li*EDGE_TYPES+e];
                        if(gt==0)continue;
                        int esum=0;
                        if(e==0) {
                            if(plan[i*4+3]==ABSENT) {
                                for(U c=0;c<3;++c){U ch=plan[i*4+c];if(last[ch]<length)esum+=prev[ch];}
                            }
                            for(U p=i+1;p<nodes;++p) {
                                if(plan[p*4+3]==ABSENT && last[p]<length && (plan[p*4]==i || plan[p*4+1]==i || plan[p*4+2]==i))esum+=prev[p];
                            }
                        } else if(e==1) {
                            for(U p=i+1;p<nodes;++p) {
                                if(plan[p*4+3]!=ABSENT || last[p]>=length)continue;
                                U c0=plan[p*4],c1=plan[p*4+1],c2=plan[p*4+2];
                                if(i==c0 && last[c1]<length)esum+=prev[c1];
                                else if(i==c1){if(last[c0]<length)esum+=prev[c0];if(last[c2]<length)esum+=prev[c2];}
                                else if(i==c2 && last[c1]<length)esum+=prev[c1];
                            }
                        } else if(e==2) {
                            for(U j=0;j<i;++j) {
                                if(last[j]<length && level[j]==li && last[j]+1==first[i]) {
                                    if(read_law==1u && li==0u && ids[row*s[2]+last[j]]==query_tid)esum+=16;
                                    else esum+=prev[j];
                                }
                            }
                        } else if(e==3) {
                            if(li==0) {
                                U tid=ids[row*s[2]+last[i]];
                                if(read_law==0u) {
                                    for(U j=0;j<i;++j)if(last[j]<length && level[j]==0 && ids[row*s[2]+last[j]]==tid)esum=prev[j];
                                } else {
                                    U matches=0u;
                                    if(sweep==0u && last[i]+1<length && tid==query_tid) {
                                        esum+=8;
                                        ++matches;
                                    }
                                    U j=i;
                                    while(j>0u && matches<RECENT_MATCH_CAP) {
                                        --j;
                                        if(last[j]<length && level[j]==0 && ids[row*s[2]+last[j]]==tid) {
                                            esum+=prev[j];
                                            ++matches;
                                        }
                                    }
                                }
                            }
                        } else if(e==4) {
                            if(read_law==0u) {
                                for(U j=0;j<i;++j)if(last[j]<length && level[j]==li && frame[j]==frame[i])esum=prev[j];
                            } else {
                                U matches=0u,j=i;
                                while(j>0u && matches<RECENT_MATCH_CAP) {
                                    --j;
                                    if(last[j]<length && level[j]==li && frame[j]==frame[i]) {
                                        esum+=prev[j];
                                        ++matches;
                                    }
                                }
                            }
                        }
                        rel[i]+=gt>0?esum:-esum;
                        if(read_law==1u && (bypass_mask&8u)==0u && (U)magnitude(rel[i])>max_rel_bound) {
                            *refusal_status=14u;
                            return false;
                        }
                    }
                }
                for(U i=0;i<nodes;++i)prev[i]=rel[i];
            }
            if(trace) {
                U* rtrace=trace+(s[4]*s[1])*(TRACE_ROW+s[5]*TRACE_LEVEL)+(row*s[1]+b)*s[6];
                for(U i=0;i<s[6];++i)rtrace[i]=i<nodes?(U)rel[i]:ABSENT;
            }
        }
        U* rec=trace ? trace+(row*s[1]+b)*(TRACE_ROW+s[5]*TRACE_LEVEL) : 0;
        if(rec){for(U i=0;i<TRACE_ROW+s[5]*TRACE_LEVEL;++i)rec[i]=ABSENT;rec[0]=fq;rec[3]=root_id(u,g);}
        U terms=1u;
        for(U l=0;l<s[5];++l) {
            int gate=gates[b*s[5]+l];
            if(gate<-1 || gate>1){*refusal_status=10u;return false;}
            U* lv=rec ? rec+TRACE_ROW+l*TRACE_LEVEL : 0;
            if(lv)lv[0]=(U)(gate+1);
            if(gate==0 && !lv)continue;
            U top[MAX_KEPT],key_d[MAX_KEPT],key_h[MAX_KEPT],found=0,eligible=0;
            int key_r[MAX_KEPT];
            for(U node=0;node<nodes;++node) {
                if(level[node]!=l || last[node]+1>=length)continue;
                ++eligible;
                U distance,hamming;
                if(!positioned_root_cost(s,query_position,g,transport[node],transport_root[node],&distance,&hamming)){*refusal_status=2u;return false;}
                int rscore=rel[node];
                U at=found;
                while(at>0) {
                    U o=top[at-1];
                    bool better=rscore>key_r[at-1] || (rscore==key_r[at-1] && (distance<key_d[at-1]
                        || (distance==key_d[at-1] && (hamming<key_h[at-1]
                        || (hamming==key_h[at-1] && (last[node]>last[o] || (last[node]==last[o] && node<o)))))));
                    if(!better)break;
                    if(at<kept){top[at]=top[at-1];key_r[at]=key_r[at-1];key_d[at]=key_d[at-1];key_h[at]=key_h[at-1];}
                    --at;
                }
                if(at<kept){top[at]=node;key_r[at]=rscore;key_d[at]=distance;key_h[at]=hamming;if(found<kept)++found;}
            }
            if(lv){lv[1]=found;lv[3]=eligible;}
            if(found==0)continue;
            int sum[8]={0,0,0,0,0,0,0,0};
            for(U k=0;k<found;++k){
                U node=top[k];int carried[8];
                U a=(transport_mode==1u && l==0u)?g[IDENTITY]:transport[node];
                action(g,a,payload+node*8,carried);
                for(U d=0;d<8;++d)sum[d]+=carried[d];
                if(lv){lv[4+k]=node;lv[12+k]=a;lv[20+k]=root_id(carried,g);}
            }
            U m=snap(sum,s[12]==1?1u:found,g);
            if(lv)lv[2]=m;
            if(gate==0)continue;
            if(read_law==0u) {
                for(U d=0;d<8;++d){int v=(int)g[ROOT+m*8+d];u[d]+=gate>0?v:-v;}
                ++terms;
            } else {
                for(U d=0;d<8;++d){int v=(int)g[ROOT+m*8+d];int wv=v<<1;u[d]+=gate>0?wv:-wv;}
                terms+=2u;
            }
        }
        U out=snap(u,s[12]==1?1u:terms,g);
        if(b==0u)pass1_out_first=out;
        else if(b+1u==total_blocks)pass1_out_last=out;
        if(s[12]==2){if(rec){rec[1]=ABSENT;rec[2]=terms;}for(U d=0;d<8;++d)output[b*8+d]=u[d];continue;}
        if(rec){rec[1]=out;rec[2]=terms;}
        for(U d=0;d<8;++d)output[b*8+d]=(int)g[ROOT+out*8+d];
    }
    return true;
}

extern "C" __global__ void discrete_within_cell_writer_eval(
    const int* table,const U* routes,const int* gates,const U* ids,const int* input,
    const U* targets,const U* plan,const U* g,const U* s,const U* position_steps,
    const U* wc_cfg,const U* frame_remap,
    int* out_q,U* out_admitted_cells,U* out_top8_tokens,U* out_metrics,U* out_trace,U* status
) {
    U row=blockIdx.x*blockDim.x+threadIdx.x;if(row>=s[4])return;
    int q[MAX_WIDTH];
    U refusal=0u;
    if(!flock_vector_versioned(input,ids,gates,plan,g,s,position_steps,wc_cfg,frame_remap,row,q,out_trace,&refusal)) {
        atomicExch(status,refusal?refusal:2u);
        return;
    }
    U width=s[1]*8u;
    for(U d=0;d<width;++d)out_q[row*width+d]=q[d];
    if((wc_cfg[5]&32u)!=0u)return;
    U cells[16],prediction=ABSENT,tr=0,ar=0,arithmetic_status=0;W tm=0,am=0,telemetry[6];
    W total=writer(q,table,g,s,targets[row],2,0,0,&prediction,&tr,&ar,cells,&tm,&am,telemetry,&arithmetic_status);
    if(total==~(W)0){atomicExch(status,arithmetic_status?arithmetic_status:8u);return;}
    for(U i=0;i<16;++i)out_admitted_cells[row*16+i]=cells[i];

    U ranked[8],distance[8],found=0,cell_count=0;
    for(U i=0;i<8;++i){ranked[i]=ABSENT;distance[i]=ABSENT;}
    for(U i=0;i<16;++i)if(cells[i]!=ABSENT)++cell_count;

    U target_tok=targets[row];
    U target_cell=token_cell(table,g,s,target_tok,2,0,0);
    U target_dist=token_distance(q,table,g,s,target_tok,2,0,0);
    U within_cell_rivals=0u,cross_cell_rivals=0u,admitted_token_count=0u;

    for(U token=0;token<s[0];++token) {
        U cell=token_cell(table,g,s,token,2,0,0);
        bool admitted=false;
        for(U i=0;i<16;++i)if(cells[i]==cell){admitted=true;break;}
        if(!admitted)continue;
        ++admitted_token_count;
        U d=token_distance(q,table,g,s,token,2,0,0);
        if(token!=target_tok && (d<target_dist || (d==target_dist && token<target_tok))) {
            if(cell==target_cell)++within_cell_rivals;
            else ++cross_cell_rivals;
        }
        U at=found;
        while(at>0) {
            U prev=at-1;
            if(!(d<distance[prev] || (d==distance[prev] && token<ranked[prev])))break;
            if(at<8){ranked[at]=ranked[prev];distance[at]=distance[prev];}
            --at;
        }
        if(at<8){ranked[at]=token;distance[at]=d;if(found<8)++found;}
    }
    U hit1=ranked[0]==target_tok?1u:0u,hit8=0u;
    for(U i=0;i<8;++i){
        out_top8_tokens[row*8+i]=ranked[i];
        if(ranked[i]==target_tok)hit8=1u;
    }
    U target_admitted=(cell_count>ar)?1u:0u;
    U* m=out_metrics+row*8;
    m[0]=target_admitted;
    m[1]=hit1;
    m[2]=hit8;
    m[3]=(U)telemetry[1];
    m[4]=admitted_token_count;
    m[5]=tr;
    m[6]=within_cell_rivals;
    m[7]=cross_cell_rivals;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn within_cell_writer_module_generates_two_entries_from_admitted_helpers() {
        let source = include_str!("discrete.cu");
        let generated = generate(source).expect("within_cell_writer generation");
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
