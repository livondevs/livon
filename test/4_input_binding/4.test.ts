import { describe, it, expect } from "vitest";
import InputBinding from "./4.lun";

describe("InputBinding", () => {
  it("should display input value", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    InputBinding().mount(container);
    const input = container.querySelector("input")!;
    input.value = "test";
    input.dispatchEvent(new Event("input"));
    await Promise.resolve();
    expect(container.textContent).toContain("test");
  });

  it("should append xxx when button clicked", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = InputBinding();
    component.mount(container);
    const input = container.querySelector("input")!;
    input.value = "a";
    input.dispatchEvent(new Event("input"));
    const btn = container.querySelector("button")!;
    btn.click();
    await Promise.resolve();
    expect(container.textContent).toContain("axxx");
  });
});
