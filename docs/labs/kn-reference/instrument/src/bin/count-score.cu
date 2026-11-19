// Ahead-of-time sm_89 device for the finite, fit and reload-score instrument.
#include <cuda_runtime.h>
#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <stdexcept>
#include <string>
#include <vector>
using U32 = uint32_t;
using U64 = unsigned long long;
using ID = uint16_t;
constexpr unsigned V = 4096, Q = 1 << 24;
void check(cudaError_t e) {
    if (e != cudaSuccess) throw std::runtime_error(cudaGetErrorString(e));
}
template<class T> std::vector<T> read(const char *path) {
    std::ifstream in(path, std::ios::binary | std::ios::ate);
    if (!in) throw std::runtime_error("input open failed");
    auto n = static_cast<size_t>(in.tellg());
    if (!n || n % sizeof(T)) throw std::runtime_error("invalid input length");
    std::vector<T> data(n / sizeof(T));
    in.seekg(0);
    if (!in.read(reinterpret_cast<char*>(data.data()), n)) throw std::runtime_error("short input");
    return data;
}
template<class T> struct Device {
    T *p;
    explicit Device(size_t n) { check(cudaMalloc(&p, n*sizeof(T))); }
    ~Device() { cudaFree(p); }
};
template<class T> void upload(T *p, const std::vector<T> &v) {
    check(cudaMemcpy(p, v.data(), v.size()*sizeof(T), cudaMemcpyHostToDevice));
}
// Raw CUDA and frozen serving stores independently compute the cumulative allocation.
__global__ void normalize(U32 *table, const U32 *uni_counts, U32 *uni, U32 *present) {
    unsigned row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row > V) return;
    const bool base = row == V;
    U32 *output = base ? uni : table + row*V;
    const U32 *input = base ? uni_counts : output;
    U64 total = 0;
    for (unsigned t=1; t<V; ++t) total += input[t] + U64(base);
    if (!base) present[row] = total != 0;
    if (total == 0) return;
    const U64 available = base ? Q-(V-1) : Q;
    U64 cumulative=0, before=0;
    for (unsigned t=1; t<V; ++t) {
        cumulative += input[t] + U64(base);
        U64 end = available*cumulative/total;
        output[t] = static_cast<U32>(end-before) + unsigned(base);
        before=end;
    }
    output[0]=0;
}
__device__ U64 mixture(U32 uni, U32 cond, U32 w) {
    return U64(256-w)*uni + U64(w)*cond;
}
__global__ void targets(const ID *tokens, size_t n, const U32 *table, const U32 *uni,
                        const U32 *present, const U32 *weight, U64 *masses, U64 *sums) {
    size_t i = size_t(blockIdx.x)*blockDim.x + threadIdx.x;
    if (i >= n) return;
    unsigned t=tokens[i];
    if (t==0) { if(masses) masses[i]=0; return; }
    unsigned p=tokens[i-1], w=*weight;
    U32 c=present[p] ? table[p*V+t] : uni[t];
    U64 m=mixture(uni[t], c, w);
    if(masses) masses[i]=m;
    if(sums) {
        U64 resp=((U64(w)*c << 16) + m/2)/m;
        atomicAdd(sums, resp);
        atomicAdd(sums+1, 1ULL);
    }
}
__global__ void finish_em(U32 *w, const U64 *sums) {
    U64 value=(sums[0]+sums[1]*128)/(sums[1]*256);
    *w=static_cast<U32>(value<1 ? 1 : (value>255 ? 255 : value));
}
__global__ void greedy(const U32 *table, const U32 *uni, const U32 *present, const U32 *weight, ID *best) {
    unsigned p=blockIdx.x*blockDim.x+threadIdx.x;
    if (p>=V) return;
    U64 largest=0;
    ID choice=1;
    for(unsigned t=1;t<V;++t) {
        U64 m=mixture(uni[t], present[p] ? table[p*V+t] : uni[t], *weight);
        if(m>largest) { choice=static_cast<ID>(t); largest=m; }
    }
    best[p]=choice;
}
// Exact Q12 log accountant for Q32 masses, identical to bits_q12(mass << 16).
__global__ void account(const U64 *masses, size_t n, U32 *bits) {
    size_t i=size_t(blockIdx.x)*blockDim.x+threadIdx.x;
    if(i>=n) return;
    U64 m=masses[i];
    if(!m) { bits[i]=0; return; }
    unsigned exponent=63-__clzll(m);
    U64 norm=m<<(63-exponent);
    unsigned fraction=0;
    for(unsigned k=0;k<12;++k) {
        U64 hi=__umul64hi(norm,norm), lo=norm*norm;
        unsigned top=hi>>63;
        norm=top ? hi : ((hi<<1)|(lo>>63));
        fraction=(fraction<<1)|top;
    }
    bits[i]=(32-exponent)*4096-fraction;
}
template<class T> void save(std::ofstream &out, size_t n, T *source) {
    std::vector<T> v(n);
    check(cudaMemcpy(v.data(), source, n*sizeof(T), cudaMemcpyDeviceToHost));
    out.write(reinterpret_cast<const char*>(v.data()), n*sizeof(T));
    if (!out) throw std::runtime_error("failed output");
}
int main(int argc, char **argv) try {
    if (argc!=6) throw std::runtime_error("expected fit|score WORDS IDS OUTPUT WEIGHT");
    std::string mode=argv[1];
    if (mode!="fit" && mode!="score") throw std::runtime_error("unknown mode");
    auto words=read<U32>(argv[2]);
    auto ids=read<ID>(argv[3]);
    if(words.size()<V+1 || (words.size()-V-1)%2 || words[1]!=0 || ids.front()!=0 || ids.back()!=1)
        throw std::runtime_error("invalid input shape");
    for(size_t i=0;i<ids.size();++i)
        if(ids[i]>=V || ((ids[i]==0)!=(i==0 || ids[i-1]==1))) throw std::runtime_error("bad record");
    std::vector<U32> raw(V*V);
    for(size_t i=V+1;i<words.size();i+=2) {
        unsigned p=words[i]&65535, t=words[i]>>16;
        if(p>=V || !t || t>=V || !words[i+1] || raw[p*V+t]) throw std::runtime_error("bad count");
        raw[p*V+t]=words[i+1];
    }
    Device<U32> table(V*V), uni(V), uni_counts(V), present(V), weight(1);
    Device<ID> tokens(ids.size());
    upload(tokens.p, ids);
    upload(table.p, raw);
    check(cudaMemcpy(uni_counts.p, words.data()+1, V*sizeof(U32), cudaMemcpyHostToDevice));
    normalize<<<(V+128)/128,128>>>(table.p,uni_counts.p,uni.p,present.p);
    int argweight=std::stoi(argv[5]);
    if(argweight<0 || argweight>255) throw std::runtime_error("bad weight");
    U32 w=static_cast<U32>(argweight);
    check(cudaMemcpy(weight.p,&w,sizeof(w),cudaMemcpyHostToDevice));
    std::ofstream out(argv[4],std::ios::binary);
    auto blocks=static_cast<unsigned>((ids.size()+255)/256);
    if(mode=="fit") {
        Device<U64> sums(2);
        for(unsigned pass=0;pass<4;++pass) {
            check(cudaMemset(sums.p,0,2*sizeof(U64)));
            targets<<<blocks,256>>>(tokens.p,ids.size(),table.p,uni.p,present.p,weight.p,nullptr,sums.p);
            finish_em<<<1,1>>>(weight.p,sums.p);
            save(out,1,weight.p);
        }
    } else {
        Device<U64> masses(ids.size());
        Device<ID> best(V);
        Device<U32> bits(ids.size());
        targets<<<blocks,256>>>(tokens.p,ids.size(),table.p,uni.p,present.p,weight.p,masses.p,nullptr);
        greedy<<<(V+127)/128,128>>>(table.p,uni.p,present.p,weight.p,best.p);
        account<<<blocks,256>>>(masses.p,ids.size(),bits.p);
        save(out,V,best.p);
        save(out,ids.size(),masses.p);
        save(out,ids.size(),bits.p);
    }
    check(cudaDeviceSynchronize());
    return 0;
} catch(const std::exception &e) {
    std::fprintf(stderr,"%s\n",e.what());
    return 1;
}
