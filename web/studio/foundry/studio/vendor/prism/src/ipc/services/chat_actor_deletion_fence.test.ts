import { describe, expect, it } from "vitest";
import { PrismErrorKind } from "@/errors/prism_error";
import {
  assertChatActorAdmissionOpen,
  beginChatActorDeletion,
} from "./chat_actor_deletion_fence";

describe("chat actor deletion fence", () => {
  it("blocks admission until every deletion lease is released", () => {
    const releaseFirst = beginChatActorDeletion(7);
    const releaseSecond = beginChatActorDeletion(7);

    expect(() => assertChatActorAdmissionOpen(7)).toThrowError(
      expect.objectContaining({ kind: PrismErrorKind.Precondition }),
    );
    releaseFirst();
    expect(() => assertChatActorAdmissionOpen(7)).toThrowError(
      expect.objectContaining({ kind: PrismErrorKind.Precondition }),
    );

    releaseSecond();
    expect(() => assertChatActorAdmissionOpen(7)).not.toThrow();
  });
});
