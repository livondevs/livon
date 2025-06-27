import { describe, it, expect } from "vitest";
import ParentComponentStatic from "./19.lun";

describe("ParentComponentStatic", () => {
  it("should render parent component with static content", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    ParentComponentStatic().mount(container);
    expect(container.innerHTML).toEqualNormalizedHtml(
      '<div class="parent">This is the parent component<div id="child">This is child component</div> Hello Lunas</div>',
    );
  });
});
