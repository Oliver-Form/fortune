from PIL import Image

# Atlas settings (must match your code)
TILE_SIZE = 32
ATLAS_COLS = 4
ATLAS_ROWS = 2

# Colors for each tile type (order: grass, water, desert, stone, wood, unknown, ...)
TILE_COLORS = [
    (80, 200, 80),    # Grass (green)
    (60, 120, 220),   # Water (blue)
    (220, 200, 120),  # Desert (tan)
    (120, 120, 120),  # Stone (gray)
    (160, 80, 40),    # Wood (brown)
    (255, 0, 255),    # Unknown (magenta)
    (0, 0, 0),        # Extra (black)
    (255, 255, 255),  # Extra (white)
]

atlas = Image.new("RGB", (TILE_SIZE * ATLAS_COLS, TILE_SIZE * ATLAS_ROWS))

for idx, color in enumerate(TILE_COLORS):
    x = idx % ATLAS_COLS
    y = idx // ATLAS_COLS
    if y >= ATLAS_ROWS:
        break
    tile = Image.new("RGB", (TILE_SIZE, TILE_SIZE), color)
    atlas.paste(tile, (x * TILE_SIZE, y * TILE_SIZE))

atlas.save("tiles_atlas.png")
print("Generated tiles_atlas.png with solid colors.")
