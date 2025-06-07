import { glob } from "glob";
import path from "path";
import type { Plugin } from "vite";

/**
 * Vite plugin for automatic routing based on .lun files in the specified pages directory.
 */
/**
 * Vite plugin for automatic route generation based on the file structure within a specified pages directory.
 * 
 * This plugin scans the provided `pagesDir` directory, generates route definitions,
 * and exposes them as a virtual module (`virtual:generated-routes`) for use in the application.
 * 
 * @param options - Plugin options.
 * @param options.pagesDir - Relative path to the directory containing page components.
 * @returns A Vite plugin object that handles automatic route generation.
 */
export function lunasAutoRoutingPlugin(options: { pagesDir: string }): Plugin {
  let projectRoot: string;

  return {
    name: "vite-plugin-lunas-auto-routing",
    enforce: "pre",

    /**
     * Store the resolved project root directory.
     */
    configResolved(config) {
      projectRoot = config.root;
    },

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
     * Load and generate the virtual module for routes.
     */
    load(id) {
      if (id === "virtual:generated-routes") {
        const absPagesDir = path.resolve(projectRoot, options.pagesDir);
        if (!absPagesDir.startsWith(projectRoot)) {
          throw new Error(
            `Security error: pagesDir must be inside the project root: ${absPagesDir}`
          );
        }

        const routesArray = generateRoutes(absPagesDir, projectRoot);
        return `export const routes = ${routesArray};`;
      }
      return null;
    },
  };
}
/**
 * Scan .lun files under absDir and produce a JSON array string of route objects.
 * @param absDir Absolute path to the pages directory
 */
function generateRoutes(absDir: string, projectRoot: string): string {
  const pattern = path.posix.join(absDir.replace(/\\/g, "/"), "**/*.lun");
  const files = glob.sync(pattern, { nodir: true });

  const routes = files.map((file) => {
    // Create a relative path from the project root for imports
    const relPath = path.relative(projectRoot, file).split(path.sep).join("/");
    const baseName = path.basename(file, ".lun");

    // Determine route path
    const routeName = baseName.toLowerCase() === "index" ? "" : baseName;
    const safeName = routeName.toLowerCase().replace(/[^a-z0-9_-]/g, "");
    const routePath = `/${safeName}`;

    // Dynamic import for the component
    const importLiteral = JSON.stringify(`./${relPath}`);
    return `{ path: ${JSON.stringify(
      routePath
    )}, component: () => import(${importLiteral}) }`;
  });

  return `[${routes.join(",")}]`;
}
