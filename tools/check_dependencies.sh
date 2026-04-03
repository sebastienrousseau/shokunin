#!/bin/sh

# Set the path to Cargo.toml relative to the script's location
script_dir="$(cd "$(dirname "$0")" && pwd)"
cargo_toml="${script_dir}/../Cargo.toml"

# Directories to search for dependency usage
search_dirs="${script_dir}/../src/ ${script_dir}/../benches/ ${script_dir}/../examples/ ${script_dir}/../tests/"

# Extract dependency names specifically from the [dependencies] section
dependencies=$(awk '/\[dependencies\]/ {flag=1; next} /^\[/{flag=0} flag {print}' "${cargo_toml}" | grep -oE '^[a-zA-Z0-9_-]+' || true)

# Iterate over each dependency
echo "${dependencies}" | while read -r dep; do
    # Skip empty lines
    [ -z "${dep}" ] && continue

    # Prepare a pattern to match Rust module imports (e.g., http-handle becomes http_handle)
    dep_pattern=$(echo "${dep}" | tr '-' '_')

    # Check if the dependency is used in any of the specified directories
    found=false
    for dir in ${search_dirs}; do
        if grep -qir "${dep_pattern}" "${dir}"; then
            found=true
            break
        fi
    done

    # If the dependency is not found in any directory, mark it as unused
    if [ "${found}" = false ]; then
        printf "The \033[1m%s\033[0m crate is not required!\n" "${dep}"
    fi
done
