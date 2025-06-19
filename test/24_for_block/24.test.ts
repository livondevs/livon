import { describe, it, expect } from "vitest";
import ForBlock from "./24.lun";

describe("ForBlock", () => {
  it("should render only one nested block when all bools are true and counts are 1", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = ForBlock();
    component.mount(container);

    const finalBlock = container.querySelector("#block-0-0-0-0-0-0");
    expect(finalBlock).not.toBeNull();
    expect(finalBlock?.textContent).toBe("Block: 0-0-0-0-0-0");

    // Ensure only one block is rendered
    const allBlocks = container.querySelectorAll("[id^='block-']");
    expect(allBlocks.length).toBe(1);
  });

  it("should remove all blocks when the second bool is set to false", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = ForBlock();
    component.mount(container);

    const toggle = container.querySelector("#toggle-if-1") as HTMLButtonElement;
    toggle.click();
    await Promise.resolve();

    const blocks = container.querySelectorAll("[id^='block-']");
    expect(blocks.length).toBe(0);
  });

  it("should render two top-level for-blocks when count[0] is 2", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = ForBlock();
    component.mount(container);

    const inc = container.querySelector("#inc-for-0") as HTMLButtonElement;
    inc.click();
    await Promise.resolve();

    const block1 = container.querySelector("#block-0-0-0-0-0-0");
    const block2 = container.querySelector("#block-1-0-0-0-0-0");
    expect(block1).not.toBeNull();
    expect(block2).not.toBeNull();
  });

  it("should not decrement below 0", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = ForBlock();
    component.mount(container);

    const dec = container.querySelector("#dec-for-0") as HTMLButtonElement;
    dec.click(); // Already at 1, should go to 0
    dec.click(); // Try to go below 0
    await Promise.resolve();

    const label = container.querySelector("#count-for-0");
    expect(label?.textContent).toContain("0");
  });

  it("should toggle and reflect ON/OFF state", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = ForBlock();
    component.mount(container);

    const toggle = container.querySelector("#toggle-if-0") as HTMLButtonElement;
    expect(toggle.textContent).toContain("ON");
    toggle.click();
    await Promise.resolve();
    expect(toggle.textContent).toContain("OFF");
  });
});
