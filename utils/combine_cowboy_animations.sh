#!/bin/bash

# Script to combine cowboy animations into a single GLB file using Blender
# Usage: ./combine_cowboy_animations.sh

set -e

# Check if Blender is installed
if ! command -v blender &> /dev/null; then
    echo "ERROR: Blender is not installed or not in PATH"
    echo "Please install Blender first:"
    echo "  Ubuntu/Debian: sudo apt install blender"
    echo "  Or download from: https://www.blender.org/download/"
    exit 1
fi

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Cowboy Animation Combiner ==="
echo "Working directory: $SCRIPT_DIR"
echo ""

# Check if required files exist
required_files=(
    "cowboy_idle.glb"
    "cowboy_walking.glb" 
    "cowboy_shooting.glb"
    "cowboy_aiming.glb"
    "Running.fbx"
    "Pistol Aim.fbx"
)

missing_files=()
for file in "${required_files[@]}"; do
    if [[ ! -f "$file" ]]; then
        missing_files+=("$file")
    fi
done

if [[ ${#missing_files[@]} -gt 0 ]]; then
    echo "WARNING: The following files are missing:"
    for file in "${missing_files[@]}"; do
        echo "  - $file"
    done
    echo ""
    echo "The script will continue with available files..."
    echo ""
fi

echo "Running Blender to combine animations..."
echo "This may take a few minutes..."
echo ""

# Run Blender in background mode with our Python script
blender --background --python combine_cowboy_animations.py

# Check if the output file was created
if [[ -f "cowboy_combined.glb" ]]; then
    echo ""
    echo "✅ SUCCESS: Combined GLB file created!"
    echo "📁 Output file: cowboy_combined.glb"
    echo ""
    echo "File size: $(du -h cowboy_combined.glb | cut -f1)"
    echo ""
    echo "You can now use this file in your Bevy game by updating the asset paths to:"
    echo "  Scene: models/cowboy_combined.glb#Scene0"
    echo "  Animations:"
    echo "    - models/cowboy_combined.glb#Animation0 (Idle)"
    echo "    - models/cowboy_combined.glb#Animation1 (Walking)"
    echo "    - models/cowboy_combined.glb#Animation2 (Shooting)"
    echo "    - models/cowboy_combined.glb#Animation3 (Aiming)"
    echo "    - models/cowboy_combined.glb#Animation4 (Running)"
    echo "    - models/cowboy_combined.glb#Animation5 (Holster)"
    echo ""
else
    echo "❌ ERROR: Failed to create combined GLB file"
    echo "Check the Blender output above for error messages"
    exit 1
fi
