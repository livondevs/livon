#!/usr/bin/env node
import * as fs from "fs";
import * as path from "path";
import {
  formatParsedContent,
  parseCustomSyntax,
  ParsedContent,
} from "./format";

/**
 * Main execution function.
 */
async function main(): Promise<void> {
  // Get the input and output files from command-line arguments.
  const inputFile: string = process.argv[2] || "input.txt";
  const outputFile: string = process.argv[3] || "output.txt";
  const inputFilePath: string = path.resolve(process.cwd(), inputFile);
  const outputFilePath: string = path.resolve(process.cwd(), outputFile);

  let fileContent: string;
  try {
    fileContent = fs.readFileSync(inputFilePath, "utf8");
  } catch (error: unknown) {
    console.error(`Failed to read file: ${inputFilePath}`);
    process.exit(1);
  }

  // Parse the custom syntax.
  const parsed: ParsedContent = parseCustomSyntax(fileContent);
  // Format each section and build the output.
  const output: string = await formatParsedContent(parsed);

  try {
    fs.writeFileSync(outputFilePath, output, "utf8");
  } catch (error: unknown) {
    console.error(`Failed to write file: ${outputFilePath}`);
    process.exit(1);
  }
}

main();
