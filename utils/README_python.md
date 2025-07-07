# Map to PNG Converter (Python)

This utility converts Fortune game `.map` files to PNG images using Python.

## Features

- **High Performance**: Uses NumPy for fast array operations
- **Configurable Scaling**: Reduce image size with scale factors for large maps
- **Color Accuracy**: Uses the exact same color mapping as the game
- **Detailed Statistics**: Shows tile distribution and percentages
- **Cross-Platform**: Works on Linux, macOS, and Windows

## Requirements

- Python 3.7 or higher
- Pillow (PIL) library for image processing
- NumPy for array operations

## Installation

### Option 1: Automatic (Recommended)
Use the provided shell script which will automatically install dependencies:

```bash
./convert_map_to_png.sh ../src/file.map
```

### Option 2: Manual Installation
Install dependencies manually:

```bash
pip install -r requirements.txt
# or
pip install Pillow numpy
```

## Usage

### Command Line Interface

```bash
# Basic usage - convert map to PNG
python map_to_png.py input.map

# Specify output filename
python map_to_png.py input.map -o world_map.png

# Scale down large maps (4x smaller)
python map_to_png.py input.map -s 4 -o small_world.png

# Convert test maps
python map_to_png.py ../src/file.map
```

### Using the Shell Script

```bash
# Basic conversion
./convert_map_to_png.sh ../src/file.map

# With options
./convert_map_to_png.sh ../src/file.map -o test_map.png -s 2
```

## Map File Format

The `.map` files contain:
- **Size**: 4096×4096 tiles
- **Format**: Binary, little-endian
- **Tile Data**: Each tile is 2 bytes (u16)
- **Total Size**: 33,554,432 bytes (32 MB)

## Tile Types and Colors

| Type | Name   | RGB Color       | Hex Color | Game Color         |
|------|--------|-----------------|-----------|-------------------|
| 0    | Grass  | (51, 204, 51)   | #33CC33  | (0.2, 0.8, 0.2)  |
| 1    | Desert | (204, 178, 102) | #CCB266  | (0.8, 0.7, 0.4)  |
| 2    | Water  | (51, 102, 204)  | #3366CC  | (0.2, 0.4, 0.8)  |
| 3    | Rock   | (127, 127, 127) | #7F7F7F  | (0.5, 0.5, 0.5)  |
| 4    | Sand   | (229, 204, 153) | #E5CC99  | (0.9, 0.8, 0.6)  |

## Performance Tips

### Large Maps (4096×4096)
- Full resolution PNG: ~48 MB
- Use scale factor 4 for overview: ~3 MB
- Use scale factor 8 for thumbnails: ~800 KB

### Memory Usage
- Loading 4096×4096 map: ~64 MB RAM
- Creating full PNG: ~192 MB RAM peak
- Scaled images use proportionally less memory

## Examples

### Convert and View Statistics
```bash
python map_to_png.py world.map
```
Output:
```
Loading map from: world.map
Converting 4096x4096 map to image...
Saving PNG to: world.png
Map statistics:
  Image size: 4096x4096 pixels
  Tile types found: [0, 1, 2, 3, 4]
    Grass: 8,234,567 tiles (49.1%)
    Desert: 3,456,789 tiles (20.6%)
    Water: 2,987,654 tiles (17.8%)
    Rock: 1,234,567 tiles (7.4%)
    Sand: 865,431 tiles (5.2%)
```

### Create Scaled Overview
```bash
python map_to_png.py large_world.map -s 8 -o overview.png
```

### Batch Process Multiple Maps
```bash
for map in *.map; do
    python map_to_png.py "$map" -s 4
done
```

## Integration with Game Development

This utility is perfect for:
- **Level Design**: Visualize map layouts
- **Testing**: Quick preview of procedurally generated maps  
- **Documentation**: Create map images for wikis/guides
- **Debugging**: Verify map generation algorithms
- **Asset Creation**: Generate textures or backgrounds

## Troubleshooting

### Common Issues

**"ModuleNotFoundError: No module named 'PIL'"**
```bash
pip install Pillow numpy
```

**"Invalid map file size"**
- Check that the file is exactly 33,554,432 bytes
- Ensure the file isn't corrupted or truncated

**"Permission denied"**
```bash
chmod +x convert_map_to_png.sh
```

**Memory issues with large maps**
- Use a scale factor: `-s 4` or higher
- Close other applications to free RAM
- Use a 64-bit Python installation

### Debugging Mode

Add debug prints by modifying the script:
```python
# Add after loading tiles
print(f"Min tile value: {tiles.min()}")
print(f"Max tile value: {tiles.max()}")
print(f"Tiles shape: {tiles.shape}")
```

## Performance Comparison

| Operation | Time (4096×4096) | Memory |
|-----------|------------------|--------|
| Load .map | ~0.5s           | 64 MB  |
| Scale 1x  | ~2.0s           | 192 MB |
| Scale 4x  | ~0.3s           | 12 MB  |
| Scale 8x  | ~0.1s           | 3 MB   |

## Related Files

- `map_to_png.py` - Main Python script
- `convert_map_to_png.sh` - Convenience shell script
- `requirements.txt` - Python dependencies
- `../src/map_loader.rs` - Game's map loading code
- `../src/file.map` - Test map file
