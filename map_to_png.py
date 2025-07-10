#!/usr/bin/env python3
"""
Fortune Game Map to PNG Converter

This utility converts binary map files to PNG images for visualization.

Usage:
    python map_to_png.py <input_map_file> <output_png_file> [options]

Options:
    --scale SCALE       Scale factor for output image (default: 1)
    --chunk-grid        Draw chunk grid lines
    --region X Y W H    Convert only a specific region
    --palette STYLE     Color palette: default, realistic, high-contrast
"""

import struct
import sys
import argparse
from typing import Tuple, List
try:
    from PIL import Image
except ImportError:
    print("Error: PIL (Pillow) is required. Install with: pip install Pillow")
    sys.exit(1)

# Constants
MAP_SIZE = 4096
CHUNK_SIZE = 256

class TileType:
    GRASS = 0
    DESERT = 1
    WATER = 2
    ROCK = 3
    SAND = 4

# Color palettes
PALETTES = {
    'default': {
        TileType.GRASS: (32, 128, 32),      # Green
        TileType.DESERT: (204, 179, 102),   # Tan/Yellow
        TileType.WATER: (51, 102, 204),     # Blue
        TileType.ROCK: (128, 128, 128),     # Gray
        TileType.SAND: (230, 204, 153),     # Light tan
    },
    'realistic': {
        TileType.GRASS: (34, 139, 34),      # Forest green
        TileType.DESERT: (238, 203, 173),   # Peach puff
        TileType.WATER: (65, 105, 225),     # Royal blue
        TileType.ROCK: (105, 105, 105),     # Dim gray
        TileType.SAND: (244, 164, 96),      # Sandy brown
    },
    'high-contrast': {
        TileType.GRASS: (0, 255, 0),        # Bright green
        TileType.DESERT: (255, 255, 0),     # Bright yellow
        TileType.WATER: (0, 0, 255),        # Bright blue
        TileType.ROCK: (128, 128, 128),     # Gray
        TileType.SAND: (255, 192, 203),     # Pink
    }
}

def load_map(filename: str) -> List[List[int]]:
    """Load map data from binary file."""
    try:
        with open(filename, 'rb') as f:
            data = f.read()
        
        if len(data) != MAP_SIZE * MAP_SIZE * 2:
            raise ValueError(f"Invalid map file size. Expected {MAP_SIZE * MAP_SIZE * 2} bytes, got {len(data)}")
        
        map_data = [[0 for _ in range(MAP_SIZE)] for _ in range(MAP_SIZE)]
        
        for z in range(MAP_SIZE):
            for x in range(MAP_SIZE):
                index = (z * MAP_SIZE + x) * 2
                tile_value = struct.unpack('<H', data[index:index+2])[0]
                map_data[z][x] = min(tile_value, 4)  # Clamp to valid tile types
        
        return map_data
    
    except FileNotFoundError:
        raise FileNotFoundError(f"Map file '{filename}' not found")
    except Exception as e:
        raise Exception(f"Error loading map: {e}")

def map_to_image(map_data: List[List[int]], 
                palette: dict, 
                scale: int = 1,
                region: Tuple[int, int, int, int] = None,
                chunk_grid: bool = False) -> Image.Image:
    """Convert map data to PIL Image."""
    
    if region:
        x, z, width, height = region
        # Clamp region to map bounds
        x = max(0, min(x, MAP_SIZE - 1))
        z = max(0, min(z, MAP_SIZE - 1))
        width = min(width, MAP_SIZE - x)
        height = min(height, MAP_SIZE - z)
    else:
        x, z, width, height = 0, 0, MAP_SIZE, MAP_SIZE
    
    # Create image
    img_width = width * scale
    img_height = height * scale
    img = Image.new('RGB', (img_width, img_height))
    pixels = img.load()
    
    # Fill pixels
    for row in range(height):
        for col in range(width):
            tile_type = map_data[z + row][x + col]
            color = palette.get(tile_type, (255, 0, 255))  # Magenta for unknown tiles
            
            # Fill scaled pixels
            for sy in range(scale):
                for sx in range(scale):
                    pixel_x = col * scale + sx
                    pixel_y = row * scale + sy
                    if pixel_x < img_width and pixel_y < img_height:
                        pixels[pixel_x, pixel_y] = color
    
    # Draw chunk grid if requested
    if chunk_grid and scale >= 2:
        grid_color = (64, 64, 64)  # Dark gray
        
        # Vertical lines
        for chunk_x in range(0, width, CHUNK_SIZE):
            if chunk_x == 0:
                continue  # Skip first line
            pixel_x = chunk_x * scale
            if pixel_x < img_width:
                for y in range(img_height):
                    pixels[pixel_x, y] = grid_color
        
        # Horizontal lines  
        for chunk_z in range(0, height, CHUNK_SIZE):
            if chunk_z == 0:
                continue  # Skip first line
            pixel_y = chunk_z * scale
            if pixel_y < img_height:
                for x in range(img_width):
                    pixels[x, pixel_y] = grid_color
    
    return img

def main():
    parser = argparse.ArgumentParser(description="Convert Fortune map files to PNG images",
                                   formatter_class=argparse.RawDescriptionHelpFormatter,
                                   epilog=__doc__)
    
    parser.add_argument('input_map', help='Input map file path')
    parser.add_argument('output_png', help='Output PNG file path')
    parser.add_argument('--scale', type=int, default=1, 
                       help='Scale factor for output image (default: 1)')
    parser.add_argument('--chunk-grid', action='store_true',
                       help='Draw chunk grid lines')
    parser.add_argument('--region', nargs=4, type=int, metavar=('X', 'Y', 'W', 'H'),
                       help='Convert only a specific region')
    parser.add_argument('--palette', choices=['default', 'realistic', 'high-contrast'],
                       default='default', help='Color palette style')
    
    args = parser.parse_args()
    
    try:
        # Validate arguments
        if args.scale < 1:
            print("Error: Scale must be at least 1")
            sys.exit(1)
        
        if args.region:
            x, y, w, h = args.region
            if x < 0 or y < 0 or w <= 0 or h <= 0:
                print("Error: Region coordinates must be non-negative and size must be positive")
                sys.exit(1)
            if x >= MAP_SIZE or y >= MAP_SIZE:
                print(f"Error: Region starting position must be within map bounds (0-{MAP_SIZE-1})")
                sys.exit(1)
        
        # Load map data
        print(f"Loading map from {args.input_map}...")
        map_data = load_map(args.input_map)
        
        # Convert to image
        print(f"Converting to image with {args.palette} palette...")
        palette = PALETTES[args.palette]
        region = tuple(args.region) if args.region else None
        
        img = map_to_image(map_data, palette, args.scale, region, args.chunk_grid)
        
        # Save image
        print(f"Saving image to {args.output_png}...")
        img.save(args.output_png)
        
        print(f"Successfully converted map to {args.output_png}")
        print(f"Image size: {img.width}x{img.height} pixels")
        
        if region:
            x, y, w, h = region
            print(f"Region: ({x},{y}) {w}x{h} tiles")
        else:
            print(f"Full map: {MAP_SIZE}x{MAP_SIZE} tiles")
        
    except KeyboardInterrupt:
        print("\nOperation cancelled")
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
