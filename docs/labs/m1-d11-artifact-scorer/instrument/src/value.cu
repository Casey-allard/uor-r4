// Native token-identity registration and masked connector search on frozen endpoints.
#define D11_CONST __device__ __constant__
#include "d11_tables.h"
using u32 = unsigned int;
constexpr u32 NONE=0xffffffffu, WIDTH=192, STRIDE=WIDTH+3;

__device__ u32 distance(u32 a, u32 b) {
    u32 d=0;
    for (u32 k=0;k<4;++k) {
        u32 x=(a>>(4*k))&15, y=(b>>(4*k))&15;
        for (u32 j=0;j<8;++j) {
            int delta=D11_ROOTS[x][j]-D11_ROOTS[y][j];
            d += delta<0 ? -delta : delta;
        }
    }
    return d;
}

extern "C" __global__ void value_fit(const u32* data,u32 rows,u32* ids,u32* count) {
    if(blockIdx.x || threadIdx.x) return;
    u32 used=0;
    for(u32 i=0;i<rows;++i) {
        const u32* row=data+i*STRIDE;
        const u32 first=row[WIDTH+1]+1, len=row[WIDTH+2]-first;
        u32 word[4]={len,NONE,NONE,NONE};
        for(u32 j=0;j<len;++j) word[j+1]=row[first+j];
        bool found=false;
        for(u32 k=0;k<used && !found;++k) {
            found=true;
            for(u32 j=0;j<4;++j) found &= word[j]==ids[4*k+j];
        }
        if(!found) {
            for(u32 j=0;j<4;++j) ids[4*used+j]=word[j];
            ++used;
        }
    }
    *count=used;
}

struct Winner {
    u32 cost=0, len=0;
    __device__ void observe(u32 c,u32 n) {
        if (!len || c*len<cost*n || (c*len==cost*n && n>len)) {cost=c;len=n;}
    }
};

extern "C" __global__ void value_predict(const u32* data,const u32* roots,const u32* classes,
        const u32* relations,u32 nr,const u32* exact,u32 ne,u32 rows,u32* output) {
    const u32 i=blockIdx.x*blockDim.x+threadIdx.x;
    if(i>=rows) return;
    const u32* row=data+i*STRIDE;
    const u32 end=row[WIDTH+1], limit=row[WIDTH+2];
    u32* out=output+i*7;
    for(u32 k=0;k<4;++k) out[k]=NONE;
    out[4]=out[5]=out[6]=0;
    if(end==NONE) return;
    Winner regular{}, start{};
    for(u32 k=0;k<nr;++k) {
        const u32* rel=relations+5*k;
        const u32 n=rel[0], source=end+1+n, bit=1u<<(n-1);
        if(source>=limit) continue;
        u32 cost=n*distance(roots[row[source]],rel[4]);
        for(u32 j=0;j<n;++j) cost+=4*distance(roots[row[end+1+j]],rel[1+j]);
        out[4]|=bit;
        regular.observe(cost,n);
        if(classes[row[source]]==1) {
            out[5]|=bit;
            start.observe(cost,n);
        }
    }
    if(regular.len) out[0]=end+1+regular.len;
    if(start.len) out[1]=end+1+start.len;
    for(u32 k=0;k<ne;++k) {
        const u32* c=exact+k*4;
        const u32 n=c[0],bit=1u<<(n-1);
        if(!(out[4]&bit)) continue;
        bool equal=true;
        for(u32 j=0;j<n;++j) equal &= row[end+1+j]==c[1+j];
        if(equal) out[6]|=bit;
    }
    for(u32 n=1;n<=3;++n) {
        const u32 bit=1u<<(n-1);
        if(out[6]&bit) out[2]=end+1+n;
        if(out[6]&out[5]&bit) out[3]=end+1+n;
    }
}
