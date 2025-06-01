import { compile } from "lunas/compiler";
import type { Plugin } from "vite";
import { resolve as resolvePath } from "path";

// Define the project root directory for path validation
const PROJECT_ROOT = resolvePath(__dirname, "../");

/**
 * Vite plugin for compiling and handling .lun and .lun.ts files.
 */
export function lunas(): Plugin {
  // Map to store generated CSS code for each .lun file
  const cssCodeMap = new Map<string, string>();

  return {
    name: "vite-plugin-lunas",

    /**
     * Resolve virtual module IDs for .lun style imports.
     */
    resolveId(source) {
      const [filepath, query] = source.split("?", 2);
      // Check if the file is a .lun file with ?style.css or ?style query
      if (
        filepath.endsWith(".lun") &&
        (query === "style.css" || query === "style")
      ) {
        const absPath = resolvePath(filepath);
        // Only allow files inside the project root
        if (!absPath.startsWith(PROJECT_ROOT)) {
          return null;
        }
        return source;
      }
      return null;
    },

    /**
     * Transform .lun and .lun.ts files during the build process.
     */
    async transform(code, id) {
      const absId = resolvePath(id.split("?")[0]);
      // Ignore files outside the project root
      if (!absId.startsWith(PROJECT_ROOT)) {
        return null;
      }

      // Handle .lun files
      if (absId.endsWith(".lun")) {
        const result = compile(code);
        // If CSS is present, store it and add a virtual import
        if (result.css) {
          cssCodeMap.set(absId, result.css);
          const virtualImport = `import ${JSON.stringify(
            `${id}?style.css`
          )};\n`;
          return {
            code: virtualImport + result.js,
            map: null,
          };
        }
        return {
          code: result.js,
          map: null,
        };
      }

      return null;
    },

    /**
     * Load the virtual CSS module for .lun files.
     */
    load(id) {
      if (id.endsWith(".lun?style.css")) {
        const filepath = id.replace("?style.css", "");
        const absPath = resolvePath(filepath);
        // Return CSS only if the file exists in the map
        if (!cssCodeMap.has(absPath)) {
          return null;
        }
        return cssCodeMap.get(absPath)!;
      }
      return null;
    },
  };
}
