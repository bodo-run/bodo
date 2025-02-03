#!/bin/bash
# Script to split an input file containing sections in the format:
#   >>>> relative/path/to/file.txt
#   file content...
#
# Usage: ./write_files.sh input_file.txt

set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <input_file>"
    exit 1
fi

input_file="$1"
current_file=""
current_content=""

while IFS= read -r line || [ -n "$line" ]; do
    # Check if the line starts with ">>>> " indicating a new file section
    if [[ "$line" =~ ^\>\>\>\>[[:space:]]+(.*) ]]; then
        # If there's a current file, write its collected content using printf to avoid extra newline
        if [[ -n "$current_file" ]]; then
            mkdir -p "$(dirname "$current_file")"
            printf '%s' "$current_content" >"$current_file"
        fi
        # Set the new file name (relative path)
        current_file="${BASH_REMATCH[1]}"
        current_content=""
    else
        # Append the current line to the file content preserving newlines
        if [[ -z "$current_content" ]]; then
            current_content="$line"
        else
            current_content="$current_content"$'\n'"$line"
        fi
    fi
done <"$input_file"

# Write the last collected file content if any, again using printf
if [[ -n "$current_file" ]]; then
    mkdir -p "$(dirname "$current_file")"
    printf '%s' "$current_content" >"$current_file"
fi

echo "Files created successfully."
