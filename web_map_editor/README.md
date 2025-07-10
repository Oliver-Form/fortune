# Fortune Web Map Editor

A web-based map editor for Fortune game maps (.map files).

## Features

- **Visual Editing**: Paint tiles directly on a visual map representation
- **Multiple Tools**: Paint brush, flood fill, circle, line, and eyedropper tools
- **Tile Types**: Support for all 5 tile types (Grass, Desert, Water, Rock, Sand)
- **Zoom & Pan**: Navigate large maps with smooth zooming and panning
- **Minimap**: Overview of the entire map with viewport indicator
- **File Operations**: Load existing .map files, save changes, create new maps
- **Export**: Export maps as PNG images for sharing or reference
- **Keyboard Shortcuts**: Quick access to tools and tile types

## Usage

### Opening the Editor

1. Open `index.html` in a web browser
2. The editor will start with a blank grass map (4096x4096)

### Tools

- **🖌️ Paint (B)**: Paint individual tiles with the selected tile type
- **🪣 Fill (F)**: Flood fill connected areas with the selected tile type
- **⭕ Circle (C)**: Draw circles (coming soon)
- **📏 Line (L)**: Draw lines (coming soon)
- **💧 Eyedropper (E)**: Pick tile type from the map

### Tile Types

- **1 - Grass**: Green terrain (default)
- **2 - Desert**: Sandy brown terrain
- **3 - Water**: Blue terrain
- **4 - Rock**: Gray terrain
- **5 - Sand**: Light brown terrain

### Controls

- **Mouse Wheel**: Zoom in/out
- **Left Click**: Use selected tool
- **Right Click**: Pan (coming soon)
- **Brush Size Slider**: Adjust paint brush size (1-20 tiles)
- **Zoom Slider**: Manual zoom control (10%-500%)

### File Operations

- **Load Map**: Click "Load Map" and select a .map file
- **Save Map**: Click "Save Map" to download the current map
- **New Map**: Click "New Map" to create a blank map
- **Export PNG**: Click "Export PNG" to save as image

### Keyboard Shortcuts

- **1-5**: Select tile types (Grass, Desert, Water, Rock, Sand)
- **B**: Paint tool
- **F**: Fill tool
- **C**: Circle tool
- **L**: Line tool
- **E**: Eyedropper tool

## Map Format

The editor works with the same .map format as the Fortune game:
- Binary format with little-endian u16 values
- 4096x4096 tiles (16,777,216 total tiles)
- Each tile is 2 bytes representing the tile type

## Tips

- Use the minimap to quickly navigate large maps
- Zoom in for precise editing, zoom out for overview
- Use flood fill for quickly filling large areas
- The eyedropper tool is great for sampling existing terrain
- Export PNG for sharing maps or getting an overview

## Browser Compatibility

- Modern browsers with Canvas 2D support
- Chrome, Firefox, Safari, Edge (latest versions)
- File API support required for loading/saving maps
