// D29 HISTORICAL EVIDENCE: NEVER COMPILED. Removed from the compiled
// native-context module on 13 October 2026: each normalises with exp/log
// (a log-sum-exp alias table, and softmax-weighted credit in `emit` and
// `action_credit`). Kept verbatim; a launch is refused (`HISTORICAL_KERNELS`).

// ---- whole_path_alias_log (excluded by D29) ----
extern "C" __global__ void whole_path_alias_log(const float*x,const unsigned*offsets,const unsigned*indices,float*out,unsigned n){
 unsigned token=blockIdx.x*blockDim.x+threadIdx.x;if(token>=n)return;
 double m=(double)x[indices[offsets[token]]];
 for(unsigned j=offsets[token]+1;j<offsets[token+1];j++)m=fmax(m,(double)x[indices[j]]);
 double sum=0;for(unsigned j=offsets[token];j<offsets[token+1];j++)sum=__dadd_rn(sum,exp((double)x[indices[j]]-m));
 out[token]=(float)(m+log(sum));
}
// ---- emit (excluded by D29) ----
extern "C" __global__ void emit(const float*r,const float*c,const float*up,const double*q,float*out,float*dr,float*dc,unsigned*err,unsigned N,unsigned backward){
 unsigned lane=blockIdx.x*blockDim.x+threadIdx.x;if(lane>=N)return;int root=0,cat=0;for(int i=1;i<120;i++)if(r[lane*120+i]>r[lane*120+root])root=i;for(int i=1;i<33;i++)if(c[lane*33+i]>c[lane*33+cat])cat=i;double rad=cat?ldexp(1.,cat-17):0;
 if(!backward){for(int i=0;i<4;i++)writef(out,lane*4+i,mul(q[root*4+i],rad),err);return;}if(!cat)return;double g[4],a[4];for(int i=0;i<4;i++)g[i]=up[lane*4+i];tangent(q+root*4,g,a);for(int i=0;i<4;i++)a[i]=mul(a[i],rad);double p[120],total=0,mean=0;for(int i=0;i<120;i++){p[i]=exp((double)r[lane*120+i]-(double)r[lane*120+root]);total=plus(total,p[i]);}for(int i=0;i<120;i++){p[i]/=total;mean=plus(mean,mul(p[i],dot(a,q+i*4)));}for(int i=0;i<120;i++)writef(dr,lane*120+i,mul(p[i],dot(a,q+i*4)-mean),err);
 int lo=cat>1?cat-1:1,hi=cat<32?cat+1:32;double m=-__longlong_as_double(0x7ff0000000000000ULL),sum=0;mean=0;for(int i=lo;i<=hi;i++)m=fmax(m,(double)c[lane*33+i]);for(int i=lo;i<=hi;i++)sum=plus(sum,exp((double)c[lane*33+i]-m));for(int i=lo;i<=hi;i++)mean=plus(mean,mul(exp((double)c[lane*33+i]-m)/sum,ldexp(1.,i-17)));double radial=dot(g,q+root*4);for(int i=lo;i<=hi;i++)writef(dc,lane*33+i,mul(mul(radial,exp((double)c[lane*33+i]-m))/sum,ldexp(1.,i-17)-mean),err);
}
// ---- action_credit (excluded by D29) ----
// Old ambient4 and new full-choice temporal channels remain separate. A
// zero full-choice upstream therefore preserves the old gradient exactly.
extern "C" __global__ void action_credit(const float*r,const float*b,const float*up,const double*q,const unsigned*group,const unsigned*tr,const double*con,double*future,double*direct,double*choice,const double*future_utility,double*utility_now,double*utility_choice,unsigned T,unsigned t,unsigned n,unsigned l,unsigned reset,unsigned finite,unsigned N){
 unsigned idx=blockIdx.x*blockDim.x+threadIdx.x;if(idx>=N)return;int batch=idx/n,lane=idx%n,at=batch*T+t;const unsigned*old=tr+at*24,*state=old+8,*acts=old+16;double g[4],a[4],d[4];for(int i=0;i<4;i++)g[i]=plus(plus(reset?0:future[idx*4+i],(double)up[(at*n+lane)*397+153+i]),gather(con,batch,lane,i,n,l));if(!finite){double v[4];tangent(q+state[lane]*4,g,v);for(int i=0;i<4;i++)g[i]=v[i];}ham(q+old[lane]*4,q+acts[lane]*4,g,d,a);for(int i=0;i<4;i++)direct[idx*4+i]=d[i];double p[120],credit[120],m=-__longlong_as_double(0x7ff0000000000000ULL),sum=0,mean=0,utility_mean=0;const float*row=r+at*n*273;
 for(int c=0;c<120;c++){utility_now[idx*120+c]=plus(reset?0:future_utility[idx*120+c],(double)up[(at*n+lane)*397+277+c]);p[c]=score(row,b,q,old,lane,0,c,n,l);m=fmax(m,p[c]);credit[c]=dot(a,q+c*4);}for(int c=0;c<120;c++){p[c]=exp(p[c]-m);sum=plus(sum,p[c]);}for(int c=0;c<120;c++){p[c]/=sum;mean=plus(mean,mul(p[c],credit[c]));utility_mean=plus(utility_mean,mul(p[c],utility_now[idx*120+group[old[lane]*128+c]]));}for(int c=0;c<120;c++){choice[idx*120+c]=mul(p[c],credit[c]-mean);utility_choice[idx*120+c]=mul(p[c],utility_now[idx*120+group[old[lane]*128+c]]-utility_mean);}
}
