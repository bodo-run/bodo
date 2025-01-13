#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

function loadPlugin(pluginPath) {
    const absPath = path.resolve(pluginPath);
    console.error("[DEBUG] Loading plugin from:", absPath);
    try {
        const plugin = require(absPath);
        console.error("[DEBUG] Plugin loaded successfully");
        console.error("[DEBUG] Plugin exports:", Object.keys(plugin));
        return plugin;
    } catch (err) {
        console.error("[DEBUG] Failed to load plugin:", err);
        console.error("[DEBUG] Current directory:", process.cwd());
        console.error("[DEBUG] Plugin file exists:", fs.existsSync(absPath));
        return null;
    }
}

const pluginFile = process.env.BODO_PLUGIN_FILE;
const opts = process.env.BODO_OPTS ? JSON.parse(process.env.BODO_OPTS) : {};

console.error("[DEBUG] Plugin file:", pluginFile);
console.error("[DEBUG] Options:", JSON.stringify(opts));

if (!pluginFile) {
    console.error("BODO_PLUGIN_FILE environment variable not set");
    process.exit(1);
}

const plugin = loadPlugin(pluginFile);
if (!plugin) {
    console.error("Failed to load plugin");
    process.exit(1);
}

const hookName = opts.hook;
if (!hookName) {
    console.error("No hook specified in opts");
    process.exit(1);
}

if (typeof plugin[hookName] !== 'function') {
    console.error("[DEBUG] Available hooks:", Object.keys(plugin));
    console.error(`Plugin does not export a '${hookName}' function`);
    process.exit(1);
}

const result = plugin[hookName](opts);
if (result !== undefined) {
    console.log(JSON.stringify(result));
} 