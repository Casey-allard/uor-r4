import type { TextStreamPart, ToolSet } from "ai";

export const MODEL_REFUSAL_WARNING =
  '<prism-output type="warning" message="Model refused to respond for safety reasons">The model’s safety system rejected this request. Try switching to a different model.</prism-output>';

export function isModelRefusal(part: TextStreamPart<ToolSet>): boolean {
  return (
    part.type === "finish" &&
    part.finishReason === "content-filter" &&
    part.rawFinishReason === "refusal"
  );
}
