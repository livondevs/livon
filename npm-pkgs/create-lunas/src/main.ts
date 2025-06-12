#!/usr/bin/env node
import degit from "degit";
import { existsSync } from "fs";
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

    await renameFiles(targetDir);

    console.log("✅ Project initialized.");
    console.log("👉 Next steps:");
    console.log(`   cd ${targetDir}`);
    console.log("   npm install");
    console.log("   npm run dev");
  } catch (err) {
    console.error("❌ Failed to initialize project:", err);
    process.exit(1);
  }
})();

async function renameFiles(projectName: string) {
  const filesToRename = ["README.md", "index.html", "package.json"];
  for (const file of filesToRename) {
    const filePath = path.join(projectName, file);
    let content: string;
    try {
      content = await readFile(filePath, "utf8");
    } catch (error: unknown) {
      if (
        typeof error === "object" &&
        error !== null &&
        (error as { code?: string }).code === "ENOENT"
      )
        continue; // If the file does not exist, ignore it
      throw error;
    }
    const updatedContent = content.replace(/__PROJECT_NAME__/g, projectName);
    await writeFile(filePath, updatedContent, "utf8");
  }
}
