#!/usr/bin/env ruby
require 'json'

# Get environment variables
opts_json = ENV['BODO_OPTS']
plugin_file = ENV['BODO_PLUGIN_FILE']

if opts_json.nil? || plugin_file.nil?
  STDERR.puts "Missing required environment variables"
  exit 1
end

# Parse options
begin
  opts = JSON.parse(opts_json)
rescue JSON::ParserError => e
  STDERR.puts "Failed to parse BODO_OPTS JSON: #{e}"
  exit 1
end

# Get hook name
hook_name = opts['hook']
if hook_name.nil?
  STDERR.puts "No hook specified in opts"
  exit 1
end

# Load plugin
begin
  require_relative plugin_file
rescue LoadError => e
  STDERR.puts "Failed to load plugin #{plugin_file}: #{e}"
  exit 1
end

# Get plugin module
plugin_module = Object.const_get('BodoPlugin')
unless plugin_module.respond_to?(hook_name)
  STDERR.puts "Plugin does not export a '#{hook_name}' function"
  exit 1
end

# Execute hook
begin
  result = plugin_module.send(hook_name, opts)
  puts result.to_json if result
  exit 0
rescue StandardError => e
  STDERR.puts "Plugin error: #{e}"
  exit 1
end 