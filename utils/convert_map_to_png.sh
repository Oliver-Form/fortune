#!/bin/bash

# Map to PNG Converter Shell Script
# Converts Fortune game .map files to PNG images using Python

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON_SCRIPT="$SCRIPT_DIR/map_to_png.py"

# Check if Python script exists
if [ ! -f "$PYTHON_SCRIPT" ]; then
    echo "Error: Python script not found at $PYTHON_SCRIPT"
    exit 1
fi

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo "Error: Python 3 is required but not found"
    echo "Please install Python 3 and try again"
    exit 1
fi

# Check if required packages are installed
if ! python3 -c "import PIL, numpy" 2>/dev/null; then
    echo "Installing required Python packages..."
    if [ -f "$SCRIPT_DIR/requirements.txt" ]; then
        pip3 install -r "$SCRIPT_DIR/requirements.txt"
    else
        pip3 install Pillow numpy
    fi
fi

# Run the Python script with all arguments
python3 "$PYTHON_SCRIPT" "$@"
