//! Opt-in BVG-K set selection, decision and one-epoch set commit (W3, Stage C2,
//! handoff H2a), derived from the generated K-coordinate module. Three entries
//! are appended to the generated `rowsk.cu` text, so the row-parallel and
//! K-coordinate images stay byte-identical: this is a separate module.
//!
//! The rule: `S_t` is the eligible coordinates whose chosen move strictly
//! improves, ordered by coordinate ordinal; `best` is the member with the lowest
//! single-move loss, ties to the lowest ordinal. `|S_t| = 0` commits nothing,
//! `|S_t| = 1` commits `{best}` with no verify forward, and `|S_t| >= 2` commits
//! `S_t` only if the joint loss `L(S_t)` is strictly below `L({best})`, else
//! `{best}`. The epoch advances once per batch with a commit. The joint loss is
//! computed by the device API (H2b) on a scratch state; nothing here runs it.

pub const ENTRIES: [&str; 8] = [
    "discrete_counterfactual_rows_baseline",
    "discrete_counterfactual_rows_eval",
    "discrete_counterfactual_rows_reduce",
    "discrete_counterfactual_rowsk_eval",
    "discrete_counterfactual_rowsk_reduce",
    "discrete_bvgk_choose",
    "discrete_bvgk_decide",
    "discrete_bvgk_apply",
];

/// Append the BVG-K entries to the generated K-coordinate source.
pub fn generate(rowsk_source: &str) -> Result<String, String> {
    if !rowsk_source.contains("extern \"C\" __global__ void discrete_counterfactual_rowsk_reduce(")
    {
        return Err("BVG-K anchor changed: K-coordinate reducer missing".into());
    }
    // The appended kernels call these prefix helpers; the generated rows text
    // keeps only helpers and the writer, so `resulting_port` is re-declared
    // below as `bvgk_resulting_port` with the ordinary body (pinned by test).
    for anchor in PREFIX_HELPERS {
        if rowsk_source.matches(anchor).count() != 1 {
            return Err(format!(
                "BVG-K anchor changed: prefix helper {anchor:?} differs"
            ));
        }
    }
    Ok(format!("{rowsk_source}\n{BVGK}"))
}

/// Prefix helper definitions the appended kernels rely on, verbatim.
pub const PREFIX_HELPERS: [&str; 6] = [
    "__device__ U compose(const U* g,U a,U b) {",
    "__device__ void action(const U* g,U a,const int* x,int* y) {",
    "__device__ U move_action(const U* g,U candidate) {",
    "__device__ U gate_count(const U* s) {",
    "__device__ U root_id(const int* v,const U* g) {",
    "__device__ int gate_value(const int* gates,U at,U kind,U index,U candidate) {",
];

/// Exact body of the ordinary `resulting_port` (`discrete.cu`).
#[cfg(test)]
const ORDINARY_RESULTING_PORT_BODY: &str = r#"    U a=move_action(g,candidate);
    if(kind==0)return compose(g,a,routes[index]);
    int out[8];action(g,a,table+index*8,out);return root_id(out,g);
}"#;

/// Exact lane rule of `discrete_choose` (`discrete.cu`).
#[cfg(test)]
const ORDINARY_CHOOSER_RULE: &str = r#"    if(kind==GATE_KIND) {
        // Strict improvement only; ties keep the lower ordinal (CURRENT, then 0, -1, +1).
        for(U i=1;i<3;++i)if(losses[i]<losses[best])best=i;
    } else {
        for(U i=1;i<16;++i) {
            bool tie=best!=0 && losses[i]==losses[best] && resulting_port(table,routes,g,s,kind,index,i)<resulting_port(table,routes,g,s,kind,index,best);
            if(losses[i]<losses[best] || tie)best=i;
        }
    }"#;

/// Exact move writes of `discrete_apply` (`discrete.cu`).
#[cfg(test)]
const ORDINARY_APPLY_MOVE: &str = r#"    U a=move_action(g,(U)proposal[0]);
    if(kind==0)routes[index]=compose(g,a,routes[index]);
    else {int out[8];action(g,a,table+index*8,out);for(U d=0;d<8;++d)table[index*8+d]=out[d];}"#;

const BVGK: &str = r#"
// ---- BVG-K set selection and one-epoch set commit (opt-in, Stage C2) ----
// All k coordinates of one launch are one kind (0 route, 1 vector block,
// GATE_KIND gate), scored by discrete_counterfactual_rowsk_*:
//   losses[c*16 + lane], status[c]           C1 outputs, read unchanged
//   eligible[c]                              0 = refused by policy (FrozenZero)
//   records[c*8 + w]                         per-coordinate proposal record:
//     w0 chosen lane, w1 loss(lane 0), w2 loss(chosen), w3 epoch at choice,
//     w4 objective sensitivity (w0..w4 = the discrete_choose proposal words),
//     w5 kind, w6 index, w7 the coordinate's C1 status word
//   member[c]                                1 iff c is in S_t
//   summary[0..3]                            |S_t|, best ordinal, L({best});
//                                            best and L are ~0 when S_t empty
//   joint[0..2]                              present flag, L(S_t) (H2b fills
//                                            these on a scratch state clone)
//   commit[c]                                1 iff c is committed
// Status codes (written only on refusal; the caller zeroes the word first):
//   3 stale epoch; 4 out of range (lane, kind, index or summary); 5 a member
//   is not strictly improving or the epoch is at MAX; 9 gated mismatch;
//   14 a member's recorded kind differs from the launch kind; 15 two members
//   share one index (storage would not be disjoint); 16 the commit set is
//   empty; 17 |S_t| >= 2 with the joint loss absent.
// decide zeroes commit[0..k] and commit_count before either refusal return,
// so a refused batch never leaves a previous batch's commit set for apply.
// Status 16 is a misuse code: the host never launches apply when
// commit_count == 0 (an empty S_t is a legal no-op batch, not a refusal).
// Apply validates every member before any write; on refusal nothing changes.
#define BVGK_RECORD 8u
__device__ bool bvgk_in_range(const U* s,U kind,U index) {
    if(kind==GATE_KIND)return index<gate_count(s);
    if(kind==0)return index<s[1]*s[5];
    if(kind==1)return index<s[0]*s[1];
    return false;
}
__device__ U bvgk_resulting_port(const int* table,const U* routes,const U* g,const U* s,U kind,U index,U candidate) {
    U a=move_action(g,candidate);
    if(kind==0)return compose(g,a,routes[index]);
    int out[8];action(g,a,table+index*8,out);return root_id(out,g);
}
extern "C" __global__ void discrete_bvgk_choose(const W* all_losses,const U* all_status,const U* eligible,U k,const W* epoch,const int* table,const U* routes,const U* g,const U* s,U kind,const U* indices,W* records,U* member,W* summary) {
    if(blockIdx.x)return;
    for(U c=threadIdx.x;c<k;c+=blockDim.x) {
        const W* losses=all_losses+c*16;W* p=records+c*BVGK_RECORD;U index=indices[c];
        U best=0;W sensitive=0;U active=kind==GATE_KIND?3u:16u;
        bool ok=all_status[c]==0 && eligible[c]!=0 && bvgk_in_range(s,kind,index);
        if(ok) {
            for(U i=1;i<active;++i)if(losses[i]!=losses[0])sensitive=1;
            if(kind==GATE_KIND) {
                for(U i=1;i<3;++i)if(losses[i]<losses[best])best=i;
            } else {
                for(U i=1;i<16;++i) {
                    bool tie=best!=0 && losses[i]==losses[best] && bvgk_resulting_port(table,routes,g,s,kind,index,i)<bvgk_resulting_port(table,routes,g,s,kind,index,best);
                    if(losses[i]<losses[best] || tie)best=i;
                }
            }
        }
        p[0]=best;p[1]=losses[0];p[2]=losses[best];p[3]=*epoch;p[4]=sensitive;p[5]=kind;p[6]=index;p[7]=all_status[c];
        member[c]=ok && best!=0 && losses[best]<losses[0] ? 1u : 0u;
    }
    __syncthreads();
    if(threadIdx.x)return;
    W count=0,best=~(W)0,best_loss=~(W)0;
    for(U c=0;c<k;++c) {
        if(!member[c])continue;++count;
        W l=records[c*BVGK_RECORD+2];
        if(l<best_loss){best_loss=l;best=c;}
    }
    summary[0]=count;summary[1]=best;summary[2]=best_loss;
}
extern "C" __global__ void discrete_bvgk_decide(const U* member,const W* summary,U k,const W* joint,U* commit,W* commit_count,U* status) {
    if(blockIdx.x || threadIdx.x)return;
    W count=summary[0],best=summary[1];
    for(U c=0;c<k;++c)commit[c]=0u;
    *commit_count=0;
    if(count>k || (count && (best>=k || !member[best]))){*status=4u;return;}
    if(count>=2 && joint[0]==0){*status=17u;return;}
    bool whole=count>=2 && joint[1]<summary[2];
    W n=0;
    for(U c=0;c<k;++c) {
        U m=count==0 ? 0u : (whole ? member[c] : (c==best ? 1u : 0u));
        commit[c]=m;n+=m;
    }
    *commit_count=n;
}
extern "C" __global__ void discrete_bvgk_apply(int* table,U* routes,int* gates,const U* g,const U* s,const W* records,const U* commit,U kind,U k,W* epoch,U* status) {
    if(blockIdx.x || threadIdx.x)return;
    bool gated=s[8]==MODE_FLOCK || s[8]==MODE_RELATION;
    U n=0;
    for(U c=0;c<k;++c) {
        if(!commit[c])continue;++n;
        const W* p=records+c*BVGK_RECORD;
        if(p[5]!=kind){*status=14u;return;}
        if(p[3]!=*epoch){*status=3u;return;}
        if((kind==GATE_KIND && !gated) || (kind==0 && gated)){*status=9u;return;}
        if(p[0]>=(kind==GATE_KIND?3u:16u) || p[6]>=(W)~0u || !bvgk_in_range(s,kind,(U)p[6])){*status=4u;return;}
        if(p[0]==0 || p[2]>=p[1] || *epoch==~(W)0){*status=5u;return;}
        for(U d=0;d<c;++d)if(commit[d] && records[d*BVGK_RECORD+6]==p[6]){*status=15u;return;}
    }
    if(n==0){*status=16u;return;}
    for(U c=0;c<k;++c) {
        if(!commit[c])continue;
        const W* p=records+c*BVGK_RECORD;U lane=(U)p[0],index=(U)p[6];
        if(kind==GATE_KIND){gates[index]=gate_value(gates,index,GATE_KIND,index,lane);continue;}
        U a=move_action(g,lane);
        if(kind==0)routes[index]=compose(g,a,routes[index]);
        else {int out[8];action(g,a,table+index*8,out);for(U d=0;d<8;++d)table[index*8+d]=out[d];}
    }
    ++*epoch;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn rowsk() -> String {
        let rows = crate::rows_source::generate(include_str!("discrete.cu")).expect("rows module");
        crate::rowsk_source::generate(&rows).expect("rows-K module")
    }

    fn entries(text: &str) -> Vec<&str> {
        text.match_indices("extern \"C\" __global__ void ")
            .map(|(at, prefix)| {
                let name = &text[at + prefix.len()..];
                &name[..name.find('(').expect("entry signature")]
            })
            .collect()
    }

    #[test]
    fn bvgk_module_appends_exactly_three_entries_to_the_rowsk_module() {
        let rowsk = rowsk();
        let bvgk = generate(&rowsk).expect("BVG-K module");
        assert!(
            bvgk.starts_with(&rowsk),
            "the K-coordinate text is a prefix, unchanged"
        );
        let mut expected = crate::rowsk_source::ENTRIES.to_vec();
        expected.extend([
            "discrete_bvgk_choose",
            "discrete_bvgk_decide",
            "discrete_bvgk_apply",
        ]);
        assert_eq!(ENTRIES.to_vec(), expected);
        assert_eq!(entries(&bvgk), ENTRIES);
    }

    #[test]
    fn bvgk_text_is_integer_only() {
        let lower = BVGK.to_ascii_lowercase();
        for token in [
            "float",
            "double",
            "half",
            "exp",
            "log",
            "sigmoid",
            "tanh",
            "softplus",
            concat!("soft", "max"),
            "sqrt",
            "mma.",
            "mma_",
            "wmma",
            "tensor",
            "matmul",
            "gemm",
            "cublas",
            "atomicadd",
        ] {
            assert!(!lower.contains(token), "forbidden token {token:?} in BVG-K");
        }
    }

    fn reindent(text: &str, extra: &str) -> String {
        text.lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .map(|line| format!("{extra}{line}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn bvgk_mirrors_the_ordinary_chooser_apply_and_port_rules() {
        // The ordinary kernels live after the rows prefix cut, so the mirrors
        // are pinned against discrete.cu itself: a change there fails here.
        let ordinary = include_str!("discrete.cu");
        for anchor in [
            ORDINARY_CHOOSER_RULE,
            ORDINARY_APPLY_MOVE,
            ORDINARY_RESULTING_PORT_BODY,
        ] {
            assert_eq!(ordinary.matches(anchor).count(), 1, "{anchor}");
        }
        let chooser = reindent(ORDINARY_CHOOSER_RULE, "        ")
            .replace("resulting_port(", "bvgk_resulting_port(");
        assert_eq!(BVGK.matches(chooser.as_str()).count(), 1);
        let apply = reindent(ORDINARY_APPLY_MOVE, "    ").replace("(U)proposal[0]", "lane");
        assert_eq!(BVGK.matches(apply.as_str()).count(), 1);
        let port = format!(
            "__device__ U bvgk_resulting_port(const int* table,const U* routes,const U* g,const U* s,U kind,U index,U candidate) {{\n{ORDINARY_RESULTING_PORT_BODY}"
        );
        assert_eq!(BVGK.matches(port.as_str()).count(), 1);
        // Chooser sensitivity line, proposal words, gate write and the port
        // signature appear in both texts (the proposal record is `p[w]` here
        // and `proposal[w]` in the ordinary kernel).
        for anchor in [
            "for(U i=1;i<active;++i)if(losses[i]!=losses[0])sensitive=1;",
            "gates[index]=gate_value(gates,index,GATE_KIND,index,",
        ] {
            assert_eq!(ordinary.matches(anchor).count(), 1, "{anchor}");
            assert_eq!(BVGK.matches(anchor).count(), 1, "{anchor}");
        }
        for (text, name) in [(ordinary, "proposal"), (BVGK, "p")] {
            let words = format!(
                "{name}[0]=best;{name}[1]=losses[0];{name}[2]=losses[best];{name}[3]=*epoch;{name}[4]=sensitive;"
            );
            assert_eq!(text.matches(words.as_str()).count(), 1, "{words}");
        }
        let signature =
            "(const int* table,const U* routes,const U* g,const U* s,U kind,U index,U candidate) {";
        assert_eq!(
            ordinary
                .matches(format!("__device__ U resulting_port{signature}").as_str())
                .count(),
            1
        );
        assert_eq!(
            BVGK.matches(format!("__device__ U bvgk_resulting_port{signature}").as_str())
                .count(),
            1
        );
    }

    #[test]
    fn bvgk_decide_zeroes_the_commit_set_before_every_refusal() {
        let at = BVGK.find("void discrete_bvgk_decide(").expect("decide");
        let end = BVGK.find("void discrete_bvgk_apply(").expect("apply");
        let decide = &BVGK[at..end];
        let zero = decide
            .find("for(U c=0;c<k;++c)commit[c]=0u;")
            .expect("zeroing loop");
        let count = decide.find("*commit_count=0;").expect("count reset");
        for refusal in ["*status=4u;return;", "*status=17u;return;"] {
            let r = decide.find(refusal).expect(refusal);
            assert!(zero < r && count < r, "zeroing must precede {refusal}");
        }
    }

    #[test]
    fn changed_anchors_are_refused() {
        let rowsk = rowsk();
        for helper in PREFIX_HELPERS {
            assert!(generate(&rowsk.replace(helper, "__device__ void gone() {")).is_err());
        }
        assert!(generate("no reducer here").is_err());
    }
}
