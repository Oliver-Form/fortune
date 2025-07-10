use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;
use crate::{components::*, constants::*, resources::*};

pub fn update_explored_chunks(
    player_query: Query<&Transform, With<Player>>,
    mut world_data: ResMut<WorldData>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let chunk_x = (player_transform.translation.x / (CHUNK_SIZE as f32 * TILE_SIZE)) as i32;
        let chunk_z = (player_transform.translation.z / (CHUNK_SIZE as f32 * TILE_SIZE)) as i32;
        let current_chunk = IVec2::new(chunk_x, chunk_z);

        world_data.explored_chunks.insert(current_chunk);
    }
}

pub fn manage_world_chunks(
    player_query: Query<&Transform, With<Player>>,
    mut world_data: ResMut<WorldData>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunk_entities: Query<(Entity, &ChunkEntity)>,
    chunk_border_visible: Res<ChunkBorderVisible>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let player_chunk = IVec2::new(
            (player_transform.translation.x / (CHUNK_SIZE as f32 * TILE_SIZE)) as i32,
            (player_transform.translation.z / (CHUNK_SIZE as f32 * TILE_SIZE)) as i32,
        );

        let max_chunk_x = MAP_WIDTH / CHUNK_SIZE;
        let max_chunk_y = MAP_HEIGHT / CHUNK_SIZE;

        let mut chunks_to_keep = std::collections::HashSet::new();

        // Generate and spawn chunks around player
        for x in -RENDER_DISTANCE..=RENDER_DISTANCE {
            for z in -RENDER_DISTANCE..=RENDER_DISTANCE {
                let chunk_pos = player_chunk + IVec2::new(x, z);
                chunks_to_keep.insert(chunk_pos);

                // Check if chunk is within world boundaries
                if chunk_pos.x >= 0
                    && chunk_pos.x < max_chunk_x
                    && chunk_pos.y >= 0
                    && chunk_pos.y < max_chunk_y
                {
                    if !world_data.chunks.contains_key(&chunk_pos) {
                        if let Some(chunk_data) = generate_chunk_from_map(&world_data, chunk_pos) {
                            spawn_chunk(&chunk_data, &mut commands, &mut meshes, &mut materials, &chunk_border_visible);
                            world_data.chunks.insert(chunk_pos, chunk_data);
                        }
                    }
                }
            }
        }

        // Despawn chunks that are too far away
        for (entity, chunk_comp) in chunk_entities.iter() {
            if !chunks_to_keep.contains(&chunk_comp.position) {
                commands.entity(entity).despawn_recursive();
                world_data.chunks.remove(&chunk_comp.position);
            }
        }
    }
}

pub fn toggle_chunk_borders(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut chunk_border_visible: ResMut<ChunkBorderVisible>,
) {
    if keyboard_input.just_pressed(KeyCode::F3) {
        chunk_border_visible.0 = !chunk_border_visible.0;
    }
}

pub fn get_tile_at_position(world_data: &WorldData, x: i32, y: i32) -> TileType {
    if x >= 0 && x < MAP_WIDTH && y >= 0 && y < MAP_HEIGHT {
        let index = (y * MAP_WIDTH + x) as usize;
        if index < world_data.tiles.len() {
            return world_data.tiles[index];
        }
    }
    TileType::Unknown
}

fn generate_chunk_from_map(world_data: &WorldData, chunk_pos: IVec2) -> Option<ChunkData> {
    let mut tiles = [[TileType::Unknown; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
    let start_x = chunk_pos.x * CHUNK_SIZE;
    let start_y = chunk_pos.y * CHUNK_SIZE;

    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let world_x = start_x + x;
            let world_y = start_y + y;
            tiles[y as usize][x as usize] = get_tile_at_position(world_data, world_x, world_y);
        }
    }

    Some(ChunkData {
        position: chunk_pos,
        tiles,
    })
}

fn spawn_chunk(
    chunk_data: &ChunkData,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk_border_visible: &Res<ChunkBorderVisible>,
) {
    let chunk_world_x = chunk_data.position.x as f32 * CHUNK_SIZE as f32 * TILE_SIZE;
    let chunk_world_z = chunk_data.position.y as f32 * CHUNK_SIZE as f32 * TILE_SIZE;

    // --- Single Mesh with Vertex Colors ---
    // This approach creates one single mesh for the entire chunk.
    // Each tile is a quad with its vertices colored according to the tile type.
    // This is extremely performant as it results in only one draw call per chunk.
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();
    let mut vertex_count: u32 = 0;

    for y in 0..CHUNK_SIZE as usize {
        for x in 0..CHUNK_SIZE as usize {
            let tile_type = chunk_data.tiles[y][x];

            if tile_type == TileType::Water {
                continue; // Skip rendering water tiles
            }

            let tile_color = tile_type.get_color().as_rgba_f32();
            let x_pos = x as f32 * TILE_SIZE;
            let z_pos = y as f32 * TILE_SIZE;

            // Define the 4 vertices of the quad, with correct winding order
            positions.extend([
                // Bottom-left
                [x_pos, 0.0, z_pos],
                // Bottom-right
                [x_pos + TILE_SIZE, 0.0, z_pos],
                // Top-right
                [x_pos + TILE_SIZE, 0.0, z_pos + TILE_SIZE],
                // Top-left
                [x_pos, 0.0, z_pos + TILE_SIZE],
            ]);

            // All vertices of a tile have the same color and normal
            for _ in 0..4 {
                normals.push([0.0, 1.0, 0.0]);
                colors.push(tile_color);
            }

            // Add UVs for the quad
            uvs.extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);

            // Add indices for the two triangles of the quad, with correct winding order
            indices.extend([
                vertex_count,
                vertex_count + 2,
                vertex_count + 1,
                vertex_count,
                vertex_count + 3,
                vertex_count + 2,
            ]);

            vertex_count += 4;
        }
    }

    if positions.is_empty() {
        // Spawn an empty parent entity even for water-only chunks to track them
        commands.spawn((
            SpatialBundle {
                transform: Transform::from_xyz(chunk_world_x, 0.0, chunk_world_z),
                ..default()
            },
            ChunkEntity {
                position: chunk_data.position,
            },
        ));
        return;
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));

    let mesh_handle = meshes.add(mesh);

    // Use a single material that respects vertex colors.
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE, // Set to white to not tint the vertex colors
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    });

    // Create a parent entity for the whole chunk and add the mesh as a child
    let parent_chunk_entity = commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(chunk_world_x, 0.0, chunk_world_z),
                ..default()
            },
            ChunkEntity {
                position: chunk_data.position,
            },
        ))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: mesh_handle,
                material: material.clone(),
                ..default()
            });
        })
        .id();

    // Draw chunk border if enabled
    if chunk_border_visible.0 {
        let border_color = Color::BLACK;
        let border_thickness = 0.05;
        let border_length = CHUNK_SIZE as f32 * TILE_SIZE;
        let y = 0.01; // Slightly above ground to avoid z-fighting

        let border_material = materials.add(StandardMaterial::from(border_color));

        let borders = [
            // Top
            (
                Vec3::new(border_length / 2.0, y, 0.0),
                Quat::IDENTITY,
                Vec3::new(border_length, border_thickness, border_thickness),
            ),
            // Bottom
            (
                Vec3::new(border_length / 2.0, y, border_length),
                Quat::IDENTITY,
                Vec3::new(border_length, border_thickness, border_thickness),
            ),
            // Left
            (
                Vec3::new(0.0, y, border_length / 2.0),
                Quat::IDENTITY,
                Vec3::new(border_thickness, border_thickness, border_length),
            ),
            // Right
            (
                Vec3::new(border_length, y, border_length / 2.0),
                Quat::IDENTITY,
                Vec3::new(border_thickness, border_thickness, border_length),
            ),
        ];

        let border_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        commands.entity(parent_chunk_entity).with_children(|parent| {
            for (translation, _rotation, scale) in borders.iter() {
                parent.spawn(PbrBundle {
                    mesh: border_mesh.clone(),
                    material: border_material.clone(),
                    transform: Transform {
                        translation: *translation,
                        scale: *scale,
                        ..default()
                    },
                    ..default()
                });
            }
        });
    }

    // Add decorations
    spawn_chunk_decorations(
        chunk_data,
        commands,
        meshes,
        materials,
        parent_chunk_entity,
    );
}

fn spawn_chunk_decorations(
    chunk_data: &ChunkData,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent_chunk: Entity,
) {
    let cactus_mesh = meshes.add(Cuboid::new(0.2, 1.5, 0.2));
    let tree_mesh = meshes.add(Cuboid::new(0.3, 2.0, 0.3));
    let enemy_mesh = meshes.add(Cuboid::new(0.8, 1.8, 0.8));
    let cactus_material = materials.add(Color::rgb(0.2, 0.7, 0.2));
    let tree_material = materials.add(Color::rgb(0.4, 0.2, 0.1));
    let enemy_material = materials.add(Color::rgb(0.6, 0.2, 0.8)); // Purple enemies

    // Only add a few decorations per chunk to keep performance good
    for i in 0..CHUNK_SIZE as usize {
        for j in 0..CHUNK_SIZE as usize {
            let tile_type = chunk_data.tiles[j][i];

            // Use a hash of the position for deterministic "randomness"
            let mut hasher = DefaultHasher::new();
            hasher.write_i32(chunk_data.position.x * CHUNK_SIZE + i as i32);
            hasher.write_i32(chunk_data.position.y * CHUNK_SIZE + j as i32);
            let hash = hasher.finish();

            let world_x = (chunk_data.position.x * CHUNK_SIZE + i as i32) as f32 * TILE_SIZE;
            let world_z = (chunk_data.position.y * CHUNK_SIZE + j as i32) as f32 * TILE_SIZE;

            // Spawn enemies sporadically across all biomes (except water)
            if tile_type != TileType::Water && (hash % 200) == 0 {
                let enemy_entity = commands
                    .spawn((
                        PbrBundle {
                            mesh: enemy_mesh.clone(),
                            material: enemy_material.clone(),
                            transform: Transform::from_xyz(world_x, 0.9, world_z),
                            ..default()
                        },
                        Enemy,
                    ))
                    .id();
                commands.entity(parent_chunk).add_child(enemy_entity);
            }

            match tile_type {
                TileType::Desert => {
                    if (hash % 50) == 0 {
                        // More frequent cacti
                        let cactus_entity = commands
                            .spawn((
                                PbrBundle {
                                    mesh: cactus_mesh.clone(),
                                    material: cactus_material.clone(),
                                    transform: Transform::from_xyz(world_x, 0.75, world_z),
                                    ..default()
                                },
                                Cactus,
                            ))
                            .id();
                        commands.entity(parent_chunk).add_child(cactus_entity);
                    }
                }
                TileType::Grass => {
                    if (hash % 100) == 0 {
                        // Less frequent trees
                        let tree_entity = commands
                            .spawn((
                                PbrBundle {
                                    mesh: tree_mesh.clone(),
                                    material: tree_material.clone(),
                                    transform: Transform::from_xyz(world_x, 1.0, world_z),
                                    ..default()
                                },
                                Tree,
                            ))
                            .id();
                        commands.entity(parent_chunk).add_child(tree_entity);
                    }
                }
                _ => {} // No decorations for other types
            }
        }
    }
}
