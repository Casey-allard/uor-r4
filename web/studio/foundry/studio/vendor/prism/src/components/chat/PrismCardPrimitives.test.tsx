import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { PrismCard, PrismStateIndicator } from "./PrismCardPrimitives";

describe("PrismCard", () => {
  it("lifts the card surface on hover in dark mode", () => {
    render(<PrismCard data-testid="card">Content</PrismCard>);

    const card = screen.getByTestId("card");
    expect(card.className).toContain("dark:bg-(--background-lighter)");
    expect(card.className).toContain("dark:hover:bg-muted/50");
  });
});

describe("PrismStateIndicator", () => {
  it("renders warning state with an amber indicator", () => {
    const { container } = render(
      <PrismStateIndicator state="warning" warningLabel="Needs attention" />,
    );

    expect(screen.getByText("Needs attention")).toBeTruthy();
    expect(container.querySelector(".text-amber-600")).toBeTruthy();
  });
});
