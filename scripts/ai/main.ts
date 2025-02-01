#!/usr/bin/env -S deno run --allow-all

import OpenAI from "npm:openai";
import type {
  ChatCompletionCreateParams,
  ChatCompletionChunk,
} from "npm:openai/resources/chat/completions";
import * as path from "https://deno.land/std/path/mod.ts";

//
// ────────────────────────────────────────────────────────────────────────────────
//   CONSTANTS & PROMPTS
// ────────────────────────────────────────────────────────────────────────────────
//

// Use emojis
const FILENAME_TAG = "📁FILENAME";
const FILE_CONTENT_START_TAG = "🗄️FILE_CONTENT_START🗄️";
const FILE_CONTENT_END_TAG = "🔚FILE_CONTENT_END🔚";

const ADD_TESTS_PROMPT = `
Your task is to add or expand Rust tests to improve coverage in the specified file. Focus on edge cases, error handling, and untested paths. Follow these guidelines:

1. Do not remove or rename existing tests unless strictly necessary.
2. Write idiomatic Rust tests that thoroughly cover both typical and edge-case scenarios.
3. Include positive and negative tests (checking for expected errors, panics, or invalid inputs).
4. Preserve the existing code logic; only add or modify tests to increase coverage.
5. Use clear, descriptive function names for new tests.
6. Ensure the project compiles and all tests pass without warnings.
7. Avoid introducing dependencies not already in the project unless absolutely necessary.
8. Add inline comments where needed to explain complex or non-trivial test cases.
9. Aim for high coverage but do not compromise code readability or maintainability.
10. Focus only on the file you are given.
11. Always put tests in the tests/ directory.

When providing your response, wrap the complete file content as follows:

${FILENAME_TAG}
<the relative path to the file>
${FILE_CONTENT_START_TAG}
<the complete file content here>
${FILE_CONTENT_END_TAG}
`.trim();

const FIX_TESTS_PROMPT = `
We have failing tests. Your task is to fix them by adjusting the relevant code. 
Keep these rules in mind:

1. Do not remove or rename existing tests unless absolutely necessary.
2. Preserve existing functionality; fix only the issues causing test failures.
3. Write concise, clear changes. If an extensive refactor is required, explain why in the code.
4. Do not alter unrelated logic or style unless it directly affects test results.
5. Return the complete file content with your fixes.

When providing your response, wrap the complete file content as follows:

${FILENAME_TAG}
<the relative path to the file>
${FILE_CONTENT_START_TAG}
<the complete file content here>
${FILE_CONTENT_END_TAG}
`.trim();

const REVIEW_PROMPT = `
Review the proposed code changes. Check correctness, clarity, and performance.
If the changes are acceptable, respond with PASS. 
If the changes are insufficient or break something, respond with NOT GOOD anywhere in the text.

1. Focus on correctness of the logic, test coverage, and maintainability.
2. If the changes are inadequate or dangerous, you must include the phrase "NOT GOOD" in your response.
3. If they are acceptable, provide a brief overall verdict.
4. Include no extraneous commentary or code beyond that verdict.

Respond only with PASS or NOT GOOD. Nothing else.
`.trim();

const MODEL_NAME = "o1-preview";

//
// ────────────────────────────────────────────────────────────────────────────────
//   ENV & OPENAI CLIENT SETUP
// ────────────────────────────────────────────────────────────────────────────────
//

const apiKey = Deno.env.get("OPENAI_API_KEY");
if (!apiKey) {
  throw new Error("Missing OPENAI_API_KEY env var.");
}
const openai = new OpenAI({ apiKey });

//
// ────────────────────────────────────────────────────────────────────────────────
//   PATHS, FILE UTILITIES & PROCESS EXECUTION
// ────────────────────────────────────────────────────────────────────────────────
//

const __dirname = path.dirname(new URL(import.meta.url).pathname);
const repoRoot = path.resolve(__dirname, "../..");
const coverageDir = path.join(repoRoot, "coverage");

function existsSync(filePath: string): boolean {
  try {
    Deno.statSync(filePath);
    return true;
  } catch (err) {
    if (err instanceof Deno.errors.NotFound) return false;
    throw err;
  }
}

function readFileForCoverage(fileName: string): string {
  const filePath = path.join(repoRoot, fileName);
  return existsSync(filePath)
    ? Deno.readTextFileSync(filePath)
    : "File not found in repository.";
}

async function writeFileContent(filePath: string, content: string) {
  const fullPath = path.join(repoRoot, filePath);
  console.log("Writing file content to:", fullPath);
  try {
    await Deno.writeTextFile(fullPath, content);
    console.log("File written successfully.");
  } catch (err) {
    console.error("Failed to write file:", err);
    throw err;
  }
}

/**
 * Executes a command with given arguments and logs its progress.
 */
async function runCommand(
  cmd: string,
  args: string[] = [],
  options?: { showOutput?: boolean }
): Promise<{ code: number; stdout: Uint8Array; stderr: Uint8Array }> {
  console.log("$", `${cmd} ${args.join(" ")}`);
  const proc = new Deno.Command(cmd, {
    cwd: repoRoot,
    args,
    stdout: options?.showOutput ? "inherit" : "piped",
    stderr: options?.showOutput ? "inherit" : "piped",
    stdin: "inherit",
    env: {
      ...Deno.env.toObject(),
      FORCE_COLOR: "1",
      CARGO_TERM_COLOR: "always",
      RUST_BACKTRACE: "1",
      LLVM_PROFILE_FILE: path.join(coverageDir, "%p-%m.profraw"),
    },
  });
  const { code, stdout, stderr } = await proc.output();

  // Special handling for test and clippy commands
  if (code !== 0) {
    if (cmd === "cargo" && args[0] === "test") return { code, stdout, stderr };
    if (cmd === "cargo" && args[0] === "clippy") {
      console.log("Clippy failed but continuing...");
      return { code, stdout, stderr };
    }
    throw new Error(
      `Command "${cmd} ${args.join(" ")}" failed with code ${code}`
    );
  }
  return { code, stdout, stderr };
}

function serializeRepo(maxTokens: number): string {
  // Placeholder for the repository context.
  return `Repository context would be here (limited to ~${maxTokens} tokens)...`;
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   AI RESPONSE PARSING & HANDLING
// ────────────────────────────────────────────────────────────────────────────────
//

function parseAiResponse(
  content: string
): { filename: string; content: string } | null {
  if (
    !(
      content.includes(FILENAME_TAG) &&
      content.includes(FILE_CONTENT_START_TAG) &&
      content.includes(FILE_CONTENT_END_TAG)
    )
  )
    return null;

  const parts = content.split(FILENAME_TAG);
  if (!parts[1]) return null;
  const afterFileName = parts[1].trim();
  const newlineIndex = afterFileName.indexOf("\n");
  if (newlineIndex < 0) return null;

  const filename = afterFileName.slice(0, newlineIndex).trim();
  const rest = afterFileName.slice(newlineIndex).trim();
  const startIndex = rest.indexOf(FILE_CONTENT_START_TAG);
  const endIndex = rest.indexOf(FILE_CONTENT_END_TAG);
  if (startIndex < 0 || endIndex < 0) return null;

  let fileContent = rest
    .slice(startIndex + FILE_CONTENT_START_TAG.length, endIndex)
    .trim();

  // Strip code fences if present.
  if (fileContent.startsWith("```")) {
    const fenceEnd = fileContent.indexOf("\n");
    if (fenceEnd !== -1) fileContent = fileContent.slice(fenceEnd + 1);
  }
  if (fileContent.endsWith("```"))
    fileContent = fileContent.slice(0, -3).trim();

  return { filename, content: fileContent };
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   COVERAGE SETUP & COLLECTION
// ────────────────────────────────────────────────────────────────────────────────
//

interface GrcovOptions {
  outputPath: string;
  outputFormat: "lcov" | "covdir";
}

async function runGrcov(options: GrcovOptions): Promise<void> {
  await runCommand("grcov", [
    ".",
    "--binary-path",
    "./target/debug/deps",
    "-s",
    ".",
    "-t",
    options.outputFormat,
    "--branch",
    "--ignore-not-existing",
    "--ignore",
    "target/*",
    "--ignore",
    "**/tests/*",
    "--ignore",
    "**/deps/*",
    "--ignore",
    "**/.cargo/*",
    "--filter",
    "covered",
    "--output-path",
    options.outputPath,
  ]);
}

async function runTests(
  jsonFormat = false,
  showOutput = false
): Promise<{
  code: number;
  stdout: Uint8Array;
  stderr: Uint8Array;
}> {
  const args = ["test", "--tests"];
  if (jsonFormat) {
    args.push("--message-format=json", "--no-fail-fast");
  }
  return runCommand("cargo", args, { showOutput });
}

async function collectCoverage(_: number): Promise<void> {
  console.log("Running tests to generate coverage profiles...");
  try {
    await runTests();
  } catch {
    console.log("Tests failed - generating partial coverage data...");
  }
  console.log("Converting coverage data using grcov...");

  await runGrcov({
    outputPath: path.join(coverageDir, "lcov.info"),
    outputFormat: "lcov",
  });

  await runGrcov({
    outputPath: path.join(coverageDir, "coverage.json"),
    outputFormat: "covdir",
  });

  console.log("Coverage data collected.");
}

function parseCoverageData(threshold: number): {
  coverageData: CoverageData;
  belowThresholdFiles: string[];
} {
  const coveragePath = path.join(coverageDir, "coverage.json");
  if (!existsSync(coveragePath)) {
    throw new Error("No coverage.json produced. Check grcov usage.");
  }
  const coverageStr = Deno.readTextFileSync(coveragePath);
  const coverageData = JSON.parse(coverageStr);
  const belowThresholdFiles: string[] = [];

  if (coverageData.children?.src?.children) {
    for (const [file, info] of Object.entries(
      coverageData.children.src.children as Record<
        string,
        { coveragePercent: number }
      >
    )) {
      if (info.coveragePercent < threshold) belowThresholdFiles.push(file);
    }
  } else if (coverageData.children) {
    for (const [file, info] of Object.entries(
      coverageData.children as Record<string, { coveragePercent: number }>
    )) {
      if (file.startsWith("src/") && info.coveragePercent < threshold) {
        belowThresholdFiles.push(file);
      }
    }
  } else if (typeof coverageData.coveragePercent === "number") {
    console.log(`Overall coverage: ${coverageData.coveragePercent}%`);
    if (coverageData.coveragePercent < threshold) {
      console.log("Coverage below threshold, analyzing individual files...");
      belowThresholdFiles.push(
        ...Object.keys(coverageData.files || {})
          .filter((f) => f.startsWith("src/"))
          .filter((f) => {
            const fileCov = coverageData.files[f].coveragePercent;
            return typeof fileCov === "number" && fileCov < threshold;
          })
      );
    }
  }
  console.log("Below-threshold files:", belowThresholdFiles);
  return { coverageData, belowThresholdFiles };
}

interface CoverageData {
  children?: {
    src?: {
      children?: Record<string, { coveragePercent: number }>;
    };
  };
}

async function reCollectCoverage(): Promise<CoverageData> {
  console.log("Re-running tests for updated coverage...");
  try {
    await runTests();
  } catch {
    console.log("Tests failed during re-collection - continuing...");
  }

  const jsonPath = path.join(coverageDir, "coverage.json");
  await runGrcov({
    outputPath: jsonPath,
    outputFormat: "covdir",
  });

  if (!existsSync(jsonPath)) {
    throw new Error(
      "No coverage.json produced on re-collect. Check grcov usage."
    );
  }

  return processCoverageData(jsonPath);
}

function processCoverageData(jsonPath: string): CoverageData {
  const coverageStr = Deno.readTextFileSync(jsonPath);
  const coverageData = JSON.parse(coverageStr);

  if (coverageData.children?.src?.children) {
    return coverageData;
  }

  const normalizedData = {
    children: {
      src: {
        children: Object.fromEntries(
          Object.entries(coverageData.children || coverageData.files || {})
            .filter(([file]) => file.startsWith("src/"))
            .map(([k, v]) => [
              k,
              { coveragePercent: (v as any).coveragePercent as number },
            ])
        ),
      },
    },
  };

  return normalizedData;
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   TEST FAILURE HANDLING & AI PROMPTS FOR TEST FIXES
// ────────────────────────────────────────────────────────────────────────────────
//

interface TestFailure {
  name: string;
  path: string;
  message: string;
}

function parseTestFailures(output: string, errors: string): TestFailure[] {
  return output
    .split("\n")
    .filter(Boolean)
    .map((line) => {
      try {
        const msg = JSON.parse(line);
        if (msg?.type === "test" && msg?.event === "failed") {
          return {
            name: msg.name || "",
            path:
              msg.target?.src_path ||
              (msg.name && msg.name.split("::")[0]) ||
              "",
            message: msg.stdout || msg.stderr || errors || "test failed",
          };
        }
        return null;
      } catch {
        return null;
      }
    })
    .filter((test): test is TestFailure => test !== null);
}

// Add these types and functions after the existing imports
interface AiRequestParams {
  fileName: string;
  currentContent: string;
  repoSnapshot: string;
  prompt: string;
  stream?: boolean;
}

interface StreamChunk {
  choices: Array<{
    delta: {
      content?: string;
    };
  }>;
}

async function makeAiRequest(params: AiRequestParams): Promise<string> {
  const chatParams: ChatCompletionCreateParams = {
    model: MODEL_NAME,
    stream: params.stream ?? false,
    messages: [
      {
        role: "user",
        content: [
          "==== Repo ====",
          params.repoSnapshot,
          "==== File ====",
          params.fileName,
          "==== Current content ====",
          params.currentContent,
          "==== Instructions ====",
          params.prompt,
        ].join("\n\n"),
      },
    ],
  };

  if (params.stream) {
    const streamResponse = await openai.chat.completions.create(chatParams);
    let completeResponse = "";

    try {
      const stream = streamResponse as AsyncIterable<ChatCompletionChunk>;
      for await (const part of stream) {
        const content = part.choices?.[0]?.delta?.content ?? "";
        process.stdout.write(content);
        completeResponse += content;
      }
    } catch (err) {
      console.error("Error processing stream:", err);
      return "";
    }

    console.log("\nFinal AI response:");
    console.log(completeResponse);
    return completeResponse;
  } else {
    const completion = await openai.chat.completions.create(chatParams);
    return completion.choices[0].message?.content ?? "";
  }
}

// Replace the existing AI interaction functions with:
async function generateTestsForFile(
  fileName: string,
  repoSnapshot: string
): Promise<void> {
  const currentContent = readFileForCoverage(fileName);
  const aiResponse = await makeAiRequest({
    fileName,
    currentContent,
    repoSnapshot,
    prompt: ADD_TESTS_PROMPT,
    stream: true,
  });

  const parsed = parseAiResponse(aiResponse);
  if (parsed && parsed.content) {
    console.log("AI provided new file content; updating file...");
    await writeFileContent(parsed.filename, parsed.content);
  } else {
    console.log("No valid file content returned from AI for test generation.");
  }
}

async function fixTestsForFile(
  fileName: string,
  errorOutput: string,
  repoSnapshot: string
): Promise<void> {
  const currentContent = readFileForCoverage(fileName);
  const aiResponse = await makeAiRequest({
    fileName,
    currentContent,
    repoSnapshot,
    prompt: FIX_TESTS_PROMPT,
  });

  const parsed = parseAiResponse(aiResponse);
  if (parsed && parsed.content) {
    console.log(`AI fixed tests for ${fileName}. Writing update...`);
    await writeFileContent(parsed.filename, parsed.content);
  } else {
    console.log("AI did not return valid file content for test fixes.");
  }
}

async function aiCodeReview(
  fileName: string,
  repoSnapshot: string
): Promise<string> {
  const currentContent = readFileForCoverage(fileName);
  return makeAiRequest({
    fileName,
    currentContent,
    repoSnapshot,
    prompt: REVIEW_PROMPT,
  });
}

// Add back the fixTests function
async function fixTests(
  failedTestsJson: string,
  repoSnapshot: string
): Promise<boolean> {
  try {
    // Run normal tests for detailed output.
    const normalTest = await runTests(false, true);
    console.log(
      "Test output (if not already visible):",
      new TextDecoder().decode(normalTest.stderr)
    );

    // Run tests with JSON output.
    const { code, stdout, stderr } = await runTests(true);
    if (code === 0) return true;

    const output = new TextDecoder().decode(stdout);
    const errors = new TextDecoder().decode(stderr);
    const failedTests = parseTestFailures(output, errors);

    if (failedTests.length > 0) {
      console.log("Detected failed tests:", failedTests);
      // Group failures by file.
      const testsByFile = failedTests.reduce<Record<string, TestFailure[]>>(
        (acc, test) => {
          if (test.path) {
            (acc[test.path] = acc[test.path] || []).push(test);
          }
          return acc;
        },
        {}
      );

      for (const [filePath, tests] of Object.entries(testsByFile)) {
        const errorOutput = tests
          .map((t) => `${t.name}: ${t.message}`)
          .join("\n");
        await fixTestsForFile(filePath, errorOutput, repoSnapshot);
      }

      // Lint and format.
      await runCommand("cargo", ["fmt"]);
      try {
        await runCommand("cargo", ["clippy", "--fix", "--allow-dirty"]);
      } catch (e) {
        console.log("Clippy reported issues:", e);
      }
    }
  } catch (err: any) {
    console.log(`Error during test fixing: ${err.message || err}`);
    return false;
  }
  return false;
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   MAIN PIPELINE & FINALIZATION
// ────────────────────────────────────────────────────────────────────────────────
//

async function improveCoverageForFile(
  filePath: string,
  threshold: number,
  maxIterations = 10
): Promise<void> {
  const fileName = filePath.startsWith("src/") ? filePath : `src/${filePath}`;
  const repoSnapshot = serializeRepo(5000);
  let iteration = 0;
  let coverageReached = false;
  let testsPass = false;

  while (iteration < maxIterations && !coverageReached) {
    iteration++;
    console.log(`\n=== Iteration #${iteration} for ${fileName} ===`);

    await generateTestsForFile(fileName, repoSnapshot);
    await runCommand("cargo", ["fmt"]);
    await runCommand("cargo", ["clippy", "--fix", "--allow-dirty"]);

    testsPass = await fixTests(filePath, repoSnapshot);

    try {
      const diff = await runCommand("git", ["diff", "--name-only"]);
      if (diff.code === 0) {
        await runCommand("git", ["add", "."]);
        await runCommand("git", [
          "commit",
          "-m",
          `Improve coverage for ${fileName} (iteration ${iteration})`,
        ]);
      } else {
        console.log("No changes detected to commit; skipping commit step.");
        continue;
      }
    } catch {
      console.log("Git commit encountered an issue; proceeding...");
    }

    const reviewVerdict = await aiCodeReview(fileName, repoSnapshot);
    const coverageData = await reCollectCoverage();
    let coveragePct = 0;
    if (coverageData.children?.src?.children) {
      const fileCoverage = Object.entries(
        coverageData.children.src.children
      ).find(([key]) => key === fileName);
      if (fileCoverage) coveragePct = fileCoverage[1].coveragePercent;
    }

    const approved = reviewVerdict.toLowerCase().includes("pass");
    if (coveragePct >= threshold && testsPass && approved) {
      console.log(
        `${fileName} now at ${coveragePct}% coverage with tests passing and AI approval. Done.`
      );
      coverageReached = true;
    } else {
      console.log(
        `Iteration ${iteration}: Coverage ${coveragePct}%, testsPass=${testsPass}, AI verdict=${reviewVerdict}. Reverting changes...`
      );
      await runCommand("git", ["reset", "--hard"]);
    }
  }
}

async function finalFormatting(): Promise<void> {
  console.log("Performing final formatting & linting...");
  try {
    await runCommand("cargo", ["fmt"]);
  } catch {
    console.log("cargo fmt failed or not available.");
  }
  try {
    await runCommand("cargo", ["clippy", "--fix", "--allow-dirty"]);
  } catch {
    console.log("cargo clippy failed or not available.");
  }
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   ENTRY POINT
// ────────────────────────────────────────────────────────────────────────────────
//

try {
  let testPass = false;
  const MAX_RETRIES = 3;
  let retryCount = 0;

  while (!testPass && retryCount < MAX_RETRIES) {
    retryCount++;
    console.log(`\nAttempt ${retryCount} to fix failing tests...`);

    const normalTest = await runTests(false, true);
    console.log(
      "Test output (if not already visible):",
      new TextDecoder().decode(normalTest.stderr)
    );

    const { code, stdout } = await runTests(true);
    if (code === 0) {
      testPass = true;
      break;
    } else {
      const output = new TextDecoder().decode(stdout);
      const failedTests = parseTestFailures(output, "");
      if (failedTests.length > 0) {
        console.log(
          `Found ${failedTests.length} failed tests on attempt ${retryCount}`
        );
        const repoSnapshot = serializeRepo(12000);
        try {
          testPass = await fixTests(JSON.stringify(failedTests), repoSnapshot);
        } catch (err) {
          console.log("Failed to fix tests:", err);
        }
      }
    }

    if (!testPass) {
      console.log(
        `Test fix attempt ${retryCount} failed. ${
          retryCount < MAX_RETRIES ? "Retrying..." : "Giving up."
        }`
      );
    }
  }

  if (!testPass) {
    console.log(
      "Failed to fix tests after maximum retries. Proceeding with coverage collection anyway..."
    );
  }

  await collectCoverage(90);
  const { belowThresholdFiles } = parseCoverageData(100);

  if (belowThresholdFiles.length > 0) {
    for (const file of belowThresholdFiles) {
      await improveCoverageForFile(file, 100);
    }
  } else {
    console.log("All files meet the coverage threshold.");
  }

  await finalFormatting();
  console.log("Pipeline complete.");
  Deno.exit(0);
} catch (err) {
  console.error("Pipeline failed:", err);
  Deno.exit(1);
}
