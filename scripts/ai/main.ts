#!/usr/bin/env -S deno run --allow-all

import OpenAI from "npm:openai";
import stripAnsi from "npm:strip-ansi";
import type { ChatCompletionCreateParams } from "npm:openai/resources/chat/completions";
import path from "node:path";

const FILE_TAG = "__FILENAME__";
const START_TAG = "__FILE_CONTENT_START__";
const END_TAG = "__FILE_CONTENT_END__";
const ALL_GOOD_TAG = "DONE_ALL_TESTS_PASS_AND_COVERAGE_IS_GOOD";
const DEFAULT_PROMPT = `
You are given the full respository, results of the test run, and the coverage report.
Your task is first to fix the tests that are failing. DO NOT remove any existing implementations to make the tests pass.
If all tests are passing, pay attention to the coverage report and add new tests to add more 
coverage as needed. Code coverage should be executed 100%;
Provide only and only code updates. Do not provide any other text. You response can be multiple files.
DO NOT DELETE EXISTING IMPLEMENTATIONS. DO NOT DELETE EXISTING TEST. ONLY ADD TESTS.

Important: Add test files to the tests/ directory. Do not add tests in src/ files
`.trim();

// make sure coverage dir exists
Deno.mkdirSync("coverage", { recursive: true });

// Run cargo-llvm-cov in one shot
const { code, stdout, stderr } = await runCommand("cargo", [
  "llvm-cov",
  "test",
  "--no-fail-fast",
]);
// Serialize repo
const { stdout: repo } = await runCommand("yek", [
  "--tokens",
  "120k",
  "--ignore-patterns",
  "scripts/ai/*",
  "--ignore-patterns",
  ".github/**/*",
  "--ignore-patterns",
  "docs/**/*",
]);

// Get a summary of changes made so far
const summary = await getChangesSummary(repo);

const aiPrompt = Deno.env.get("AI_PROMPT") || DEFAULT_PROMPT;

// Ask AI to write code
const textToAi = [
  `Repository:`,
  repo,
  `Summary of changes we made so far:`,
  summary,
  `cargo llvm-cov exit code: ${code}`,
  `STDOUT:`,
  stdout,
  `STDERR:`,
  stderr,
  `Instructions:`,
  aiPrompt,
  `If all tests pass, and coverage is at 100%, return "${ALL_GOOD_TAG}".`,
  `When you return updated code, format your response as follows:`,
  FILE_TAG,
  `<relative/path/to/file>`,
  START_TAG,
  `<complete updated file content>`,
  END_TAG,
]
  .map((line) => stripAnsi(line))
  .join("\n")
  .trim();

console.log("AI prompt:", textToAi);

const aiContent = await callAi(textToAi);

// Otherwise, parse out any updated code
const updatedFiles = parseUpdatedFiles(aiContent);
if (!updatedFiles.length) {
  console.log("No updated files from AI");
  Deno.exit(1);
}

// Write new content
for (const f of updatedFiles) {
  await writeFileContent(f.filename, f.content);
}

// Format & fix code
await runCommand("cargo", ["fmt"], { showOutput: true });
await runCommand("cargo", ["clippy", "--fix", "--allow-dirty"], {
  showOutput: true,
});

// Review changes again
const allGood = await reviewChanges();
if (!allGood) {
  await runCommand("git", ["add", "reset", "--hard"]);
  console.log("Changes are not good. Reverted to previous state.");
}

// ------------------ Utils -------------------

function getOpenAiClient() {
  const provider = Deno.env.get("AI_PROVIDER") || "ollama";

  switch (provider) {
    case "ollama": {
      const apiKey = "";
      const openai = new OpenAI({
        apiKey,
        baseURL: "http://127.0.0.1:11434/v1",
      });
      return {
        openai,
        modelName: "mistral-small",
      };
    }
    case "fireworks": {
      const apiKey = Deno.env.get("FIREWORKS_AI_API_KEY");
      if (!apiKey) throw new Error("Missing FIREWORKS_AI_API_KEY env var.");

      const openai = new OpenAI({
        apiKey,
        baseURL: "https://api.fireworks.ai/inference/v1/",
      });
      const modelName = "accounts/fireworks/models/deepseek-r1";
      return { openai, modelName };
    }
    case "openai": {
      const apiKey = Deno.env.get("OPENAI_API_KEY");
      if (!apiKey) throw new Error("Missing OPENAI_API_KEY env var.");
      const openai = new OpenAI({ apiKey });
      const modelName = "o1-preview";
      return { openai, modelName };
    }
    case "deepseek": {
      const apiKey = Deno.env.get("DEEPSEEK_API_KEY");
      if (!apiKey) throw new Error("Missing DEEPSEEK_API_KEY env var.");
      const openai = new OpenAI({ apiKey });
      const modelName = "deepseek-reasoner";
      return { openai, modelName, baseURL: "https://api.deepseek.com/v1" };
    }
    default: {
      throw new Error(`Unknown AI provider: ${provider}`);
    }
  }
}

async function writeFileContent(filePath: string, content: string) {
  console.log("Writing updated content to:", filePath);
  // make sure directories exists first
  const dir = path.dirname(filePath);
  Deno.mkdirSync(dir, { recursive: true });
  await Deno.writeTextFile(filePath, content);
}

async function runCommand(
  cmd: string,
  args: string[] = [],
  options: { showOutput?: boolean } = {}
): Promise<{ code: number; stdout: string; stderr: string }> {
  console.log("$", [cmd, ...args].join(" "));
  const proc = new Deno.Command(cmd, {
    args,
    stdout: options.showOutput ? "inherit" : "piped",
    stderr: options.showOutput ? "inherit" : "piped",
    stdin: "inherit",
    env: {
      ...Deno.env.toObject(),
      RUSTFLAGS: "-Cinstrument-coverage",
      LLVM_PROFILE_FILE: "coverage/merged-%p-%m.profraw",
      CARGO_TERM_COLOR: "always",
      RUST_BACKTRACE: "1",
      FORCE_COLOR: "1",
    },
  });
  const output = await proc.output();
  return {
    code: output.code,
    stdout: options.showOutput ? "" : new TextDecoder().decode(output.stdout),
    stderr: options.showOutput ? "" : new TextDecoder().decode(output.stderr),
  };
}

async function callAi(
  text: string,
  { printOutput = true }: { printOutput?: boolean } = {}
) {
  const { openai, modelName } = getOpenAiClient();
  const encoder = new TextEncoder();
  const chatParams: ChatCompletionCreateParams = {
    model: modelName,
    stream: true,
    messages: [{ role: "user", content: text }],
  };

  const res = await openai.chat.completions.create(chatParams);
  const contents = [];
  for await (const chunk of res) {
    const content = chunk.choices[0].delta.content ?? "";
    contents.push(content);
    if (printOutput) {
      Deno.stdout.writeSync(encoder.encode(content));
    }
  }

  return contents.join("");
}

async function reviewChanges() {
  const { stdout: changes } = await runCommand("git", ["diff"]);
  if (!changes) return "No changes";
  console.log("Asking AI to review changes...");
  const review = await callAi(
    [
      `Changes:`,
      changes,
      `Instructions: Review the changes made so far. In bullet points. Short and concise.`,
      `If changes are removing implementations, or change source in a way to only pass the tests, reject them.`,
      `Important: If the changes are good, return "${ALL_GOOD_TAG}".`,
    ].join("\n")
  );
  return review.includes(ALL_GOOD_TAG);
}

async function getChangesSummary(repo: string) {
  const baseBranch = Deno.env.get("BASE_BRANCH") || "main";
  const { stdout: changes } = await runCommand("git", ["diff", baseBranch]);
  if (!changes) return "No changes";
  console.log("Asking AI to summarize changes...");
  const summaryAndThinking = await callAi(
    [
      `Repository:`,
      repo,
      `Changes:`,
      changes,
      `Instructions: Summerize the changes made so far in the repo. In bullet points. Short and concise.`,
    ].join("\n")
  );

  // remove the thinking part
  const summary = summaryAndThinking.replace(/<think>\n.*?\n<\/think>/s, "");
  return summary;
}

function parseUpdatedFiles(
  content: string
): Array<{ filename: string; content: string }> {
  // Quick and simple parse for multiple updates in one message

  const results: Array<{ filename: string; content: string }> = [];

  if (!content.includes(FILE_TAG)) return results;

  // Split on the special file tag
  const chunks = content.split(FILE_TAG);
  for (const chunk of chunks) {
    const trimmedChunk = chunk.trim();
    if (!trimmedChunk) continue;

    if (!trimmedChunk.includes(START_TAG)) continue;
    if (!trimmedChunk.includes(END_TAG)) continue;

    const lines = trimmedChunk.split("\n");
    const filename = lines[0].trim();
    const rest = lines.slice(1).join("\n").trim();

    const startIdx = rest.indexOf(START_TAG);
    const endIdx = rest.indexOf(END_TAG);
    if (startIdx < 0 || endIdx < 0) continue;

    let fileContent = rest
      .substring(startIdx + START_TAG.length, endIdx)
      .trim();
    // Remove any triple backticks
    if (fileContent.startsWith("```")) {
      fileContent = fileContent.replace(/^```[^\n]*\n?/, "");
    }
    if (fileContent.endsWith("```")) {
      fileContent = fileContent.replace(/```$/, "");
    }

    results.push({ filename, content: fileContent.trim() });
  }
  return results;
}
