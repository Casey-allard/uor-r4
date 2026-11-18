// Ahead-of-time integer CUDA implementation of span_selector.rs.
#define D11_CONST __device__ __constant__
#include "d11_tables.h"
using u32 = unsigned int;
constexpr u32 NONE = 0xffffffffu, WIDTH = 192, STRIDE = WIDTH+3, WORDS = 5;
__device__ u32 distance(u32 a,u32 b) {
    u32 cost=0;
    for(u32 k=0;k<4;++k) {
        const u32 x=(a>>(4*k))&15, y=(b>>(4*k))&15;
        for(u32 j=0;j<8;++j) {
            const int d=D11_ROOTS[x][j]-D11_ROOTS[y][j];
            cost+=d<0?-d:d;
        }
    }
    return cost;
}
extern "C" __global__ void span_fit(const u32 *data,const u32 *roots,
                                    u32 rows,u32 *bank,u32 *count) {
    if (blockIdx.x || threadIdx.x) return;
    u32 used=0;
    for(u32 i=0;i<rows;++i) {
        const u32* row=data+i*STRIDE;
        const u32 end=row[WIDTH+1], source=row[WIDTH+2], len=source-end-1;
        u32 r[WORDS]={len,NONE,NONE,NONE,roots[row[source]]};
        for(u32 j=0;j<len;++j) r[1+j]=roots[row[end+1+j]];
        bool found=false;
        for(u32 k=0;k<used && !found;++k) {
            found=true;
            for(u32 j=0;j<WORDS;++j) found &= r[j]==bank[k*WORDS+j];
        }
        if(!found) {
            for(u32 j=0;j<WORDS;++j) bank[used*WORDS+j]=r[j];
            ++used;
        }
    }
    *count=used;
}
struct Span {u32 left,right;};
__device__ Span bounds(const u32 *row,const u32 *classes,u32 at) {
    Span span={at,at};
    if(classes[row[at]]==0) return span;
    while(span.left && classes[row[span.left]]==2 && classes[row[span.left-1]]!=0) --span.left;
    while(span.right+1<row[WIDTH] && classes[row[span.right+1]]==2) ++span.right;
    return span;
}
__device__ u32 source_at(const u32 *row,const u32 *roots,const u32 *bank,
                         u32 count,u32 end,u32 limit) {
    u32 bestlen=0,best=0;
    for(u32 i=0;i<count;++i) {
        const u32 *r=bank+i*WORDS;
        const u32 n=r[0],s=end+1+n;
        if(s>=limit) continue;
        u32 cost=n*distance(roots[row[s]],r[4]);
        for(u32 j=0;j<n;++j) cost+=4*distance(roots[row[end+1+j]],r[1+j]);
        if(!bestlen || cost*bestlen<best*n || (cost*bestlen==best*n && n>bestlen)) {
            best=cost;
            bestlen=n;
        }
    }
    return bestlen?end+1+bestlen:NONE;
}
__device__ void choice(u32* out,u32 q,u32 f,Span qs,Span fs,u32 src) {
    out[0]=q;out[1]=f;out[2]=qs.left;out[3]=qs.right;out[4]=fs.left;out[5]=fs.right;out[6]=src;
}
extern "C" __global__ void span_predict(const u32 *data,const u32 *roots,const u32 *classes,
        const u32 *bank,u32 count,u32 oldroot,u32 rows,u32 *output) {
    const u32 i=threadIdx.x+blockIdx.x*blockDim.x;
    if(i>=rows) return;
    const u32* row=data+i*STRIDE;
    const u32 q=row[WIDTH+1], f=row[WIDTH+2];
    u32* out=output+i*21;
    if(q==NONE) {
        for(u32 j=0;j<21;++j) out[j]=NONE;
        return;
    }
    const Span qs=bounds(row,classes,q),fs=bounds(row,classes,f);
    bool match=fs.right<qs.left && f-fs.left==q-qs.left && qs.right-qs.left==fs.right-fs.left;
    for(u32 j=0;match && j<=fs.right-fs.left;++j) match=row[fs.left+j]==row[qs.left+j];
    u32 variable=NONE,fixed=NONE;
    if(match) {
        variable=source_at(row,roots,bank,count,fs.right,qs.left);
        if(fs.right+1<qs.left) {
            fixed=fs.right+1;
            if(fixed+1<qs.left && distance(roots[row[fixed]],oldroot)<64) ++fixed;
        }
    }
    choice(out,q,f,qs,fs,variable);
    choice(out+7,q,f,qs,fs,fixed);
    choice(out+14,q,f,{q,q},{f,f},source_at(row,roots,bank,count,f,q));
}
