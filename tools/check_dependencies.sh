#!/bin/bash

# Set the path to Cargo.toml relative to the script's location
cargo_toml="$(dirname "$0")/../Cargo.toml"
# Set the directories to search in relative to the script's location
search_dirs=(
    "$(dirname "$0")/../src/"
    "$(dirname "$0")/../benches/"
    "$(dirname "$0")/../examples/"
    "$(dirname "$0")/../tests/"
)

# Extract dependency names specifically from the `[dependencies]` section
dependencies=$(awk '/\[dependencies\]/ {flag=1; next} /^\[/{flag=0} flag {print}' "${cargo_toml}" | grep -oE '^[a-zA-Z0-9_-]+' || true)

# Iterate over each dependency
while read -r dep; do
    # Skip empty lines
    [[ -z "${dep}" ]] && continue

    # Prepare a pattern to match Rust module imports (e.g., http-handle becomes http_handle)
    dep_pattern=$(echo "${dep}" | tr '-' '_')

    # Check if the dependency is used in any of the specified directories
    found=false
    for dir in "${search_dirs[@]}"; do
        if grep -qir "${dep_pattern}" "${dir}"; then
            found=true
            break
        fi
    done

    # If the dependency is not found in any directory, mark it as unused
    if [[ "${found}" = false ]]; then
        printf "🗑️ The \033[1m%s\033[0m crate is not required!\n" "${dep}"
    fi
done <<< "${dependencies}"
