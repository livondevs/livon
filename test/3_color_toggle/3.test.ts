import { describe, it, expect } from "vitest";
import ColorToggle from "./3.lun";

describe("ColorToggle", () => {
  it("should display red initially", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    ColorToggle().mount(container);
    const span = container.querySelector("span")!;
    expect(
      span.style.color === "red" || span.getAttribute("style")?.includes("red")
    ).toBeTruthy();
  });

  it("should change to yellow on yellow button click", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = ColorToggle();
    component.mount(container);
    const btn = container.querySelectorAll("button")[0];
    btn.click();
    await Promise.resolve();
    const span = container.querySelector("span")!;
    expect(span.getAttribute("style")?.includes("yellow")).toBeTruthy();
  });

  it("should change to blue on blue button click", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = ColorToggle();
    component.mount(container);
    const btn = container.querySelectorAll("button")[2];
    btn.click();
    await Promise.resolve();
    const span = container.querySelector("span")!;
    expect(span.getAttribute("style")?.includes("blue")).toBeTruthy();
  });
});
