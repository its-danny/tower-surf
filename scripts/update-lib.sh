#!/bin/sh

source="README.md"
destination="src/lib.rs"
temp=$(mktemp)

# Extract the "Overview" section and everything following it from the README.
block=$(awk '/^## 🏄‍♂️ Overview$/,0' "$source")

# Remove the NOTE block, as it's not supported by docs.rs.
block=$(echo "$block" | awk '!/^> \[!NOTE\]/ && !/^> /')

# Make sure we don't try to run doc code.
block=$(echo "$block" | sd '^```rust$' '```rust, no_run')

# Remove all existing doc comments.
grep -v '^//!' "$destination" >"$temp"
mv "$temp" "$destination"

# Prepend doc comment string to each line of the extracted block.
block=$(echo "$block" | sd '^' '//! ')

# Remove the last line from the block because it's just an empty
# doc-comment and I'm not sure why.
block=$(echo "$block" | head -n -1)

# Prepend the new doc comments.
printf "%s\n%s" "$block" "$(cat "$destination")" >"$destination"
