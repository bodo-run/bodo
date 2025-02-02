#!/usr/bin/env -S deno run --allow-all

import OpenAI from "npm:openai";
import stripAnsi from "npm:strip-ansi";
import type { ChatCompletionCreateParams } from "npm:openai/resources/chat/completions";

const AI_PROMPT = `
You are given the full respository, results of the test run, and the coverage report.
Your task is first to fix the tests that are failing. DO NOT remove any existing implementations to make the tests pass.
If all tests are passing, pay attention to the coverage report and add new tests to add more 
coverage as needed. Code coverage should be 100%;
If all tests pass, and coverage is at 100%, return "DONE_ALL_TESTS_PASS_AND_COVERAGE_GOOD".
Provide only and only code updates. Do not provide any other text. You response can be multiple files.

When you return updated code, format your response as follows:

__FILENAME__
<relative/path/to/file>
__FILE_CONTENT_START__
<complete updated file content>
__FILE_CONTENT_END__
`.trim();

function getOpenAiClient() {
  const provider = Deno.env.get("AI_PROVIDER");
  if (provider === "fireworks") {
    const apiKey = Deno.env.get("FIREWORKS_AI_API_KEY");
    if (!apiKey) throw new Error("Missing FIREWORKS_AI_API_KEY env var.");

    const openai = new OpenAI({
      apiKey,
      baseURL: "https://api.fireworks.ai/inference/v1/",
    });
    const modelName = "accounts/fireworks/models/deepseek-r1";
    return { openai, modelName };
  } else if (provider === "openai") {
    const apiKey = Deno.env.get("OPENAI_API_KEY");
    if (!apiKey) throw new Error("Missing OPENAI_API_KEY env var.");
    const openai = new OpenAI({ apiKey });
    const modelName = "o1-preview";
    return { openai, modelName };
  }
  throw new Error(`Unknown AI provider: ${provider}`);
}

function existsSync(filePath: string): boolean {
  try {
    Deno.statSync(filePath);
    return true;
  } catch {
    return false;
  }
}

async function writeFileContent(filePath: string, content: string) {
  console.log("Writing updated content to:", filePath);
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

async function main() {
  const { openai, modelName } = getOpenAiClient();
  const MAX_ATTEMPTS = Number(Deno.env.get("MAX_ATTEMPTS")) || 5;

  // make sure coverage dir exists
  Deno.mkdirSync("coverage", { recursive: true });

  for (let i = 1; i <= MAX_ATTEMPTS; i++) {
    console.log(`\n=== Iteration #${i} ===`);
    // Run cargo-llvm-cov in one shot
    const { code, stdout, stderr } = await runCommand("cargo", [
      "llvm-cov",
      "--json",
      "--output-path",
      "coverage/coverage.json",
    ]);

    const coveragePath = "coverage/coverage.json";
    let coverageJson = "";
    if (existsSync(coveragePath)) {
      coverageJson = await Deno.readTextFile(coveragePath);
    }

    // If everything passed and coverage is presumably fine
    // cargo-llvm-cov will exit 0. We can check coverage JSON if we want.
    if (code === 0) {
      console.log("cargo llvm-cov completed with exit code 0.");
      // You could parse coverageJson to verify coverage thresholds if needed.
      // For now, let's see if AI says it's "DONE_ALL_TESTS_PASS_AND_COVERAGE_GOOD"
    }

    // Send AI the entire cargo llvm-cov output + coverage JSON
    const textToAi = [
      `cargo llvm-cov exit code: ${code}`,
      `STDOUT:`,
      stdout,
      `STDERR:`,
      stderr,
      `Coverage JSON:`,
      coverageJson,
      `Instructions:`,
      AI_PROMPT,
    ]
      .map((line) => stripAnsi(line))
      .join("\n")
      .trim();

    // Append to attempts.txt
    await Deno.writeTextFile(
      `attempts.txt`,
      `Attempt ${i} Request:\n\n${textToAi}\n\n`,
      {
        append: true,
      }
    );

    const chatParams: ChatCompletionCreateParams = {
      model: modelName,
      messages: [{ role: "user", content: textToAi }],
    };
    console.log("Sending request to AI...");
    const response = await openai.chat.completions.create(chatParams);
    const aiContent = response.choices?.[0]?.message?.content ?? "";

    // Append to attempts.txt
    await Deno.writeTextFile(
      `attempts.txt`,
      `Attempt ${i} Response:\n\n${aiContent}\n\n`,
      {
        append: true,
      }
    );

    Deno.env.set("LAST_ATTEMPT", i.toString());

    // If the AI says coverage is good, we're done
    if (aiContent.includes("DONE_ALL_TESTS_PASS_AND_COVERAGE_GOOD")) {
      console.log("All tests pass and coverage is good. Done.");
      Deno.exit(0);
    }

    // Otherwise, parse out any updated code
    const updatedFiles = parseUpdatedFiles(aiContent);
    if (!updatedFiles.length) {
      console.log("No updated files from AI. Exiting...");
      Deno.exit(0);
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
  }

  console.log("Reached maximum attempts without success.");
}

function parseUpdatedFiles(
  content: string
): Array<{ filename: string; content: string }> {
  // Quick and simple parse for multiple updates in one message
  const FILE_TAG = "__FILENAME__";
  const START_TAG = "__FILE_CONTENT_START__";
  const END_TAG = "__FILE_CONTENT_END__";

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

await main();
