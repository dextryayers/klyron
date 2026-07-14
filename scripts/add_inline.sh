#!/bin/bash
# Script to add #[inline] annotations to hot-path functions across all klyron crates
# Scans for getter methods, small utility functions, builder patterns, and conversion functions

set -euo pipefail

CRATES_DIR="crates"

# Patterns that indicate functions needing #[inline]:
# 1. Simple getters: pub fn field(&self) -> &Type { self.field }
# 2. Builder methods: pub fn field(mut self, ...) -> Self
# 3. Conversion functions: pub fn to_*, pub fn from_*, pub fn into_*
# 4. Small utility functions (< 5 lines)
# 5. New/constructor methods

add_inline_if_missing() {
    local file="$1"
    local pattern="$2"
    local tmp=$(mktemp)
    local added=0

    while IFS= read -r line; do
        echo "$line" >> "$tmp"
        # Check if this line is a function definition that needs #[inline]
        if [[ "$line" =~ ^[[:space:]]*pub\ (fn|unsafe\ fn)\ .*self\ *(\&|$) ]] || \
           [[ "$line" =~ ^[[:space:]]*pub\ fn\ (new|default|from_|to_|into_|as_|is_|get_|set_|with_|builder|build|clone_|status_code|ok|text|header|body|signal|json|method|url|host|port|path|search|hash|href|origin|protocol|hostname) ]] || \
           [[ "$line" =~ ^[[:space:]]*pub\ fn\ (len|is_empty|has|keys|values|entries|sort|clear|delete|append|remove|contains|get|set|add|push|pop|insert|swap|split|join|trim|clone|copy|into_|from_|to_|as_) ]]; then
            # Check if the previous line already has #[inline]
            if ! grep -q '#\[inline' <<< "$(tail -1 "$tmp" 2>/dev/null)"; then
                echo "    #[inline]" >> "$tmp"
                added=$((added + 1))
            fi
        fi
    done < "$file"
    mv "$tmp" "$file"
    echo "  Added $added #[inline] annotations to $file"
}

# Process all lib.rs files in klyron crates
for crate in crates/klyron_*; do
    lib="$crate/src/lib.rs"
    if [ -f "$lib" ]; then
        echo "Processing $lib..."
        add_inline_if_missing "$lib"
    fi
done

echo "Done! Added #[inline] annotations across all crates."
