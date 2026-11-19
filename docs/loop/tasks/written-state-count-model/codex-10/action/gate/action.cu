// Reuse the released geometry, tokenizer bounds and AOT/host I/O types.
#define main previous_experiment_main
#include "../../selector/cross.cu"
#undef main
struct Evidence { U word[14]; };
constexpr U RESULT=20;
HD U evidence_distance(const Tables& t,const Evidence& x,const Evidence& y){
    U sum=0;
    for(unsigned i=0;i<12;++i) sum+=distance(t,x.word[i],y.word[i]);
    for(unsigned i=12;i<14;++i)
        sum+=64*(x.word[i]>y.word[i]?x.word[i]-y.word[i]:y.word[i]-x.word[i]);
    return sum;
}
HD void gate(const Tables& t,const Site* roles,const Evidence* bank,U positive,U total,
             const Row& row,U* out){
    for(U i=0;i<RESULT;++i) out[i]=X;
    out[19]=0;
    if(row.q==X || row.source==X || !t.cls[row.ids[row.q]]) return;
    Span chosen=word(t,row,row.q);
    U nom=X,best=X;
    for(U p=0;p<row.n;++p){
        if(t.cls[row.ids[p]]!=1) continue;
        Site c=context(t,row,word(t,row,p));
        U e=contrast(nearest(t,roles,0,c),nearest(t,roles,1,c));
        if(e<best){nom=p;best=e;}
    }
    if(nom==X) return;
    Span named=word(t,row,nom);
    U count=0;
    for(U f=0;f<nom && count<2;++f){
        if(t.cls[row.ids[f]]==1 && same(row,nom,f,named,word(t,row,f))) ++count;
    }
    Evidence features;
    Site chosen_ctx=context(t,row,chosen),nominee_ctx=context(t,row,named);
    for(U i=0;i<6;++i){
        features.word[i]=chosen_ctx.x[i];
        features.word[i+6]=nominee_ctx.x[i];
    }
    features.word[12]=count;features.word[13]=(chosen.l==named.l);
    for(U i=0;i<14;++i) out[i]=features.word[i];
    out[14]=nom;out[15]=chosen.l;out[16]=row.wrong;
    for(U i=0;i<total;++i){
        U group=(i<positive)?17:18, d=evidence_distance(t,bank[i],features);
        if(d<out[group]) out[group]=d;
    }
    out[19]=(out[17]<out[18]);
}
__global__ void action_kernel(const Tables* t,const Site* roles,const Evidence* bank,U pos,U total,
                              const Row* rows,U n,U* out){
    U r=blockIdx.x*blockDim.x+threadIdx.x;
    if(r<n)gate(*t,roles,bank,pos,total,rows[r],out+r*RESULT);
}
int main(){
    U h[4];read(h,4);U n=h[0],role_count=h[1],pos=h[2],total=h[3];
    if(!n||n>4096||role_count>24576||!role_count||total>8192||pos>total)return 2;
    Tables t;read(&t,1);
    if(!std::is_sorted(t.ends,t.ends+K)||t.ends[K-1]!=role_count||!t.ends[0]||t.ends[1]<=t.ends[0])return 2;
    std::vector<Site> roles(role_count);read(roles.data(),role_count);
    std::vector<Evidence> bank(std::max(total,1u));read(bank.data(),total);
    std::vector<Row> rows(n);read(rows.data(),n);
    if(std::fgetc(stdin)!=EOF)return 2;
    for(auto& r:rows){
        if(!r.n||r.n>W||(r.q!=X&&r.q>=r.n)||(r.source!=X&&r.source>=r.n))return 2;
        for(U i=0;i<r.n;++i)if(r.ids[i]>=4096)return 2;
    }
    auto dt=device(&t,1);auto dp=device(roles.data(),roles.size());
    auto db=device(bank.data(),bank.size());auto dr=device(rows.data(),n);
    std::vector<U> output(n*RESULT);U* dout;check(cudaMalloc(&dout,output.size()*4));
    action_kernel<<<(n+63)/64,64>>>(dt,dp,db,pos,total,dr,n,dout);
    check(cudaGetLastError());check(cudaMemcpy(output.data(),dout,output.size()*4,cudaMemcpyDeviceToHost));
    for(U i=0;i<n;++i){
        U ref[RESULT];gate(t,roles.data(),bank.data(),pos,total,rows[i],ref);
        if(!std::equal(ref,ref+RESULT,output.data()+i*RESULT))return 2;
    }
    for(auto p:{(void*)dt,(void*)dp,(void*)db,(void*)dr,(void*)dout})check(cudaFree(p));
    return std::fwrite(output.data(),4,output.size(),stdout)==output.size()?0:2;
}
