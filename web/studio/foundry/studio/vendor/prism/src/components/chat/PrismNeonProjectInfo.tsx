import React from "react";
import { CustomTagState } from "./stateTypes";
import { PrismDbProjectInfo } from "./PrismDbProjectInfo";

interface PrismNeonProjectInfoProps {
  node: {
    properties: {
      state?: CustomTagState;
    };
  };
  children: React.ReactNode;
}

export function PrismNeonProjectInfo(props: PrismNeonProjectInfoProps) {
  return <PrismDbProjectInfo provider="Neon" {...props} />;
}
