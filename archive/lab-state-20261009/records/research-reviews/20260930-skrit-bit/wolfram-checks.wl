(* Executed with Wolfram Language evaluator, 10 second limit, 2026-09-30.
   These are exact finite illustrations, not a compiler or general theorem proof. *)
ac = 2^9 - 1;
hal = 2^42 - 2^9;
domain = Range[0, 11];
a = Select[domain, EvenQ];
b = Select[domain, Mod[#,3] == 0 &];
both = Intersection[a,b];
family = {domain,a,b,both};
resolve[x_, fam_] := Module[{m = Select[fam, MemberQ[#,x]&]},
  Select[m, Function[s, !AnyTrue[m,
    Function[t, t =!= s && Complement[t,s] === {}]]]]];
result = <|
 "acDecimal"->ac, "acHex"->IntegerString[ac,16],
 "halBits"->Total[IntegerDigits[hal,2]],
 "partitionDisjoint"->(BitAnd[ac,hal]==0),
 "partitionComplete"->(BitOr[ac,hal]==2^42-1),
 "evenOnly"->Complement[a,b], "thirdOnly"->Complement[b,a],
 "intersection"->both,
 "ambiguityWithoutIntersectionAt6"->resolve[6,{domain,a,b}],
 "uniqueWithIntersectionForAll12"->And@@Table[Length[resolve[x,family]]==1,{x,domain}],
 "declarationOrderInvariant24Permutations"->And@@Flatten[
   Table[resolve[x,p]===resolve[x,family],{x,domain},{p,Permutations[family]}]],
 "rewriteRootFirst"->(f[a0]/.{f[a0]->b0,a0->c0}),
 "rewriteInnerFirstNormalForm"->(f[a0]/.a0->c0),
 "boundedCategorySubsetCount"->2^42|>;
result
