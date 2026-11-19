// The same integer operators run on the host and the device. The host runner
// rejects any different transition word before returning the CUDA result.
#include <cuda_runtime.h>
#include <algorithm>
#include <array>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <vector>
using U = uint32_t;
constexpr U X = UINT32_MAX, W = 192, STRIDE = W * 3 + 8, D = 6, K = 5;
#define HD __host__ __device__
struct Tables { U roots[4096], cls[4096], dist[256], ends[K]; };
struct Site { U x[D]; };
struct Row {
    U ids[W], old[W][2], n, q, other_q, f, source, wrong, spare1, spare2;
};
static_assert(sizeof(Row)==STRIDE*sizeof(U));
struct Span { U l,r; };
HD Span word(const Tables& t,const Row& r,U at){
    U l=at,h=at;
    if(t.cls[r.ids[at]]){
        while(l && t.cls[r.ids[l]]==2 && t.cls[r.ids[l-1]]) --l;
        while(h+1<r.n && t.cls[r.ids[h+1]]==2) ++h;
    }
    return {l,h};
}
HD U root(const Tables& t,const Row& r,int i){
    return i>=0 && (U)i<r.n ? t.roots[r.ids[i]] : X;
}
HD Site context(const Tables& t,const Row& r,Span w){
    Site s;
    for(int i=0;i<3;++i){
        s.x[i]=root(t,r,(int)w.l-3+i);
        s.x[i+3]=root(t,r,(int)w.r+1+i);
    }
    return s;
}
HD Site onset(const Tables& t,const Row& r,U end,U at){
    Site s;
    for(int i=0;i<3;++i){
        int prev=(int)at-3+i;
        s.x[i]=prev>(int)end ? root(t,r,prev) : X;
        s.x[i+3]=root(t,r,(int)at+i);
    }
    return s;
}
HD U distance(const Tables& t,U a,U b){
    if(a==b) return 0;
    if(a==X || b==X) return 64;
    U sum=0;
    for(U i=0;i<4;++i) sum+=t.dist[((a>>(4*i))&15)*16+((b>>(4*i))&15)];
    return sum;
}
HD U nearest(const Tables& t,const Site* bank,U kind,Site s){
    U best=X;
    for(U j=kind?t.ends[kind-1]:0;j<t.ends[kind] && best;++j){
        U total=0;
        for(U i=0;i<D && total<best;++i) total+=distance(t,s.x[i],bank[j].x[i]);
        if(total<best) best=total;
    }
    return best;
}
HD U contrast(U pos,U neg){ return pos+(neg<48?48-neg:0); }
HD bool same(const Row& r,U q,U f,Span a,Span b){
    if(r.ids[q]!=r.ids[f] || b.r>=a.l || q-a.l!=f-b.l || a.r-a.l!=b.r-b.l) return false;
    for(U j=0;j<=a.r-a.l;++j) if(r.ids[a.l+j]!=r.ids[b.l+j]) return false;
    return true;
}
HD void fit(const Tables& t,const Row& r,U* out){
    for(U j=0;j<6*7;++j) out[j]=X;
    U at[3]={r.q,r.other_q,r.f};
    for(U k=0;k<6;++k){
        Site s; U kind=k;
        if(k<3){
            if(at[k]==X) continue;
            s=context(t,r,word(t,r,at[k]));
        } else {
            U end=word(t,r,r.f).r, start=end+2+(k-3);
            if(start>=word(t,r,r.q).l || t.cls[r.ids[start]]!=1) continue;
            kind=start==r.source?3:4;
            s=onset(t,r,end,start);
        }
        out[7*k]=kind;
        for(U j=0;j<D;++j) out[7*k+1+j]=s.x[j];
    }
}
struct Pick { U q,f,cost; };
HD void choose(Pick& p,U q,U f,U cost){ if(cost<p.cost) p={q,f,cost}; }
HD void predict(const Tables& t,const Site* bank,const Row& r,U* out){
    Span bounds[W]; U role[W],fact[W];
    Pick picks[5];
    for(U arm=0;arm<5;++arm) picks[arm]={X,X,X};
    for(U q=0;q<r.n;++q){
        Span a=word(t,r,q); bounds[q]=a;
        if(q==a.l){
            Site s=context(t,r,a);
            role[q]=contrast(nearest(t,bank,0,s),nearest(t,bank,1,s));
            fact[q]=nearest(t,bank,2,s);
        } else {role[q]=role[a.l]; fact[q]=fact[a.l];}
        for(U f=0;f+1<q;++f){
            if(!same(r,q,f,a,bounds[f])) continue;
            U old=r.old[q][0]+r.old[f][1];
            choose(picks[0],q,f,old);
            choose(picks[1],q,f,role[q]+fact[f]);
            bool qm=r.q!=X && a.l<=r.q && r.q<=a.r;
            bool fm=r.f!=X && bounds[f].l<=r.f && r.f<=bounds[f].r;
            if(qm) choose(picks[2],q,f,old);
            if(fm) choose(picks[3],q,f,old);
            if(qm && fm) choose(picks[4],q,f,old);
        }
    }
    for(U arm=0;arm<5;++arm){
        Pick p=picks[arm]; U best=X,start=X;
        if(p.q!=X){
            U end=bounds[p.f].r;
            for(U j=end+2;j<=end+4 && j<bounds[p.q].l;++j){
                if(t.cls[r.ids[j]]!=1) continue;
                Site s=onset(t,r,end,j);
                U e=contrast(nearest(t,bank,3,s),nearest(t,bank,4,s));
                if(e<best){best=e;start=j;}
            }
        }
        out[4*arm]=p.q; out[4*arm+1]=p.f; out[4*arm+2]=start; out[4*arm+3]=p.cost;
    }
}
__global__ void execute(const Tables* t,const Site* bank,const Row* rows,U n,U isfit,U* out){
    U i=blockDim.x*blockIdx.x+threadIdx.x;
    if(i<n){
        if(isfit) fit(*t,rows[i],out+i*42);
        else predict(*t,bank,rows[i],out+i*20);
    }
}
static void check(cudaError_t e){
    if(e!=cudaSuccess){std::fprintf(stderr,"%s\n",cudaGetErrorString(e));std::exit(2);}
}
template<class T> static T* device(const T* p,size_t n){
    T* d; check(cudaMalloc(&d,n*sizeof(T)));
    check(cudaMemcpy(d,p,n*sizeof(T),cudaMemcpyHostToDevice));return d;
}
template<class T> static void read(T* p,size_t n){
    if(std::fread(p,sizeof(T),n,stdin)!=n){std::fprintf(stderr,"short input\n");std::exit(2);}
}
int main(){
    U hdr[3];read(hdr,3);
    U isfit=hdr[0],n=hdr[1],nb=hdr[2];
    if(isfit>1 || !n || n>4096 || nb>6*4096) return 2;
    Tables t;read(&t,1);
    if(!std::is_sorted(t.ends,t.ends+K) || t.ends[K-1]!=nb) return 2;
    std::vector<Site> bank(std::max(1u,nb));read(bank.data(),nb);
    std::vector<Row> rows(n);read(rows.data(),n);
    if(std::fgetc(stdin)!=EOF) return 2;
    for(const auto& r:rows){
        if(!r.n || r.n>W) return 2;
        for(U i=0;i<r.n;++i) if(r.ids[i]>=4096) return 2;
        for(U v:{r.q,r.f,r.other_q,r.source,r.wrong}) if(v!=X && v>=r.n) return 2;
        if(isfit && (r.q==X||r.f==X||r.source==X)) return 2;
    }
    auto dt=device(&t,1); auto db=device(bank.data(),bank.size()); auto dr=device(rows.data(),n);
    U width=isfit?42:20;
    std::vector<U> result(n*width);
    U* dout;check(cudaMalloc(&dout,result.size()*sizeof(U)));
    execute<<<(n+63)/64,64>>>(dt,db,dr,n,isfit,dout);
    check(cudaGetLastError());check(cudaMemcpy(result.data(),dout,result.size()*4,cudaMemcpyDeviceToHost));
    for(U i=0;i<n;++i){
        U ref[42];
        if(isfit) fit(t,rows[i],ref); else predict(t,bank.data(),rows[i],ref);
        if(!std::equal(ref,ref+width,result.data()+i*width)){
            std::fprintf(stderr,"device/host mismatch row %u\n",i);return 2;
        }
    }
    for(auto p:{(void*)dt,(void*)db,(void*)dr,(void*)dout}) check(cudaFree(p));
    return std::fwrite(result.data(),sizeof(U),result.size(),stdout)==result.size()?0:2;
}
