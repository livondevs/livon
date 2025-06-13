import { describe, it, expect } from "vitest";
import ToggleBlock from "./8.lun";
import { valueObj } from "lunas/engine";

describe("ToggleBlock", () => {
  it("初期状態で表示される", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    ToggleBlock({ showBlock: new valueObj(true) }).mount(container);
    expect(container.textContent).toContain("THIS IS IF BLOCK");
  });

  it("初期状態で非表示の場合、ブロックが表示されない", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    ToggleBlock({ showBlock: new valueObj(false) }).mount(container);
    expect(container.textContent).not.toContain("THIS IS IF BLOCK");
  });

  it("ボタンクリックでブロックが非表示になる", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = ToggleBlock({ showBlock: new valueObj(true) });
    component.mount(container);

    const button = container.querySelector("button")!;
    button.click();
    await Promise.resolve();

    expect(container.textContent).not.toContain("THIS IS IF BLOCK");
  });

  it("ボタンを2回クリックすると再び表示される", async () => {
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

  it("showBlockの値表示が状態と一致する", async () => {
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

  it("unmountするとDOMが消える", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = ToggleBlock({ showBlock: new valueObj(true) });
    component.mount(container);
    component.__unmount();

    expect(container.innerHTML).toBe("");
  });
});
