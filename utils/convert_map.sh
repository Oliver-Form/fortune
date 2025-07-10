#!/bin/bash

# Map to PNG Converter Script
# Usage: ./convert_map.sh [input.map] [output.png]

UTILS_DIR="$(dirname "$0")"
cd "$UTILS_DIR"

# Check if Rust/Cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo not found. Please install Rust and Cargo."
    exit 1
fi

# Build the utility if needed
if [ ! -f "target/release/map_to_png" ] || [ "src/main.rs" -nt "target/release/map_to_png" ]; then
    echo "Building map converter..."
    cargo build --release
    if [ $? -ne 0 ]; then
        echo "Error: Failed to build the converter."
        exit 1
    fi
fi

# Run the converter with provided arguments
echo "Converting map to PNG..."
if [ $# -eq 0 ]; then
    # No arguments - use defaults
    cargo run --release
elif [ $# -eq 1 ]; then
    # One argument - input file only
    cargo run --release -- "$1"
elif [ $# -eq 2 ]; then
    # Two arguments - input and output files
    cargo run --release -- "$1" "$2"
else
    echo "Usage: $0 [input.map] [output.png]"
    echo "Examples:"
    echo "  $0                              # Use default paths"
    echo "  $0 my_map.map                  # Convert my_map.map to my_map.png"
    echo "  $0 input.map output.png        # Convert input.map to output.png"
    exit 1
fi
