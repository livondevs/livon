import { describe, it, expect } from "vitest";
import ResetMessage from "./5.lun";

describe("ResetMessage", () => {
  it("should reset message when button clicked", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    ResetMessage().mount(container);
    const input = container.querySelector("input")!;
    input.value = "hello";
    input.dispatchEvent(new Event("input"));
    const btn = container.querySelector("button")!;
    btn.click();
    await Promise.resolve();
    expect(container.textContent).toContain("The entered message: ");
  });
});
