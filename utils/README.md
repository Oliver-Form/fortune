# Map to PNG Converter

This utility converts your game's `.map` files to PNG images for visualization.

## Building

From the `utils` directory, run:

```bash
cargo build --release
```

## Usage

### Option 1: Default paths
```bash
cargo run
```
This will convert `../src/file_checkers.map` to `map_visualization.png`

### Option 2: Specify input file only
```bash
cargo run -- path/to/your/file.map
```
This will create a PNG with the same name (e.g., `file.png`)

### Option 3: Specify both input and output
```bash
cargo run -- path/to/input.map path/to/output.png
```

## Examples

```bash
# Convert the default map file
cargo run

# Convert a specific map file
cargo run -- ../src/file.map my_map.png

# Convert with custom output name
cargo run -- ../src/file_checkers.map world_map.png
```

## Output

The utility will:
1. Load the binary map data
2. Display statistics about tile types and their distribution
3. Create a PNG image where each pixel represents one tile
4. Save the image with the specified filename

## Color Mapping

- **Grass**: Forest Green (RGB: 34, 139, 34)
- **Water**: Dodger Blue (RGB: 30, 144, 255)
- **Desert**: Peach Puff (RGB: 238, 203, 173)
- **Stone**: Gray (RGB: 128, 128, 128)
- **Wood**: Saddle Brown (RGB: 139, 69, 19)
- **Unknown**: Bright Magenta (RGB: 255, 0, 255) - for debugging

## Map Dimensions

The utility expects a map of 1000x1000 tiles (1,000,000 total tiles). If your map has different dimensions, update the constants `MAP_WIDTH` and `MAP_HEIGHT` in `src/main.rs`.

## File Format

The utility expects binary files where each tile is represented as a 2-byte little-endian unsigned integer (u16):
- 0 = Grass
- 1 = Water  
- 2 = Desert
- 3 = Stone
- 4 = Wood
- Any other value = Unknown
