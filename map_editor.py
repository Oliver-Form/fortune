#!/usr/bin/env python3
"""
Fortune Game Map Editor CLI Tool

This tool allows you to create and edit map files for the Fortune game.
You can place different tile types in rectangular, circular, or custom-shaped areas.

Usage:
    python map_editor.py create <map_file>                    # Create a new map
    python map_editor.py info <map_file>                      # Show map information
    python map_editor.py fill <map_file> <tile_type> [x y w h]  # Fill rectangle with tile type
    python map_editor.py circle <map_file> <tile_type> <x> <y> <radius>  # Fill circle
    python map_editor.py line <map_file> <tile_type> <x1> <y1> <x2> <y2> [width]  # Draw line
    python map_editor.py noise <map_file> <tile_type> <density> [x y w h]  # Add noise pattern
    python map_editor.py view <map_file> [x y w h]             # View ASCII representation
    python map_editor.py copy <src_map> <dst_map> <src_x> <src_y> <dst_x> <dst_y> <w> <h>  # Copy region

Tile Types:
    0 = Grass (default)
    1 = Desert
    2 = Water
    3 = Rock
    4 = Sand
"""

import struct
import sys
import os
import math
import random
import argparse
from typing import List, Tuple, Optional

# Constants from the game
MAP_SIZE = 4096
CHUNK_SIZE = 256

class TileType:
    GRASS = 0
    DESERT = 1
    WATER = 2
    ROCK = 3
    SAND = 4
    
    NAMES = {
        0: "Grass",
        1: "Desert", 
        2: "Water",
        3: "Rock",
        4: "Sand"
    }
    
    SYMBOLS = {
        0: ".",  # Grass
        1: "~",  # Desert
        2: "≈",  # Water
        3: "▲",  # Rock
        4: ":",  # Sand
    }

class MapEditor:
    def __init__(self):
        self.map_data = [[TileType.GRASS for _ in range(MAP_SIZE)] for _ in range(MAP_SIZE)]
    
    def load_map(self, filename: str) -> bool:
        """Load a map from a binary file."""
        try:
            with open(filename, 'rb') as f:
                data = f.read()
            
            if len(data) != MAP_SIZE * MAP_SIZE * 2:
                print(f"Error: Invalid map file size. Expected {MAP_SIZE * MAP_SIZE * 2} bytes, got {len(data)}")
                return False
            
            for z in range(MAP_SIZE):
                for x in range(MAP_SIZE):
                    index = (z * MAP_SIZE + x) * 2
                    tile_value = struct.unpack('<H', data[index:index+2])[0]
                    self.map_data[z][x] = min(tile_value, 4)  # Clamp to valid tile types
            
            return True
        except FileNotFoundError:
            print(f"Error: Map file '{filename}' not found")
            return False
        except Exception as e:
            print(f"Error loading map: {e}")
            return False
    
    def save_map(self, filename: str) -> bool:
        """Save the map to a binary file."""
        try:
            with open(filename, 'wb') as f:
                for z in range(MAP_SIZE):
                    for x in range(MAP_SIZE):
                        tile_value = self.map_data[z][x]
                        f.write(struct.pack('<H', tile_value))
            return True
        except Exception as e:
            print(f"Error saving map: {e}")
            return False
    
    def create_new_map(self, filename: str) -> bool:
        """Create a new map filled with grass."""
        self.map_data = [[TileType.GRASS for _ in range(MAP_SIZE)] for _ in range(MAP_SIZE)]
        return self.save_map(filename)
    
    def get_tile(self, x: int, z: int) -> int:
        """Get tile at position (x, z)."""
        if 0 <= x < MAP_SIZE and 0 <= z < MAP_SIZE:
            return self.map_data[z][x]
        return TileType.GRASS
    
    def set_tile(self, x: int, z: int, tile_type: int):
        """Set tile at position (x, z)."""
        if 0 <= x < MAP_SIZE and 0 <= z < MAP_SIZE and 0 <= tile_type <= 4:
            self.map_data[z][x] = tile_type
    
    def fill_rectangle(self, x: int, z: int, width: int, height: int, tile_type: int):
        """Fill a rectangular area with the specified tile type."""
        for dz in range(height):
            for dx in range(width):
                self.set_tile(x + dx, z + dz, tile_type)
    
    def fill_circle(self, center_x: int, center_z: int, radius: int, tile_type: int):
        """Fill a circular area with the specified tile type."""
        for z in range(max(0, center_z - radius), min(MAP_SIZE, center_z + radius + 1)):
            for x in range(max(0, center_x - radius), min(MAP_SIZE, center_x + radius + 1)):
                distance = math.sqrt((x - center_x) ** 2 + (z - center_z) ** 2)
                if distance <= radius:
                    self.set_tile(x, z, tile_type)
    
    def draw_line(self, x1: int, z1: int, x2: int, z2: int, tile_type: int, width: int = 1):
        """Draw a line between two points."""
        dx = abs(x2 - x1)
        dz = abs(z2 - z1)
        sx = 1 if x1 < x2 else -1
        sz = 1 if z1 < z2 else -1
        err = dx - dz
        
        x, z = x1, z1
        
        while True:
            # Draw with width
            for dw in range(-(width//2), (width//2) + 1):
                for dh in range(-(width//2), (width//2) + 1):
                    self.set_tile(x + dw, z + dh, tile_type)
            
            if x == x2 and z == z2:
                break
            
            e2 = 2 * err
            if e2 > -dz:
                err -= dz
                x += sx
            if e2 < dx:
                err += dx
                z += sz
    
    def add_noise(self, x: int, z: int, width: int, height: int, tile_type: int, density: float):
        """Add random noise pattern to an area."""
        for dz in range(height):
            for dx in range(width):
                if random.random() < density:
                    self.set_tile(x + dx, z + dz, tile_type)
    
    def copy_region(self, src_x: int, src_z: int, dst_x: int, dst_z: int, width: int, height: int):
        """Copy a region from one location to another."""
        # Create temporary storage
        temp_data = []
        for dz in range(height):
            row = []
            for dx in range(width):
                row.append(self.get_tile(src_x + dx, src_z + dz))
            temp_data.append(row)
        
        # Copy to destination
        for dz in range(height):
            for dx in range(width):
                self.set_tile(dst_x + dx, dst_z + dz, temp_data[dz][dx])
    
    def get_map_info(self) -> dict:
        """Get information about the current map."""
        tile_counts = {i: 0 for i in range(5)}
        
        for z in range(MAP_SIZE):
            for x in range(MAP_SIZE):
                tile_counts[self.map_data[z][x]] += 1
        
        total_tiles = MAP_SIZE * MAP_SIZE
        info = {
            'size': f"{MAP_SIZE}x{MAP_SIZE}",
            'total_tiles': total_tiles,
            'tile_counts': tile_counts,
            'tile_percentages': {tile: (count / total_tiles) * 100 for tile, count in tile_counts.items()}
        }
        
        return info
    
    def view_ascii(self, x: int = 0, z: int = 0, width: int = 80, height: int = 40) -> str:
        """Generate ASCII representation of a map region."""
        lines = []
        
        # Clamp to map bounds
        x = max(0, min(x, MAP_SIZE - 1))
        z = max(0, min(z, MAP_SIZE - 1))
        width = min(width, MAP_SIZE - x)
        height = min(height, MAP_SIZE - z)
        
        for dz in range(height):
            line = ""
            for dx in range(width):
                tile_type = self.get_tile(x + dx, z + dz)
                line += TileType.SYMBOLS.get(tile_type, "?")
            lines.append(line)
        
        return "\n".join(lines)

def main():
    parser = argparse.ArgumentParser(description="Fortune Game Map Editor", 
                                   formatter_class=argparse.RawDescriptionHelpFormatter,
                                   epilog=__doc__)
    
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Create command
    create_parser = subparsers.add_parser('create', help='Create a new map')
    create_parser.add_argument('map_file', help='Output map file path')
    
    # Info command
    info_parser = subparsers.add_parser('info', help='Show map information')
    info_parser.add_argument('map_file', help='Map file path')
    
    # Fill command
    fill_parser = subparsers.add_parser('fill', help='Fill rectangle with tile type')
    fill_parser.add_argument('map_file', help='Map file path')
    fill_parser.add_argument('tile_type', type=int, choices=[0,1,2,3,4], help='Tile type (0-4)')
    fill_parser.add_argument('x', type=int, nargs='?', default=0, help='X coordinate')
    fill_parser.add_argument('y', type=int, nargs='?', default=0, help='Y coordinate')
    fill_parser.add_argument('w', type=int, nargs='?', default=MAP_SIZE, help='Width')
    fill_parser.add_argument('h', type=int, nargs='?', default=MAP_SIZE, help='Height')
    
    # Circle command
    circle_parser = subparsers.add_parser('circle', help='Fill circle with tile type')
    circle_parser.add_argument('map_file', help='Map file path')
    circle_parser.add_argument('tile_type', type=int, choices=[0,1,2,3,4], help='Tile type (0-4)')
    circle_parser.add_argument('x', type=int, help='Center X coordinate')
    circle_parser.add_argument('y', type=int, help='Center Y coordinate')
    circle_parser.add_argument('radius', type=int, help='Circle radius')
    
    # Line command
    line_parser = subparsers.add_parser('line', help='Draw line')
    line_parser.add_argument('map_file', help='Map file path')
    line_parser.add_argument('tile_type', type=int, choices=[0,1,2,3,4], help='Tile type (0-4)')
    line_parser.add_argument('x1', type=int, help='Start X coordinate')
    line_parser.add_argument('y1', type=int, help='Start Y coordinate')
    line_parser.add_argument('x2', type=int, help='End X coordinate')
    line_parser.add_argument('y2', type=int, help='End Y coordinate')
    line_parser.add_argument('width', type=int, nargs='?', default=1, help='Line width')
    
    # Noise command
    noise_parser = subparsers.add_parser('noise', help='Add noise pattern')
    noise_parser.add_argument('map_file', help='Map file path')
    noise_parser.add_argument('tile_type', type=int, choices=[0,1,2,3,4], help='Tile type (0-4)')
    noise_parser.add_argument('density', type=float, help='Noise density (0.0-1.0)')
    noise_parser.add_argument('x', type=int, nargs='?', default=0, help='X coordinate')
    noise_parser.add_argument('y', type=int, nargs='?', default=0, help='Y coordinate')
    noise_parser.add_argument('w', type=int, nargs='?', default=MAP_SIZE, help='Width')
    noise_parser.add_argument('h', type=int, nargs='?', default=MAP_SIZE, help='Height')
    
    # View command
    view_parser = subparsers.add_parser('view', help='View ASCII representation')
    view_parser.add_argument('map_file', help='Map file path')
    view_parser.add_argument('x', type=int, nargs='?', default=0, help='X coordinate')
    view_parser.add_argument('y', type=int, nargs='?', default=0, help='Y coordinate')
    view_parser.add_argument('w', type=int, nargs='?', default=80, help='Width')
    view_parser.add_argument('h', type=int, nargs='?', default=40, help='Height')
    
    # Copy command
    copy_parser = subparsers.add_parser('copy', help='Copy region between maps')
    copy_parser.add_argument('src_map', help='Source map file')
    copy_parser.add_argument('dst_map', help='Destination map file')
    copy_parser.add_argument('src_x', type=int, help='Source X coordinate')
    copy_parser.add_argument('src_y', type=int, help='Source Y coordinate')
    copy_parser.add_argument('dst_x', type=int, help='Destination X coordinate')
    copy_parser.add_argument('dst_y', type=int, help='Destination Y coordinate')
    copy_parser.add_argument('w', type=int, help='Width')
    copy_parser.add_argument('h', type=int, help='Height')
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    editor = MapEditor()
    
    try:
        if args.command == 'create':
            if editor.create_new_map(args.map_file):
                print(f"Created new map: {args.map_file}")
            else:
                print("Failed to create map")
                sys.exit(1)
        
        elif args.command == 'info':
            if editor.load_map(args.map_file):
                info = editor.get_map_info()
                print(f"Map: {args.map_file}")
                print(f"Size: {info['size']}")
                print(f"Total tiles: {info['total_tiles']:,}")
                print("\nTile distribution:")
                for tile_type, count in info['tile_counts'].items():
                    name = TileType.NAMES[tile_type]
                    percentage = info['tile_percentages'][tile_type]
                    print(f"  {name:>6}: {count:>8,} ({percentage:>5.1f}%)")
            else:
                sys.exit(1)
        
        elif args.command == 'fill':
            if editor.load_map(args.map_file):
                editor.fill_rectangle(args.x, args.y, args.w, args.h, args.tile_type)
                if editor.save_map(args.map_file):
                    tile_name = TileType.NAMES[args.tile_type]
                    print(f"Filled rectangle ({args.x},{args.y}) {args.w}x{args.h} with {tile_name}")
                else:
                    print("Failed to save map")
                    sys.exit(1)
            else:
                sys.exit(1)
        
        elif args.command == 'circle':
            if editor.load_map(args.map_file):
                editor.fill_circle(args.x, args.y, args.radius, args.tile_type)
                if editor.save_map(args.map_file):
                    tile_name = TileType.NAMES[args.tile_type]
                    print(f"Filled circle at ({args.x},{args.y}) radius {args.radius} with {tile_name}")
                else:
                    print("Failed to save map")
                    sys.exit(1)
            else:
                sys.exit(1)
        
        elif args.command == 'line':
            if editor.load_map(args.map_file):
                editor.draw_line(args.x1, args.y1, args.x2, args.y2, args.tile_type, args.width)
                if editor.save_map(args.map_file):
                    tile_name = TileType.NAMES[args.tile_type]
                    print(f"Drew line from ({args.x1},{args.y1}) to ({args.x2},{args.y2}) with {tile_name}")
                else:
                    print("Failed to save map")
                    sys.exit(1)
            else:
                sys.exit(1)
        
        elif args.command == 'noise':
            if args.density < 0.0 or args.density > 1.0:
                print("Error: Density must be between 0.0 and 1.0")
                sys.exit(1)
            
            if editor.load_map(args.map_file):
                editor.add_noise(args.x, args.y, args.w, args.h, args.tile_type, args.density)
                if editor.save_map(args.map_file):
                    tile_name = TileType.NAMES[args.tile_type]
                    print(f"Added {tile_name} noise (density {args.density}) to rectangle ({args.x},{args.y}) {args.w}x{args.h}")
                else:
                    print("Failed to save map")
                    sys.exit(1)
            else:
                sys.exit(1)
        
        elif args.command == 'view':
            if editor.load_map(args.map_file):
                ascii_view = editor.view_ascii(args.x, args.y, args.w, args.h)
                print(f"Map view: ({args.x},{args.y}) {args.w}x{args.h}")
                print("Legend: . = Grass, ~ = Desert, ≈ = Water, ▲ = Rock, : = Sand")
                print(ascii_view)
            else:
                sys.exit(1)
        
        elif args.command == 'copy':
            # Load source map
            src_editor = MapEditor()
            if not src_editor.load_map(args.src_map):
                print(f"Failed to load source map: {args.src_map}")
                sys.exit(1)
            
            # Load or create destination map
            if os.path.exists(args.dst_map):
                if not editor.load_map(args.dst_map):
                    print(f"Failed to load destination map: {args.dst_map}")
                    sys.exit(1)
            else:
                print(f"Creating new destination map: {args.dst_map}")
            
            # Copy data from source to destination
            for dz in range(args.h):
                for dx in range(args.w):
                    tile = src_editor.get_tile(args.src_x + dx, args.src_y + dz)
                    editor.set_tile(args.dst_x + dx, args.dst_y + dz, tile)
            
            if editor.save_map(args.dst_map):
                print(f"Copied region from {args.src_map} to {args.dst_map}")
            else:
                print("Failed to save destination map")
                sys.exit(1)
    
    except KeyboardInterrupt:
        print("\nOperation cancelled")
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
