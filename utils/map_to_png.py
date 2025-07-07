#!/usr/bin/env python3
"""
Map to PNG Converter for Fortune Game

This script converts .map files (binary format) to PNG images.
The map files contain a 4096x4096 grid of tiles, where each tile is represented as a u16 (2 bytes).

Map format:
- Size: 4096x4096 tiles
- Each tile: 2 bytes (little-endian u16)
- Total file size: 4096 * 4096 * 2 = 33,554,432 bytes

Tile types and their RGB colors:
- 0: Grass  -> (51, 204, 51)    [0.2, 0.8, 0.2] * 255
- 1: Desert -> (204, 178, 102)  [0.8, 0.7, 0.4] * 255
- 2: Water  -> (51, 102, 204)   [0.2, 0.4, 0.8] * 255
- 3: Rock   -> (127, 127, 127)  [0.5, 0.5, 0.5] * 255
- 4: Sand   -> (229, 204, 153)  [0.9, 0.8, 0.6] * 255
"""

import argparse
import struct
import sys
from pathlib import Path
from typing import Tuple

try:
    from PIL import Image
    import numpy as np
except ImportError:
    print("Error: Required packages not found. Please install them with:")
    print("pip install Pillow numpy")
    sys.exit(1)

# Constants from the game
MAP_SIZE = 4096
BYTES_PER_TILE = 2

# Color mapping for tile types (RGB values 0-255)
TILE_COLORS = {
    0: (51, 204, 51),    # Grass - green
    1: (204, 178, 102),  # Desert - sandy brown
    2: (51, 102, 204),   # Water - blue
    3: (127, 127, 127),  # Rock - gray
    4: (229, 204, 153),  # Sand - light sandy
}

def load_map_file(file_path: str) -> np.ndarray:
    """
    Load a .map file and return a 2D numpy array of tile types.
    
    Args:
        file_path: Path to the .map file
        
    Returns:
        2D numpy array of shape (MAP_SIZE, MAP_SIZE) containing tile type values
        
    Raises:
        FileNotFoundError: If the map file doesn't exist
        ValueError: If the map file has incorrect size
    """
    path = Path(file_path)
    
    if not path.exists():
        raise FileNotFoundError(f"Map file not found: {file_path}")
    
    expected_size = MAP_SIZE * MAP_SIZE * BYTES_PER_TILE
    actual_size = path.stat().st_size
    
    if actual_size != expected_size:
        raise ValueError(
            f"Invalid map file size. Expected {expected_size} bytes, got {actual_size} bytes"
        )
    
    # Read the binary data
    with open(file_path, 'rb') as f:
        data = f.read()
    
    # Convert to array of u16 values (little-endian)
    tile_values = struct.unpack(f'<{MAP_SIZE * MAP_SIZE}H', data)
    
    # Reshape into 2D array (z, x) -> (row, col)
    tiles = np.array(tile_values).reshape(MAP_SIZE, MAP_SIZE)
    
    return tiles

def tiles_to_image(tiles: np.ndarray, scale_factor: int = 1) -> Image.Image:
    """
    Convert a 2D array of tile types to a PIL Image.
    
    Args:
        tiles: 2D numpy array of tile type values
        scale_factor: Factor to scale down the image (1 = full size, 2 = half size, etc.)
        
    Returns:
        PIL Image object
    """
    height, width = tiles.shape
    
    # Create RGB image array
    if scale_factor > 1:
        # Downsample the tiles array first for performance
        new_height = height // scale_factor
        new_width = width // scale_factor
        
        # Take every nth pixel for downsampling
        downsampled_tiles = tiles[::scale_factor, ::scale_factor]
        tiles = downsampled_tiles
        height, width = new_height, new_width
    
    # Create RGB array
    rgb_array = np.zeros((height, width, 3), dtype=np.uint8)
    
    # Map each tile type to its color
    for tile_type, color in TILE_COLORS.items():
        mask = tiles == tile_type
        rgb_array[mask] = color
    
    # Handle unknown tile types (default to grass)
    unknown_mask = ~np.isin(tiles, list(TILE_COLORS.keys()))
    rgb_array[unknown_mask] = TILE_COLORS[0]  # Grass color
    
    # Convert to PIL Image
    image = Image.fromarray(rgb_array, 'RGB')
    return image

def convert_map_to_png(input_path: str, output_path: str = None, scale_factor: int = 1) -> str:
    """
    Convert a .map file to a PNG image.
    
    Args:
        input_path: Path to the input .map file
        output_path: Path for the output PNG file (optional)
        scale_factor: Factor to scale down the image (1 = full size)
        
    Returns:
        Path to the created PNG file
    """
    input_path = Path(input_path)
    
    # Generate output path if not provided
    if output_path is None:
        output_path = input_path.with_suffix('.png')
    else:
        output_path = Path(output_path)
    
    print(f"Loading map from: {input_path}")
    tiles = load_map_file(str(input_path))
    
    print(f"Converting {MAP_SIZE}x{MAP_SIZE} map to image...")
    if scale_factor > 1:
        print(f"Scaling down by factor of {scale_factor}")
    
    image = tiles_to_image(tiles, scale_factor)
    
    print(f"Saving PNG to: {output_path}")
    image.save(str(output_path), 'PNG')
    
    # Print some statistics
    unique_tiles = np.unique(tiles)
    print(f"Map statistics:")
    print(f"  Image size: {image.size[0]}x{image.size[1]} pixels")
    print(f"  Tile types found: {unique_tiles.tolist()}")
    
    for tile_type in unique_tiles:
        if tile_type in TILE_COLORS:
            count = np.sum(tiles == tile_type)
            percentage = (count / tiles.size) * 100
            tile_names = ['Grass', 'Desert', 'Water', 'Rock', 'Sand']
            if tile_type < len(tile_names):
                name = tile_names[tile_type]
            else:
                name = f'Unknown({tile_type})'
            print(f"    {name}: {count:,} tiles ({percentage:.1f}%)")
    
    return str(output_path)

def main():
    """Main entry point for the script."""
    parser = argparse.ArgumentParser(
        description="Convert Fortune game .map files to PNG images",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python map_to_png.py world.map
  python map_to_png.py world.map -o world_map.png
  python map_to_png.py world.map -s 4 -o small_world.png
  python map_to_png.py ../src/file.map
        """
    )
    
    parser.add_argument('input', help='Input .map file path')
    parser.add_argument('-o', '--output', help='Output PNG file path (default: input_name.png)')
    parser.add_argument('-s', '--scale', type=int, default=1, 
                       help='Scale factor to reduce image size (default: 1 = full size)')
    
    args = parser.parse_args()
    
    # Validate scale factor
    if args.scale < 1:
        print("Error: Scale factor must be 1 or greater")
        sys.exit(1)
    
    if args.scale > MAP_SIZE:
        print(f"Error: Scale factor cannot be greater than {MAP_SIZE}")
        sys.exit(1)
    
    try:
        output_path = convert_map_to_png(args.input, args.output, args.scale)
        print(f"\nSuccess! PNG image saved to: {output_path}")
        
    except FileNotFoundError as e:
        print(f"Error: {e}")
        sys.exit(1)
    except ValueError as e:
        print(f"Error: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"Unexpected error: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main()
