//! Offline native-context CUDA kernels. No served-runtime code uses this module.
//! Strict q4 forward scores classes and time in parallel; backward stages reverse
//! time with parallel class credit and double scratch. Speed remains unmeasured.
#![allow(unsafe_code)]
use candle_core::{CudaDevice, Error, Result};
use cudarc::driver::{CudaView, LaunchConfig, PushKernelArg};
use std::sync::OnceLock;
pub(crate) enum Arg<'a> {
    F(CudaView<'a, f32>),
    D(CudaView<'a, f64>),
    U(CudaView<'a, u32>),
    N(u32),
}
pub(crate) fn launch(
    device: &CudaDevice,
    name: &str,
    count: usize,
    args: &[Arg<'_>],
) -> Result<()> {
    let count =
        u32::try_from(count).map_err(|_| Error::Msg("native CUDA launch overflow".into()))?;
    if count == 0 {
        candle_core::bail!("empty native CUDA launch");
    }
    let fun = device.get_or_load_custom_func(name, "uor_native_context", ptx()?)?;
    let mut builder = fun.builder();
    for arg in args {
        match arg {
            Arg::F(v) => {
                builder.arg(v);
            }
            Arg::D(v) => {
                builder.arg(v);
            }
            Arg::U(v) => {
                builder.arg(v);
            }
            Arg::N(v) => {
                builder.arg(v);
            }
        }
    }
    // SAFETY: callers validate dimensions and allocate every indexed buffer;
    // arguments exactly match the corresponding signatures below.
    unsafe {
        builder.launch(LaunchConfig {
            grid_dim: (count.div_ceil(64), 1, 1),
            block_dim: (64, 1, 1),
            shared_mem_bytes: 0,
        })
    }
    .map_err(|e| Error::Cuda(Box::new(e)))?;
    Ok(())
}
fn ptx() -> Result<&'static str> {
    static PTX: OnceLock<std::result::Result<String, String>> = OnceLock::new();
    match PTX.get_or_init(|| {
        cudarc::nvrtc::compile_ptx_with_opts(
            SOURCE,
            cudarc::nvrtc::CompileOptions {
                use_fast_math: Some(false),
                prec_div: Some(true),
                prec_sqrt: Some(true),
                ftz: Some(false),
                name: Some("native_context.cu".into()),
                ..Default::default()
            },
        )
        .map(|p| p.to_src())
        .map_err(|e| e.to_string())
    }) {
        Ok(p) => Ok(p),
        Err(e) => Err(Error::Msg(format!("native context NVRTC: {e}"))),
    }
}
const SOURCE: &str = r#"
// NVRTC supplies device math builtins without host system headers.
// Explicit rounding prevents a CUDA fused multiply-add changing score ties.
__device__ double plus(double a,double b){return __dadd_rn(a,b);}
__device__ double mul(double a,double b){return __dmul_rn(a,b);}
__device__ double dot(const double*a,const double*b){double z=0;for(int i=0;i<4;i++)z=plus(z,mul(a[i],b[i]));return z;}
__device__ int classes(int f){return f==2?33:120;}
__device__ int toff(int f,int n){return f==0?0:(f==1?120*n:240*n);}
__device__ int boff(int f,int n,int l,int nei){return toff(f,n)*4*(l>1?2:1)+(nei?n*classes(f)*4:0);}
__device__ int neighbor(int lane,int l){return lane/l*l+(lane+1)%l;}
__device__ double score(const float*r,const float*b,const double*q,const unsigned*s,int lane,int f,int c,int n,int l){
 int k=boff(f,n,l,0)+(lane*classes(f)+c)*4;
 double v[4];for(int i=0;i<4;i++)v[i]=b[k+i];
 double z=plus((double)r[toff(f,n)+lane*classes(f)+c],dot(q+s[lane]*4,v));
 if(l>1){k=boff(f,n,l,1)+(lane*classes(f)+c)*4;for(int i=0;i<4;i++)v[i]=b[k+i];z=plus(z,dot(q+s[neighbor(lane,l)]*4,v));}return z;
}
__device__ void writef(float*out,int at,double z,unsigned*err){float v=(float)z;if(!isfinite(v))atomicExch(err,1u);out[at]=v;}
__device__ void tangent(const double*q,const double*g,double*out){double r=dot(q,g)/dot(q,q);for(int i=0;i<4;i++)out[i]=g[i]-mul(q[i],r);}
__device__ void ham(const double*a,const double*b,const double*g,double*d,double*h){
 d[0]=plus(plus(plus(mul(g[0],b[0]),mul(g[1],b[1])),mul(g[2],b[2])),mul(g[3],b[3]));
 d[1]=plus(plus(plus(-mul(g[0],b[1]),mul(g[1],b[0])),-mul(g[2],b[3])),mul(g[3],b[2]));
 d[2]=plus(plus(plus(-mul(g[0],b[2]),mul(g[1],b[3])),mul(g[2],b[0])),-mul(g[3],b[1]));
 d[3]=plus(plus(plus(-mul(g[0],b[3]),-mul(g[1],b[2])),mul(g[2],b[1])),mul(g[3],b[0]));
 h[0]=plus(plus(plus(mul(g[0],a[0]),mul(g[1],a[1])),mul(g[2],a[2])),mul(g[3],a[3]));
 h[1]=plus(plus(plus(-mul(g[0],a[1]),mul(g[1],a[0])),mul(g[2],a[3])),-mul(g[3],a[2]));
 h[2]=plus(plus(plus(-mul(g[0],a[2]),-mul(g[1],a[3])),mul(g[2],a[0])),mul(g[3],a[1]));
 h[3]=plus(plus(plus(-mul(g[0],a[3]),mul(g[1],a[2])),-mul(g[2],a[1])),mul(g[3],a[0]));
}
extern "C" __global__ void context_fwd(const float*r,const float*b,const double*q,const unsigned*group,const unsigned*native,unsigned*trace,float*out,unsigned*err,unsigned B,unsigned T,unsigned n,unsigned l,unsigned reset,unsigned hard){
 unsigned batch=blockIdx.x*blockDim.x+threadIdx.x;if(batch>=B)return;unsigned s[8];for(int i=0;i<8;i++)s[i]=1;
 for(unsigned t=0;t<T;t++){int at=batch*T+t;if(reset)for(int i=0;i<8;i++)s[i]=1;unsigned old[8],act[8];for(int i=0;i<8;i++){old[i]=s[i];act[i]=1;}
 const float*row=r+at*n*273;
 if(hard){for(int i=0;i<8;i++){if(native[at*24+i]!=old[i])atomicExch(err,2u);s[i]=native[at*24+8+i];act[i]=native[at*24+16+i];}}
 else for(unsigned lane=0;lane<n;lane++){int win=0;double z=score(row,b,q,old,lane,0,0,n,l);for(int c=1;c<120;c++){double v=score(row,b,q,old,lane,0,c,n,l);if(v>z){z=v;win=c;}}act[lane]=win;s[lane]=group[old[lane]*128+win];}
 for(int i=0;i<8;i++){trace[at*24+i]=old[i];trace[at*24+8+i]=s[i];trace[at*24+16+i]=act[i];}
 for(unsigned lane=0;lane<n;lane++){int o=(at*n+lane)*397;for(int c=0;c<120;c++)writef(out,o+c,score(row,b,q,s,lane,1,c,n,l),err);for(int c=0;c<33;c++)writef(out,o+120+c,score(row,b,q,s,lane,2,c,n,l),err);for(int i=0;i<4;i++)writef(out,o+153+i,q[s[lane]*4+i],err);for(int c=0;c<120;c++){writef(out,o+157+group[old[lane]*128+c],score(row,b,q,old,lane,0,c,n,l),err);writef(out,o+277+c,c==s[lane]?1.:0.,err);}}
 }}
extern "C" __global__ void finish(const double*in,float*out,unsigned*err,unsigned N,unsigned B){unsigned i=blockIdx.x*blockDim.x+threadIdx.x;if(i>=N)return;double v=0;for(unsigned b=0;b<B;b++)v=plus(v,in[b*N+i]);writef(out,i,v,err);}
extern "C" __global__ void emit(const float*r,const float*c,const float*up,const double*q,float*out,float*dr,float*dc,unsigned*err,unsigned N,unsigned backward){
 unsigned lane=blockIdx.x*blockDim.x+threadIdx.x;if(lane>=N)return;int root=0,cat=0;for(int i=1;i<120;i++)if(r[lane*120+i]>r[lane*120+root])root=i;for(int i=1;i<33;i++)if(c[lane*33+i]>c[lane*33+cat])cat=i;double rad=cat?ldexp(1.,cat-17):0;
 if(!backward){for(int i=0;i<4;i++)writef(out,lane*4+i,mul(q[root*4+i],rad),err);return;}if(!cat)return;double g[4],a[4];for(int i=0;i<4;i++)g[i]=up[lane*4+i];tangent(q+root*4,g,a);for(int i=0;i<4;i++)a[i]=mul(a[i],rad);double p[120],total=0,mean=0;for(int i=0;i<120;i++){p[i]=exp((double)r[lane*120+i]-(double)r[lane*120+root]);total=plus(total,p[i]);}for(int i=0;i<120;i++){p[i]/=total;mean=plus(mean,mul(p[i],dot(a,q+i*4)));}for(int i=0;i<120;i++)writef(dr,lane*120+i,mul(p[i],dot(a,q+i*4)-mean),err);
 int lo=cat>1?cat-1:1,hi=cat<32?cat+1:32;double m=-__longlong_as_double(0x7ff0000000000000ULL),sum=0;mean=0;for(int i=lo;i<=hi;i++)m=fmax(m,(double)c[lane*33+i]);for(int i=lo;i<=hi;i++)sum=plus(sum,exp((double)c[lane*33+i]-m));for(int i=lo;i<=hi;i++)mean=plus(mean,mul(exp((double)c[lane*33+i]-m)/sum,ldexp(1.,i-17)));double radial=dot(g,q+root*4);for(int i=lo;i<=hi;i++)writef(dc,lane*33+i,mul(mul(radial,exp((double)c[lane*33+i]-m))/sum,ldexp(1.,i-17)-mean),err);
}
extern "C" __global__ void finite_check(const float*x,unsigned*err,unsigned N){unsigned i=blockIdx.x*blockDim.x+threadIdx.x;if(i<N&&!isfinite(x[i]))atomicExch(err,1u);}

extern "C" __global__ void q4_round(const float*x,float*out,unsigned*err,unsigned N){unsigned i=blockIdx.x*blockDim.x+threadIdx.x;if(i<N){if(!isfinite(x[i])||fabsf(x[i])>1.75f)atomicExch(err,1u);out[i]=roundf(x[i]*4.f)*.25f;}}
extern "C" __global__ void native_parallel_fwd(const float*r,const float*b,const double*q,const unsigned*group,const unsigned*tr,float*out,unsigned*err,unsigned T,unsigned n,unsigned l,unsigned N){
 unsigned idx=blockIdx.x*blockDim.x+threadIdx.x;if(idx>=N)return;int c=idx%120,lane=(idx/120)%n,at=idx/(120*n),o=(at*n+lane)*397;const unsigned*old=tr+at*24;const unsigned*state=old+8;const float*row=r+at*n*273;
 writef(out,o+c,score(row,b,q,state,lane,1,c,n,l),err);if(c<33)writef(out,o+120+c,score(row,b,q,state,lane,2,c,n,l),err);if(c<4)writef(out,o+153+c,q[state[lane]*4+c],err);writef(out,o+157+group[old[lane]*128+c],score(row,b,q,old,lane,0,c,n,l),err);writef(out,o+277+c,c==state[lane]?1.:0.,err);
}
// Each class owns its token/basis gradient cells. Cross-lane state credit is
// written separately, then reduced in deterministic class order on device.
extern "C" __global__ void readout_credit(const float*b,const float*up,const double*q,const unsigned*tr,double*dr,double*db,double*con,unsigned T,unsigned t,unsigned n,unsigned l,unsigned basislen,unsigned N){
 unsigned idx=blockIdx.x*blockDim.x+threadIdx.x;if(idx>=N)return;int c=idx%120,lane=(idx/120)%n,batch=idx/(120*n),at=batch*T+t,o=(at*n+lane)*397;const unsigned*state=tr+at*24+8;double*partial=db+batch*basislen;
 for(int i=0;i<8;i++)con[idx*8+i]=0;
 for(int f=1;f<3;f++){if(c>=classes(f))continue;double g=up[o+(f==1?0:120)+c];dr[at*n*273+toff(f,n)+lane*classes(f)+c]=g;for(int nei=0;nei<(l>1?2:1);nei++){int target=nei?neighbor(lane,l):lane,k=boff(f,n,l,nei)+(lane*classes(f)+c)*4;for(int i=0;i<4;i++){partial[k+i]=plus(partial[k+i],mul(g,q[state[target]*4+i]));con[idx*8+nei*4+i]=plus(con[idx*8+nei*4+i],mul(g,(double)b[k+i]));}}}
}
__device__ double gather(const double*con,int batch,int lane,int coord,int n,int l){double z=0;for(int c=0;c<120;c++)z=plus(z,con[((batch*n+lane)*120+c)*8+coord]);if(l>1){int prev=lane/l*l+(lane+l-1)%l;for(int c=0;c<120;c++)z=plus(z,con[((batch*n+prev)*120+c)*8+4+coord]);}return z;}
// Old ambient4 and new full-choice temporal channels remain separate. A
// zero full-choice upstream therefore preserves the old gradient exactly.
extern "C" __global__ void action_credit(const float*r,const float*b,const float*up,const double*q,const unsigned*group,const unsigned*tr,const double*con,double*future,double*direct,double*choice,const double*future_utility,double*utility_now,double*utility_choice,unsigned T,unsigned t,unsigned n,unsigned l,unsigned reset,unsigned finite,unsigned N){
 unsigned idx=blockIdx.x*blockDim.x+threadIdx.x;if(idx>=N)return;int batch=idx/n,lane=idx%n,at=batch*T+t;const unsigned*old=tr+at*24,*state=old+8,*acts=old+16;double g[4],a[4],d[4];for(int i=0;i<4;i++)g[i]=plus(plus(reset?0:future[idx*4+i],(double)up[(at*n+lane)*397+153+i]),gather(con,batch,lane,i,n,l));if(!finite){double v[4];tangent(q+state[lane]*4,g,v);for(int i=0;i<4;i++)g[i]=v[i];}ham(q+old[lane]*4,q+acts[lane]*4,g,d,a);for(int i=0;i<4;i++)direct[idx*4+i]=d[i];double p[120],credit[120],m=-__longlong_as_double(0x7ff0000000000000ULL),sum=0,mean=0,utility_mean=0;const float*row=r+at*n*273;
 for(int c=0;c<120;c++){utility_now[idx*120+c]=plus(reset?0:future_utility[idx*120+c],(double)up[(at*n+lane)*397+277+c]);p[c]=score(row,b,q,old,lane,0,c,n,l);m=fmax(m,p[c]);credit[c]=dot(a,q+c*4);}for(int c=0;c<120;c++){p[c]=exp(p[c]-m);sum=plus(sum,p[c]);}for(int c=0;c<120;c++){p[c]/=sum;mean=plus(mean,mul(p[c],credit[c]));utility_mean=plus(utility_mean,mul(p[c],utility_now[idx*120+group[old[lane]*128+c]]));}for(int c=0;c<120;c++){choice[idx*120+c]=mul(p[c],credit[c]-mean);utility_choice[idx*120+c]=mul(p[c],utility_now[idx*120+group[old[lane]*128+c]]-utility_mean);}
}
extern "C" __global__ void transition_credit(const float*b,const float*up,const double*q,const unsigned*group,const unsigned*tr,const double*choice,const double*utility_choice,double*dr,double*db,double*con,double*utility_con,unsigned T,unsigned t,unsigned n,unsigned l,unsigned basislen,unsigned N){
 unsigned idx=blockIdx.x*blockDim.x+threadIdx.x;if(idx>=N)return;int c=idx%120,lane=(idx/120)%n,batch=idx/(120*n),at=batch*T+t;const unsigned*old=tr+at*24;double oldg=plus((double)up[(at*n+lane)*397+157+group[old[lane]*128+c]],choice[idx]);double newg=utility_choice[idx],g=plus(oldg,newg);dr[at*n*273+lane*120+c]=g;double*partial=db+batch*basislen;for(int i=0;i<8;i++){con[idx*8+i]=0;utility_con[idx*8+i]=0;}for(int nei=0;nei<(l>1?2:1);nei++){int target=nei?neighbor(lane,l):lane,k=boff(0,n,l,nei)+(lane*120+c)*4;for(int i=0;i<4;i++){partial[k+i]=plus(partial[k+i],mul(g,q[old[target]*4+i]));con[idx*8+nei*4+i]=mul(oldg,(double)b[k+i]);utility_con[idx*8+nei*4+i]=mul(newg,(double)b[k+i]);}}
}
extern "C" __global__ void old_credit(const double*con,const double*direct,double*future,unsigned n,unsigned l,unsigned N){unsigned idx=blockIdx.x*blockDim.x+threadIdx.x;if(idx>=N)return;int coord=idx%4,lane=(idx/4)%n,batch=idx/(4*n);future[idx]=plus(direct[idx],gather(con,batch,lane,coord,n,l));}
// Gather each own/neighbor four-coordinate score dependency once. The next
// kernel lifts it into120 root utilities; no120x120 Jacobian is materialized.
extern "C" __global__ void old_utility_basis(const double*con,double*basis,unsigned n,unsigned l,unsigned N){unsigned idx=blockIdx.x*blockDim.x+threadIdx.x;if(idx>=N)return;int coord=idx%4,lane=(idx/4)%n,batch=idx/(4*n);basis[idx]=gather(con,batch,lane,coord,n,l);}
extern "C" __global__ void old_utility_credit(const double*now,const double*basis,const double*q,const unsigned*group,const unsigned*tr,double*future,unsigned T,unsigned t,unsigned n,unsigned N){unsigned idx=blockIdx.x*blockDim.x+threadIdx.x;if(idx>=N)return;int r=idx%120,slot=idx/120,batch=slot/n,lane=slot%n,at=batch*T+t;unsigned action=tr[at*24+16+lane];future[idx]=plus(now[slot*120+group[r*128+action]],dot(basis+slot*4,q+r*4));}
extern "C" __global__ void q4_clip(const float*x,float*out,unsigned*err,unsigned N){unsigned i=blockIdx.x*blockDim.x+threadIdx.x;if(i<N){if(!isfinite(x[i]))atomicExch(err,1u);out[i]=fminf(1.75f,fmaxf(-1.75f,x[i]));}}
"#;
