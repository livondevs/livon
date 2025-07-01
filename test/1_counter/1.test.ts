import { describe, it, expect } from "vitest";
import Counter from "./1.lun";

describe("Counter", () => {
  it("should display 0 initially", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    Counter().mount(container);

    const countDiv = container.querySelector("#count")!;
    expect(countDiv.textContent).toBe("0");

    expect(container.innerHTML).toBe(
      '<div><div id="count">0</div><button id="increment-btn">+1</button></div>'
    );
  });

  it("should display 1 after one click", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = Counter();
    component.mount(container);

    const button = container.querySelector(
      "#increment-btn"
    ) as HTMLButtonElement;
    button.click();
    await Promise.resolve();

    const countDiv = container.querySelector("#count")!;
    expect(countDiv.textContent).toBe("1");
    expect(container.innerHTML).toEqualNormalizedHtml(
      '<div><div id="count">1</div><button id="increment-btn">+1</button></div>'
    );
  });

  it("should display 3 after three clicks", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = Counter();
    component.mount(container);

    const button = container.querySelector(
      "#increment-btn"
    ) as HTMLButtonElement;
    button.click();
    button.click();
    button.click();
    await Promise.resolve();

    const countDiv = container.querySelector("#count")!;
    expect(countDiv.textContent).toBe("3");
    expect(container.innerHTML).toBe(
      '<div><div id="count">3</div><button id="increment-btn">+1</button></div>'
    );
  });

  it("should clear DOM after unmount", async () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const component = Counter();
    component.mount(container);
    component.__unmount();

    expect(container.innerHTML).toBe("");
  });
});
