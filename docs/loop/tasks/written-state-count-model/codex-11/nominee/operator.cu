// Reuse the frozen D11 root metric, exact whole-word comparisons and context coordinates.
#define main original_cross_main
#include "../../codex-10/selector/cross.cu"
#undef main
struct Extra { U upper[4096],delimiter[4096]; };
struct Example { Site context; U upper, left; };
struct Evidence { U w[14]; };
struct Header { U mode,n,roles,positive,examples,allowed,permissions; };
constexpr U FIELDS=10, OUT=18;
HD Example feature(const Tables& t,const Extra& e,const Row& r,U p){
    return {context(t,r,word(t,r,p)), e.upper[r.ids[p]], p?e.delimiter[r.ids[p-1]]:0};
}
HD U cost(const Tables& t,const Example& a,const Example& b){
    if(a.upper!=b.upper)return 4096;
    U d=(a.left!=b.left)*64;
    for(U i=0;i<6;++i)d+=distance(t,a.context.x[i],b.context.x[i]);
    return d;
}
HD void extract(const Tables& t,const Extra& e,const Row& r,U* out){
    for(U i=0;i<W*FIELDS;++i)out[i]=X;
    for(U i=0;i<r.n;++i)if(t.cls[r.ids[i]]==1){
        Example x=feature(t,e,r,i);U *slot=out+i*FIELDS;
        for(U j=0;j<6;++j)slot[j]=x.context.x[j];
        slot[6]=x.upper;slot[7]=x.left;slot[8]=(i==r.q);
        slot[9]=r.old[i][0] && i?r.ids[i-1]:X;
    }
}
// Layout of each arm is nominee, first eligible earlier fact, support count, all earlier matches.
HD void support(const Tables& t,const Extra& e,const Row& r,U q,U* out){
    out[0]=q;out[1]=X;out[2]=out[3]=0;
    if(q==X)return;
    auto a=word(t,r,q);
    for(U i=0;i+1<q;++i)if(t.cls[r.ids[i]]==1 && same(r,q,i,a,word(t,r,i))){
        ++out[3];
        if(i && e.delimiter[r.ids[i-1]]){
            if(out[1]==X)out[1]=i;
            ++out[2];
        }
    }
}
HD U permission(const Tables& t,Site a,Site b,U count,bool agree,const Evidence& x){
    U sum=0;
    for(U i=0;i<6;++i)sum+=distance(t,a.x[i],x.w[i])+distance(t,b.x[i],x.w[i+6]);
    int c=static_cast<int>(count>2?2:count)-static_cast<int>(x.w[12]);
    return sum+64*((c<0?-c:c)+(agree!=x.w[13]));
}
HD void choose_request(const Header& h,const Tables& t,const Extra& e,const Site* roles,
                       const Example* bank,const Evidence* gate,const Row& r,U* out){
    U old=X,fresh=X,old_score=X,best_pos=X; int best_delta=8193;
    for(U i=0;i<r.n;++i){
        if(t.cls[r.ids[i]]!=1)continue;
        Example x=feature(t,e,r,i);
        U role=contrast(nearest(t,roles,0,x.context),nearest(t,roles,1,x.context));
        if(role<old_score){old_score=role;old=i;}
        U pos=4096,neg=4096;
        for(U j=0;j<h.examples;++j){
            U d=cost(t,x,bank[j]);U& best=j<h.positive?pos:neg;
            if(d<best)best=d;
        }
        int delta=static_cast<int>(pos)-static_cast<int>(neg);
        if(delta<best_delta||(delta==best_delta&&pos<best_pos)){best_delta=delta;best_pos=pos;fresh=i;}
    }
    support(t,e,r,old,out); support(t,e,r,fresh,out+4); support(t,e,r,r.q,out+8);
    out[12]=out[13]=X;out[14]=0;out[15]=best_pos;out[16]=8193+best_delta;out[17]=old_score;
    if(r.f==X||r.source==X||fresh==X||!t.cls[r.ids[r.f]])return;
    auto chosen=word(t,r,r.f); Site a=context(t,r,chosen),b=context(t,r,word(t,r,fresh));
    for(U j=0;j<h.permissions;++j){
        U& best=out[j<h.allowed?12:13];U d=permission(t,a,b,out[7],chosen.l==fresh,gate[j]);
        if(d<best)best=d;
    }
    out[14]=(out[12]<out[13]);
}
__global__ void nominate(Header h,Tables* t,Extra* e,Site* roles,Example* bank,Evidence* gate,Row* rows,U width,U* out){
    U i=blockIdx.x*blockDim.x+threadIdx.x;if(i>=h.n)return;
    if(h.mode==0)extract(*t,*e,rows[i],out+i*width);
    else choose_request(h,*t,*e,roles,bank,gate,rows[i],out+i*width);
}
int main(){
    Header h;read(&h,1);
    if(h.mode>1||!h.n||h.n>8192||!h.roles||h.roles>32768||h.positive>h.examples||h.examples>32768||h.allowed>h.permissions||h.permissions>8192)return 2;
    Tables t; Extra e;read(&t,1);read(&e,1);
    if(!std::is_sorted(t.ends,t.ends+K)||t.ends[K-1]!=h.roles)return 2;
    std::vector<Site> roles(h.roles);read(roles.data(),h.roles);
    std::vector<Example> bank(std::max(1u,h.examples));read(bank.data(),h.examples);
    std::vector<Evidence> gate(std::max(1u,h.permissions));read(gate.data(),h.permissions);
    std::vector<Row> rows(h.n);read(rows.data(),h.n);
    if(std::fgetc(stdin)!=EOF)return 2;
    for(const auto& r:rows){
        if(!r.n||r.n>W)return 2;
        for(U j=0;j<r.n;++j)if(r.ids[j]>=4096)return 2;
        for(U at:{r.q,r.f,r.source})if(at!=X&&at>=r.n)return 2;
    }
    U width=h.mode?OUT:W*FIELDS;
    auto dt=device(&t,1);auto de=device(&e,1);auto dr=device(rows.data(),h.n);
    auto roles_d=device(roles.data(),roles.size());auto bank_d=device(bank.data(),bank.size());auto gate_d=device(gate.data(),gate.size());
    std::vector<U> out(h.n*width),ref(width);
    U* dout;check(cudaMalloc(&dout,out.size()*4));
    nominate<<<(h.n+63)/64,64>>>(h,dt,de,roles_d,bank_d,gate_d,dr,width,dout);
    check(cudaGetLastError());check(cudaMemcpy(out.data(),dout,out.size()*4,cudaMemcpyDeviceToHost));
    for(U i=0;i<h.n;++i){
        if(!h.mode)extract(t,e,rows[i],ref.data());
        else choose_request(h,t,e,roles.data(),bank.data(),gate.data(),rows[i],ref.data());
        if(!std::equal(ref.begin(),ref.end(),out.begin()+i*width)){std::fprintf(stderr,"host/device mismatch row %u\n",i);return 3;}
    }
    for(auto p:{(void*)dt,(void*)de,(void*)dr,(void*)roles_d,(void*)bank_d,(void*)gate_d,(void*)dout})check(cudaFree(p));
    return std::fwrite(out.data(),4,out.size(),stdout)==out.size()?0:2;
}
