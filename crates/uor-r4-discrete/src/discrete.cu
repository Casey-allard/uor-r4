// Integer-only SpiralCore/FBS path. Geometry arrays are compiled from the
// source-bound signed-axis action graph; no learned coefficient grid exists.
// Storage indexing uses integer products. Model values use gather,
// sign, add/subtract, comparison and finite action-table lookup.
typedef unsigned int U;
typedef unsigned long long W;
#define ROOT 0
#define AXIS 128
#define COMPOSE 640
#define BFS 4736
#define BRANCH 4992
#define GENERATOR 5040
#define INVERSE 5055
#define IDENTITY 5119
#define ABSENT 0xffffffffu
#define MAX_WIDTH 128

__device__ int magnitude(int x) { return x < 0 ? -x : x; }
__device__ U compose(const U* g,U a,U b) { return g[COMPOSE+a*64+b]; }
__device__ void action(const U* g,U a,const int* x,int* y) {
    for(U d=0;d<8;++d) { U axis=g[AXIS+a*8+d]; int v=x[axis&7]; y[d]=(axis&8)?-v:v; }
}
__device__ U move_action(const U* g,U candidate) { return candidate ? g[GENERATOR+candidate-1] : g[IDENTITY]; }
__device__ void table_block(const int* table,const U* g,U token,U block,U blocks,U kind,U index,U candidate,int* out) {
    const int* p=table+(token*blocks+block)*8;
    if(kind==1 && index==token*blocks+block && candidate) action(g,move_action(g,candidate),p,out);
    else for(U d=0;d<8;++d) out[d]=p[d];
}
__device__ void input_block(const int* input,const U* ids,const U* g,U row,U pos,U block,const U* s,U kind,U index,U candidate,int* out) {
    U at=row*s[2]+pos;
    const int* p=input+(at*s[1]+block)*8;
    if(kind==1 && index==ids[at]*s[1]+block && candidate) action(g,move_action(g,candidate),p,out);
    else for(U d=0;d<8;++d) out[d]=p[d];
}
// Coordinate displacement is primary. Coordinate Hamming breaks equal
// displacement; retained port identity resolves a remaining tie.
__device__ void root_cost(const int* x,const U* g,U root,U* distance,U* hamming) {
    *distance=0;*hamming=0;
    for(U d=0;d<8;++d) { int y=(int)g[ROOT+root*8+d]; *distance+=(U)magnitude(x[d]-y); *hamming+=(x[d]!=y); }
}
__device__ bool route_vector(const int* input,const U* ids,const U* routes,const U* plan,const U* g,const U* s,U row,U kind,U index,U candidate,int* output) {
    plan+=row*(3+s[6]*4+16);
    U length=plan[0],nodes=plan[1],frontier=plan[2];plan+=3;
    if(length==0 || length>s[2] || nodes>s[6] || frontier==0 || frontier>16 || s[13]>1u || (s[13]==1u && kind==0u))return false;
    if(s[13]==1u){for(U block=0;block<s[1];++block)input_block(input,ids,g,row,length-1,block,s,kind,index,candidate,output+block*8);return true;}
    for(U block=0;block<s[1];++block) {
        int query[8];input_block(input,ids,g,row,length-1,block,s,kind,index,candidate,query);
        U node=ABSENT,root=0,total=g[IDENTITY];
        for(U stage=0;stage<s[5];++stage) {
            U parameter=block*s[5]+stage;
            U frame=routes[parameter];
            if(frame>=64) return false;
            if(kind==0 && index==parameter && candidate) frame=compose(g,move_action(g,candidate),frame);
            int q[8];action(g,frame,query,q);
            U count=stage==0?frontier:3;
            U best=ABSENT,best_distance=ABSENT,best_hamming=ABSENT,goal=0;
            for(U port=0;port<count;++port) {
                U r=stage==0?port:g[BRANCH+root*3+port];
                U distance,hamming;root_cost(q,g,r,&distance,&hamming);
                if(distance<best_distance || (distance==best_distance && hamming<best_hamming)) {
                    best=port;best_distance=distance;best_hamming=hamming;goal=r;
                }
            }
            if(best==ABSENT) return false;
            total=compose(g,g[BFS+root*16+goal],total);root=goal;
            node=stage==0?plan[s[6]*4+best]:plan[node*4+best];
            if(node>=nodes) return false;
            U atom=plan[node*4+3];
            if(atom!=ABSENT) {
                if(atom>=length) return false;
                int carrier[8];input_block(input,ids,g,row,atom,block,s,kind,index,candidate,carrier);
                action(g,total,carrier,output+block*8);break;
            }
            if(stage+1==s[5]) return false;
        }
    }
    return true;
}
// ---- Flock & Relation read (Claude; Antigravity ReadSpec::Relation extension) ----
// Device spec words appended after s[0..7]: s[8] read mode (0 Route, 1 Flock, 2 Relation),
// s[9] kept K, s[10] sweeps S, s[12] snap scale (0 snap_n, 1 unit snap_1 control, 2 raw C_raw control:
// the output block is the unsnapped Z^8 residual u, traced output root ABSENT; 1 and 2 Flock/Relation only),
// s[13] route bypass (0 normal Route, 1 R-Off query-block bypass; Route only, refuses kind 0 proposals).
// s[14] position transport (0 legacy endpoint BFS, 1 causal prefix holonomy, 2 matched endpoint control,
// 3 identity; nonzero Flock/Relation only), with position_steps[block*16+root] candidates in 0..16.
// Gates are i32 trits: block*stages+level (level 0 = atoms),
// plus in Relation mode 5*stages relation gates at blocks*stages + level*5 + edge_type.
#define MODE_FLOCK 1u
#define MODE_RELATION 2u
#define EDGE_TYPES 5u
#define MAX_KEPT 8u
#define MAX_SWEEPS 5u
#define MAX_NODES 364u
#define GATE_KIND 3u
__device__ U gate_count(const U* s) { return s[1]*s[5] + (s[8]==MODE_RELATION ? EDGE_TYPES*s[5] : 0u); }
__device__ U root_id(const int* v,const U* g);
// Gate value under an optional virtual Gate move: ordinal 0 keeps the current
// trit; ordinals 1,2 are the siblings in the fixed order [0,-1,+1] skipping it.
__device__ int gate_value(const int* gates,U at,U kind,U index,U candidate) {
    int current=gates[at];
    if(kind!=GATE_KIND || index!=at || candidate==0)return current;
    const int order[3]={0,-1,1};U k=0;
    for(U i=0;i<3;++i){if(order[i]==current)continue;++k;if(k==candidate)return order[i];}
    return current;
}
// snap_n(x): the orbit root r minimising (L1(x - n*r), Hamming(x, n*r), r).
// n*r is built by repeated addition; n <= 1+MAX_NODES is never reached (n <= 9).
__device__ U snap(const int* x,U n,const U* g) {
    U best=ABSENT,best_distance=ABSENT,best_hamming=ABSENT;
    for(U r=0;r<16;++r) {
        U distance=0,hamming=0;
        for(U d=0;d<8;++d){int y=0;int v=(int)g[ROOT+r*8+d];for(U i=0;i<n;++i)y+=v;distance+=(U)magnitude(x[d]-y);hamming+=(x[d]!=y);}
        if(distance<best_distance || (distance==best_distance && hamming<best_hamming)){best=r;best_distance=distance;best_hamming=hamming;}
    }
    return best;
}
// Frame block of payload block b: the first and last endpoints, never b itself.
__device__ U frame_block(U b,U blocks) { return b==0 ? blocks-1 : 0; }
// Flock/Relation read for one row. Payload block b of each kept candidate is carried
// by the s[14] action: legacy BFS, causal prefix holonomy, matched endpoint control,
// or identity. Ranking uses the same transported frame as payload consensus. The
// consensus is snap_K of the carried sum; gates add (+1) or subtract (-1) each
// level's consensus to the query payload; out = snap_n of that exact residual.
// Optional trace (forward only; null elsewhere) from this SAME path, per (row, block):
//   [fq, out root, terms, query payload root], then per level l, TRACE_LEVEL words:
//   [gate+1, kept, consensus root, eligible count, kept node ids[8], actions[8], carried roots[8]]
// Unused words are ABSENT. Selection is traced even for gate 0 (output unchanged).
// In Relation mode, per-node relation scores rel[0..s[6]] are written after the
// Flock trace region at trace + (s[4]*s[1])*(TRACE_ROW+s[5]*TRACE_LEVEL).
#define TRACE_ROW 4u
#define TRACE_LEVEL 28u
#define MAX_POSITIONS 243u
__device__ U position_step(const U* steps,U block,U root,const U* g) {
    U candidate=steps[block*16+root];
    return candidate==0?g[IDENTITY]:g[GENERATOR+candidate-1];
}
__device__ U position_action(const U* steps,const unsigned char* prefix,const U* g,const U* s,U block,U from_root,U query_root,U from_pos,U query_pos) {
    if(s[14]==0)return g[BFS+from_root*16+query_root];
    if(s[14]==1)return compose(g,prefix[query_pos],g[INVERSE+prefix[from_pos]]);
    if(s[14]==2)return compose(g,position_step(steps,block,query_root,g),g[INVERSE+position_step(steps,block,from_root,g)]);
    return g[IDENTITY];
}
__device__ bool positioned_root_cost(const U* s,const int* query,const U* g,U action_index,U root,U* distance,U* hamming) {
    if(s[14]==0)root_cost(query,g,root,distance,hamming);
    else {
        int source[8],transported[8];
        for(U d=0;d<8;++d)source[d]=(int)g[ROOT+root*8+d];
        action(g,action_index,source,transported);
        U mapped=root_id(transported,g);if(mapped==ABSENT)return false;
        root_cost(query,g,mapped,distance,hamming);
    }
    return true;
}
__device__ bool flock_vector(const int* input,const U* ids,const int* gates,const U* plan,const U* g,const U* s,const U* position_steps,U row,U kind,U index,U candidate,int* output,U* trace) {
    plan+=row*(3+s[6]*4+16);
    U length=plan[0],nodes=plan[1];plan+=3;
    U kept=s[9],sweeps=s[10];
    if(length==0 || length>s[2] || nodes>s[6] || nodes>MAX_NODES || kept==0 || kept>MAX_KEPT || s[1]<2)return false;
    if(s[8]==MODE_RELATION && (sweeps==0 || sweeps>MAX_SWEEPS))return false;
    U level[MAX_NODES],first[MAX_NODES],last[MAX_NODES],frame[MAX_NODES];
    for(U node=0;node<nodes;++node) {
        U atom=plan[node*4+3];
        if(atom!=ABSENT){if(atom>=length)return false;level[node]=0;first[node]=atom;last[node]=atom;}
        else {
            U c0=plan[node*4],c2=plan[node*4+2];
            if(c0>=node || plan[node*4+1]>=node || c2>=node)return false;
            level[node]=level[c0]+1;first[node]=first[c0];last[node]=last[c2];
        }
    }
    for(U b=0;b<s[1];++b) {
        U f=frame_block(b,s[1]);
        int query_frame[8],u[8];
        input_block(input,ids,g,row,length-1,f,s,kind,index,candidate,query_frame);
        input_block(input,ids,g,row,length-1,b,s,kind,index,candidate,u);
        U fq=root_id(query_frame,g);if(fq==ABSENT)return false;
        int query_payload[8];for(U d=0;d<8;++d)query_payload[d]=u[d];
        int* query_position=s[14]==0?query_frame:query_payload;
        U action_query=fq;
        if(s[14]==2){action_query=root_id(u,g);if(action_query==ABSENT)return false;}
        unsigned char prefix[MAX_POSITIONS];
        if(s[14]==1) {
            U acc=g[IDENTITY];
            for(U p=0;p<length;++p) {
                int fv[8];input_block(input,ids,g,row,p,b,s,kind,index,candidate,fv);
                U root=root_id(fv,g);if(root==ABSENT)return false;
                acc=compose(g,position_step(position_steps,b,root,g),acc);
                prefix[p]=(unsigned char)acc;
            }
        }
        // Node frames and block-b payloads, children before parents.
        int payload[MAX_NODES*8];
        for(U node=0;node<nodes;++node) {
            U atom=plan[node*4+3];
            if(atom!=ABSENT) {
                int fv[8];input_block(input,ids,g,row,atom,f,s,kind,index,candidate,fv);
                frame[node]=root_id(fv,g);if(frame[node]==ABSENT)return false;
                input_block(input,ids,g,row,atom,b,s,kind,index,candidate,payload+node*8);
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
                int rv[8];input_block(input,ids,g,row,last[node],b,s,kind,index,candidate,rv);
                tr=root_id(rv,g);if(tr==ABSENT)return false;
            }
            transport_root[node]=(unsigned char)tr;
            transport[node]=(unsigned char)position_action(position_steps,prefix,g,s,b,tr,action_query,last[node],length-1);
        }
        int rel[MAX_NODES];
        for(U node=0;node<nodes;++node)rel[node]=0;
        if(s[8]==MODE_RELATION) {
            for(U rg=0;rg<EDGE_TYPES*s[5];++rg) {
                int gt=gate_value(gates,s[1]*s[5]+rg,kind,index,candidate);
                if(gt<-1 || gt>1)return false;
            }
            int prev[MAX_NODES];
            for(U node=0;node<nodes;++node) {
                U distance,hamming;if(!positioned_root_cost(s,query_position,g,transport[node],transport_root[node],&distance,&hamming))return false;
                prev[node]=8-(int)distance;
            }
            for(U sweep=0;sweep<sweeps;++sweep) {
                for(U i=0;i<nodes;++i) {
                    if(last[i]>=length)continue;
                    U li=level[i];
                    for(U e=0;e<EDGE_TYPES;++e) {
                        int gt=gate_value(gates,s[1]*s[5]+li*EDGE_TYPES+e,kind,index,candidate);
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
                            for(U j=0;j<i;++j)if(last[j]<length && level[j]==li && last[j]+1==first[i])esum+=prev[j];
                        } else if(e==3) {
                            if(li==0) {
                                U tid=ids[row*s[2]+last[i]];
                                for(U j=0;j<i;++j)if(last[j]<length && level[j]==0 && ids[row*s[2]+last[j]]==tid)esum=prev[j];
                            }
                        } else if(e==4) {
                            for(U j=0;j<i;++j)if(last[j]<length && level[j]==li && frame[j]==frame[i])esum=prev[j];
                        }
                        rel[i]+=gt>0?esum:-esum;
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
        U terms=1;
        for(U l=0;l<s[5];++l) {
            int gate=gate_value(gates,b*s[5]+l,kind,index,candidate);
            if(gate<-1 || gate>1)return false;
            U* lv=rec ? rec+TRACE_ROW+l*TRACE_LEVEL : 0;
            if(lv)lv[0]=(U)(gate+1);
            if(gate==0 && !lv)continue;
            // Eligible candidates: level l, every leaf strictly before the query.
            U top[MAX_KEPT],key_d[MAX_KEPT],key_h[MAX_KEPT],found=0,eligible=0;
            int key_r[MAX_KEPT];
            for(U node=0;node<nodes;++node) {
                if(level[node]!=l || last[node]+1>=length)continue;
                ++eligible;
                U distance,hamming;if(!positioned_root_cost(s,query_position,g,transport[node],transport_root[node],&distance,&hamming))return false;
                int rscore=rel[node];
                // Insert by (rel_score desc, distance asc, hamming asc, recency desc, node asc).
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
            for(U k=0;k<found;++k){U node=top[k];int carried[8];U a=transport[node];
                action(g,a,payload+node*8,carried);
                for(U d=0;d<8;++d)sum[d]+=carried[d];
                if(lv){lv[4+k]=node;lv[12+k]=a;lv[20+k]=root_id(carried,g);}}
            U m=snap(sum,s[12]==1?1u:found,g);
            if(lv)lv[2]=m;
            if(gate==0)continue;
            for(U d=0;d<8;++d){int v=(int)g[ROOT+m*8+d];u[d]+=gate>0?v:-v;}
            ++terms;
        }
        if(s[12]==2){if(rec){rec[1]=ABSENT;rec[2]=terms;}for(U d=0;d<8;++d)output[b*8+d]=u[d];continue;}
        U out=snap(u,s[12]==1?1u:terms,g);
        if(rec){rec[1]=out;rec[2]=terms;}
        for(U d=0;d<8;++d)output[b*8+d]=(int)g[ROOT+out*8+d];
    }
    return true;
}
// Mode dispatch: Route (default), Flock, or Relation. A Route proposal in
// Flock/Relation mode and a Gate proposal in Route mode are refused (status 9).
__device__ bool read_vector(const int* input,const U* ids,const U* routes,const int* gates,const U* plan,const U* g,const U* s,const U* position_steps,U row,U kind,U index,U candidate,int* output,U* trace) {
    if(s[8]==MODE_FLOCK || s[8]==MODE_RELATION)return flock_vector(input,ids,gates,plan,g,s,position_steps,row,kind,index,candidate,output,trace);
    return route_vector(input,ids,routes,plan,g,s,row,kind,index,candidate,output);
}
__device__ U address_distance(const int* q,const U* g,const U* s,U cell) {
    U a,ha,b,hb;root_cost(q,g,cell/16,&a,&ha);root_cost(q+(s[1]-1)*8,g,cell%16,&b,&hb);return a+b;
}
__device__ U token_distance(const int* q,const int* table,const U* g,const U* s,U token,U kind,U index,U candidate) {
    U sum=0;
    for(U block=0;block<s[1];++block) { int v[8];table_block(table,g,token,block,s[1],kind,index,candidate,v);for(U d=0;d<8;++d) sum+=(U)magnitude(q[block*8+d]-v[d]); }
    return sum;
}
__device__ U root_id(const int* v,const U* g) {
    for(U r=0;r<16;++r){bool equal=true;for(U d=0;d<8;++d)if(v[d]!=(int)g[ROOT+r*8+d])equal=false;if(equal)return r;}
    return ABSENT;
}
__device__ U token_cell(const int* table,const U* g,const U* s,U token,U kind,U index,U candidate) {
    int first[8],last[8];table_block(table,g,token,0,s[1],kind,index,candidate,first);
    table_block(table,g,token,s[1]-1,s[1],kind,index,candidate,last);
    U a=root_id(first,g),b=root_id(last,g);return a==ABSENT || b==ABSENT?ABSENT:a*16+b;
}
__device__ W scaled_add(W value,U times) { W total=0;while(times){if(times&1)total+=value;value<<=1;times>>=1;}return total; }
__device__ W checked_scaled_add(W value,U times) {
    W total=0;
    while(times){
        if(times&1){if(~(W)0-total<value)return ~(W)0;total+=value;}
        times>>=1;
        if(times){if(value>(~(W)0>>1))return ~(W)0;value<<=1;}
    }
    return total;
}
__device__ bool cell_precedes(U distance,U cell,U other_distance,U other_cell) {
    return distance<other_distance || (distance==other_distance && cell<other_cell);
}
// The writer's cell address metric, s[15] (#2834): 0 the two-block outer key
// above; 1 the cell envelope, min over the cell's tokens of token_distance;
// 2 the two-scale (envelope << 8) + the sum of the cell's token distances.
// env and sum are filled only when s[15] != 0. Host oracles: address_cells.rs
// cell_outer_l1, cell_envelope_l1 and full_vector_cell_l1. The two-scale metric
// orders admission and rank; its address hinge keeps the envelope's scale (the
// sum term only breaks ties), so the evaluation's hinge range contract holds.
__device__ W cell_distance(const int* q,const U* g,const U* s,U cell,const U* env,const W* sum) {
    if(s[15]==0u)return address_distance(q,g,s,cell);
    return s[15]==1u?(W)env[cell]:((W)env[cell]<<8)+sum[cell];
}
__device__ bool cell_precedes_w(W distance,U cell,W other_distance,U other_cell) {
    return distance<other_distance || (distance==other_distance && cell<other_cell);
}
// Product addresses derive from current codebook geometry, including virtual
// moves. Full bounded vocabulary scans retain every crowded-cell identity.
// An absent comparison has zero hinge and an explicit eligibility count.
__device__ W writer(const int* q,const int* table,const U* g,const U* s,U target,U kind,U index,U candidate,U* prediction,U* token_rivals,U* address_rivals,U* admitted,W* token_hinge=0,W* address_hinge=0,W* telemetry=0,U* arithmetic_status=0) {
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
}
// Statuses added with the flock/relation read: 9 = read-mode/proposal-kind mismatch
// (Gate in Route mode, Route in Flock/Relation mode); 10 = gate trit outside {-1,0,1}
// or read mode/K/sweeps out of range. 13 = writer address metric word s[15] > 2.
extern "C" __global__ void discrete_init(int* table,U* routes,int* gates,const U* g,const U* s,U seed,U* status) {
    U i=blockIdx.x*blockDim.x+threadIdx.x;
    U elements=s[0]*s[1]*8;
    if(i<elements) {
        U block=i/8;U state=seed^block^0x9e3779b9u;
        state^=state<<13;state^=state>>17;state^=state<<5;
        table[i]=(int)g[ROOT+(state&15)*8+(i&7)];
    }
    if(i<s[1]*s[5])routes[i]=g[IDENTITY];
    if(i<gate_count(s))gates[i]=0;
    if(i==0)*status=0;
}
extern "C" __global__ void discrete_encode(const int* table,const U* ids,U count,U width,U vocabulary,int* out,U* status) {
    U i=blockIdx.x*blockDim.x+threadIdx.x;if(i>=count*width)return;
    U token=ids[i/width];if(token>=vocabulary){atomicExch(status,1u);return;}out[i]=table[token*width+i%width];
}
// Read-only current-checkpoint token cells, in token-ID order. Only cells/status
// are writable; the existing identity-candidate helper supplies all geometry.
extern "C" __global__ void discrete_export_token_cells(const int* table,const U* g,const U* s,const U* position_steps,U* cells,U* status) {
    U token=blockIdx.x*blockDim.x+threadIdx.x;if(token>=s[0])return;
    U cell=token_cell(table,g,s,token,2,0,0);
    if(cell==ABSENT || cell>255u){atomicExch(status,8u);return;}
    cells[token]=cell;
}
// trace: Flock/Relation mode only, rows*blocks*(TRACE_ROW+stages*TRACE_LEVEL) words
// (+ rows*blocks*nodes_count relation score words in Relation mode); ignored in Route mode.
extern "C" __global__ void discrete_forward(const int* table,const U* routes,const int* gates,const U* ids,const int* input,const U* targets,const U* plan,const U* g,const U* s,const U* position_steps,int* out,U* predictions,W* losses,U* token_rivals,U* address_rivals,U* admitted,U* trace,U* status) {
    U row=blockIdx.x*blockDim.x+threadIdx.x;if(row>=s[4])return;
    int q[MAX_WIDTH];if(!read_vector(input,ids,routes,gates,plan,g,s,position_steps,row,2,0,0,q,(s[8]==MODE_FLOCK || s[8]==MODE_RELATION)?trace:0)){atomicExch(status,2u);return;}
    for(U d=0;d<s[1]*8;++d)out[row*s[1]*8+d]=q[d];
    U arithmetic_status=0;
    losses[row]=writer(q,table,g,s,targets[row],2,0,0,predictions+row,token_rivals+row,address_rivals+row,admitted+row*16,0,0,0,&arithmetic_status);
    if(losses[row]==~(W)0)atomicExch(status,arithmetic_status?arithmetic_status:8u);
}
// Sixteen fixed lanes. Gate (kind 3) uses ordinals 0..2 only; lanes 3..15 stay MAX.
extern "C" __global__ void discrete_counterfactual(const int* table,const U* routes,const int* gates,const U* ids,const int* input,const U* targets,const U* plan,const U* g,const U* s,const U* position_steps,U kind,U index,W* losses,U* status) {
    U candidate=blockIdx.x*blockDim.x+threadIdx.x;if(candidate>=16)return;
    W sum=0;U eligible=0;losses[candidate]=~(W)0;
    bool gated=s[8]==MODE_FLOCK || s[8]==MODE_RELATION;
    if((kind==GATE_KIND && !gated) || (kind==0 && gated)){atomicExch(status,9u);return;}
    if(kind==GATE_KIND && index>=gate_count(s)){atomicExch(status,4u);return;}
    if(kind==GATE_KIND && candidate>=3)return;
    for(U row=0;row<s[4];++row){int q[MAX_WIDTH],baseline[MAX_WIDTH];U prediction,tr,ar;
        if(!read_vector(input,ids,routes,gates,plan,g,s,position_steps,row,2,0,0,baseline,0)){atomicExch(status,2u);return;}
        U arithmetic_status=0;
        W base=writer(baseline,table,g,s,targets[row],2,0,0,&prediction,&tr,&ar,0,0,0,0,&arithmetic_status);
        if(base==~(W)0){atomicExch(status,arithmetic_status?arithmetic_status:8u);return;}
        // The baseline fixes eligible rows. A virtual move cannot improve by
        // removing its supervised competitor; absent baseline comparisons skip.
        if(tr==0)continue;++eligible;
        if(!read_vector(input,ids,routes,gates,plan,g,s,position_steps,row,kind,index,candidate,q,0)){atomicExch(status,2u);return;}
        W cost=writer(q,table,g,s,targets[row],kind,index,candidate,&prediction,&tr,&ar,0,0,0,0,&arithmetic_status);
        if(arithmetic_status){atomicExch(status,arithmetic_status);return;}
        if(cost==~(W)0 || tr==0){sum=~(W)0;break;}
        if(~(W)0-sum<cost){atomicExch(status,12u);return;}sum+=cost;
    }
    if(eligible==0){atomicExch(status,8u);return;}
    losses[candidate]=sum;
}
__device__ U resulting_port(const int* table,const U* routes,const U* g,const U* s,U kind,U index,U candidate) {
    U a=move_action(g,candidate);
    if(kind==0)return compose(g,a,routes[index]);
    int out[8];action(g,a,table+index*8,out);return root_id(out,g);
}
extern "C" __global__ void discrete_choose(const W* losses,const W* epoch,const int* table,const U* routes,const int* gates,const U* g,const U* s,const U* position_steps,U kind,U index,W* proposal) {
    if(blockIdx.x || threadIdx.x)return;U best=0;
    // Report objective sensitivity independently of improvement. All-worse
    // proposals are sensitive; all-tied proposals are not. Gate MAX padding
    // is outside the active inventory and must never manufacture sensitivity.
    W sensitive=0;U active=kind==GATE_KIND?3u:16u;
    for(U i=1;i<active;++i)if(losses[i]!=losses[0])sensitive=1;
    if(kind==GATE_KIND) {
        // Strict improvement only; ties keep the lower ordinal (CURRENT, then 0, -1, +1).
        for(U i=1;i<3;++i)if(losses[i]<losses[best])best=i;
    } else {
        for(U i=1;i<16;++i) {
            bool tie=best!=0 && losses[i]==losses[best] && resulting_port(table,routes,g,s,kind,index,i)<resulting_port(table,routes,g,s,kind,index,best);
            if(losses[i]<losses[best] || tie)best=i;
        }
    }
    proposal[0]=best;proposal[1]=losses[0];proposal[2]=losses[best];proposal[3]=*epoch;proposal[4]=sensitive;
}
extern "C" __global__ void discrete_apply(int* table,U* routes,int* gates,const U* g,const U* s,const U* position_steps,const W* proposal,U kind,U index,W* epoch,U* status) {
    if(blockIdx.x || threadIdx.x)return;
    if(proposal[3]!=*epoch){*status=3;return;}
    bool gated=s[8]==MODE_FLOCK || s[8]==MODE_RELATION;
    if(kind==GATE_KIND) {
        if(!gated){*status=9;return;}
        if(proposal[0]>=3 || index>=gate_count(s)){*status=4;return;}
        if(proposal[0]==0)return;
        if(proposal[2]>=proposal[1] || *epoch==~(W)0){*status=5;return;}
        gates[index]=gate_value(gates,index,GATE_KIND,index,(U)proposal[0]);
        ++*epoch;return;
    }
    if(kind==0 && gated){*status=9;return;}
    if(proposal[0]>=16 || kind>1 || (kind==0 && index>=s[1]*s[5]) || (kind==1 && index>=s[0]*s[1])){*status=4;return;}
    if(proposal[0]==0)return;
    if(proposal[2]>=proposal[1] || *epoch==~(W)0){*status=5;return;}
    U a=move_action(g,(U)proposal[0]);
    if(kind==0)routes[index]=compose(g,a,routes[index]);
    else {int out[8];action(g,a,table+index*8,out);for(U d=0;d<8;++d)table[index*8+d]=out[d];}
    ++*epoch;
}
extern "C" __global__ void discrete_validate(const int* table,const U* routes,const int* gates,const U* g,const U* s,const U* position_steps,U* status) {
    U i=blockIdx.x*blockDim.x+threadIdx.x;
    if(i<s[0]*s[1]) {
        bool found=false;
        for(U r=0;r<16;++r){bool equal=true;for(U d=0;d<8;++d)if(table[i*8+d]!=(int)g[r*8+d])equal=false;found|=equal;}
        if(!found)atomicExch(status,6u);
    }
    if(i<s[1]*s[5] && routes[i]>=64)atomicExch(status,7u);
    if(i<gate_count(s) && (gates[i]<-1 || gates[i]>1))atomicExch(status,10u);
    if(i<s[1]*16 && position_steps[i]>=16)atomicExch(status,10u);
    if(i==0 && (s[11]>1 || s[8]>MODE_RELATION || s[12]>2 || (s[12]!=0 && s[8]!=MODE_FLOCK && s[8]!=MODE_RELATION) || s[13]>1 || (s[13]!=0 && s[8]!=0)
        || s[14]>3 || (s[14]!=0 && s[8]!=MODE_FLOCK && s[8]!=MODE_RELATION)
        || (s[8]==MODE_FLOCK && (s[9]==0 || s[9]>MAX_KEPT || s[10]!=0 || s[1]<2))
        || (s[8]==MODE_RELATION && (s[9]==0 || s[9]>MAX_KEPT || s[10]==0 || s[10]>MAX_SWEEPS || s[1]<2))))atomicExch(status,10u);
}

// Exact model evaluation. Same read/writer as forward; top8 is a GLOBAL order
// over all admitted identities, never concatenated independent per-cell ranks.
// Row metric words: T,A,total,token_rivals,address_rivals,hit1,hit8,admitted_cells.
extern "C" __global__ void discrete_evaluate(const int* table,const U* routes,const int* gates,const U* ids,const int* input,const U* targets,const U* plan,const U* g,const U* s,const U* position_steps,U* top8,W* metrics,U* status) {
    U row=blockIdx.x*blockDim.x+threadIdx.x;if(row>=s[4])return;
    int q[MAX_WIDTH];
    if(!read_vector(input,ids,routes,gates,plan,g,s,position_steps,row,2,0,0,q,0)){atomicExch(status,2u);return;}
    U cells[16],prediction,tr,ar,arithmetic_status=0;W tm,am,telemetry[6];
    W total=writer(q,table,g,s,targets[row],2,0,0,&prediction,&tr,&ar,cells,&tm,&am,telemetry,&arithmetic_status);
    if(total==~(W)0){atomicExch(status,arithmetic_status?arithmetic_status:8u);return;}
    U ranked[8],distance[8],found=0,cell_count=0;
    for(U i=0;i<8;++i){ranked[i]=ABSENT;distance[i]=ABSENT;}
    for(U i=0;i<16;++i)if(cells[i]!=ABSENT)++cell_count;
    for(U token=0;token<s[0];++token){
        U cell=token_cell(table,g,s,token,2,0,0);bool admitted=false;
        for(U i=0;i<16;++i)if(cells[i]==cell){admitted=true;break;}
        if(!admitted)continue;
        U d=token_distance(q,table,g,s,token,2,0,0),at=found;
        while(at>0){U prev=at-1;
            if(!(d<distance[prev] || (d==distance[prev] && token<ranked[prev])))break;
            if(at<8){ranked[at]=ranked[prev];distance[at]=distance[prev];}--at;
        }
        if(at<8){ranked[at]=token;distance[at]=d;if(found<8)++found;}
    }
    if(found==0 || ranked[0]!=prediction){atomicExch(status,11u);return;}
    W hit8=0;for(U i=0;i<8;++i){top8[row*8+i]=ranked[i];if(ranked[i]==targets[row])hit8=1;}
    W* out=metrics+row*14;
    out[0]=tm;out[1]=am;out[2]=(tm<<3)+scaled_add(am,s[1]*8);out[3]=tr;out[4]=ar;
    out[5]=ranked[0]==targets[row];out[6]=hit8;out[7]=cell_count;
    for(U field=0;field<6;++field)out[8+field]=telemetry[field];
}
__device__ bool metric_add(W* value,W add,U* status) {
    if(~(W)0-*value<add){atomicExch(status,12u);return false;}*value+=add;return true;
}
// Output words: rows,hit1,hit8,sumT,sumA,sumtotal,token_eligible_rows,
// address_eligible_rows,sum_token_rivals,sum_address_rivals,sum_cells,
// eligible_any_rows,eligible_total_zero_rows,eligible_token_zero_rows,
// eligible_address_zero_rows,eligible_strict_token_wins,8*width.
// Absent-comparison zeros are never successes; strict T<width does not alter loss.
// summaries=0 reduces row metrics; summaries=1 combines unchanged completed
// batch receipts. In both cases all arithmetic and denominator checks are CUDA.
extern "C" __global__ void discrete_evaluation_aggregate(const W* metrics,U count,U width,U summaries,W* out,U* status) {
    if(blockIdx.x || threadIdx.x)return;
    if(count==0 || width<16 || width>MAX_WIDTH || summaries>1){*status=12u;return;}
    for(U i=0;i<1020;++i)out[i]=0;out[16]=(W)width<<3;
    for(U row=0;row<count;++row){
        if(summaries){const W* p=metrics+(W)row*1020;if(p[16]!=out[16] || (row && p[23]!=out[23])){*status=12u;return;}
            out[23]=p[23];
            for(U i=0;i<999;++i)if(i!=16 && i!=23 && !metric_add(out+i,p[i],status))return;
        } else {const W* p=metrics+(W)row*14;
            if(p[0]>640 || p[1]>72 || p[8]>255 || p[9]<1 || p[9]>256 || p[10]<1 || p[10]>256 || p[7]<1 || p[7]>16 || p[4]>p[7] || p[7]-p[4]>1 || (row && p[10]!=out[23])){*status=12u;return;}
            W values[16]={1,p[5],p[6],p[0],p[1],p[2],p[3]>0,p[4]>0,p[3],p[4],p[7],
                p[3]>0 || p[4]>0,p[2]==0 && (p[3]>0 || p[4]>0),
                p[3]>0 && p[0]==0,p[4]>0 && p[1]==0,p[3]>0 && p[0]<width};
            for(U i=0;i<16;++i)if(!metric_add(out+i,values[i],status))return;
            W weighted_token=checked_scaled_add(p[0],8),weighted_address=checked_scaled_add(p[1],width),weighted_rank=checked_scaled_add(p[8],width);
            if(weighted_token==~(W)0 || weighted_address==~(W)0 || weighted_rank==~(W)0 || ~(W)0-weighted_token<weighted_address || ~(W)0-weighted_token<weighted_rank){*status=12u;return;}
            if(p[2]!=weighted_token+weighted_address || p[12]!=weighted_token+weighted_rank || p[11]!=p[3]+p[7]-p[4] || (p[13]!=p[2] && p[13]!=p[12]) || p[9]>p[10] || p[7]>p[10] || p[11]<p[7] || ((p[8]==0)!=(p[7]>p[4])) || p[8]!=(p[9]>p[7]?p[9]-p[7]:0)){*status=12u;return;}
            W extra[12]={p[8],weighted_token,weighted_address,weighted_rank,p[2],p[12],0,p[11],p[7]-p[4],p[3]>0 && p[0]<width && p[7]>p[4],p[8]==0,p[13]};
            out[23]=p[10];
            for(U i=0;i<12;++i)if(i!=6 && !metric_add(out+17+i,extra[i],status))return;
            if(!metric_add(out+29+p[0],1,status) || !metric_add(out+670+p[1],1,status) || !metric_add(out+743+p[8],1,status))return;
        }
    }
    U starts[3]={29,670,743},bins[3]={641,73,256},percent[7]={0,25,50,75,90,99,100};
    if(out[0]==0){*status=12u;return;}
    for(U hinge=0;hinge<3;++hinge){
        W histogram_count=0;
        for(U value=0;value<bins[hinge];++value)if(!metric_add(&histogram_count,out[starts[hinge]+value],status))return;
        if(histogram_count!=out[0]){*status=12u;return;}
        for(U quantile=0;quantile<7;++quantile){
            W product=checked_scaled_add(out[0],percent[quantile]);
            if(product==~(W)0 || product>~(W)0-99){*status=12u;return;}
            W threshold=percent[quantile]?(product+99)/100:1,cumulative=0;
            for(U value=0;value<bins[hinge];++value){
                if(!metric_add(&cumulative,out[starts[hinge]+value],status))return;
                if(cumulative>=threshold){out[999+hinge*7+quantile]=value;break;}
            }
        }
    }
}
// Caller supplies masked-target identities only. Frequency accumulation is
// CUDA; TRAIN and diagnostic DEV always use different zero-initialized tables.
extern "C" __global__ void discrete_count(const U* targets,U count,U vocabulary,W* counts,U* status) {
    U i=blockIdx.x*blockDim.x+threadIdx.x;if(i>=count)return;
    U token=targets[i];if(token>=vocabulary){atomicExch(status,1u);return;}
    atomicAdd(counts+token,(W)1);
}
// Raw count ranking: count descending, token ID ascending, including zero
// frequency tokens. Metrics: eval_rows,hit1,hit8,zero_count_targets,source_count,
// nonzero_tokens. This does not normalize counts or inspect model parameters.
extern "C" __global__ void discrete_count_score(const W* counts,U vocabulary,const U* targets,U count,U* top8,W* metrics,U* status) {
    if(blockIdx.x || threadIdx.x)return;
    if(vocabulary<2 || vocabulary>8192 || count==0){*status=12u;return;}
    U ranked[8],found=0;W frequency[8];
    for(U i=0;i<8;++i){ranked[i]=ABSENT;frequency[i]=0;}
    for(U i=0;i<6;++i)metrics[i]=0;
    for(U token=0;token<vocabulary;++token){W f=counts[token];
        if(!metric_add(metrics+4,f,status))return;if(f)++metrics[5];
        U at=found;while(at>0){U prev=at-1;
            if(!(f>frequency[prev] || (f==frequency[prev] && token<ranked[prev])))break;
            if(at<8){ranked[at]=ranked[prev];frequency[at]=frequency[prev];}--at;
        }
        if(at<8){ranked[at]=token;frequency[at]=f;if(found<8)++found;}
    }
    for(U i=0;i<8;++i)top8[i]=ranked[i];
    for(U i=0;i<count;++i){U target=targets[i];if(target>=vocabulary){*status=1u;return;}
        ++metrics[0];if(target==ranked[0])++metrics[1];
        for(U j=0;j<found;++j)if(target==ranked[j]){++metrics[2];break;}
        if(counts[target]==0)++metrics[3];
    }
}
// Carrier hard selection (carrier-credit-hard-selection, #2819). One block of
// CARRIER_THREADS; integer only. Reads one batch's discrete_evaluate rows and
// writes out[0]=token, out[1]=value. Mode 0 (credit): a row's token+address
// hinge goes to its target and, when top-1 is wrong, to its strongest admitted
// rival top8[0]; the largest credit wins, ties to the lowest id, and zero credit
// falls back to the lowest target (value = credit). Mode 1: the k-th distinct
// target, k = mix(key) mod D (value = D). Mode 2: the target of row mix(key) mod
// rows (value = row). Modes 1 and 2 never read a model score.
#define CARRIER_THREADS 256u
__device__ W carrier_mix(W z) {
    z+=0x9E3779B97F4A7C15ull;
    z=(z^(z>>30))*0xBF58476D1CE4E5B9ull;
    z=(z^(z>>27))*0x94D049BB133111EBull;
    return z^(z>>31);
}
extern "C" __global__ void discrete_carrier_select(const U* targets,const U* top8,const W* metrics,U rows,U vocabulary,U mode,W key,W* scratch,W* out,U* status) {
    __shared__ W value[CARRIER_THREADS];
    __shared__ U token[CARRIER_THREADS];
    __shared__ U low[CARRIER_THREADS];
    U t=threadIdx.x;
    if(blockDim.x!=CARRIER_THREADS || gridDim.x!=1u || rows==0u || vocabulary==0u || mode>2u){if(t==0u)atomicExch(status,4u);return;}
    U chunk=(vocabulary+CARRIER_THREADS-1u)/CARRIER_THREADS,first=t*chunk,last=first+chunk;
    if(first>vocabulary)first=vocabulary;
    if(last>vocabulary)last=vocabulary;
    for(U v=first;v<last;++v)scratch[v]=0;
    __syncthreads();
    U lowest=ABSENT;
    for(U r=t;r<rows;r+=CARRIER_THREADS){
        U target=targets[r];
        if(target>=vocabulary){atomicExch(status,4u);continue;}
        if(target<lowest)lowest=target;
        if(mode==0u){
            W h=metrics[r*14u]+metrics[r*14u+1u];
            if(h==0)continue;
            atomicAdd(scratch+target,h);
            U rival=top8[r*8u];
            if(metrics[r*14u+5u]==0 && rival<vocabulary && rival!=target)atomicAdd(scratch+rival,h);
        }else if(mode==1u)atomicExch(scratch+target,(W)1);
    }
    low[t]=lowest;
    __syncthreads();
    W best=0;U at=ABSENT;
    for(U v=first;v<last;++v){W c=scratch[v];if(mode==0u){if(c>best){best=c;at=v;}}else best+=c;}
    value[t]=best;token[t]=at;
    __syncthreads();
    if(t!=0u)return;
    U lowest_target=ABSENT;
    for(U i=0;i<CARRIER_THREADS;++i)if(low[i]<lowest_target)lowest_target=low[i];
    if(lowest_target==ABSENT){atomicExch(status,4u);return;}
    if(mode==2u){W r=carrier_mix(key)%(W)rows;out[0]=targets[r];out[1]=r;return;}
    if(mode==0u){
        W credit=0;U winner=ABSENT;
        for(U i=0;i<CARRIER_THREADS;++i)if(value[i]>credit){credit=value[i];winner=token[i];}
        if(credit==0){winner=lowest_target;}
        out[0]=winner;out[1]=credit;return;
    }
    W distinct=0;
    for(U i=0;i<CARRIER_THREADS;++i)distinct+=value[i];
    if(distinct==0){atomicExch(status,4u);return;}
    W k=carrier_mix(key)%distinct;
    for(U i=0;i<CARRIER_THREADS;++i){
        if(k>=value[i]){k-=value[i];continue;}
        U from=i*chunk,to=from+chunk;if(to>vocabulary)to=vocabulary;
        for(U v=from;v<to;++v)if(scratch[v]){if(k==0){out[0]=v;out[1]=distinct;return;}--k;}
    }
    atomicExch(status,4u);
}
