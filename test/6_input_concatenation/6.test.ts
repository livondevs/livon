import { describe, it, expect } from "vitest";
import InputConcat from "./6.lun";

describe("InputConcat", () => {
  it("should display input value", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    InputConcat().mount(container);
    const input = container.querySelector("input")!;
    input.value = "foo";
    input.dispatchEvent(new Event("input"));
    await Promise.resolve();
    expect(container.textContent).toContain("foo");
  });

  it("should append 'foo' when setFoo button clicked", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = InputConcat();
    component.mount(container);
    const btn = container.querySelectorAll("button")[0];
    btn.click();
    await Promise.resolve();
    expect(container.textContent).toContain("foo");
  });

  it("should append 'hoge' when addFoo1 button clicked", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = InputConcat();
    component.mount(container);
    const btn = container.querySelectorAll("button")[1];
    btn.click();
    await Promise.resolve();
    expect(container.textContent).toContain("hoge");
  });
});
