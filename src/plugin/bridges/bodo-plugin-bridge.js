#!/usr/bin/env node

const optsJson = process.env.BODO_OPTS;
if (!optsJson) {
  console.error("No BODO_OPTS provided");
  process.exit(1);
}

const pluginFile = process.env.BODO_PLUGIN_FILE;
if (!pluginFile) {
  console.error("No BODO_PLUGIN_FILE provided");
  process.exit(1);
}

// Parse JSON
let opts;
try {
  opts = JSON.parse(optsJson);
} catch (err) {
  console.error("Failed to parse BODO_OPTS JSON:", err);
  process.exit(1);
}

// Dynamically require the plugin
let plugin;
try {
  plugin = require(pluginFile);
} catch (err) {
  console.error(`Failed to load plugin file ${pluginFile}:`, err);
  process.exit(1);
}

// Get the hook name and function
const hookName = opts.hook;
if (!hookName) {
  console.error("No hook specified in opts");
  process.exit(1);
}

const hookFn = plugin[hookName];
if (typeof hookFn !== "function") {
  console.error(`Plugin does not export a '${hookName}' function`);
  process.exit(1);
}

// Call the function with our data
Promise.resolve(hookFn(opts))
  .then(() => {
    process.exit(0);
  })
  .catch(err => {
    console.error("Plugin error:", err);
    process.exit(1);
  }); 