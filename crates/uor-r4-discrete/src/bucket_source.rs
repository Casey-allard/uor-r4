//! Ahead-of-time CUDA source generator for the CSR bucketed writer (`m1-faster-writer`, #2840).
//!
//! Preserves all helper device functions in `discrete.cu` up to `// Statuses added`
//! and emits seven admitted `.entry` kernels in a separate `bucket.ptx` / `bucket.cubin`
//! module without modifying `discrete.cu` or `ordinary.cubin`:
//! - `discrete_bucket_build`: builds the 256-cell CSR index `(offsets, members, cells, absent_count)` on GPU.
//! - `discrete_bucket_update`: incrementally updates the CSR index after a single-token move on GPU.
//! - `discrete_bucket_writer_eval`: evaluates `bucket_writer` on pre-supplied query vectors `q` and records exact per-row comparison counters.
//! - `discrete_fullscan_writer_eval`: evaluates the exact `discrete.cu` full-scan `writer` on pre-supplied query vectors `q` and records exact per-row comparison counters.
//! - `discrete_bucket_forward`: drop-in `read_vector` + `bucket_writer` forward kernel.
//! - `discrete_bucket_counterfactual`: drop-in 16-lane `read_vector` + `bucket_writer` counterfactual kernel.
//! - `discrete_bucket_evaluate`: drop-in `read_vector` + `bucket_writer` evaluation kernel producing `top8` and 14 metric words per row.

pub const ENTRIES: [&str; 7] = [
    "discrete_bucket_build",
    "discrete_bucket_update",
    "discrete_bucket_writer_eval",
    "discrete_fullscan_writer_eval",
    "discrete_bucket_forward",
    "discrete_bucket_counterfactual",
    "discrete_bucket_evaluate",
];

pub fn generate(source: &str) -> Result<String, String> {
    let writer_end = source
        .find("// Statuses added")
        .ok_or("bucket source anchor changed: writer end missing")?;
    if source.matches(ORDINARY_WRITER).count() != 1 {
        return Err("bucket source anchor changed: ordinary writer body differs".into());
    }
    let helpers_and_writer = &source[..writer_end];
    Ok(format!("{helpers_and_writer}\n{BUCKET}"))
}

/// Exact text of `writer` in `src/discrete.cu`. If `discrete.cu` changes its writer
/// implementation, `generate` refuses at build time rather than silently diverging.
pub const ORDINARY_WRITER: &str = r#"__device__ W writer(const int* q,const int* table,const U* g,const U* s,U target,U kind,U index,U candidate,U* prediction,U* token_rivals,U* address_rivals,U* admitted,W* token_hinge=0,W* address_hinge=0,W* telemetry=0,U* arithmetic_status=0) {
    if(s[15]>2u){if(arithmetic_status)*arithmetic_status=13;return ~(W)0;}
    bool occupied[256],used[256];U env[256];W sum[256];for(U i=0;i<256;++i){occupied[i]=false;used[i]=false;}
    if(s[15]!=0u)for(U i=0;i<256;++i){env[i]=ABSENT;sum[i]=0;}
    for(U token=0;token<s[0];++token){U c=token_cell(table,g,s,token,kind,index,candidate);if(c==ABSENT)return ~(W)0;occupied[c]=true;
        if(s[15]!=0u){U e=token_distance(q,table,g,s,token,kind,index,candidate);if(e<env[c])env[c]=e;sum[c]+=e;}}
    U best_token=ABSENT,best_distance=ABSENT,rival_distance=ABSENT;W rival_address=~(W)0;
    U target_cell=token_cell(table,g,s,target,kind,index,candidate);
    *token_rivals=0;*address_rivals=0;if(admitted)for(U i=0;i<16;++i)admitted[i]=ABSENT;
    for(U k=0;k<s[3];++k) {
        U cell=ABSENT;W cost=~(W)0;
        for(U c=0;c<256;++c)if(occupied[c] && !used[c]){W d=cell_distance(q,g,s,c,env,sum);if(cell_precedes_w(d,c,cost,cell)){cell=c;cost=d;}}
        if(cell==ABSENT)break;
        used[cell]=true;if(admitted)admitted[k]=cell;
        if(cell!=target_cell){++*address_rivals;W h=s[15]==2u?(W)env[cell]:cost;if(h<rival_address)rival_address=h;}
        for(U token=0;token<s[0];++token)if(token_cell(table,g,s,token,kind,index,candidate)==cell){
            U d=token_distance(q,table,g,s,token,kind,index,candidate);
            if(d<best_distance || (d==best_distance && token<best_token)){best_token=token;best_distance=d;}
            if(token!=target){++*token_rivals;if(d<rival_distance)rival_distance=d;}
        }
    }
    *prediction=best_token;if(best_token==ABSENT)return ~(W)0;
    U width=s[1]*8,td=token_distance(q,table,g,s,target,kind,index,candidate);W ad=cell_distance(q,g,s,target_cell,env,sum);
    W tm=rival_distance==ABSENT || width+td<=rival_distance ? 0 : width+td-rival_distance;
    W ah=s[15]==2u?(W)env[target_cell]:ad;
    W am=rival_address==~(W)0 || 8+ah<=rival_address ? 0 : 8+ah-rival_address;
    U rank=1,occupancy=0;
    if(s[11]==1 || telemetry)for(U cell=0;cell<256;++cell)if(occupied[cell]){
        ++occupancy;W distance=cell_distance(q,g,s,cell,env,sum);
        if(cell_precedes_w(distance,cell,ad,target_cell))++rank;
    }
    W rm=rank>s[3]?rank-s[3]:0;
    W weighted_token=checked_scaled_add(tm,8),weighted_address=checked_scaled_add(am,width),weighted_rank=checked_scaled_add(rm,width);
    if(weighted_token==~(W)0 || weighted_address==~(W)0 || weighted_rank==~(W)0
        || ~(W)0-weighted_token<weighted_address || ~(W)0-weighted_token<weighted_rank){
        if(arithmetic_status)*arithmetic_status=12;return ~(W)0;
    }
    W loss_ta=weighted_token+weighted_address,loss_tr=weighted_token+weighted_rank;
    if(telemetry){telemetry[0]=rm;telemetry[1]=rank;telemetry[2]=occupancy;
        telemetry[3]=*token_rivals+used[target_cell];telemetry[4]=loss_tr;telemetry[5]=s[11]==1?loss_tr:loss_ta;}
    if(token_hinge)*token_hinge=tm;if(address_hinge)*address_hinge=am;
    return s[11]==1?loss_tr:loss_ta;
}"#;

const BUCKET: &str = r#"
// ---- CSR Bucketed Writer (m1-faster-writer, #2840) ----
// Walk modes: 0 = IdOrder (full writer parity), 1 = ServingPruned (skip cell when address_distance > best_distance).
// Planted fault modes (§8 negative controls):
//   0 = None
//   1 = StaleBucket (handled by skipping discrete_bucket_update on host/caller)
//   2 = ReversedTie (token > best_token on equal token_distance)
//   3 = OverPruned (skip cell when address_distance >= best_distance in ServingPruned)
//   4 = CorruptedCsr (corrupt cell 0 CSR slice end)
#define BUCKET_WALK_ID_ORDER 0u
#define BUCKET_WALK_SERVING_PRUNED 1u
#define BUCKET_FAULT_NONE 0u
#define BUCKET_FAULT_STALE_BUCKET 1u
#define BUCKET_FAULT_REVERSED_TIE 2u
#define BUCKET_FAULT_OVER_PRUNED 3u
#define BUCKET_FAULT_CORRUPTED_CSR 4u

__device__ W bucket_writer(
    const int* q,const int* table,const U* g,const U* s,
    const U* offsets,const U* members,const U* cells,const U* absent_count,
    U target,U kind,U index,U candidate,
    U walk_mode,U fault_mode,
    U* prediction,U* token_rivals,U* address_rivals,U* admitted,
    W* token_hinge,W* address_hinge,W* telemetry,U* arithmetic_status,
    U* top8_out,U* counters_out
) {
    U c_token_cell=0u,c_cell_dist=0u,c_token_dist=0u,c_max_bucket=0u;
    if(s[15]!=0u){
        if(arithmetic_status)*arithmetic_status=13u;
        if(counters_out){counters_out[0]=0u;counters_out[1]=0u;counters_out[2]=0u;counters_out[3]=0u;}
        return ~(W)0;
    }
    bool has_move=(fault_mode!=BUCKET_FAULT_STALE_BUCKET && kind==1u && candidate!=0u && index<s[0]*s[1]);
    U moved_token=has_move?index/s[1]:ABSENT;
    U moved_old_cell=has_move?cells[moved_token]:ABSENT;
    U moved_new_cell=ABSENT;
    if(has_move){
        ++c_token_cell;
        moved_new_cell=token_cell(table,g,s,moved_token,kind,index,candidate);
    }
    U cur_absent=absent_count[0];
    if(has_move){
        if(moved_old_cell==ABSENT && cur_absent>0u)--cur_absent;
        if(moved_new_cell==ABSENT)++cur_absent;
    }
    if(cur_absent>0u || target>=s[0]){
        if(counters_out){counters_out[0]=c_token_cell;counters_out[1]=c_cell_dist;counters_out[2]=c_token_dist;counters_out[3]=c_max_bucket;}
        return ~(W)0;
    }
    U target_cell=(has_move && target==moved_token)?moved_new_cell:cells[target];
    if(target_cell>=256u){
        if(counters_out){counters_out[0]=c_token_cell;counters_out[1]=c_cell_dist;counters_out[2]=c_token_dist;counters_out[3]=c_max_bucket;}
        return ~(W)0;
    }
    bool occupied[256],used[256];
    W dist_cell[256];
    U corrupted_cell=ABSENT;
    if(fault_mode==BUCKET_FAULT_CORRUPTED_CSR){
        for(U c=0;c<256u;++c)if(offsets[c+1]>offsets[c]){corrupted_cell=c;break;}
    }
    for(U c=0;c<256u;++c){
        U b_start=offsets[c],b_end=offsets[c+1];
        if(c==corrupted_cell)b_end=b_start;
        U cnt=b_end>=b_start?b_end-b_start:0u;
        if(has_move && moved_old_cell!=moved_new_cell){
            if(c==moved_old_cell && cnt>0u)--cnt;
            if(c==moved_new_cell)++cnt;
        }
        if(cnt>c_max_bucket)c_max_bucket=cnt;
        occupied[c]=(cnt>0u);
        used[c]=false;
        ++c_cell_dist;
        dist_cell[c]=(W)address_distance(q,g,s,c);
    }
    U best_token=ABSENT,best_distance=ABSENT,rival_distance=ABSENT;
    W rival_address=~(W)0;
    *token_rivals=0u;*address_rivals=0u;
    if(admitted)for(U i=0;i<16u;++i)admitted[i]=ABSENT;
    U adm_cells[16],adm_count=0u;
    for(U k=0;k<s[3];++k){
        U cell=ABSENT;W cost=~(W)0;
        for(U c=0;c<256u;++c)if(occupied[c] && !used[c]){
            W d=dist_cell[c];
            bool cell_win=(fault_mode==BUCKET_FAULT_REVERSED_TIE)?(d<cost || (d==cost && c>cell)):cell_precedes_w(d,c,cost,cell);
            if(cell_win){cell=c;cost=d;}
        }
        if(cell==ABSENT)break;
        used[cell]=true;
        if(admitted)admitted[k]=cell;
        adm_cells[adm_count++]=cell;
        if(cell!=target_cell){
            ++*address_rivals;
            if(cost<rival_address)rival_address=cost;
        }
    }
    U ranked[8],rdist[8],found=0u;
    if(top8_out)for(U i=0;i<8u;++i){ranked[i]=ABSENT;rdist[i]=ABSENT;}
    U walk_cells[16];
    for(U i=0;i<adm_count;++i)walk_cells[i]=adm_cells[i];
    if(walk_mode==BUCKET_WALK_ID_ORDER){
        for(U i=1;i<adm_count;++i){
            U v=walk_cells[i],j=i;
            while(j>0u && walk_cells[j-1]>v){walk_cells[j]=walk_cells[j-1];--j;}
            walk_cells[j]=v;
        }
    }
    for(U idx_k=0;idx_k<adm_count;++idx_k){
        U cell=walk_cells[idx_k];
        if(walk_mode==BUCKET_WALK_SERVING_PRUNED && best_distance!=ABSENT){
            if(fault_mode==BUCKET_FAULT_OVER_PRUNED){
                if(dist_cell[cell]>=(W)best_distance)continue;
            } else {
                if(dist_cell[cell]>(W)best_distance)continue;
            }
        }
        U b_start=offsets[cell],b_end=offsets[cell+1];
        if(cell==corrupted_cell)b_end=b_start;
        for(U pos=b_start;pos<b_end;++pos){
            U token=members[pos];
            if(has_move && token==moved_token && moved_new_cell!=cell)continue;
            ++c_token_dist;
            U d=token_distance(q,table,g,s,token,kind,index,candidate);
            bool tie_win=(fault_mode==BUCKET_FAULT_REVERSED_TIE)?(token>best_token):(token<best_token);
            if(d<best_distance || (d==best_distance && tie_win)){best_token=token;best_distance=d;}
            if(token!=target){++*token_rivals;if(d<rival_distance)rival_distance=d;}
            if(top8_out){
                U at=found;
                while(at>0u){
                    U prev=at-1u;
                    bool top_tie=(fault_mode==BUCKET_FAULT_REVERSED_TIE)?(token>ranked[prev]):(token<ranked[prev]);
                    if(!(d<rdist[prev] || (d==rdist[prev] && top_tie)))break;
                    if(at<8u){ranked[at]=ranked[prev];rdist[at]=rdist[prev];}
                    --at;
                }
                if(at<8u){ranked[at]=token;rdist[at]=d;if(found<8u)++found;}
            }
        }
        if(has_move && moved_new_cell==cell && moved_old_cell!=cell){
            U token=moved_token;
            ++c_token_dist;
            U d=token_distance(q,table,g,s,token,kind,index,candidate);
            bool tie_win=(fault_mode==BUCKET_FAULT_REVERSED_TIE)?(token>best_token):(token<best_token);
            if(d<best_distance || (d==best_distance && tie_win)){best_token=token;best_distance=d;}
            if(token!=target){++*token_rivals;if(d<rival_distance)rival_distance=d;}
            if(top8_out){
                U at=found;
                while(at>0u){
                    U prev=at-1u;
                    bool top_tie=(fault_mode==BUCKET_FAULT_REVERSED_TIE)?(token>ranked[prev]):(token<ranked[prev]);
                    if(!(d<rdist[prev] || (d==rdist[prev] && top_tie)))break;
                    if(at<8u){ranked[at]=ranked[prev];rdist[at]=rdist[prev];}
                    --at;
                }
                if(at<8u){ranked[at]=token;rdist[at]=d;if(found<8u)++found;}
            }
        }
    }
    if(top8_out)for(U i=0;i<8u;++i)top8_out[i]=ranked[i];
    *prediction=best_token;
    if(best_token==ABSENT){
        if(counters_out){counters_out[0]=c_token_cell;counters_out[1]=c_cell_dist;counters_out[2]=c_token_dist;counters_out[3]=c_max_bucket;}
        return ~(W)0;
    }
    U width=s[1]*8u,td=token_distance(q,table,g,s,target,kind,index,candidate);
    if(counters_out){counters_out[0]=c_token_cell;counters_out[1]=c_cell_dist;counters_out[2]=c_token_dist;counters_out[3]=c_max_bucket;}
    W ad=dist_cell[target_cell];
    W tm=rival_distance==ABSENT || width+td<=rival_distance ? 0 : width+td-rival_distance;
    W ah=ad;
    W am=rival_address==~(W)0 || 8+ah<=rival_address ? 0 : 8+ah-rival_address;
    U rank=1u,occupancy=0u;
    if(s[11]==1u || telemetry)for(U cell=0;cell<256u;++cell)if(occupied[cell]){
        ++occupancy;W distance=dist_cell[cell];
        if(cell_precedes_w(distance,cell,ad,target_cell))++rank;
    }
    W rm=rank>s[3]?rank-s[3]:0;
    W weighted_token=checked_scaled_add(tm,8u),weighted_address=checked_scaled_add(am,width),weighted_rank=checked_scaled_add(rm,width);
    if(weighted_token==~(W)0 || weighted_address==~(W)0 || weighted_rank==~(W)0
        || ~(W)0-weighted_token<weighted_address || ~(W)0-weighted_token<weighted_rank){
        if(arithmetic_status)*arithmetic_status=12u;return ~(W)0;
    }
    W loss_ta=weighted_token+weighted_address,loss_tr=weighted_token+weighted_rank;
    if(telemetry){telemetry[0]=rm;telemetry[1]=rank;telemetry[2]=occupancy;
        telemetry[3]=*token_rivals+used[target_cell];telemetry[4]=loss_tr;telemetry[5]=s[11]==1u?loss_tr:loss_ta;}
    if(token_hinge)*token_hinge=tm;if(address_hinge)*address_hinge=am;
    return s[11]==1u?loss_tr:loss_ta;
}

extern "C" __global__ void discrete_bucket_build(const int* table,const U* g,const U* s,U* offsets,U* members,U* cells,U* absent_count,U* status) {
    if(blockIdx.x!=0u)return;
    for(U token=threadIdx.x;token<s[0];token+=blockDim.x){
        cells[token]=token_cell(table,g,s,token,2u,0u,0u);
    }
    __syncthreads();
    if(threadIdx.x!=0u)return;
    for(U c=0;c<=256u;++c)offsets[c]=0u;
    U abs_cnt=0u;
    for(U token=0;token<s[0];++token){
        U c=cells[token];
        if(c==ABSENT)++abs_cnt;
        else if(c<256u)++offsets[c+1u];
        else {*status=8u;return;}
    }
    *absent_count=abs_cnt;
    U running=0u;
    for(U c=0;c<=256u;++c){
        running+=offsets[c];
        offsets[c]=running;
    }
    U cursor[256];
    for(U c=0;c<256u;++c)cursor[c]=offsets[c];
    for(U token=0;token<s[0];++token){
        U c=cells[token];
        if(c!=ABSENT)members[cursor[c]++]=token;
    }
}

extern "C" __global__ void discrete_bucket_update(const int* table,const U* g,const U* s,U moved_token,U* offsets,U* members,U* cells,U* absent_count,U* entries_written,U* status) {
    if(blockIdx.x || threadIdx.x)return;
    if(moved_token>=s[0]){*status=4u;return;}
    U old_cell=cells[moved_token];
    U new_cell=token_cell(table,g,s,moved_token,2u,0u,0u);
    if(new_cell!=ABSENT && new_cell>=256u){*status=8u;return;}
    if(new_cell==old_cell){*entries_written=0u;return;}
    U written=0u;
    cells[moved_token]=new_cell;++written;
    U total_present=offsets[256];
    // Locate old position if old_cell != ABSENT
    U remove_pos=ABSENT;
    if(old_cell!=ABSENT){
        U b_start=offsets[old_cell],b_end=offsets[old_cell+1u];
        for(U pos=b_start;pos<b_end;++pos){
            if(members[pos]==moved_token){remove_pos=pos;break;}
        }
        if(remove_pos==ABSENT){*status=8u;return;}
    } else if(*absent_count>0u){
        --*absent_count;++written;
    }
    if(new_cell==ABSENT){
        // Shift left from remove_pos..total_present-1
        for(U pos=remove_pos;pos+1u<total_present;++pos){
            members[pos]=members[pos+1u];++written;
        }
        for(U c=old_cell+1u;c<=256u;++c){--offsets[c];++written;}
        ++*absent_count;++written;
        *entries_written=written;
        return;
    }
    if(old_cell==ABSENT){
        // Insert into new_cell maintaining ascending token order
        U b_start=offsets[new_cell],b_end=offsets[new_cell+1u];
        U insert_pos=b_start;
        while(insert_pos<b_end && members[insert_pos]<moved_token)++insert_pos;
        for(U pos=total_present;pos>insert_pos;--pos){
            members[pos]=members[pos-1u];++written;
        }
        members[insert_pos]=moved_token;++written;
        for(U c=new_cell+1u;c<=256u;++c){++offsets[c];++written;}
        *entries_written=written;
        return;
    }
    // Both old_cell and new_cell are in 0..256 and old_cell != new_cell.
    if(old_cell<new_cell){
        U b_start=offsets[new_cell],b_end=offsets[new_cell+1u];
        U insert_before=b_start;
        while(insert_before<b_end && members[insert_before]<moved_token)++insert_before;
        // After removing remove_pos (< insert_before), target slot is insert_before - 1
        U target_pos=insert_before-1u;
        for(U pos=remove_pos;pos<target_pos;++pos){
            members[pos]=members[pos+1u];++written;
        }
        members[target_pos]=moved_token;++written;
        for(U c=old_cell+1u;c<=new_cell;++c){--offsets[c];++written;}
    } else {
        U b_start=offsets[new_cell],b_end=offsets[new_cell+1u];
        U insert_pos=b_start;
        while(insert_pos<b_end && members[insert_pos]<moved_token)++insert_pos;
        for(U pos=remove_pos;pos>insert_pos;--pos){
            members[pos]=members[pos-1u];++written;
        }
        members[insert_pos]=moved_token;++written;
        for(U c=new_cell+1u;c<=old_cell;++c){++offsets[c];++written;}
    }
    *entries_written=written;
}

extern "C" __global__ void discrete_bucket_writer_eval(
    const int* q_rows,const int* table,const U* g,const U* s,
    const U* offsets,const U* members,const U* cells,const U* absent_count,
    const U* targets,U kind,U index,U candidate,U walk_mode,U fault_mode,
    U* predictions,W* losses,U* token_rivals,U* address_rivals,U* admitted,
    U* top8,W* metrics,U* counters,U* status
) {
    U row=blockIdx.x*blockDim.x+threadIdx.x;if(row>=s[4])return;
    const int* q=q_rows+row*s[1]*8u;
    U pred=ABSENT,tr=0u,ar=0u,arith=0u;
    W tm=0,am=0,telemetry[6]={0,0,0,0,0,0};
    U row_cells[16],row_top8[8],row_counters[4]={0,0,0,0};
    W loss=bucket_writer(
        q,table,g,s,offsets,members,cells,absent_count,
        targets[row],kind,index,candidate,walk_mode,fault_mode,
        &pred,&tr,&ar,row_cells,&tm,&am,telemetry,&arith,row_top8,row_counters
    );
    predictions[row]=pred;
    losses[row]=loss;
    token_rivals[row]=tr;
    address_rivals[row]=ar;
    for(U i=0;i<16u;++i)admitted[row*16u+i]=row_cells[i];
    for(U i=0;i<4u;++i)counters[row*4u+i]=row_counters[i];
    if(loss==~(W)0){
        for(U i=0;i<8u;++i)top8[row*8u+i]=ABSENT;
        for(U i=0;i<14u;++i)metrics[row*14u+i]=0;
        atomicExch(status,arith?arith:8u);
        return;
    }
    U cell_count=0u;
    for(U i=0;i<16u;++i)if(row_cells[i]!=ABSENT)++cell_count;
    W hit8=0;
    for(U i=0;i<8u;++i){
        top8[row*8u+i]=row_top8[i];
        if(row_top8[i]==targets[row])hit8=1;
    }
    W* out=metrics+row*14u;
    out[0]=tm;out[1]=am;out[2]=(tm<<3)+scaled_add(am,s[1]*8u);out[3]=tr;out[4]=ar;
    out[5]=row_top8[0]==targets[row];out[6]=hit8;out[7]=cell_count;
    for(U field=0;field<6u;++field)out[8u+field]=telemetry[field];
}

extern "C" __global__ void discrete_fullscan_writer_eval(
    const int* q_rows,const int* table,const U* g,const U* s,
    const U* targets,U kind,U index,U candidate,
    U* predictions,W* losses,U* token_rivals,U* address_rivals,U* admitted,
    U* top8,W* metrics,U* counters,U* status
) {
    U row=blockIdx.x*blockDim.x+threadIdx.x;if(row>=s[4])return;
    const int* q=q_rows+row*s[1]*8u;
    U pred=ABSENT,tr=0u,ar=0u,arith=0u;
    W tm=0,am=0,telemetry[6]={0,0,0,0,0,0};
    U row_cells[16];
    W loss=writer(q,table,g,s,targets[row],kind,index,candidate,&pred,&tr,&ar,row_cells,&tm,&am,telemetry,&arith);
    predictions[row]=pred;
    losses[row]=loss;
    token_rivals[row]=tr;
    address_rivals[row]=ar;
    for(U i=0;i<16u;++i)admitted[row*16u+i]=row_cells[i];
    if(loss==~(W)0){
        for(U i=0;i<8u;++i)top8[row*8u+i]=ABSENT;
        for(U i=0;i<14u;++i)metrics[row*14u+i]=0;
        counters[row*4u]=s[0];counters[row*4u+1u]=0u;counters[row*4u+2u]=0u;counters[row*4u+3u]=0u;
        atomicExch(status,arith?arith:8u);
        return;
    }
    U ranked[8],distance[8],found=0u,cell_count=0u;
    for(U i=0;i<8u;++i){ranked[i]=ABSENT;distance[i]=ABSENT;}
    for(U i=0;i<16u;++i)if(row_cells[i]!=ABSENT)++cell_count;
    U admitted_tokens=0u;
    U bucket_counts[256];
    for(U c=0;c<256u;++c)bucket_counts[c]=0u;
    for(U token=0;token<s[0];++token){
        U cell=token_cell(table,g,s,token,kind,index,candidate);
        if(cell<256u)++bucket_counts[cell];
        bool is_adm=false;
        for(U i=0;i<16u;++i)if(row_cells[i]==cell){is_adm=true;break;}
        if(!is_adm)continue;
        ++admitted_tokens;
        U d=token_distance(q,table,g,s,token,kind,index,candidate),at=found;
        while(at>0u){
            U prev=at-1u;
            if(!(d<distance[prev] || (d==distance[prev] && token<ranked[prev])))break;
            if(at<8u){ranked[at]=ranked[prev];distance[at]=distance[prev];}--at;
        }
        if(at<8u){ranked[at]=token;distance[at]=d;if(found<8u)++found;}
    }
    U max_b=0u;
    for(U c=0;c<256u;++c)if(bucket_counts[c]>max_b)max_b=bucket_counts[c];
    U occ=(U)telemetry[2];
    U full_cell_dist=1u+occ;
    for(U k=0;k<cell_count;++k)full_cell_dist+=(occ-k);
    counters[row*4u]=(1u+cell_count)*s[0]+1u;
    counters[row*4u+1u]=full_cell_dist;
    counters[row*4u+2u]=admitted_tokens;
    counters[row*4u+3u]=max_b;
    W hit8=0;
    for(U i=0;i<8u;++i){top8[row*8u+i]=ranked[i];if(ranked[i]==targets[row])hit8=1;}
    W* out=metrics+row*14u;
    out[0]=tm;out[1]=am;out[2]=(tm<<3)+scaled_add(am,s[1]*8u);out[3]=tr;out[4]=ar;
    out[5]=ranked[0]==targets[row];out[6]=hit8;out[7]=cell_count;
    for(U field=0;field<6u;++field)out[8u+field]=telemetry[field];
}

extern "C" __global__ void discrete_bucket_forward(
    const int* table,const U* routes,const int* gates,const U* ids,const int* input,
    const U* targets,const U* plan,const U* g,const U* s,const U* position_steps,
    const U* offsets,const U* members,const U* cells,const U* absent_count,
    int* out,U* predictions,W* losses,U* token_rivals,U* address_rivals,U* admitted,U* trace,U* status
) {
    U row=blockIdx.x*blockDim.x+threadIdx.x;if(row>=s[4])return;
    int q[MAX_WIDTH];
    if(!read_vector(input,ids,routes,gates,plan,g,s,position_steps,row,2u,0u,0u,q,(s[8]==MODE_FLOCK || s[8]==MODE_RELATION)?trace:0)){
        atomicExch(status,2u);return;
    }
    for(U d=0;d<s[1]*8u;++d)out[row*s[1]*8u+d]=q[d];
    U arithmetic_status=0u;
    losses[row]=bucket_writer(
        q,table,g,s,offsets,members,cells,absent_count,
        targets[row],2u,0u,0u,BUCKET_WALK_ID_ORDER,BUCKET_FAULT_NONE,
        predictions+row,token_rivals+row,address_rivals+row,admitted+row*16u,
        0,0,0,&arithmetic_status,0,0
    );
    if(losses[row]==~(W)0)atomicExch(status,arithmetic_status?arithmetic_status:8u);
}

extern "C" __global__ void discrete_bucket_counterfactual(
    const int* table,const U* routes,const int* gates,const U* ids,const int* input,
    const U* targets,const U* plan,const U* g,const U* s,const U* position_steps,
    const U* offsets,const U* members,const U* cells,const U* absent_count,
    U kind,U index,W* losses,U* status
) {
    U candidate=blockIdx.x*blockDim.x+threadIdx.x;if(candidate>=16u)return;
    W sum=0;U eligible=0u;losses[candidate]=~(W)0;
    bool gated=s[8]==MODE_FLOCK || s[8]==MODE_RELATION;
    if((kind==GATE_KIND && !gated) || (kind==0u && gated)){atomicExch(status,9u);return;}
    if(kind==GATE_KIND && index>=gate_count(s)){atomicExch(status,4u);return;}
    if(kind==GATE_KIND && candidate>=3u)return;
    for(U row=0;row<s[4];++row){
        int q[MAX_WIDTH],baseline[MAX_WIDTH];U prediction,tr,ar;
        if(!read_vector(input,ids,routes,gates,plan,g,s,position_steps,row,2u,0u,0u,baseline,0)){atomicExch(status,2u);return;}
        U arithmetic_status=0u;
        W base=bucket_writer(
            baseline,table,g,s,offsets,members,cells,absent_count,
            targets[row],2u,0u,0u,BUCKET_WALK_ID_ORDER,BUCKET_FAULT_NONE,
            &prediction,&tr,&ar,0,0,0,0,&arithmetic_status,0,0
        );
        if(base==~(W)0){atomicExch(status,arithmetic_status?arithmetic_status:8u);return;}
        if(tr==0u)continue;++eligible;
        if(!read_vector(input,ids,routes,gates,plan,g,s,position_steps,row,kind,index,candidate,q,0)){atomicExch(status,2u);return;}
        W cost=bucket_writer(
            q,table,g,s,offsets,members,cells,absent_count,
            targets[row],kind,index,candidate,BUCKET_WALK_ID_ORDER,BUCKET_FAULT_NONE,
            &prediction,&tr,&ar,0,0,0,0,&arithmetic_status,0,0
        );
        if(arithmetic_status){atomicExch(status,arithmetic_status);return;}
        if(cost==~(W)0 || tr==0u){sum=~(W)0;break;}
        if(~(W)0-sum<cost){atomicExch(status,12u);return;}sum+=cost;
    }
    if(eligible==0u){atomicExch(status,8u);return;}
    losses[candidate]=sum;
}

extern "C" __global__ void discrete_bucket_evaluate(
    const int* table,const U* routes,const int* gates,const U* ids,const int* input,
    const U* targets,const U* plan,const U* g,const U* s,const U* position_steps,
    const U* offsets,const U* members,const U* cells,const U* absent_count,
    U* top8,W* metrics,U* status
) {
    U row=blockIdx.x*blockDim.x+threadIdx.x;if(row>=s[4])return;
    int q[MAX_WIDTH];
    if(!read_vector(input,ids,routes,gates,plan,g,s,position_steps,row,2u,0u,0u,q,0)){atomicExch(status,2u);return;}
    U row_cells[16],ranked[8],prediction=ABSENT,tr=0u,ar=0u,arithmetic_status=0u;
    W tm=0,am=0,telemetry[6]={0,0,0,0,0,0};
    W total=bucket_writer(
        q,table,g,s,offsets,members,cells,absent_count,
        targets[row],2u,0u,0u,BUCKET_WALK_ID_ORDER,BUCKET_FAULT_NONE,
        &prediction,&tr,&ar,row_cells,&tm,&am,telemetry,&arithmetic_status,ranked,0
    );
    if(total==~(W)0){atomicExch(status,arithmetic_status?arithmetic_status:8u);return;}
    if(ranked[0]==ABSENT || ranked[0]!=prediction){atomicExch(status,11u);return;}
    U cell_count=0u;
    for(U i=0;i<16u;++i)if(row_cells[i]!=ABSENT)++cell_count;
    W hit8=0;
    for(U i=0;i<8u;++i){top8[row*8u+i]=ranked[i];if(ranked[i]==targets[row])hit8=1;}
    W* out=metrics+row*14u;
    out[0]=tm;out[1]=am;out[2]=(tm<<3)+scaled_add(am,s[1]*8u);out[3]=tr;out[4]=ar;
    out[5]=ranked[0]==targets[row];out[6]=hit8;out[7]=cell_count;
    for(U field=0;field<6u;++field)out[8u+field]=telemetry[field];
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_writer_anchor_matches_admitted_kernel() {
        let source = include_str!("discrete.cu");
        assert_eq!(source.matches(ORDINARY_WRITER).count(), 1);
    }

    #[test]
    fn generated_bucket_module_keeps_helpers_and_adds_seven_entries() {
        let source = include_str!("discrete.cu");
        let generated = generate(source).expect("bucket source generated");
        let prefix_end = source.find("// Statuses added").expect("writer end");
        assert!(generated.starts_with(&source[..prefix_end]));
        assert_eq!(generated.matches("__global__").count(), ENTRIES.len());
        for entry in ENTRIES {
            assert_eq!(
                generated.matches(&format!("void {entry}(")).count(),
                1,
                "missing or duplicate {entry}"
            );
        }
        assert!(!generated.contains("void discrete_forward("));
        assert!(!generated.contains("void discrete_counterfactual("));
        assert!(!generated.contains("void discrete_evaluate("));
    }

    #[test]
    fn modified_ordinary_writer_refuses_generation() {
        let source = include_str!("discrete.cu");
        assert!(generate(&source.replace("for(U k=0;k<s[3];++k)", "for(U k=0;k<8;++k)")).is_err());
        assert!(generate(&source.replace("// Statuses added", "// statuses")).is_err());
    }

    #[test]
    fn committed_gpu_qualification_receipts_pass_b1_b2_b3_and_match_across_builds() {
        use crate::corpus::sha256;
        fn strip_timing(value: &mut serde_json::Value) {
            match value {
                serde_json::Value::Object(map) => {
                    map.retain(|key, _| {
                        !key.ends_with("_us")
                            && !key.ends_with("_tokens_per_s")
                            && !key.ends_with("report_sha256")
                            && key != "image_sha256"
                    });
                    for child in map.values_mut() {
                        strip_timing(child);
                    }
                }
                serde_json::Value::Array(items) => {
                    for child in items {
                        strip_timing(child);
                    }
                }
                _ => {}
            }
        }
        let build_a_raw =
            include_str!("../../../docs/labs/m1-faster-writer/qualification-build-a.json");
        let build_b_raw =
            include_str!("../../../docs/labs/m1-faster-writer/qualification-build-b.json");
        let generated = generate(include_str!("discrete.cu")).expect("bucket source generated");
        let expected_bucket_source_sha = sha256(generated.as_bytes());
        assert_eq!(
            expected_bucket_source_sha,
            "7222ef058a99178fb7080ba8ac64d52d15d3fb078c098d606e8d3ebd78620962"
        );
        let mut parsed = Vec::new();
        for raw in [build_a_raw, build_b_raw] {
            let mut v: serde_json::Value = serde_json::from_str(raw).expect("valid json receipt");
            assert_eq!(v["status"], "PASSED");
            assert_eq!(v["b1_parity_pass"], true);
            assert_eq!(v["b2_comparison_reduction_pass"], true);
            assert_eq!(v["negative_controls_pass"], true);
            assert_eq!(v["direct_seeded_sweep"]["total_mismatches"], 0);
            assert_eq!(
                v["driver_source_sha256"],
                "04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d"
            );
            assert_eq!(
                v["bucket_image"]["source_sha256"].as_str(),
                Some(expected_bucket_source_sha.as_str())
            );
            assert_eq!(
                v["bucket_image"]["ptx_sha256"],
                "e53dbb16cdbbf0a3c7d90e4df17db9934141c0b55949fd0a3d15b2f490410a41"
            );
            assert_eq!(
                v["bucket_image"]["cubin_sha256"],
                "22e82fe4eb36e8e6e8f54b8062c40ac6bcea81d55685a7e2ad7e3e427c23c556"
            );
            assert_eq!(v["planted_faults"].as_array().map(Vec::len), Some(4));
            assert_eq!(v["real_read_dev_panel"].as_array().map(Vec::len), Some(2));
            if let Some(ci) = v
                .get_mut("compiled_identity")
                .and_then(serde_json::Value::as_object_mut)
            {
                ci.remove("source_sha256");
            }
            strip_timing(&mut v);
            parsed.push(v);
        }
        assert_eq!(parsed[0], parsed[1]);
    }
}
