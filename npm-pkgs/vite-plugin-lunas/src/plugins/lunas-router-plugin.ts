import path from "path";
import { glob } from "glob";
import type { Plugin } from "vite";
import { resolve as resolvePath } from "path";

// Define the project root directory for security checks
const PROJECT_ROOT = resolvePath(__dirname, "../");

/**
 * Vite plugin for automatic routing based on .lun files in the pages directory.
 */
export function lunasAutoRoutingPlugin(options: { pagesDir: string }): Plugin {
  // Resolve the absolute path of pagesDir and ensure it's inside the project root
  const absPagesDir = resolvePath(options.pagesDir);
  if (!absPagesDir.startsWith(PROJECT_ROOT)) {
    throw new Error(
      "Security error: pagesDir must be inside the project root: " +
        options.pagesDir
    );
  }

  return {
    name: "vite-plugin-lunas-auto-routing",

    /**
     * Resolve the virtual module ID for generated routes.
     */
    resolveId(source) {
      if (source === "virtual:generated-routes") {
        return source;
      }
      return null;
    },

    /**
     * Load and generate routes dynamically based on .lun files.
     */
    load(id) {
      if (id === "virtual:generated-routes") {
        const routesArrayCode = generateRoutesCode(absPagesDir);
        return `export const routes = ${routesArrayCode};`;
      }
      return null;
    },
  };
}

/**
 * Generate a stringified array of route objects from .lun files in the pages directory.
 */
function generateRoutesCode(pagesDir: string): string {
  // Use glob to find all .lun files
  const pattern = path.join(pagesDir, "**/*.lun");
  const files = glob.sync(pattern);

  // Map each file to a route object string
  const routeObjects: string[] = files.map((file) => {
    // Convert absolute path to project-relative path and normalize separators
    const relPath = path.relative(process.cwd(), file).replace(/\\/g, "/");

    // Use empty name for index.lun, otherwise sanitize to lower-case and safe characters only
    let name = path.basename(file, ".lun");
    if (name.toLowerCase() === "index") {
      name = "";
    }
    const safeName = name.toLowerCase().replace(/[^a-z0-9_-]/g, "");

    // Generate the route path
    const routePath = "/" + safeName;

    // Use JSON.stringify to escape the import path safely
    const importPathLiteral = JSON.stringify("./" + relPath);

    // Build the route object as a string
    return `{ path: ${JSON.stringify(
      routePath
    )}, component: () => import(${importPathLiteral}) }`;
  });

  // Return the full array as a string
  return `[${routeObjects.join(",")}]`;
}
