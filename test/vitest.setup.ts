import { beforeEach, describe, it, expect } from "vitest";
import prettier from "prettier/standalone";
import parserHtml from "prettier/parser-html";
import { diffLines } from "diff"; // tiny diff util bundled with vitest deps
import pc from "picocolors"; // for nice CLI colors (already in vitest)

beforeEach(() => {
  document.body.innerHTML = "";
});

/* ------------------------------------------------------------ *
 * helpers
 * ------------------------------------------------------------ */

/** Format HTML through Prettier so that diff is line-by-line & human-readable */
function prettyHtml(html: string): string {
  try {
    return prettier
      .format(html, {
        parser: "html",
        plugins: [parserHtml],
        htmlWhitespaceSensitivity: "ignore",
      })
      .trim();
  } catch {
    // fallback: at least trim + collapse
    return html.replace(/\s+/g, " ").trim();
  }
}

/** Collapse redundant whitespace so that Prettier’s spacing around inline tags is normalized away */
function normalizeWhitespace(html: string): string {
  return html
    .replace(/>\s+/g, ">") // remove space after tag open
    .replace(/\s+</g, "<") // remove space before tag close/open
    .replace(/\s{2,}/g, " ") // collapse multiple spaces
    .trim();
}

/** create a unified diff with +/- lines and basic color */
function formatDiff(expected: string, received: string): string {
  return diffLines(expected, received)
    .map((part) => {
      if (part.added) return pc.green("+ " + part.value);
      if (part.removed) return pc.red("- " + part.value);
      return "  " + part.value;
    })
    .join("");
}

/* ------------------------------------------------------------ *
 * custom matcher
 * ------------------------------------------------------------ */

expect.extend({
  toEqualNormalizedHtml(received: string, expected: string) {
    // 1) Prettier-format both strings
    const prettyExp = prettyHtml(expected);
    const prettyRec = prettyHtml(received);

    // 2) Normalize redundant whitespace introduced by Prettier
    const normExp = normalizeWhitespace(prettyExp);
    const normRec = normalizeWhitespace(prettyRec);

    const pass = normExp === normRec;

    return {
      pass,
      message: () =>
        pass
          ? pc.green("✅ HTML matched")
          : pc.red("❌ HTML mismatch:\n\n") + formatDiff(normExp, normRec),
    };
  },
});

/* ------------------------------------------------------------ *
 * typings (optional, for TS IntelliSense)
 * ------------------------------------------------------------ */

declare module "vitest" {
  // biome-ignore lint/suspicious/noExplicitAny: Vitest matcher typing shim
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  interface Assertion<T = any> {
    toEqualNormalizedHtml(expected: string): void;
  }
}
