import prettier from "prettier";

/* 最終行が改行であれば改行を保ち、そうでなければフォーマット後にできた空白の最終行を削除しながらフォーマットする関数 */
/* 引数に元のテキストとoptionを受け取る */

async function formatTextWithPreservedNewline(
  text: string,
  options: prettier.Options,
): Promise<string> {
  const hasTrailingNewline = text.slice(0, -1).endsWith("\n");
  const formatted = await prettier.format(text, options);
  return hasTrailingNewline ? formatted : formatted.trimEnd();
}

/**
 * Valid section names.
 */
type SectionName = "html" | "script" | "style";

/**
 * Object type to store content for each section.
 */
interface Sections {
  html?: string;
  script?: string;
  style?: string;
}

/**
 * Parsed content structure containing a preamble and sections.
 */
export type ParsedContent = {
  preamble: string; // Lines that are not part of any section (e.g. directives starting with @)
  sections: Sections;
};

/**
 * Parses the input string with our custom syntax.
 *
 * The input syntax example:
 *
 *   @input message1:string
 *
 *   html:
 *     <div class="child">
 *       This is child component
 *       <div >Message from parent: ${message1}</div>
 *     </div>
 *   style:
 *     .child {
 *       border: dashed blue;
 *       padding: 5px;
 *     }
 *
 * Section header lines (e.g. "html:") must appear at the beginning of a line.
 * Lines belonging to a section must be indented exactly one level (here, 2 spaces).
 * Lines starting with "@" (and any lines before the first section header) are treated as preamble.
 *
 * @param input The input string.
 * @returns ParsedContent containing the preamble and each section's content.
 */
export function parseCustomSyntax(input: string): ParsedContent {
  const preambleLines: string[] = [];
  const sections: Sections = {};
  let currentSection: SectionName | null = null;
  const lines: string[] = input.split(/\r?\n/);

  for (const line of lines) {
    // Check if the line is a section header.
    const headerMatch: RegExpMatchArray | null = line.match(/^(\w+):\s*$/);
    if (headerMatch) {
      const sectionName: string = headerMatch[1];
      if (
        sectionName === "html" ||
        sectionName === "script" ||
        sectionName === "style"
      ) {
        currentSection = sectionName as SectionName;
        sections[currentSection] = "";
      } else {
        // If the section is not one of the target ones, reset currentSection.
        currentSection = null;
      }
      continue;
    }

    // If not in a section, add the line to the preamble.
    if (currentSection === null) {
      preambleLines.push(line);
    } else {
      // Lines within a section should have exactly one indent level (2 spaces).
      if (line.startsWith("  ")) {
        sections[currentSection] += line.slice(2) + "\n";
      } else if (line.trim() === "") {
        // Preserve empty lines.
        sections[currentSection] += "\n";
      } else {
        // If there is no expected indent, record the line as is.
        sections[currentSection] += line + "\n";
      }
    }
  }

  return {
    preamble: preambleLines.join("\n"),
    sections: sections,
  };
}

/**
 * Formats each section using Prettier and then constructs the output string.
 *
 * The output is structured so that each section header (e.g. "html:") is followed by its
 * content lines indented by one level (2 spaces). The preamble is output before any section.
 *
 * For each section, if the original last line did not have a trailing space,
 * the formatted output's last line will have its trailing whitespace trimmed.
 *
 * @param parsed The parsed content (preamble and sections).
 * @returns The formatted output string.
 */
export async function formatParsedContent(
  parsed: ParsedContent,
): Promise<string> {
  // Prettier formatting options for each language.
  const htmlOptions: prettier.Options = {
    parser: "html",
    tabWidth: 2,
    printWidth: Infinity,
    // Avoid Prettier appending a newline at the end.
    endOfLine: "lf",
    plugins: [await import("prettier/plugins/html")],
  };

  const jsOptions: prettier.Options = {
    parser: "typescript",
    tabWidth: 2,
    printWidth: Infinity,
    endOfLine: "lf",
    plugins: [await import("prettier/plugins/typescript")],
  };

  const cssOptions: prettier.Options = {
    parser: "css",
    tabWidth: 2,
    printWidth: Infinity,
    endOfLine: "lf",
    plugins: [await import("prettier/plugins/postcss")],
  };

  const formattedSections: Sections = {};

  if (parsed.sections.html !== undefined) {
    formattedSections.html = await formatTextWithPreservedNewline(
      parsed.sections.html,
      htmlOptions,
    );
  }
  if (parsed.sections.script !== undefined) {
    formattedSections.script = await formatTextWithPreservedNewline(
      parsed.sections.script,
      jsOptions,
    );
  }
  if (parsed.sections.style !== undefined) {
    formattedSections.style = await formatTextWithPreservedNewline(
      parsed.sections.style,
      cssOptions,
    );
  }

  let output = "";
  // Output the preamble (directives, etc.) as-is.
  if (parsed.preamble.trim() !== "") {
    output += parsed.preamble + "\n";
  }

  console.log(formattedSections);

  for (const [sectionIndex, key] of (
    Object.keys(formattedSections) as Array<keyof Sections>
  ).entries()) {
    if (formattedSections[key] !== undefined) {
      output += `${key}:\n`;
      let lines: string[] = formattedSections[key]!.split("\n");
      // Add 2 spaces to the beginning of each line (empty lines are preserved).
      for (const [lineIndex, line] of lines.entries()) {
        if (line.trim() !== "") {
          output += "  " + line;
        }
        if (
          lineIndex !== lines.length - 1 ||
          sectionIndex !== Object.keys(formattedSections).length - 1
        ) {
          output += "\n";
        }
      }
    }
  }

  return output;
}
