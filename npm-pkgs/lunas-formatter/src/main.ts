import {
  ParsedContent,
  parseCustomSyntax,
  formatParsedContent,
} from "./format";

export async function format(input: string): Promise<string> {
  const parsed: ParsedContent = parseCustomSyntax(input);
  return await formatParsedContent(parsed);
}
