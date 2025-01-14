#!/usr/bin/env node

/**
 * @fileoverview
 * This script asks DeepSeek to help with debugging a Rust project.
 * It serializes the project, gets test failures, and sends the content to DeepSeek.
 * The response is then printed to the console.
 */

const { execSync } = require('child_process');
const https = require('https');

const token = process.env.DEEPSEEK_API_KEY;
const testCommand = process.argv[2] || 'cargo test';

const debugEnabled = process.argv.includes('--debug');

function debug(message) {
    if (debugEnabled) {
        console.log(`[DEBUG] ${message}`);
    }
}

if (!token) {
    console.error('DEEPSEEK_API_KEY is not set');
    process.exit(1);
}

debug('Serializing...');
// DeepSeek maximum context length is 128K tokens. we leave some room for the test failures.
const maxSize = 128000 - 10000; // 10000 tokens for test failures

// Convert execSync to Promise-based execution
function execCommand(command, options = {}) {
    return new Promise((resolve, reject) => {
        try {
            const result = execSync(command, { ...options, encoding: 'utf8' });
            resolve(result);
        } catch (error) {
            if (options.returnError) {
                resolve(error.stdout + error.stderr);
            } else {
                reject(error);
            }
        }
    });
}

// Run serialization and testing in parallel
debug('Starting serialization and testing in parallel...');
Promise.all([
    execCommand(`yek --stream --max-size ${maxSize} --tokens`),
    execCommand(testCommand, { stdio: ['pipe', 'pipe', 'pipe'], returnError: true })
])
.then(([serialized, testOutput]) => {
    const testFailures = testOutput.split('\n')
        .filter(line => line.match(/test .* failed/))
        .join('\n')
        .trim();

    debug('Asking deepseek...');
    
    // Truncate and escape content if too large
    const maxContentLength = 30000; // Adjust this value as needed
    const truncateAndEscape = (str) => {
        if (str.length > maxContentLength) {
            str = str.slice(0, maxContentLength) + '... (truncated)';
        }
        return JSON.stringify(str);
    };

    const content = truncateAndEscape(`Repo:\n\n${serialized}\n\nTest failures:\n\n${testFailures}`);
    const systemPrompt = 
    `You are a an expert Rust developer. You are familiar with the Rust language and its ecosystem.
    You use modern Rust and the latest Rust features.
    You are given a Rust project and some test failures.
    Your task is to help the user debug the test failures.
    You should provide a detailed explanation of the test failures and how to fix them.
    Keep your response concise and to the point.
    Write **high-quality** and **clean** code.
    `;

    const data = JSON.stringify({
        model: 'deepseek-chat',
        messages: [
            { role: 'system', content: systemPrompt },
            { role: 'user', content }
        ],
        stream: false
    });

    const options = {
        hostname: 'api.deepseek.com',
        path: '/chat/completions',
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`,
            'Content-Length': Buffer.byteLength(data)
        }
    };

    const req = https.request(options, (res) => {
        let responseData = '';

        res.on('data', (chunk) => {
            responseData += chunk;
        });

        res.on('end', () => {
            try {
                const jsonResponse = JSON.parse(responseData);
                const content = jsonResponse?.choices?.[0]?.message?.content;
                if (content) {
                    console.log(content);
                } else {
                    console.error('No content found in the response');
                }
            } catch (error) {
                console.error('Failed to parse response:', responseData);
                process.exit(1);
            }
        });
    });

    req.on('error', (error) => {
        console.error('Error:', error);
        process.exit(1);
    });

    req.write(data);
    req.end();
})
.catch(error => {
    console.error('Error:', error);
    process.exit(1);
});
