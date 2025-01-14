#!/usr/bin/env node

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
const serialized = execSync('yek -s').toString().trim();

// Get test failures
debug('Getting test failures...');
let testFailures;
try {
    testFailures = execSync(testCommand, {
        stdio: ['pipe', 'pipe', 'pipe'],
        encoding: 'utf8'
    });
} catch (error) {
    testFailures = error.stdout + error.stderr;
}
testFailures = testFailures.split('\n')
    .filter(line => line.match(/test .* failed/))
    .join('\n')
    .trim();

debug('Asking deepseek...');

// Truncate and escape content if too large
const maxContentLength = 50000; // Adjust this value as needed
const truncateAndEscape = (str) => {
    if (str.length > maxContentLength) {
        str = str.slice(0, maxContentLength) + '... (truncated)';
    }
    return JSON.stringify(str);
};

const data = JSON.stringify({
    model: 'deepseek-chat',
    messages: [
        { role: 'system', content: 'You are a helpful assistant.' },
        { role: 'user', content: truncateAndEscape(serialized) },
        { role: 'user', content: truncateAndEscape(testFailures) }
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
