use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use image::{ImageBuffer, Rgb, RgbImage};

// Constants matching your game constants
const MAP_WIDTH: i32 = 1000;
const MAP_HEIGHT: i32 = 1000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TileType {
    Grass,    // 0
    Water,    // 1
    Desert,   // 2
    Stone,    // 3
    Wood,     // 4
    Unknown,  // fallback
}

impl TileType {
    fn from_u16(value: u16) -> Self {
        match value {
            0 => TileType::Grass,
            1 => TileType::Water,
            2 => TileType::Desert,
            3 => TileType::Stone,
            4 => TileType::Wood,
            _ => TileType::Unknown,
        }
    }

    fn to_rgb(&self) -> [u8; 3] {
        match self {
            TileType::Grass => [34, 139, 34],     // Forest green
            TileType::Water => [30, 144, 255],    // Dodger blue
            TileType::Desert => [238, 203, 173],  // Peach puff
            TileType::Stone => [128, 128, 128],   // Gray
            TileType::Wood => [139, 69, 19],      // Saddle brown
            TileType::Unknown => [255, 0, 255],   // Bright magenta for debugging
        }
    }
}

fn load_map_data<P: AsRef<Path>>(path: P) -> Result<Vec<TileType>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut tiles = Vec::new();
    let mut buffer = [0; 2]; // Read 2 bytes for u16

    while reader.read_exact(&mut buffer).is_ok() {
        let tile_value = u16::from_le_bytes(buffer);
        tiles.push(TileType::from_u16(tile_value));
    }

    println!("Loaded {} tiles from map file", tiles.len());
    
    // Validate we have the expected number of tiles
    let expected_tiles = (MAP_WIDTH * MAP_HEIGHT) as usize;
    if tiles.len() != expected_tiles {
        println!("Warning: Expected {} tiles, but got {}", expected_tiles, tiles.len());
    }

    Ok(tiles)
}

fn create_png_from_tiles(tiles: &[TileType], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let width = MAP_WIDTH as u32;
    let height = MAP_HEIGHT as u32;
    
    // Create a new RGB image
    let mut img: RgbImage = ImageBuffer::new(width, height);
    
    // Fill the image with tile colors
    for (i, tile) in tiles.iter().enumerate() {
        let x = (i as u32) % width;
        let y = (i as u32) / width;
        
        if y < height {
            let rgb = tile.to_rgb();
            let pixel = Rgb([rgb[0], rgb[1], rgb[2]]);
            img.put_pixel(x, y, pixel);
        }
    }
    
    // Save the image
    img.save(output_path)?;
    println!("Successfully saved map as PNG: {}", output_path);
    
    Ok(())
}

fn print_tile_statistics(tiles: &[TileType]) {
    let mut counts = std::collections::HashMap::new();
    
    for tile in tiles {
        *counts.entry(*tile).or_insert(0) += 1;
    }
    
    println!("\nTile Statistics:");
    println!("================");
    for (tile_type, count) in counts.iter() {
        let percentage = (*count as f64 / tiles.len() as f64) * 100.0;
        println!("{:?}: {} tiles ({:.2}%)", tile_type, count, percentage);
    }
    println!("Total tiles: {}", tiles.len());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    let (input_path, output_path) = if args.len() >= 3 {
        (args[1].clone(), args[2].clone())
    } else if args.len() == 2 {
        let input = args[1].clone();
        let output = input.replace(".map", ".png");
        (input, output)
    } else {
        // Default paths
        ("../src/file_checkers.map".to_string(), "map_visualization.png".to_string())
    };
    
    println!("Converting map file to PNG...");
    println!("Input: {}", input_path);
    println!("Output: {}", output_path);
    println!("Expected dimensions: {}x{}", MAP_WIDTH, MAP_HEIGHT);
    
    // Load the map data
    let tiles = load_map_data(&input_path)?;
    
    // Print statistics about the tiles
    print_tile_statistics(&tiles);
    
    // Create the PNG image
    create_png_from_tiles(&tiles, &output_path)?;
    
    println!("\nConversion completed successfully!");
    println!("You can now view the map visualization at: {}", output_path);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tile_type_conversion() {
        assert_eq!(TileType::from_u16(0), TileType::Grass);
        assert_eq!(TileType::from_u16(1), TileType::Water);
        assert_eq!(TileType::from_u16(2), TileType::Desert);
        assert_eq!(TileType::from_u16(3), TileType::Stone);
        assert_eq!(TileType::from_u16(4), TileType::Wood);
        assert_eq!(TileType::from_u16(999), TileType::Unknown);
    }
    
    #[test]
    fn test_tile_colors() {
        assert_eq!(TileType::Grass.to_rgb(), [34, 139, 34]);
        assert_eq!(TileType::Water.to_rgb(), [30, 144, 255]);
        assert_eq!(TileType::Desert.to_rgb(), [238, 203, 173]);
    }
}
