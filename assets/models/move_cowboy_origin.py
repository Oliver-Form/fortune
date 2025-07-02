import bpy
import sys

# Import the GLB file (replace with your filename if needed)
bpy.ops.import_scene.gltf(filepath="cowboy_combined.glb")

# Move all mesh and armature objects down by 0.5 units on Z (Blender up axis)
for obj in bpy.data.objects:
    if obj.type in {'MESH', 'ARMATURE'}:
        obj.location[2] -= 0.5

# Export the modified scene as a new GLB
bpy.ops.export_scene.gltf(filepath="cowboy_combined_fixed.glb", export_format='GLB')
