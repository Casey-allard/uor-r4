#include <cuda_runtime.h>
#include <stdint.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <vector>
#include "abi.h"
#define D11_CONST __device__ __constant__
#include "d11_tables.h"

struct Geometry {
    uint32_t shape, signature;
    uint32_t roots[10];
};

__device__ uint32_t take(const uint32_t* record, int at) {
    return (record[3+at/2] >> (16*(at&1))) & 65535;
}
__device__ void store(uint32_t* record, Geometry geometry, uint32_t label) {
    record[0]=geometry.shape|(label<<16); record[1]=geometry.signature; record[2]=NO_RADIUS;
    for (int i=0;i<5;++i) record[3+i]=geometry.roots[2*i]|(geometry.roots[2*i+1]<<16);
}
__device__ uint32_t root_distance(uint32_t a, uint32_t b) {
    uint32_t total=0;
    for (int block=0;block<4;++block) {
        auto x=D11_ROOTS[(a>>(4*block))&15];
        auto y=D11_ROOTS[(b>>(4*block))&15];
        for (int axis=0;axis<8;++axis) {
            int delta=int(x[axis])-int(y[axis]);
            total+=delta<0?-delta:delta;
        }
    }
    return total;
}
__device__ int count(Geometry g) {return int(g.shape&255)+int(g.shape>>8);}
__device__ uint32_t distance(Geometry query, const uint32_t* record) {
    uint32_t total=0;
    for(int i=0;i<count(query);++i) total+=root_distance(query.roots[i],take(record,i));
    return total;
}
__device__ bool match(Geometry x,const uint32_t* record) {
    return x.shape==(record[0]&65535) && x.signature==record[1];
}
__device__ Geometry geometry(const uint32_t* record) {
    Geometry g={record[0]&65535,record[1],{}};
    for(int i=0;i<10;++i) g.roots[i]=take(record,i);
    return g;
}
__device__ bool transform(Geometry& g,uint32_t mode,uint32_t remaining) {
    int depth=g.shape&255, slots=count(g);
    if(mode==1) {
        for(int i=0;i<slots;++i) {
            uint32_t cls=(g.signature>>(2*i))&3;
            if(!cls) continue;
            int end=i<depth?depth:slots;
            for(int j=i+1;j<end;++j) if(((g.signature>>(2*j))&3)==cls && g.roots[j]<g.roots[i]) {
                uint32_t swap=g.roots[i];g.roots[i]=g.roots[j];g.roots[j]=swap;
            }
        }
    } else if (mode==2) {
        g=Geometry{uint32_t(depth)|(remaining<<8),0,{}};
    }
    return true;
}
__device__ bool validate_row(const uint32_t* row) {
    auto length=row[2],fact=row[3],query=row[4],fence=row[5];
    if(!length || length>CONTEXT || (row[6]|row[7])>>3) return false;
    if(fence==NONE) {
        if(row[6]||row[7]) return false;
        for(int k=0;k<3;++k) {
            auto c=row+8+9*k;
            if(c[0]!=NONE) return false;
            for(int j=1;j<9;++j) if(c[j]) return false;
        }
        return true;
    }
    if(row[1]==NONE || !fact || !(fact<fence && fence<query && query<length)) return false;
    uint32_t mask=row[6]&row[7];
    for(int k=0;k<3;++k) {
        auto c=row+8+9*k;
        if(!(mask&(1u<<k))) {
            if(c[0]!=NONE) return false;
            for(int i=1;i<9;++i) if(c[i]) return false;
            continue;
        }
        uint32_t source=fact+2+k;
        if(c[0]!=source || source>=fence || c[1]<=source || c[1]>fence || c[2]>1 || c[2]!=(fence-source<8)) return false;
        if(!c[2]) {
            for(int i=3;i<9;++i) if(c[i]) return false;
            continue;
        }
        int depth=k+1,slots=depth+int(c[1]-source);
        if(slots>10 || (c[3]>>(2*slots)) || ((c[3]>>(2*depth))&3)!=1) return false;
        for(int i=0;i<10;++i) {
            auto root=(c[4+i/2]>>(16*(i&1)))&65535;
            if(i>=slots){if(root) return false;continue;}
            uint32_t cls=(c[3]>>(2*i))&3;
            if(cls>2 || (i>depth && cls!=2)) return false;
        }
    }
    for(int i=0;i<3;++i) {
        auto a=row+8+9*i;
        if(a[2]!=1) continue;
        for(int j=0;j<i;++j) {
            auto b=row+8+9*j;
            if(b[2]!=1) continue;
            uint32_t end=a[1]<b[1]?a[1]:b[1];
            for(uint32_t k=0;k<end-fact-1;++k) {
                if(((a[3]>>(2*k))&3)!=((b[3]>>(2*k))&3)
                    || ((a[4+k/2]>>(16*(k&1)))&65535)!=((b[4+k/2]>>(16*(k&1)))&65535)) return false;
            }
        }
    }
    return true;
}
__device__ Geometry candidate(const uint32_t* c,int depth) {
    Geometry g={uint32_t(depth)|((c[1]-c[0])<<8),c[3],{}};
    for(int i=0;i<10;++i) g.roots[i]=(c[4+i/2]>>(16*(i&1)))&65535;
    return g;
}
// Finite and research compilation follow the same sealed input order. One
// thread appends, then the radius kernels work independently by bank entry.
__global__ void build_bank(const uint32_t* rows,const uint32_t* labels,uint32_t n,uint32_t mode,
                           uint32_t* bank,uint32_t* family,uint32_t* meta) {
    uint32_t used=0, positives=0, negatives=0;
    uint32_t opposed=0;
    meta[0]=0;meta[1]=0;meta[2]=0;
    for(uint32_t i=0;i<n;++i) {
        auto row=rows+WIDTH*i;
        auto label=labels+LABEL_WIDTH*i;
        if(!validate_row(row)||row[0]!=label[0]||row[1]!=label[1]){meta[0]=1;return;}
        if(label[3]==NONE) continue;
        if(label[2]==NONE){meta[0]=1;return;}
        bool gold=false;
        for(int k=0;k<3;++k) {
            auto c=row+8+9*k;
            if(c[0]==NONE || !c[2]) continue;
            gold |= c[0]==label[3];
            if(used==CAP){meta[0]=2;meta[1]=used;meta[2]=opposed;return;}
            auto g=candidate(c,k+1);
            if(!transform(g,mode,row[5]-c[0])){meta[0]=1;return;}
            uint32_t cls=c[0]==label[3]?0:1;
            positives+=cls==0; negatives+=cls==1;
            for(uint32_t j=0;j<used;++j) {
                auto e=bank+j*RECORD;
                if ((e[0]>>16)!=cls && match(g,e) && distance(g,e)==0) ++opposed;
            }
            store(bank+used*RECORD,g,cls);
            family[used]=label[2];
            ++used;
        }
        if(!gold){meta[0]=4;return;}
    }
    meta[1]=used;
    meta[2]=opposed;
    if(!positives||!negatives){meta[0]=5;return;}
    if (mode!=2 && opposed) meta[0]=3;
}
__global__ void nearest_other_family(const uint32_t* bank,const uint32_t* family,uint32_t used,uint32_t* minimum) {
    uint32_t i=threadIdx.x;
    if(i>=used) return;
    auto record=bank+i*RECORD;
    minimum[i]=NONE;
    if(record[0]>>16) return;
    auto g=geometry(record);
    for(uint32_t j=0;j<used;++j) {
        auto e=bank+j*RECORD;
        if(!(e[0]>>16) && family[i]!=family[j] && match(g,e)) {
            uint32_t d=distance(g,e);
            if(d<minimum[i]) minimum[i]=d;
        }
    }
}
__global__ void assign_radius(uint32_t* bank,uint32_t used,const uint32_t* minimum) {
    uint32_t i=threadIdx.x;
    if(i>=used) return;
    auto record=bank+i*RECORD;
    auto g=geometry(record);
    uint32_t radius=0;
    bool seen=false;
    for(uint32_t j=0;j<used;++j) {
        auto e=bank+j*RECORD;
        if((e[0]>>16)||!match(g,e)) continue;
        if(minimum[j]==NONE){radius=NO_RADIUS;break;}
        seen=true;
        if(minimum[j]>radius) radius=minimum[j];
    }
    record[2]=seen?radius:NO_RADIUS;
}

__device__ int cmp(int n,int d,int m,int e) {return n*e-m*d;}
__global__ void infer(const uint32_t* rows,uint32_t n,const uint32_t* bank,uint32_t used,uint32_t mode,uint32_t* trace) {
    uint32_t rowid=blockIdx.x*blockDim.x+threadIdx.x;
    if(rowid>=n) return;
    auto row=rows+WIDTH*rowid;
    auto t=trace+TRACE_WIDTH*rowid;
    for(int i=0;i<TRACE_WIDTH;++i) t[i]=NONE;
    t[0]=row[0];t[1]=row[1];t[2]=5;
    if(!validate_row(row)) return;
    bool unseen=false;
    int bn=0,bd=0,rn=0,rd=0;
    uint32_t winner=NONE;
    for(int k=0;k<3;++k) {
        auto c=row+8+9*k;
        auto v=t+TRACE_HEAD+k*CAND_TRACE;
        if(c[0]==NONE) continue;
        v[0]=c[0];v[1]=c[1];v[2]=1;
        if(!c[2]) continue;
        auto g=candidate(c,k+1);
        if(!transform(g,mode,row[5]-c[0])) return;
        v[2]=2;v[3]=g.shape;v[4]=g.signature;
        for(int i=0;i<count(g);++i) v[10+i]=g.roots[i];
        uint32_t best[2]={NONE,NONE},radius=NO_RADIUS;
        int num=0,den=1;
        if(mode==2) {
            uint32_t votes[2]={0,0};
            for(uint32_t j=0;j<used;++j) {
                auto e=bank+j*RECORD;
                if(match(g,e)) ++votes[e[0]>>16];
            }
            v[5]=votes[0];v[6]=votes[1];v[7]=NO_RADIUS;
            if(!votes[0]&&!votes[1]){unseen=true;continue;}
            num=int(votes[0])-int(votes[1]);
        } else {
            for(uint32_t j=0;j<used;++j) {
                auto e=bank+j*RECORD;
                if(!match(g,e)) continue;
                auto d=distance(g,e),cls=e[0]>>16;
                if(d<best[cls]) best[cls]=d;
                radius=e[2];
            }
            v[5]=best[0];v[6]=best[1];v[7]=radius;
            if(best[0]==NONE||best[1]==NONE||radius==NO_RADIUS||best[0]>radius){unseen=true;continue;}
            num=int(best[1])-int(best[0]);den=count(g);
        }
        v[2]=3;v[8]=uint32_t(num);v[9]=uint32_t(den);
        if(!bd||cmp(num,den,bn,bd)>0){rn=bn;rd=bd;bn=num;bd=den;winner=c[0];}
        else if(!rd||cmp(num,den,rn,rd)>0){rn=num;rd=den;}
    }
    if(bd){t[4]=uint32_t(bn);t[5]=uint32_t(bd);}
    if(rd){t[6]=uint32_t(rn);t[7]=uint32_t(rd);}
    t[2]=unseen?2:!bd?1:bn<=0?4:(rd&&!cmp(bn,bd,rn,rd))?3:0;
    if(!t[2])t[3]=winner;
}

static void check(cudaError_t code) {
    if(code==cudaSuccess) return;
    std::fprintf(stderr,"CUDA: %s\n",cudaGetErrorString(code));
    std::exit(2);
}
static std::vector<uint32_t> read(const char* path,size_t maxwords) {
    std::ifstream file(path,std::ios::binary|std::ios::ate);
    auto size=file.tellg();
    if(size<0 || size%4 || uint64_t(size)>4*maxwords) return {};
    file.seekg(0);
    std::vector<uint32_t> words(size_t(size)/4);
    if(!file.read(reinterpret_cast<char*>(words.data()),size)) return {};
    return words;
}
template <class T> static T* copy(const T* values,size_t count) {
    T* out=nullptr;
    check(cudaMalloc(&out,count*sizeof(T)));
    if(values) check(cudaMemcpy(out,values,count*sizeof(T),cudaMemcpyHostToDevice));
    else check(cudaMemset(out,0,count*sizeof(T)));
    return out;
}
static bool write(const char* path,const std::vector<uint32_t>& words) {
    std::ofstream out(path,std::ios::binary);
    out.write(reinterpret_cast<const char*>(words.data()),words.size()*sizeof(uint32_t));
    return bool(out);
}
static bool bank_valid(const uint32_t* bank,uint32_t used,uint32_t mode) {
    if (!used || used>CAP || mode>2) return false;
    for(uint32_t i=0;i<CAP;++i) {
        auto record=bank+RECORD*i;
        if(i>=used) {
            for(int j=0;j<RECORD;++j) if(record[j]) return false;
            continue;
        }
        uint32_t l=record[0]&255,r=(record[0]>>8)&255;
        if(l<1||l>3||r<1||r>7||record[0]>>16>1
           ||(record[2]>MAX_COST && record[2]!=NO_RADIUS)
           || (record[1]>>(2*(l+r)))) return false;
        if(mode==2) {
            if(record[1] || record[2]!=NO_RADIUS) return false;
            for(int j=3;j<RECORD;++j) if(record[j]) return false;
        } else if(((record[1]>>(2*l))&3)!=1) return false;
        for(uint32_t j=0;j<10;++j) {
            uint32_t cls=(record[1]>>(2*j))&3;
            if(mode!=2 && j<l+r && (cls>2||(j>l&&cls!=2))) return false;
            if(j>=l+r && ((record[3+j/2]>>(16*(j&1)))&65535)) return false;
        }
        for(uint32_t j=0;j<i;++j) {
            auto before=bank+j*RECORD;
            if((before[0]&65535)!=(record[0]&65535)||before[1]!=record[1]) continue;
            if(before[2]!=record[2]) return false;
            if(mode!=2&&(before[0]>>16)!=(record[0]>>16)&&!std::memcmp(before+3,record+3,5*sizeof(uint32_t))) return false;
        }
    }
    return true;
}
int main(int argc,char** argv) {
    bool fit=argc==6 && !std::strcmp(argv[1],"fit");
    if(!fit && !(argc==5&&!std::strcmp(argv[1],"score"))) return 3;
    auto input=read(argv[2],HEADER+MAX_ROWS*WIDTH);
    if(input.size()<HEADER||input[0]!=INPUT_MAGIC||!input[1]||input[1]>MAX_ROWS||input[2]!=WIDTH||input[3]||
       input.size()!=HEADER+input[1]*WIDTH) return 3;
    uint32_t n=input[1];
    auto rows=copy(input.data()+HEADER,n*WIDTH);
    if(fit) {
        auto labels=read(argv[3],4+MAX_ROWS*LABEL_WIDTH);
        if(labels.size()!=4+n*LABEL_WIDTH||labels[0]!=LABEL_MAGIC||labels[1]!=n||labels[2]!=LABEL_WIDTH||labels[3]) return 3;
        int mode=argv[4][0]-'0';
        if (mode<0 || mode>2 || argv[4][1]) return 3;
        auto label_data=copy(labels.data()+4,n*LABEL_WIDTH);
        auto bank=copy<uint32_t>(nullptr,CAP*RECORD);
        auto family=copy<uint32_t>(nullptr,CAP);
        auto minima=copy<uint32_t>(nullptr,CAP);
        auto meta=copy<uint32_t>(nullptr,3);
        build_bank<<<1,1>>>(rows,label_data,n,mode,bank,family,meta);
        check(cudaGetLastError());
        uint32_t status[3]={};
        check(cudaMemcpy(status,meta,sizeof(status),cudaMemcpyDeviceToHost));
        std::fprintf(stderr,"fit_status=%u used=%u cross_class_identical_pairs=%u\n",status[0],status[1],status[2]);
        if(status[0])return 4;
        if(mode!=2) {
            nearest_other_family<<<1,CAP>>>(bank,family,status[1],minima);
            check(cudaGetLastError());
            assign_radius<<<1,CAP>>>(bank,status[1],minima);
            check(cudaGetLastError());
        }
        std::vector<uint32_t> output(4+CAP*RECORD);
        output[0]=RAW_MAGIC;output[1]=mode;output[2]=status[1];output[3]=CAP;
        check(cudaMemcpy(output.data()+4,bank,CAP*RECORD*sizeof(uint32_t),cudaMemcpyDeviceToHost));
        if(!bank_valid(output.data()+4,output[2],mode))return 4;
        if(!write(argv[5],output))return 6;
    } else {
        auto raw=read(argv[3],4+CAP*RECORD);
        if(raw.size()!=4+CAP*RECORD||raw[0]!=RAW_MAGIC||raw[3]!=CAP||!bank_valid(raw.data()+4,raw[2],raw[1]))return 3;
        auto bank=copy(raw.data()+4,CAP*RECORD);
        auto results=copy<uint32_t>(nullptr,n*TRACE_WIDTH);
        infer<<<(n+127)/128,128>>>(rows,n,bank,raw[2],raw[1],results);
        check(cudaGetLastError());
        std::vector<uint32_t> output(4+n*TRACE_WIDTH);
        output[0]=TRACE_MAGIC;output[1]=n;output[2]=TRACE_WIDTH;output[3]=raw[1];
        check(cudaMemcpy(output.data()+4,results,n*TRACE_WIDTH*sizeof(uint32_t),cudaMemcpyDeviceToHost));
        if(!write(argv[4],output))return 6;
    }
    check(cudaDeviceReset());
    return 0;
}
