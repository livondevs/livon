import { describe, it, expect } from "vitest";
import MultiIf from "./9.lun";

describe("MultiIfBlocksRandom", () => {
  it("should display Static always", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    MultiIf().mount(container);
    expect(container.textContent).toContain("Static");
  });

  it("should toggle all blocks", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = MultiIf();
    component.mount(container);
    const before = container.textContent;
    const btn = container.querySelector("button")!;
    btn.click();
    await Promise.resolve();
    expect(container.textContent).not.toBe(before);
  });
});
