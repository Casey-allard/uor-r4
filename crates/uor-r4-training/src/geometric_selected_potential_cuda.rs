//! Offline selected finite utilities. Device threads build only requested
//! endpoint/root/category alternatives; no selected utility vectors are uploaded.
#![allow(unsafe_code)]
use super::{code, root, PotentialCredit};
use crate::{invalid, Result};
use candle_core::{op::BackpropOp, CudaStorage, Device, Storage, Tensor};
use cudarc::driver::{LaunchConfig, PushKernelArg};
use std::sync::OnceLock;

pub(super) fn utilities(
    base: &PotentialCredit,
    slots: &[u32],
    expanded: &[i32],
    device: &Device,
) -> Result<Tensor> {
    let Device::Cuda(cuda) = device else {
        return Err(invalid("selected utility CUDA device missing"));
    };
    let count = slots.len() / 4;
    let mut relative = Vec::with_capacity(14400);
    for q in 0..120 {
        for k in 0..120 {
            relative.push(u32::from(base.algebra.relative(code(q)?, code(k)?).index()));
        }
    }
    let observations = base
        .content
        .iter()
        .zip(&base.context)
        .zip(&base.raw_roots)
        .flat_map(|((c, r), raw)| {
            [
                u32::from(c.root()),
                u32::from(c.radius_bin()),
                u32::from(c.present()),
                u32::from(r.root()),
                u32::from(r.radius_bin()),
                u32::from(r.present()),
                u32::from(*raw),
            ]
        })
        .collect::<Vec<_>>();
    let coeff = base
        .coefficients
        .iter()
        .map(|&c| f64::from(c) * 0.25)
        .collect::<Vec<_>>();
    let basis = (0..120).flat_map(root).collect::<Vec<_>>();
    let tables = expanded.iter().map(|&x| i64::from(x)).collect::<Vec<_>>();
    let s = Tensor::from_vec(slots.to_vec(), slots.len(), device)?;
    let o = Tensor::from_vec(observations.clone(), observations.len(), device)?;
    let r = Tensor::from_vec(relative, 14400, device)?;
    let c = Tensor::from_vec(coeff.clone(), coeff.len(), device)?;
    let q = Tensor::from_vec(basis, 480, device)?;
    let t = Tensor::from_vec(tables.clone(), tables.len(), device)?;
    let out = cuda.alloc_zeros::<f32>(count * 306)?;
    let (ss, _) = s.storage_and_layout();
    let (os, _) = o.storage_and_layout();
    let (rs, _) = r.storage_and_layout();
    let (cs, _) = c.storage_and_layout();
    let (qs, _) = q.storage_and_layout();
    let (ts, _) = t.storage_and_layout();
    fn storage(s: &Storage) -> Result<&CudaStorage> {
        match s {
            Storage::Cuda(x) => Ok(x),
            _ => Err(invalid("selected utility storage differs")),
        }
    }
    let fun =
        cuda.get_or_load_custom_func("selected_utilities", "uor_selected_potential", ptx()?)?;
    let mut launch = fun.builder();
    let sv = storage(&ss)?.as_cuda_slice::<u32>()?.as_view();
    let ov = storage(&os)?.as_cuda_slice::<u32>()?.as_view();
    let rv = storage(&rs)?.as_cuda_slice::<u32>()?.as_view();
    let cv = storage(&cs)?.as_cuda_slice::<f64>()?.as_view();
    let qv = storage(&qs)?.as_cuda_slice::<f64>()?.as_view();
    let tv = storage(&ts)?.as_cuda_slice::<i64>()?.as_view();
    let output = out.as_view();
    launch.arg(&sv);
    launch.arg(&ov);
    launch.arg(&rv);
    launch.arg(&cv);
    launch.arg(&qv);
    launch.arg(&tv);
    launch.arg(&output);
    let n = u32::try_from(count).map_err(|_| invalid("selected CUDA count overflow"))?;
    let width = base.d.width() as u32;
    launch.arg(&n);
    launch.arg(&width);
    // SAFETY: authenticated input dimensions allocate every indexed table and
    // slot; the kernel owns exactly 306 output cells per admitted physical slot.
    unsafe {
        launch.launch(LaunchConfig {
            grid_dim: (n.div_ceil(64), 1, 1),
            block_dim: (64, 1, 1),
            shared_mem_bytes: 0,
        })
    }
    .map_err(|e| invalid(e.to_string()))?;
    Ok(Tensor::from_storage(
        Storage::Cuda(CudaStorage::wrap_cuda_slice(out, cuda.clone())),
        (count, 306),
        BackpropOp::none(),
        false,
    ))
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
                name: Some("selected_potential.cu".into()),
                ..Default::default()
            },
        )
        .map(|p| p.to_src())
        .map_err(|e| e.to_string())
    }) {
        Ok(p) => Ok(p),
        Err(e) => Err(invalid(format!("selected potential NVRTC: {e}"))),
    }
}
const SOURCE: &str = r#"
__device__ double add(double a,double b){return __dadd_rn(a,b);}
__device__ double mul(double a,double b){return __dmul_rn(a,b);}
__device__ double dot4(const double*a,const double*b){double z=0;for(int i=0;i<4;i++)z=add(z,mul(a[i],b[i]));return z;}
__device__ void ham(const double*a,const double*b,double*out){
 out[0]=add(add(add(mul(a[0],b[0]),-mul(a[1],b[1])),-mul(a[2],b[2])),-mul(a[3],b[3]));
 out[1]=add(add(add(mul(a[0],b[1]),mul(a[1],b[0])),mul(a[2],b[3])),-mul(a[3],b[2]));
 out[2]=add(add(add(mul(a[0],b[2]),-mul(a[1],b[3])),mul(a[2],b[0])),mul(a[3],b[1]));
 out[3]=add(add(add(mul(a[0],b[3]),mul(a[1],b[2])),-mul(a[2],b[1])),mul(a[3],b[0]));
}
__device__ long long lane_score(const long long*t,const unsigned*rel,const unsigned*cq,const unsigned*ck,
 unsigned qr,unsigned qb,unsigned qp,unsigned kr,unsigned kb,unsigned kp){
 long long z=t[18672+(cq[2]*2+ck[2])]+t[18676+(qp*2+kp)];
 unsigned cr=0,rr=0;
 if(cq[2]&&ck[2]){cr=rel[cq[0]*120+ck[0]];z+=t[cr]+t[16624+cq[1]*32+ck[1]];}
 if(qp&&kp){rr=rel[qr*120+kr];z+=t[120+rr]+t[17648+qb*32+kb];}
 if(cq[2]&&ck[2]&&qp&&kp)z+=t[240+cr*128+rr];
 return z;
}
extern "C" __global__ void selected_utilities(const unsigned*s,const unsigned*obs,const unsigned*rel,
 const double*coef,const double*basis,const long long*tables,float*out,unsigned N,unsigned width){
 unsigned n=blockIdx.x*blockDim.x+threadIdx.x;if(n>=N)return;
 unsigned qi=s[n*4],ki=s[n*4+1],lane=s[n*4+2],same=s[n*4+3];
 const unsigned*cq=obs+qi*7,*ck=obs+ki*7;
 const long long*t=tables+lane*18680;
 double aq[4]={0,0,0,0},ak[4]={0,0,0,0};
 if(!same&&cq[5]&&ck[5]){
  double v[4];for(int j=0;j<4;j++)v[j]=coef[4*width+lane*4+j];
  if(cq[2]&&ck[2]){
   const double*dc=basis+rel[cq[0]*120+ck[0]]*4;
   for(int j=0;j<4;j++)for(int i=0;i<4;i++)v[j]=add(v[j],mul(dc[i],coef[8*width+lane*16+i*4+j]));
  }
  const double*a=basis+cq[3]*4,*z=basis+ck[3]*4;
  double ca[4]={a[0],-a[1],-a[2],-a[3]};
  for(int axis=0;axis<4;axis++){
   double unit[4]={0,0,0,0},result[4];unit[axis]=axis==0?1:-1;
   ham(unit,z,result);aq[axis]=dot4(v,result);
   unit[axis]=1;ham(ca,unit,result);ak[axis]=dot4(v,result);
  }
 }
 double qa=dot4(aq,basis),ka=dot4(ak,basis);
 for(int r=0;r<120;r++){
  out[n*306+r]=(float)(dot4(aq,basis+r*4)-qa);
  out[n*306+120+r]=(float)(dot4(ak,basis+r*4)-ka);
 }
 long long qanchor=lane_score(t,rel,cq,ck,1,0,0,same?1:ck[3],same?0:ck[4],same?0:ck[5]);
 long long kanchor=same?0:lane_score(t,rel,cq,ck,cq[3],cq[4],cq[5],1,0,0);
 for(int c=0;c<33;c++){
  unsigned qr=c?cq[6]:1,qb=c?c-1:0,qp=c?1:0;
  unsigned kr=c?ck[6]:1,kb=c?c-1:0,kp=c?1:0;
  long long qv=lane_score(t,rel,cq,ck,qr,qb,qp,same?qr:ck[3],same?qb:ck[4],same?qp:ck[5]);
  long long kv=same?0:lane_score(t,rel,cq,ck,cq[3],cq[4],cq[5],kr,kb,kp);
  out[n*306+240+c]=(float)((double)(qv-qanchor)/16777216.);
  out[n*306+273+c]=same?0:(float)((double)(kv-kanchor)/16777216.);
 }
}
"#;
