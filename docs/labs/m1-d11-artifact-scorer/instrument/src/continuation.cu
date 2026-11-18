// AOT integer-only continuation after an already-selected WordStart address.
#define D11_CONST __device__ __constant__
#include "d11_tables.h"
using u32=unsigned int;
constexpr u32 NONE=0xffffffffu,WIDTH=192,STRIDE=WIDTH+5;

__device__ u32 orbit_distance(u32 a,u32 b) {
    u32 cost=0;
    for (u32 k=0;k<4;++k) {
        const u32 x=(a>>(4*k))&15, y=(b>>(4*k))&15;
        for (u32 j=0;j<8;++j) {
            const int d=D11_ROOTS[x][j]-D11_ROOTS[y][j];
            cost+=d<0?-d:d;
        }
    }
    return cost;
}

extern "C" __global__ void continuation_predict(const u32* data,const u32* roots,
        const u32* classes,const u32* relations,u32 count,u32 rows,u32* output) {
    const u32 index=blockIdx.x*blockDim.x+threadIdx.x;
    if(index>=rows) return;
    const u32* row=data+index*STRIDE;
    const u32 end=row[WIDTH], limit=row[WIDTH+1], start=row[WIDTH+2];
    const u32 starts=row[WIDTH+3], ordered=row[WIDTH+4];
    u32* out=output+4*index;
    out[0]=start; out[1]=out[2]=out[3]=0;
    if(start==NONE) return;
    const u32 first=end+2, stop=min(first+3,limit);
    for (u32 p=first;p<stop;++p) {
        if(classes[row[p]]==0) continue;
        const u32 root=roots[row[p]], bit=1u<<(p-first);
        u32 connector=NONE, onset=NONE;
        for(u32 k=0;k<count;++k) {
            const u32* rel=relations+5*k;
            onset=min(onset,orbit_distance(root,rel[4]));
            for(u32 j=1;j<=rel[0];++j)
                connector=min(connector,orbit_distance(root,rel[j]));
        }
        if(connector<onset) out[2]|=bit;
        else out[3]|=bit;
    }
    // Ascending updates leave the longest admissible registered extension.
    for(u32 p=start+1;p<stop;++p) {
        if(!classes[row[p-1]]) break;
        if((starts & ordered) & (1u<<(p-first))) {out[0]=p;out[1]=1;}
    }
    if(out[1]) return;
    for(u32 p=start;p<stop;++p) {
        const u32 bit=1u<<(p-first);
        if(out[2]&bit) continue;
        if(p!=start && (out[3]&starts&bit)) {out[0]=p;out[1]=2;}
        break;
    }
}
