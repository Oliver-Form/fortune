# 🌍 Fortune - Open World Game Engine

A lightweight, modular open world game engine built with Rust and Bevy for creating expansive 3D worlds with efficient chunk-based loading.

## 🚀 Core Features

- **Massive World Support**: 4096×4096 tile worlds with chunk-based streaming
- **Multi-Biome System**: Grass, Desert, Water, Rock, Sand terrain types
- **3D Character System**: Skeletal animation with state machine
- **Dynamic Camera**: Isometric and free-look camera modes
- **Visual Map Editor**: Web-based world creation tools
- **Binary Map Format**: Optimized world storage and loading

## 🏗️ Architecture

Built on Bevy's ECS with modular design:
```
src/
├── world.rs        # Chunk management & biomes
├── player.rs       # Character controller & animation
├── camera.rs       # Camera systems
├── combat.rs       # Weapon & shooting mechanics
├── ui.rs          # Menus & HUD
└── main.rs        # App setup
```

## 🛠️ Development Tools

### Web Map Editor
```bash
cd web_map_editor_fast
firefox index.html
```
- Real-time tile painting
- Chunk-optimized rendering
- Import/export .map files

### CLI Tools
```bash
# Create maps programmatically
./map_editor.py create world.map
./map_editor.py fill world.map grass 0 0 1000 1000

# Convert images to maps
./image_to_map.py sketch.png world.map
```

## 🚀 Quick Start

```bash
# Clone and run
git clone https://github.com/yourusername/fortune.git
cd fortune
cargo run --release
```

### Controls
- **WASD** - Move
- **Mouse** - Look/Aim
- **ESC** - Pause menu
- **M** - Toggle map

## 🎯 Perfect For

- Open world RPGs
- Survival games
- Exploration adventures
- Large-scale simulations

---

**Build infinite worlds with Fortune Engine 🌍**