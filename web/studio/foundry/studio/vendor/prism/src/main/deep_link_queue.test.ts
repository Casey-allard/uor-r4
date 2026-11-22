import { describe, expect, it, vi } from "vitest";
import { createDeepLinkQueue } from "@/main/deep_link_queue";

describe("createDeepLinkQueue", () => {
  it("queues deep links until the app is marked ready", () => {
    const handler = vi.fn();
    const queue = createDeepLinkQueue(handler);

    queue.handle("prism://one");
    queue.handle("prism://two");

    expect(handler).not.toHaveBeenCalled();

    queue.markReady();

    expect(handler).toHaveBeenNthCalledWith(1, "prism://one");
    expect(handler).toHaveBeenNthCalledWith(2, "prism://two");
  });

  it("handles deep links immediately after the app is marked ready", () => {
    const handler = vi.fn();
    const queue = createDeepLinkQueue(handler);

    queue.markReady();
    queue.handle("prism://ready");
    queue.markReady();

    expect(handler).toHaveBeenCalledOnce();
    expect(handler).toHaveBeenCalledWith("prism://ready");
  });

  it("queues again while a newly targeted window is loading", () => {
    const handler = vi.fn();
    const queue = createDeepLinkQueue(handler);
    queue.markReady();
    queue.markNotReady();

    queue.handle("prism://new-window");
    expect(handler).not.toHaveBeenCalled();

    queue.markReady();
    expect(handler).toHaveBeenCalledWith("prism://new-window");
  });
});
