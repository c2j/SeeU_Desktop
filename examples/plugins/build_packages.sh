#!/bin/bash

# Build script for creating iTools plugin packages
# Usage: ./build_packages.sh [plugin_name]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/packages"

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Function to build a single plugin package
build_plugin() {
    local plugin_name="$1"
    local plugin_dir="$SCRIPT_DIR/$plugin_name"
    
    if [ ! -d "$plugin_dir" ]; then
        echo "Error: Plugin directory '$plugin_dir' not found"
        return 1
    fi
    
    echo "Building plugin package: $plugin_name"
    
    # Check for required files
    if [ ! -f "$plugin_dir/manifest.json" ]; then
        echo "Error: manifest.json not found in $plugin_dir"
        return 1
    fi
    
    if [ ! -f "$plugin_dir/metadata.json" ]; then
        echo "Error: metadata.json not found in $plugin_dir"
        return 1
    fi
    
    # Create package
    local package_file="$OUTPUT_DIR/${plugin_name}.itpkg"
    
    echo "Creating package: $package_file"
    
    # Change to plugin directory and create tar.gz package
    (cd "$plugin_dir" && tar -czf "$package_file" .)
    
    if [ $? -eq 0 ]; then
        echo "✅ Successfully created: $package_file"
        
        # Show package info
        local size=$(du -h "$package_file" | cut -f1)
        echo "   Package size: $size"
        
        # Verify package contents
        echo "   Package contents:"
        tar -tzf "$package_file" | head -10 | sed 's/^/     /'
        
        local total_files=$(tar -tzf "$package_file" | wc -l)
        if [ "$total_files" -gt 10 ]; then
            echo "     ... and $((total_files - 10)) more files"
        fi
        
        echo ""
    else
        echo "❌ Failed to create package: $package_file"
        return 1
    fi
}

# Function to validate plugin structure
validate_plugin() {
    local plugin_name="$1"
    local plugin_dir="$SCRIPT_DIR/$plugin_name"
    
    echo "Validating plugin: $plugin_name"
    
    # Check required files
    local required_files=("manifest.json" "metadata.json")
    for file in "${required_files[@]}"; do
        if [ ! -f "$plugin_dir/$file" ]; then
            echo "❌ Missing required file: $file"
            return 1
        fi
    done
    
    # Validate JSON files
    if ! python3 -m json.tool "$plugin_dir/manifest.json" > /dev/null 2>&1; then
        echo "❌ Invalid JSON in manifest.json"
        return 1
    fi
    
    if ! python3 -m json.tool "$plugin_dir/metadata.json" > /dev/null 2>&1; then
        echo "❌ Invalid JSON in metadata.json"
        return 1
    fi
    
    # Check entry point exists
    local entry_point=$(python3 -c "
import json
with open('$plugin_dir/metadata.json') as f:
    data = json.load(f)
    print(data.get('entry_point', ''))
" 2>/dev/null)
    
    if [ -n "$entry_point" ] && [ ! -f "$plugin_dir/$entry_point" ]; then
        echo "❌ Entry point not found: $entry_point"
        return 1
    fi
    
    echo "✅ Plugin validation passed"
    return 0
}

# Main script logic
if [ $# -eq 0 ]; then
    # Build all plugins
    echo "Building all plugin packages..."
    echo "================================"
    
    for plugin_dir in "$SCRIPT_DIR"/*/; do
        if [ -d "$plugin_dir" ] && [ "$(basename "$plugin_dir")" != "packages" ]; then
            plugin_name=$(basename "$plugin_dir")
            
            if validate_plugin "$plugin_name"; then
                build_plugin "$plugin_name"
            else
                echo "❌ Skipping $plugin_name due to validation errors"
                echo ""
            fi
        fi
    done
    
elif [ $# -eq 1 ]; then
    # Build specific plugin
    plugin_name="$1"
    echo "Building plugin package: $plugin_name"
    echo "================================"
    
    if validate_plugin "$plugin_name"; then
        build_plugin "$plugin_name"
    else
        echo "❌ Plugin validation failed"
        exit 1
    fi
    
else
    echo "Usage: $0 [plugin_name]"
    echo ""
    echo "Examples:"
    echo "  $0                    # Build all plugins"
    echo "  $0 simple-calculator # Build specific plugin"
    exit 1
fi

echo "Build completed!"
echo "Packages are available in: $OUTPUT_DIR"
