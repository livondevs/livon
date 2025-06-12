#!/usr/bin/env node
import degit from "degit";
import { existsSync, access } from "fs";
import { mkdir, writeFile, readFile } from "fs/promises";
import { prompt } from "enquirer";
import path from "path";

(async () => {
  try {
    // Template repository
    const repo = "lunasrun/lunas-template";

    // Prompt for project name
    const { project } = await prompt<{ project: string }>({
      type: "input",
      name: "project",
      message: "Project name:",
      initial: "your-lunas-project-name",
    });
    const targetDir = project.trim();

    // Check if directory already exists
    if (existsSync(targetDir)) {
      console.error(`❌ Directory "${targetDir}" already exists.`);
      process.exit(1);
    }

    console.log(`📦 Initializing project in "${targetDir}"...`);
    const emitter = degit(repo, { cache: false, force: true, verbose: true });
    await mkdir(targetDir, { recursive: true });
    await emitter.clone(targetDir);

    await renameFiles(targetDir, project);

    console.log("✅ Project initialized.");
    console.log("👉 Next steps:");
    console.log(`   cd ${targetDir}`);
    console.log("   npm install");
    console.log("   npm run dev");
    // process.exit(1);
  } catch (err) {
    console.error("❌ Failed to initialize project:", err);
    process.exit(1);
  }
})();

async function renameFiles(dir: string, projectName: string) {
  try {
    const filesToRename = ["README.md", "index.html", "package.json"];
    for (const file of filesToRename) {
      const filePath = path.join(dir, file);
      let content: string;
      try {
        content = await readFile(filePath, "utf8");
      } catch (err: any) {
        if (err.code === "ENOENT") continue; // ファイルがなければ無視
        throw err;
      }
      const updatedContent = content.replace(/__PROJECT_NAME__/g, projectName);
      await writeFile(filePath, updatedContent, "utf8");
    }
  } catch (err) {
    console.error(`❌ Error renaming files in ${dir}:`, err);
    process.exit(1);
  }
}
