import React from "react";
import { CustomTagState } from "./stateTypes";
import { PrismDbProjectInfo } from "./PrismDbProjectInfo";

interface PrismSupabaseProjectInfoProps {
  node: {
    properties: {
      state?: CustomTagState;
    };
  };
  children: React.ReactNode;
}

export function PrismSupabaseProjectInfo(props: PrismSupabaseProjectInfoProps) {
  return <PrismDbProjectInfo provider="Supabase" {...props} />;
}
