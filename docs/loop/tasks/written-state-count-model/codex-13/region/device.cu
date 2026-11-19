// D11 packed root coordinates, sparse role cells and original half-open spans.
#include <cuda_runtime.h>
#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <cstdint>
#include <climits>
#include <vector>
using U = uint32_t;
constexpr U N=4096, W=192, O=6, R=65537, X=UINT32_MAX, Y=O*R;
struct Site { U root, cls; };
struct Cell { U key, counts[3]; };
struct Span { U start, end; };
struct Row { U ids[W], n, region_start, region_end; };
struct Rank { Span span; int best, next; };
static_assert(sizeof(Row)==(W+3)*sizeof(U));

__device__ U features(const Site* sites,const Row& row,U start,U* keys) {
    U end=start+1;
    while(end<row.n && sites[row.ids[end]].cls==2) ++end;
    U at[O]={start-3,start-2,start-1,end,end+1,end+2};
    for(U i=0;i<O;++i) keys[i]=i*R+(at[i]<row.n?sites[row.ids[at[i]]].root:R-1);
    return end;
}
__device__ const Cell* lookup(const Cell* cells,U n,U key) {
    U a=0,b=n;
    while(a<b){
        U m=(a+b)/2;
        if(cells[m].key<key)a=m+1;else b=m;
    }
    return a<n && cells[a].key==key?cells+a:nullptr;
}
__global__ void predict(const Site* sites,const Cell* cells,U cells_n,const Row* rows,U n,U* result) {
    U i=blockIdx.x*blockDim.x+threadIdx.x;
    if(i>=n)return;
    const Row& row=rows[i];
    Rank ranks[2]={{{X,X},INT_MIN,INT_MIN},{{X,X},INT_MIN,INT_MIN}};
    const int weights[O]={1,2,4,4,2,1};
    for(U p=row.region_start;p<row.region_end;++p){
        if(sites[row.ids[p]].cls!=1)continue;
        U keys[O]; U end=features(sites,row,p,keys);
        if(end>row.region_end)continue;
        int scores[2]={0,0};
        for(U k=0;k<O;++k){
            const Cell* c=lookup(cells,cells_n,keys[k]);
            if(c==nullptr)continue;
            int total=c->counts[0]+c->counts[1]+c->counts[2];
            for(U role=0;role<2;++role){
                int signed_evidence=16*static_cast<int>(c->counts[role+1])-3*total;
                scores[role]+=weights[k]*(signed_evidence/(total+2));
            }
        }
        for(U role=0;role<2;++role){
            int score=scores[role];
            if(score<=scores[1-role])continue;
            Rank& rank=ranks[role];
            if(score>rank.best) rank={{p,end},score,rank.best};
            else if(score>rank.next) rank.next=score;
        }
    }
    U* out=result+8*i;
    for(U role=0;role<2;++role){
        Rank rank=ranks[role];
        bool allowed=rank.best>=12 && rank.best>rank.next;
        out[2*role]=allowed?rank.span.start:X;
        out[2*role+1]=allowed?rank.span.end:X;
        out[4+role]=static_cast<U>(rank.best);
        out[6+role]=static_cast<U>(rank.next);
    }
}
static void check(cudaError_t e) {
    if(e==cudaSuccess)return;
    std::fprintf(stderr,"%s\n",cudaGetErrorString(e)); std::exit(2);
}
template<class T> static void read(T* p,size_t n) {
    if(std::fread(p,sizeof(T),n,stdin)!=n)std::exit(2);
}
template<class T> static T* upload(const T* p,size_t n) {
    T* d; check(cudaMalloc(&d,n*sizeof(T)));
    check(cudaMemcpy(d,p,n*sizeof(T),cudaMemcpyHostToDevice)); return d;
}
int main() {
    U h[2]; read(h,2);
    const U n=h[0],cells_n=h[1];
    if(n==0 || n>8192 || cells_n==0 || cells_n>Y)return 2;
    std::vector<Site> sites(N); read(sites.data(),N);
    for(auto s:sites)if(s.root>=R-1 || s.cls>2)return 2;
    std::vector<Cell> cells(cells_n); read(cells.data(),cells_n);
    U last=X;
    for(auto c:cells){
        if(c.key>=Y || (last!=X && c.key<=last))return 2;
        for(U count:c.counts)if(count>8192*W)return 2;
        last=c.key;
    }
    std::vector<Row> rows(n); read(rows.data(),n);
    if(std::fgetc(stdin)!=EOF)return 2;
    for(const auto& r:rows){
        if(r.n==0 || r.n>W || r.ids[0]!=0)return 2;
        for(U p=1;p<r.n;++p)if(r.ids[p]<2 || r.ids[p]>=N)return 2;
        if(r.region_start>r.region_end || r.region_end>r.n)return 2;
    }
    auto* ds=upload(sites.data(),N); auto* dr=upload(rows.data(),n);
    auto* dc=upload(cells.data(),cells_n);
    std::vector<U> result(8*n);
    U* dout; check(cudaMalloc(&dout,result.size()*sizeof(U)));
    predict<<<(n+63)/64,64>>>(ds,dc,cells_n,dr,n,dout);
    check(cudaGetLastError());
    check(cudaMemcpy(result.data(),dout,result.size()*sizeof(U),cudaMemcpyDeviceToHost));
    check(cudaFree(ds)); check(cudaFree(dr)); check(cudaFree(dc)); check(cudaFree(dout));
    return std::fwrite(result.data(),sizeof(U),result.size(),stdout)==result.size()?0:2;
}
