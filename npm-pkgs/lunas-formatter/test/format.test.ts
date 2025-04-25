/* use vitest */
import { expect, test } from "vitest";
import { promises as fs } from "fs";
import {
  formatParsedContent,
  parseCustomSyntax,
  ParsedContent,
} from "../src/format";
const testFiles = ["counter-game", "pass-value"];

async function testMain() {
  for (const testFile of testFiles) {
    test(`test formatting ${testFiles}`, async () => {
      const input = await fs.readFile(`./test/${testFile}.lun`, "utf8");
      const correctOutput = await fs.readFile(
        `./test/${testFile}.formatted.lun`,
        "utf8",
      );

      // Parse the custom syntax.
      const parsed: ParsedContent = parseCustomSyntax(input);
      // Format each section and build the output.
      const testOutput: string = await formatParsedContent(parsed);

      expect(testOutput).toEqual(correctOutput);
    });
  }
}

testMain();
