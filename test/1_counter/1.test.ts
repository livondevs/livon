// 02/2.test.ts
import { describe, it, expect } from "vitest";
import Counter from "./1.lun";

describe("Counter", () => {
  it("should display 0 initially", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    Counter().mount(container);
    expect(container.textContent).toContain("0");
  });

  it("should display 1 after one click", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = Counter();
    component.mount(container);

    const button = container.querySelector("button")!;
    button.click();
    await Promise.resolve();

    expect(container.textContent).toContain("1");
  });

  it("should display 3 after three clicks", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = Counter();
    component.mount(container);

    const button = container.querySelector("button")!;
    button.click();
    button.click();
    button.click();
    await Promise.resolve();

    expect(container.textContent).toContain("3");
  });

  it("should clear DOM after unmount", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = Counter();
    component.mount(container);
    component.__unmount();

    expect(container.innerHTML).toBe("");
  });
});
