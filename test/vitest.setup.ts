import { beforeEach, afterEach, describe, it, expect } from "vitest";

let root: HTMLDivElement;

beforeEach(() => {
  document.body.innerHTML = "";
});

function normalizeHtmlString(html: string): string {
  return html
    .replace(/>\s+</g, "><") // タグ間の空白・改行除去
    .replace(/>\s+/g, ">") // タグ直後の空白除去
    .replace(/\s+</g, "<") // タグ直前の空白除去
    .replace(/\s{2,}/g, " ") // 複数空白を1つに
    .replace(/^\s+|\s+$/g, "") // 前後の空白除去
    .replace(/\s*(=)\s*/g, "$1") // = 前後の空白除去
    .replace(/"\s+/g, '"') // 属性値内の空白
    .replace(/\s+"/g, '"');
}

// 型付きのアサーションを登録
expect.extend({
  toEqualNormalizedHtml(received: string, expected: string) {
    const normA = normalizeHtmlString(received);
    const normB = normalizeHtmlString(expected);
    const pass = normA === normB;

    return {
      pass,
      message: () =>
        pass
          ? `✅ HTML matched`
          : `❌ HTML mismatch.\n\nExpected:\n${normB}\n\nReceived:\n${normA}`,
    };
  },
});
