import "vitest";

declare module "vitest" {
  interface Assertion<T = any> {
    toEqualNormalizedHtml(expected: string): void;
  }

  interface AsymmetricMatchersContaining {
    toEqualNormalizedHtml(expected: string): void;
  }
}
