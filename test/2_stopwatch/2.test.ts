import { describe, it, expect, beforeEach, afterEach } from "vitest";
import Stopwatch from "./2.lun";

function expectedHTML(count: number, interval: number | null) {
  return (
    `<div>` +
    `<h1>Stopwatch by Lunas</h1>` +
    `<div>${count}</div>` +
    `<button>${interval == null ? "Start" : "Stop"}</button>` +
    `<button>Clear</button>` +
    `</div>`
  );
}

describe("Stopwatch", () => {
  let container: HTMLDivElement;

  beforeEach(() => {
    container = document.createElement("div");
    document.body.appendChild(container);
  });

  afterEach(() => {
    container.remove();
  });

  it("should mount with correct initial HTML", () => {
    Stopwatch().mount(container);
    expect(container.innerHTML).toBe(expectedHTML(0, expect.any(Number)));
  });

  it("should stop interval and update button text to Start", async () => {
    const component = Stopwatch();
    component.mount(container);
    const toggleBtn = container.querySelectorAll("button")[0];
    toggleBtn.click(); // Stop
    await Promise.resolve();
    expect(container.innerHTML).toBe(expectedHTML(0, null));
  });

  it("should start interval again and update button text to Stop", async () => {
    const component = Stopwatch();
    component.mount(container);
    const toggleBtn = container.querySelectorAll("button")[0];
    toggleBtn.click(); // Stop
    await Promise.resolve();
    toggleBtn.click(); // Start
    await Promise.resolve();
    expect(container.innerHTML).toBe(expectedHTML(0, expect.any(Number)));
  });
});
