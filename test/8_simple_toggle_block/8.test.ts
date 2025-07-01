import { describe, it, expect } from "vitest";
import ToggleBlock from "./8.lun";
import { valueObj } from "lunas/engine";

describe("ToggleBlock", () => {
  it("should be visible initially", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    ToggleBlock({ showBlock: new valueObj(true) }).mount(container);
    expect(container.textContent).toContain("THIS IS IF BLOCK");
  });

  it("should not display the block if initially hidden", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    ToggleBlock({ showBlock: new valueObj(false) }).mount(container);
    expect(container.textContent).not.toContain("THIS IS IF BLOCK");
  });

  it("should hide the block when the button is clicked", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = ToggleBlock({ showBlock: new valueObj(true) });
    component.mount(container);

    const button = container.querySelector("button")!;
    button.click();
    await Promise.resolve();

    expect(container.textContent).not.toContain("THIS IS IF BLOCK");
  });

  it("should show the block again after clicking the button twice", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = ToggleBlock({ showBlock: new valueObj(true) });
    component.mount(container);

    const button = container.querySelector("button")!;
    button.click();
    await Promise.resolve();
    button.click();
    await Promise.resolve();

    expect(container.textContent).toContain("THIS IS IF BLOCK");
  });

  it("should display the showBlock value according to the state", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = ToggleBlock({ showBlock: new valueObj(true) });
    component.mount(container);

    expect(container.textContent).toContain("showBlock value: true");

    const button = container.querySelector("button")!;
    button.click();
    await Promise.resolve();

    expect(container.textContent).toContain("showBlock value: false");
  });

  it("should clear the DOM after unmount", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = ToggleBlock({ showBlock: new valueObj(true) });
    component.mount(container);
    component.__unmount();

    expect(container.innerHTML).toBe("");
  });
});
