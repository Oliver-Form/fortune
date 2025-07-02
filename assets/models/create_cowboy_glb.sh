#!/bin/bash

# Script to combine Mixamo animations into a single GLB file
# Make sure you have Blender installed and the FBX files downloaded

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODELS_DIR="$SCRIPT_DIR"

echo "=== Mixamo to GLB Animation Combiner ==="
echo "Models directory: $MODELS_DIR"

# Check if Blender is installed
if ! command -v blender &> /dev/null; then
    echo "Error: Blender is not installed or not in PATH"
    echo "Please install Blender: sudo apt install blender"
    exit 1
fi

# Check for required FBX files
required_files=(
    "mixamo_idle.fbx"
    "mixamo_walking.fbx" 
    "mixamo_running.fbx"
    "mixamo_aiming.fbx"
    "mixamo_shooting.fbx"
    "mixamo_holster.fbx"
)

echo "Checking for required FBX files..."
missing_files=()

for file in "${required_files[@]}"; do
    if [ ! -f "$MODELS_DIR/$file" ]; then
        missing_files+=("$file")
    else
        echo "✓ Found: $file"
    fi
done

if [ ${#missing_files[@]} -ne 0 ]; then
    echo ""
    echo "❌ Missing FBX files:"
    for file in "${missing_files[@]}"; do
        echo "   - $file"
    done
    echo ""
    echo "Please download these animations from Mixamo:"
    echo "1. Go to https://mixamo.adobe.com"
    echo "2. Select a cowboy character"
    echo "3. Download these animations:"
    echo "   - idle (WITH SKIN) → save as mixamo_idle.fbx"
    echo "   - walking (WITHOUT SKIN) → save as mixamo_walking.fbx"
    echo "   - running (WITHOUT SKIN) → save as mixamo_running.fbx"
    echo "   - aiming (WITHOUT SKIN) → save as mixamo_aiming.fbx" 
    echo "   - shooting (WITHOUT SKIN) → save as mixamo_shooting.fbx"
    echo "   - holster (WITHOUT SKIN) → save as mixamo_holster.fbx"
    echo "4. Place all files in: $MODELS_DIR"
    echo "5. Run this script again"
    exit 1
fi

echo ""
echo "✅ All FBX files found!"
echo "Running Blender to combine animations..."

# Run Blender with the Python script
blender --background --python "$MODELS_DIR/combine_animations.py"

if [ $? -eq 0 ]; then
    echo ""
    echo "🎉 SUCCESS! cowboy.glb created successfully!"
    echo "📁 Location: $MODELS_DIR/cowboy.glb"
    echo ""
    echo "Animation indices in the GLB file:"
    echo "   0: idle"
    echo "   1: walking"
    echo "   2: running" 
    echo "   3: aiming"
    echo "   4: shooting"
    echo "   5: holster"
    echo ""
    echo "You can now run your Bevy game with: cargo run"
else
    echo "❌ Error occurred during Blender processing"
    exit 1
fi
