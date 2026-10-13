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
    if(length==0 || length>s[2] || nodes>s[6] || frontier==0 || frontier>16)return false;
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
// ---- Flock read (Claude; design docs/labs/discrete-murmuration-read-2026-10-13) ----
// Device spec words appended after s[0..7]: s[8] read mode (0 Route, 1 Flock), s[9] kept K.
// Gates are i32 trits, index block*stages+level (level 0 = atoms).
#define MODE_FLOCK 1u
#define MAX_KEPT 8u
#define MAX_NODES 364u
#define GATE_KIND 3u
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
// Flock read for one row. Payload block b of each kept candidate is carried by the
// actual BFS action between the candidate's and the query's FRAME roots; the
// consensus is snap_K of the carried sum; gates add (+1) or subtract (-1) each
// level's consensus to the query payload; out = snap_n of that exact residual.
// Optional trace (forward only; null elsewhere) from this SAME path, per (row, block):
//   [fq, out root, terms, query payload root], then per level l, TRACE_LEVEL words:
//   [gate+1, kept, consensus root, eligible count, kept node ids[8], actions[8], carried roots[8]]
// Unused words are ABSENT. Selection is traced even for gate 0 (output unchanged).
#define TRACE_ROW 4u
#define TRACE_LEVEL 28u
__device__ bool flock_vector(const int* input,const U* ids,const int* gates,const U* plan,const U* g,const U* s,U row,U kind,U index,U candidate,int* output,U* trace) {
    plan+=row*(3+s[6]*4+16);
    U length=plan[0],nodes=plan[1];plan+=3;
    U kept=s[9];
    if(length==0 || length>s[2] || nodes>s[6] || nodes>MAX_NODES || kept==0 || kept>MAX_KEPT || s[1]<2)return false;
    U level[MAX_NODES],last[MAX_NODES],frame[MAX_NODES];
    for(U node=0;node<nodes;++node) {
        U atom=plan[node*4+3];
        if(atom!=ABSENT){if(atom>=length)return false;level[node]=0;last[node]=atom;}
        else {
            U c0=plan[node*4],c2=plan[node*4+2];
            if(c0>=node || plan[node*4+1]>=node || c2>=node)return false;
            level[node]=level[c0]+1;last[node]=last[c2];
        }
    }
    for(U b=0;b<s[1];++b) {
        U f=frame_block(b,s[1]);
        int query_frame[8],u[8];
        input_block(input,ids,g,row,length-1,f,s,kind,index,candidate,query_frame);
        input_block(input,ids,g,row,length-1,b,s,kind,index,candidate,u);
        U fq=root_id(query_frame,g);if(fq==ABSENT)return false;
        // Node frames and block-b payloads, children before parents.
        int payload[MAX_NODES*8];
        for(U node=0;node<nodes;++node) {
            U atom=plan[node*4+3];
            if(atom!=ABSENT) {
                int fv[8];input_block(input,ids,g,row,atom,f,s,kind,index,candidate,fv);
                frame[node]=root_id(fv,g);if(frame[node]==ABSENT)return false;
                input_block(input,ids,g,row,atom,b,s,kind,index,candidate,payload+node*8);
            } else {
                U first=plan[node*4];frame[node]=frame[first];
                int sum[8]={0,0,0,0,0,0,0,0};
                for(U c=0;c<3;++c){U child=plan[node*4+c];int carried[8];
                    action(g,g[BFS+frame[child]*16+frame[first]],payload+child*8,carried);
                    for(U d=0;d<8;++d)sum[d]+=carried[d];}
                U r=snap(sum,3,g);for(U d=0;d<8;++d)payload[node*8+d]=(int)g[ROOT+r*8+d];
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
            for(U node=0;node<nodes;++node) {
                if(level[node]!=l || last[node]+1>=length)continue;
                ++eligible;
                U distance,hamming;root_cost(query_frame,g,frame[node],&distance,&hamming);
                // Insert by (distance asc, hamming asc, recency desc, node asc).
                U at=found;
                while(at>0) {
                    U o=top[at-1];
                    bool better=distance<key_d[at-1] || (distance==key_d[at-1] && (hamming<key_h[at-1]
                        || (hamming==key_h[at-1] && (last[node]>last[o] || (last[node]==last[o] && node<o)))));
                    if(!better)break;
                    if(at<kept){top[at]=top[at-1];key_d[at]=key_d[at-1];key_h[at]=key_h[at-1];}
                    --at;
                }
                if(at<kept){top[at]=node;key_d[at]=distance;key_h[at]=hamming;if(found<kept)++found;}
            }
            if(lv){lv[1]=found;lv[3]=eligible;}
            if(found==0)continue;
            int sum[8]={0,0,0,0,0,0,0,0};
            for(U k=0;k<found;++k){U node=top[k];int carried[8];U a=g[BFS+frame[node]*16+fq];
                action(g,a,payload+node*8,carried);
                for(U d=0;d<8;++d)sum[d]+=carried[d];
                if(lv){lv[4+k]=node;lv[12+k]=a;lv[20+k]=root_id(carried,g);}}
            U m=snap(sum,found,g);
            if(lv)lv[2]=m;
            if(gate==0)continue;
            for(U d=0;d<8;++d){int v=(int)g[ROOT+m*8+d];u[d]+=gate>0?v:-v;}
            ++terms;
        }
        U out=snap(u,terms,g);
        if(rec){rec[1]=out;rec[2]=terms;}
        for(U d=0;d<8;++d)output[b*8+d]=(int)g[ROOT+out*8+d];
    }
    return true;
}
// Mode dispatch: Route (default) or Flock. A Route proposal in Flock mode and a
// Gate proposal in Route mode are refused before any read (status 9).
__device__ bool read_vector(const int* input,const U* ids,const U* routes,const int* gates,const U* plan,const U* g,const U* s,U row,U kind,U index,U candidate,int* output,U* trace) {
    if(s[8]==MODE_FLOCK)return flock_vector(input,ids,gates,plan,g,s,row,kind,index,candidate,output,trace);
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
// Product addresses derive from current codebook geometry, including virtual
// moves. Full bounded vocabulary scans retain every crowded-cell identity.
// An absent comparison has zero hinge and an explicit eligibility count.
__device__ W writer(const int* q,const int* table,const U* g,const U* s,U target,U kind,U index,U candidate,U* prediction,U* token_rivals,U* address_rivals,U* admitted) {
    bool occupied[256],used[256];for(U i=0;i<256;++i){occupied[i]=false;used[i]=false;}
    for(U token=0;token<s[0];++token){U c=token_cell(table,g,s,token,kind,index,candidate);if(c==ABSENT)return ~(W)0;occupied[c]=true;}
    U best_token=ABSENT,best_distance=ABSENT,rival_distance=ABSENT,rival_address=ABSENT;
    U target_cell=token_cell(table,g,s,target,kind,index,candidate);
    *token_rivals=0;*address_rivals=0;if(admitted)for(U i=0;i<16;++i)admitted[i]=ABSENT;
    for(U k=0;k<s[3];++k) {
        U cell=ABSENT,cost=ABSENT;
        for(U c=0;c<256;++c)if(occupied[c] && !used[c]){U d=address_distance(q,g,s,c);if(d<cost){cell=c;cost=d;}}
        if(cell==ABSENT)break;
        used[cell]=true;if(admitted)admitted[k]=cell;
        if(cell!=target_cell){++*address_rivals;if(cost<rival_address)rival_address=cost;}
        for(U token=0;token<s[0];++token)if(token_cell(table,g,s,token,kind,index,candidate)==cell){
            U d=token_distance(q,table,g,s,token,kind,index,candidate);
            if(d<best_distance || (d==best_distance && token<best_token)){best_token=token;best_distance=d;}
            if(token!=target){++*token_rivals;if(d<rival_distance)rival_distance=d;}
        }
    }
    *prediction=best_token;if(best_token==ABSENT)return ~(W)0;
    U width=s[1]*8,td=token_distance(q,table,g,s,target,kind,index,candidate),ad=address_distance(q,g,s,target_cell);
    W tm=rival_distance==ABSENT || width+td<=rival_distance ? 0 : width+td-rival_distance;
    W am=rival_address==ABSENT || 8+ad<=rival_address ? 0 : 8+ad-rival_address;
    return (tm<<3)+scaled_add(am,width);
}
// Statuses added with the flock read: 9 = read-mode/proposal-kind mismatch
// (Gate in Route mode, Route in Flock mode); 10 = gate trit outside {-1,0,1}
// or read mode/K out of range.
extern "C" __global__ void discrete_init(int* table,U* routes,int* gates,const U* g,const U* s,U seed,U* status) {
    U i=blockIdx.x*blockDim.x+threadIdx.x;
    U elements=s[0]*s[1]*8;
    if(i<elements) {
        U block=i/8;U state=seed^block^0x9e3779b9u;
        state^=state<<13;state^=state>>17;state^=state<<5;
        table[i]=(int)g[ROOT+(state&15)*8+(i&7)];
    }
    if(i<s[1]*s[5]){routes[i]=g[IDENTITY];gates[i]=0;}
    if(i==0)*status=0;
}
extern "C" __global__ void discrete_encode(const int* table,const U* ids,U count,U width,U vocabulary,int* out,U* status) {
    U i=blockIdx.x*blockDim.x+threadIdx.x;if(i>=count*width)return;
    U token=ids[i/width];if(token>=vocabulary){atomicExch(status,1u);return;}out[i]=table[token*width+i%width];
}
// trace: Flock mode only, rows*blocks*(TRACE_ROW+stages*TRACE_LEVEL) words; ignored in Route mode.
extern "C" __global__ void discrete_forward(const int* table,const U* routes,const int* gates,const U* ids,const int* input,const U* targets,const U* plan,const U* g,const U* s,int* out,U* predictions,W* losses,U* token_rivals,U* address_rivals,U* admitted,U* trace,U* status) {
    U row=blockIdx.x*blockDim.x+threadIdx.x;if(row>=s[4])return;
    int q[MAX_WIDTH];if(!read_vector(input,ids,routes,gates,plan,g,s,row,2,0,0,q,s[8]==MODE_FLOCK?trace:0)){atomicExch(status,2u);return;}
    for(U d=0;d<s[1]*8;++d)out[row*s[1]*8+d]=q[d];
    losses[row]=writer(q,table,g,s,targets[row],2,0,0,predictions+row,token_rivals+row,address_rivals+row,admitted+row*16);
    if(losses[row]==~(W)0)atomicExch(status,8u);
}
// Sixteen fixed lanes. Gate (kind 3) uses ordinals 0..2 only; lanes 3..15 stay MAX.
extern "C" __global__ void discrete_counterfactual(const int* table,const U* routes,const int* gates,const U* ids,const int* input,const U* targets,const U* plan,const U* g,const U* s,U kind,U index,W* losses,U* status) {
    U candidate=blockIdx.x*blockDim.x+threadIdx.x;if(candidate>=16)return;
    W sum=0;U eligible=0;losses[candidate]=~(W)0;
    bool flock=s[8]==MODE_FLOCK;
    if((kind==GATE_KIND && !flock) || (kind==0 && flock)){atomicExch(status,9u);return;}
    if(kind==GATE_KIND && index>=s[1]*s[5]){atomicExch(status,4u);return;}
    if(kind==GATE_KIND && candidate>=3)return;
    for(U row=0;row<s[4];++row){int q[MAX_WIDTH],baseline[MAX_WIDTH];U prediction,tr,ar;
        if(!read_vector(input,ids,routes,gates,plan,g,s,row,2,0,0,baseline,0)){atomicExch(status,2u);return;}
        W base=writer(baseline,table,g,s,targets[row],2,0,0,&prediction,&tr,&ar,0);
        if(base==~(W)0){atomicExch(status,8u);return;}
        // The baseline fixes eligible rows. A virtual move cannot improve by
        // removing its supervised competitor; absent baseline comparisons skip.
        if(tr==0)continue;++eligible;
        if(!read_vector(input,ids,routes,gates,plan,g,s,row,kind,index,candidate,q,0)){atomicExch(status,2u);return;}
        W cost=writer(q,table,g,s,targets[row],kind,index,candidate,&prediction,&tr,&ar,0);
        if(cost==~(W)0 || tr==0){sum=~(W)0;break;}
        sum+=cost;
    }
    if(eligible==0){atomicExch(status,8u);return;}
    losses[candidate]=sum;
}
__device__ U resulting_port(const int* table,const U* routes,const U* g,const U* s,U kind,U index,U candidate) {
    U a=move_action(g,candidate);
    if(kind==0)return compose(g,a,routes[index]);
    int out[8];action(g,a,table+index*8,out);return root_id(out,g);
}
extern "C" __global__ void discrete_choose(const W* losses,const W* epoch,const int* table,const U* routes,const int* gates,const U* g,const U* s,U kind,U index,W* proposal) {
    if(blockIdx.x || threadIdx.x)return;U best=0;
    if(kind==GATE_KIND) {
        // Strict improvement only; ties keep the lower ordinal (CURRENT, then 0, -1, +1).
        for(U i=1;i<3;++i)if(losses[i]<losses[best])best=i;
    } else {
        for(U i=1;i<16;++i) {
            bool tie=best!=0 && losses[i]==losses[best] && resulting_port(table,routes,g,s,kind,index,i)<resulting_port(table,routes,g,s,kind,index,best);
            if(losses[i]<losses[best] || tie)best=i;
        }
    }
    proposal[0]=best;proposal[1]=losses[0];proposal[2]=losses[best];proposal[3]=*epoch;
}
extern "C" __global__ void discrete_apply(int* table,U* routes,int* gates,const U* g,const U* s,const W* proposal,U kind,U index,W* epoch,U* status) {
    if(blockIdx.x || threadIdx.x)return;
    if(proposal[3]!=*epoch){*status=3;return;}
    if(kind==GATE_KIND) {
        if(s[8]!=MODE_FLOCK){*status=9;return;}
        if(proposal[0]>=3 || index>=s[1]*s[5]){*status=4;return;}
        if(proposal[0]==0)return;
        if(proposal[2]>=proposal[1] || *epoch==~(W)0){*status=5;return;}
        gates[index]=gate_value(gates,index,GATE_KIND,index,(U)proposal[0]);
        ++*epoch;return;
    }
    if(kind==0 && s[8]==MODE_FLOCK){*status=9;return;}
    if(proposal[0]>=16 || kind>1 || (kind==0 && index>=s[1]*s[5]) || (kind==1 && index>=s[0]*s[1])){*status=4;return;}
    if(proposal[0]==0)return;
    if(proposal[2]>=proposal[1] || *epoch==~(W)0){*status=5;return;}
    U a=move_action(g,(U)proposal[0]);
    if(kind==0)routes[index]=compose(g,a,routes[index]);
    else {int out[8];action(g,a,table+index*8,out);for(U d=0;d<8;++d)table[index*8+d]=out[d];}
    ++*epoch;
}
extern "C" __global__ void discrete_validate(const int* table,const U* routes,const int* gates,const U* g,const U* s,U* status) {
    U i=blockIdx.x*blockDim.x+threadIdx.x;
    if(i<s[0]*s[1]) {
        bool found=false;
        for(U r=0;r<16;++r){bool equal=true;for(U d=0;d<8;++d)if(table[i*8+d]!=(int)g[r*8+d])equal=false;found|=equal;}
        if(!found)atomicExch(status,6u);
    }
    if(i<s[1]*s[5] && routes[i]>=64)atomicExch(status,7u);
    if(i<s[1]*s[5] && (gates[i]<-1 || gates[i]>1))atomicExch(status,10u);
    if(i==0 && (s[8]>MODE_FLOCK || (s[8]==MODE_FLOCK && (s[9]==0 || s[9]>MAX_KEPT || s[1]<2))))atomicExch(status,10u);
}
