import { describe, it, expect } from "vitest";
import StaticMessage from "./7.lun";

describe("StaticMessage", () => {
  it("should display static message", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    StaticMessage().mount(container);
    expect(container.textContent).toContain("Hello Lunas");
  });
});
