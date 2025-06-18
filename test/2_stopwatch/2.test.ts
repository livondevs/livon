import { describe, it, expect } from "vitest";
import Stopwatch from "./2.lun";

describe("Stopwatch", () => {
  it("should display 0 initially", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    Stopwatch().mount(container);
    expect(container.textContent).toContain("0");
  });

  it("should increment after interval", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = Stopwatch();
    component.mount(container);
    await new Promise((r) => setTimeout(r, 2100));
    expect(
      Number(container.textContent?.match(/\d+/)?.[0] || 0)
    ).toBeGreaterThan(0);
  });

  it("should clear count when clear button is clicked", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = Stopwatch();
    component.mount(container);
    const clearBtn = container.querySelectorAll("button")[1];
    clearBtn.click();
    await Promise.resolve();
    expect(container.textContent).toContain("0");
  });
});
