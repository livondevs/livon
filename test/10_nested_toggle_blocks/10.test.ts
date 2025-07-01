import { describe, it, expect } from "vitest";
import NestedToggleBlocks from "./10.lun";

describe("NestedToggleBlocks", () => {
  it("should show only Block2 initially", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    NestedToggleBlocks().mount(container);
    // Block1は非表示、Block2も非表示（Block1が非表示なので）
    expect(container.textContent).not.toContain("This is Block 1");
    expect(container.textContent).not.toContain("This is Block 2");
    expect(container.textContent).toContain("hello world");
    // Block2のボタンはVisible、Block1のボタンはHidden
    expect(container.querySelector("#btn1")?.textContent).toContain("Hidden");
    expect(container.querySelector("#btn2")?.textContent).toContain("Visible");
  });

  it("should show Block1 and Block2 after btn1 click", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = NestedToggleBlocks();
    component.mount(container);
    const btn1 = container.querySelector("#btn1") as HTMLButtonElement;
    btn1.click();
    await Promise.resolve();
    expect(container.textContent).toContain("This is Block 1");
    expect(container.textContent).toContain("This is Block 2");
    expect(container.textContent).toContain("hello world!");
    // Block1のボタンはVisible
    expect(btn1.textContent).toContain("Visible");
  });

  it("should hide Block2 after btn2 click (when Block1 is visible)", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = NestedToggleBlocks();
    component.mount(container);
    // Block1を表示
    const btn1 = container.querySelector("#btn1") as HTMLButtonElement;
    btn1.click();
    await Promise.resolve();
    // Block2を非表示
    const btn2 = container.querySelector("#btn2") as HTMLButtonElement;
    btn2.click();
    await Promise.resolve();
    expect(container.textContent).toContain("This is Block 1");
    expect(container.textContent).not.toContain("This is Block 2");
    expect(container.textContent).toContain("hello world!!");
    // Block2のボタンはHidden
    expect(btn2.textContent).toContain("Hidden");
  });

  it("should toggle Block1 and Block2 independently", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = NestedToggleBlocks();
    component.mount(container);
    const btn1 = container.querySelector("#btn1") as HTMLButtonElement;
    const btn2 = container.querySelector("#btn2") as HTMLButtonElement;
    // Block1表示
    btn1.click();
    await Promise.resolve();
    expect(container.textContent).toContain("This is Block 1");
    // Block2非表示
    btn2.click();
    await Promise.resolve();
    expect(container.textContent).not.toContain("This is Block 2");
    // Block1非表示
    btn1.click();
    await Promise.resolve();
    expect(container.textContent).not.toContain("This is Block 1");
    // Block1再表示（Block2は非表示のまま）
    btn1.click();
    await Promise.resolve();
    expect(container.textContent).toContain("This is Block 1");
    expect(container.textContent).not.toContain("This is Block 2");
    // Block2再表示
    btn2.click();
    await Promise.resolve();
    expect(container.textContent).toContain("This is Block 2");
  });

  it("should append '!' to 'hello world' on each toggle", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = NestedToggleBlocks();
    component.mount(container);
    const btn1 = container.querySelector("#btn1") as HTMLButtonElement;
    const btn2 = container.querySelector("#btn2") as HTMLButtonElement;
    // 初期
    // expect(container.textContent).toContain("hello world");
    expect(container.querySelector("#message")?.textContent).toBe(
      "Current message: hello world"
    );
    // btn1トグル
    btn1.click();
    await Promise.resolve();
    expect(container.querySelector("#message")?.textContent).toBe(
      "Current message: hello world!"
    );
    // btn2トグル
    btn2.click();
    await Promise.resolve();
    expect(container.textContent).toContain("hello world!!");
    // btn1トグル
    btn1.click();
    await Promise.resolve();
    expect(container.textContent).toContain("hello world!!!");
    // btn2トグル
    btn2.click();
    await Promise.resolve();
    expect(container.textContent).toContain("hello world!!!!");
  });
});
