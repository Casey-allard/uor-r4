import React, { useState } from "react";
import { useAtomValue } from "jotai";
import { CustomTagState } from "./stateTypes";
import {
  PrismCard,
  PrismCardHeader,
  PrismCardContent,
} from "./PrismCardPrimitives";
import { PauseCircle, Play, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useStreamChat } from "@/hooks/useStreamChat";
import { selectedChatIdAtom } from "@/atoms/chatAtoms";
import { hasPendingReviewContinuation } from "@/hooks/subagentReviewContinuation";

interface PrismStepLimitProps {
  node: {
    properties: {
      steps?: string;
      limit?: string;
      state?: CustomTagState;
    };
  };
  children?: React.ReactNode;
}

export function PrismStepLimit({ node, children }: PrismStepLimitProps) {
  const { steps = "50", limit: _limit = "50", state } = node.properties;
  const isFinished = state === "finished";
  const content = typeof children === "string" ? children : "";
  const chatId = useAtomValue(selectedChatIdAtom);
  const { streamMessage, clearPauseOnly } = useStreamChat();
  const [isLoading, setIsLoading] = useState(false);

  const handleContinue = () => {
    if (!chatId) return;
    setIsLoading(true);
    streamMessage({
      prompt: "Continue",
      chatId,
      onSettled: ({ success, pausedByStepLimit }) => {
        setIsLoading(false);
        if (
          success &&
          !pausedByStepLimit &&
          !hasPendingReviewContinuation(chatId)
        ) {
          clearPauseOnly();
        }
      },
    });
  };

  return (
    <PrismCard state={state} accentColor="amber" isExpanded={true}>
      <PrismCardHeader icon={<PauseCircle size={15} />} accentColor="amber">
        <span className="font-medium text-sm text-foreground">
          Paused after {steps} tool calls
        </span>
        {isFinished && (
          <Button
            variant="outline"
            size="sm"
            disabled={isLoading}
            onClick={handleContinue}
            className="ml-auto hover:cursor-pointer"
          >
            {isLoading ? (
              <Loader2 size={14} className="mr-1 animate-spin" />
            ) : (
              <Play size={14} className="mr-1" />
            )}
            Continue
          </Button>
        )}
      </PrismCardHeader>
      <PrismCardContent isExpanded={true}>
        {content && (
          <div className="p-3 text-sm text-muted-foreground">{content}</div>
        )}
      </PrismCardContent>
    </PrismCard>
  );
}
