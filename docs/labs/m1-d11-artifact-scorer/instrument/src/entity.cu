// Join the same frozen content energy to the span identity predicate before
// selecting the minimum. Nonword tokens remain singleton spans as in span.cu.
#define ENTITY_PAIR_CLASS_GATE
#define content_bind entity_bind
__device__ void word_bounds(const unsigned *ids,unsigned n,const unsigned *classes,
                            unsigned at,unsigned& l,unsigned& r) {
    l=r=at;
    if (classes[ids[at]]==0) return;
    while (l>0 && classes[ids[l]]==2 && classes[ids[l-1]]!=0) --l;
    while (r+1<n && classes[ids[r+1]]==2) ++r;
}
__device__ bool entity_pair_matches(const unsigned *ids,unsigned n,const unsigned *classes,
                                    unsigned q,unsigned f) {
    unsigned ql,qr,fl,fr;
    word_bounds(ids,n,classes,q,ql,qr);
    word_bounds(ids,n,classes,f,fl,fr);
    if (fr>=ql || q-ql!=f-fl || qr-ql!=fr-fl) return false;
    for (unsigned i=0;i<=fr-fl;++i) if (ids[fl+i]!=ids[ql+i]) return false;
    return true;
}
#include "content.cu"
