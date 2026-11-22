import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { PrismExecuteSql } from "./PrismExecuteSql";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) =>
      ({
        changesDatabaseSchema: "Changes database schema",
        destructiveDataChange: "Destructive data change",
      })[key] ?? key,
  }),
}));

describe("PrismExecuteSql", () => {
  it("shows a schema mutation indicator for DDL", () => {
    render(<PrismExecuteSql>CREATE TABLE users (id bigint);</PrismExecuteSql>);

    expect(screen.getByText("Changes database schema")).toBeTruthy();
  });

  it("extracts SQL text from string children mixed with React nodes", () => {
    render(
      <PrismExecuteSql>
        {"CREATE "}
        <span>ignored</span>
        {"TABLE users (id bigint);"}
      </PrismExecuteSql>,
    );

    expect(screen.getByText("Changes database schema")).toBeTruthy();
  });

  it("omits the schema mutation indicator for ordinary queries", () => {
    render(<PrismExecuteSql>SELECT * FROM users;</PrismExecuteSql>);

    expect(screen.queryByText("Changes database schema")).toBeNull();
  });

  it("shows a destructive data indicator for deletes", () => {
    render(<PrismExecuteSql>DELETE FROM users WHERE id = 1;</PrismExecuteSql>);

    expect(screen.getByText("Destructive data change")).toBeTruthy();
  });
});
