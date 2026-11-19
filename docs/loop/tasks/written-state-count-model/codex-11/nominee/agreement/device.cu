// The frozen selector word equality and the writer's recorded state are the only inputs.
#define main original_cross_main
#include "../../../codex-10/selector/cross.cu"
#undef main
constexpr U LIMIT=16;
struct Step {U copied,release,id,source_id,before[4],after[4];};
struct Run {U source,n; Step step[LIMIT];};
struct Proof {U n,ids[W],offset,count;};
HD void candidates(const Tables& t,const U* delimiter,const Row& r,U* out){
    for(U i=0;i<W;++i)out[i]=X;
    if(r.q==X)return;
    auto q=word(t,r,r.q); U ct=0;
    for(U i=1;i+1<r.q;++i){
        if(t.cls[r.ids[i]]==1&&delimiter[r.ids[i-1]]&&same(r,r.q,i,q,word(t,r,i)))out[ct++]=i;
    }
}
HD bool complete(const Proof& p,const Run& r){
    if(r.source>=p.n||r.n<2||r.n>LIMIT)return false;
    for(U i=0;i<r.n;++i){
        const Step& s=r.step[i];U at=r.source+i;bool final=(i+1==r.n);
        if(at>=p.n||s.before[0]!=1||s.before[1]!=r.source||s.before[2]!=at||s.before[3]!=p.n||
           s.after[1]!=r.source||s.after[3]!=p.n)return false;
        if(final){
            if(s.copied!=0||s.release!=1||s.id!=1||s.source_id!=X||s.after[0]||s.after[2]!=at)return false;
        }else if(s.copied!=1||s.release!=0||s.id<2||s.id!=p.ids[at]||s.source_id!=s.id||
                 s.after[0]!=1||s.after[2]!=at+1)return false;
    }
    return true;
}
// Outputs earliest source, number complete, disagreement flag and support count.
HD void agree(const Proof& p,const Run* runs,U* o){
    o[0]=X;o[1]=o[2]=0;o[3]=p.count;
    if(!p.count)return;
    const Run& first=runs[p.offset];
    for(U i=0;i<p.count;++i){
        const Run& a=runs[p.offset+i];if(complete(p,a))++o[1];
        if(first.n!=a.n)o[2]=1;
        if(first.n==a.n && a.n<=LIMIT)for(U j=0;j<a.n;++j)if(first.step[j].id!=a.step[j].id)o[2]=1;
    }
    if(o[1]==p.count&&!o[2])o[0]=first.source;
}
__global__ void enumerate_kernel(const Tables* t,const U* d,const Row* r,U n,U* o){
    U i=blockIdx.x*blockDim.x+threadIdx.x;if(i<n)candidates(*t,d,r[i],o+i*W);
}
__global__ void agreement_kernel(const Proof* p,const Run* r,U n,U* o){
    U i=blockIdx.x*blockDim.x+threadIdx.x;if(i<n)agree(p[i],r,o+4*i);
}
int main(){
    U mode,n;read(&mode,1);read(&n,1);if(mode>1||!n||n>8192)return 2;
    U width=mode?4:W;std::vector<U> result(width*n),ref(width*n);
    U* dout;check(cudaMalloc(&dout,result.size()*4));
    if(!mode){
        Tables t{};U d[4096];read(t.cls,4096);read(d,4096);std::vector<Row> rows(n);read(rows.data(),n);
        for(const auto& r:rows){
            if(!r.n||r.n>W||(r.q!=X&&r.q>=r.n))return 2;
            for(U j=0;j<r.n;++j)if(r.ids[j]>=4096)return 2;
        }
        auto dt=device(&t,1);auto dd=device(d,4096);auto dr=device(rows.data(),n);
        enumerate_kernel<<<(n+63)/64,64>>>(dt,dd,dr,n,dout);
        check(cudaGetLastError());check(cudaMemcpy(result.data(),dout,result.size()*4,cudaMemcpyDeviceToHost));
        for(U i=0;i<n;++i)candidates(t,d,rows[i],ref.data()+i*width);
        for(auto p:{(void*)dt,(void*)dd,(void*)dr})check(cudaFree(p));
    }else{
        U cnt;read(&cnt,1);if(cnt>n*W)return 2;
        std::vector<Proof> p(n);read(p.data(),n);std::vector<Run> r(std::max(1u,cnt));read(r.data(),cnt);
        for(auto& v:p)if(v.n>W||v.offset>cnt||v.count>cnt-v.offset)return 2;
        for(U i=0;i<cnt;++i)if(r[i].n>LIMIT)return 2;
        auto dp=device(p.data(),n);auto dr=device(r.data(),r.size());
        agreement_kernel<<<(n+63)/64,64>>>(dp,dr,n,dout);
        check(cudaGetLastError());check(cudaMemcpy(result.data(),dout,result.size()*4,cudaMemcpyDeviceToHost));
        for(U i=0;i<n;++i)agree(p[i],r.data(),ref.data()+4*i);
        check(cudaFree(dp));check(cudaFree(dr));
    }
    check(cudaFree(dout));
    if(std::fgetc(stdin)!=EOF||result!=ref)return 2;
    return std::fwrite(result.data(),4,result.size(),stdout)==result.size()?0:2;
}
