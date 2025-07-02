#!/usr/bin/env python3
"""
Blender script to combine multiple GLB/FBX files with animations into a single GLB file.
This script loads the cowboy character from the idle GLB and adds animations from other files.
"""

import bpy
import os
import sys

def clear_scene():
    """Clear all objects from the scene"""
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete(use_global=False)

def import_base_character(filepath):
    """Import the base character (with mesh and armature) from idle GLB"""
    if filepath.endswith('.glb'):
        bpy.ops.import_scene.gltf(filepath=filepath)
    elif filepath.endswith('.fbx'):
        bpy.ops.import_scene.fbx(filepath=filepath)
    
    # Find the armature
    armature = None
    for obj in bpy.context.scene.objects:
        if obj.type == 'ARMATURE':
            armature = obj
            break
    
    return armature

def import_animation_only(filepath, armature, animation_name):
    """Import animation from a file and apply it to the existing armature"""
    # Store current selection
    current_selection = bpy.context.selected_objects.copy()
    current_active = bpy.context.active_object
    
    # Import the animation file
    if filepath.endswith('.glb'):
        bpy.ops.import_scene.gltf(filepath=filepath)
    elif filepath.endswith('.fbx'):
        bpy.ops.import_scene.fbx(filepath=filepath)
    
    # Find the imported armature with animation
    imported_armature = None
    for obj in bpy.context.scene.objects:
        if obj.type == 'ARMATURE' and obj != armature:
            imported_armature = obj
            break
    
    if imported_armature and imported_armature.animation_data and imported_armature.animation_data.action:
        # Copy the action to our main armature
        action = imported_armature.animation_data.action
        action.name = animation_name
        
        # Create a copy of the action for our armature
        new_action = action.copy()
        new_action.name = animation_name
        
        # Store the action in the blend file
        new_action.use_fake_user = True
        
        print(f"Imported animation: {animation_name}")
    
    # Clean up imported objects except our main armature
    bpy.ops.object.select_all(action='DESELECT')
    for obj in bpy.context.scene.objects:
        if obj != armature and obj.type in ['ARMATURE', 'MESH']:
            if obj.parent != armature:  # Don't delete children of our main armature
                obj.select_set(True)
    
    bpy.ops.object.delete()
    
    # Restore selection
    bpy.ops.object.select_all(action='DESELECT')
    for obj in current_selection:
        if obj.name in bpy.context.scene.objects:
            bpy.context.scene.objects[obj.name].select_set(True)
    
    if current_active and current_active.name in bpy.context.scene.objects:
        bpy.context.view_layer.objects.active = bpy.context.scene.objects[current_active.name]

def combine_animations():
    """Main function to combine all cowboy animations"""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Animation files mapping
    animations = {
        "Idle": "cowboy_idle.glb",
        "Walking": "cowboy_walking.glb", 
        "Shooting": "cowboy_shooting.glb",
        "Aiming": "cowboy_aiming.glb",
        "Running": "Running.fbx",
        "Holster": "Pistol Aim.fbx"
    }
    
    # Clear the scene
    clear_scene()
    
    # Import base character from idle animation
    base_file = os.path.join(script_dir, animations["Idle"])
    print(f"Loading base character from: {base_file}")
    armature = import_base_character(base_file)
    
    if not armature:
        print("ERROR: Could not find armature in base file!")
        return
    
    print(f"Found armature: {armature.name}")
    
    # Import animations from other files
    for anim_name, filename in animations.items():
        if anim_name == "Idle":
            # Rename the existing action
            if armature.animation_data and armature.animation_data.action:
                armature.animation_data.action.name = "Idle"
                armature.animation_data.action.use_fake_user = True
            continue
            
        filepath = os.path.join(script_dir, filename)
        if os.path.exists(filepath):
            print(f"Importing {anim_name} from {filename}")
            import_animation_only(filepath, armature, anim_name)
        else:
            print(f"WARNING: File not found: {filepath}")
    
    # List all actions (animations) in the blend file
    print("\nAvailable animations:")
    for action in bpy.data.actions:
        print(f"  - {action.name}")
    
    # Select all objects for export
    bpy.ops.object.select_all(action='SELECT')
    
    # Export as GLB
    output_path = os.path.join(script_dir, "cowboy_combined.glb")
    print(f"\nExporting to: {output_path}")
    
    bpy.ops.export_scene.gltf(
        filepath=output_path,
        check_existing=False,
        export_format='GLB',
        export_animations=True,
        export_frame_range=False,
        export_nla_strips=False,
        export_def_bones=True,
        export_optimize_animation_size=False,
        export_anim_slide_to_zero=False,
        export_bake_animation=False
    )
    
    print(f"SUCCESS: Combined GLB saved to {output_path}")
    print("\nAnimation indices in the GLB file:")
    for i, action in enumerate(bpy.data.actions):
        print(f"  Animation{i}: {action.name}")

if __name__ == "__main__":
    combine_animations()
