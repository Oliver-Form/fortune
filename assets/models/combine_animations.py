#!/usr/bin/env python3
"""
Blender script to combine multiple FBX animations into a single GLB file
Usage: blender --background --python combine_animations.py
"""

import bpy
import os

def clear_scene():
    """Clear all objects from the scene"""
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete(use_global=False)

def import_base_model(filepath):
    """Import the base model with skin"""
    print(f"Importing base model: {filepath}")
    bpy.ops.import_scene.fbx(filepath=filepath)
    
    # Get the armature (skeleton)
    armature = None
    for obj in bpy.context.scene.objects:
        if obj.type == 'ARMATURE':
            armature = obj
            break
    
    if not armature:
        raise Exception("No armature found in base model!")
    
    return armature

def import_and_merge_animation(armature, filepath, animation_name):
    """Import an animation and add it to the armature"""
    print(f"Importing animation: {animation_name} from {filepath}")
    
    # Store current objects
    objects_before = set(bpy.context.scene.objects)
    
    # Import the animation FBX
    bpy.ops.import_scene.fbx(filepath=filepath)
    
    # Find the new armature from this import
    new_objects = set(bpy.context.scene.objects) - objects_before
    temp_armature = None
    
    for obj in new_objects:
        if obj.type == 'ARMATURE':
            temp_armature = obj
            break
    
    if not temp_armature:
        print(f"Warning: No armature found in {filepath}")
        return
    
    # Copy animation data from temp armature to main armature
    if temp_armature.animation_data and temp_armature.animation_data.action:
        action = temp_armature.animation_data.action.copy()
        action.name = animation_name
        
        # Ensure main armature has animation_data
        if not armature.animation_data:
            armature.animation_data_create()
        
        # Store the action in the blend file
        action.use_fake_user = True
        
        print(f"Successfully imported animation: {animation_name}")
    
    # Clean up temporary objects
    for obj in new_objects:
        bpy.data.objects.remove(obj, do_unlink=True)

def setup_nla_tracks(armature):
    """Set up NLA tracks for each animation"""
    if not armature.animation_data:
        return
    
    # Clear existing NLA tracks
    armature.animation_data.nla_tracks.clear()
    
    # Create NLA tracks for each action
    for action in bpy.data.actions:
        if action.name in ["idle", "walking", "running", "aiming", "shooting", "holster"]:
            track = armature.animation_data.nla_tracks.new()
            track.name = action.name
            
            # Add strip to track
            strip = track.strips.new(action.name, 1, action)
            strip.frame_start = 1
            strip.frame_end = action.frame_range[1]

def main():
    """Main function to combine animations"""
    # Define file paths (adjust these to match your downloaded files)
    base_dir = "/home/oli/git/fortune/assets/models"
    
    animation_files = {
        "idle": f"{base_dir}/mixamo_idle.fbx",        # With skin
        "walking": f"{base_dir}/mixamo_walking.fbx",   # Without skin
        "running": f"{base_dir}/mixamo_running.fbx",   # Without skin
        "aiming": f"{base_dir}/mixamo_aiming.fbx",     # Without skin
        "shooting": f"{base_dir}/mixamo_shooting.fbx", # Without skin
        "holster": f"{base_dir}/mixamo_holster.fbx",   # Without skin
    }
    
    # Check if files exist
    for name, filepath in animation_files.items():
        if not os.path.exists(filepath):
            print(f"Warning: File not found: {filepath}")
            print("Please download the FBX files from Mixamo and place them in assets/models/")
            return
    
    # Clear scene
    clear_scene()
    
    # Import base model (with skin)
    armature = import_base_model(animation_files["idle"])
    
    # Import all other animations
    for name, filepath in animation_files.items():
        if name != "idle":  # Skip idle as it's already imported
            import_and_merge_animation(armature, filepath, name)
    
    # Setup NLA tracks
    setup_nla_tracks(armature)
    
    # Select the armature and all its children
    bpy.ops.object.select_all(action='DESELECT')
    armature.select_set(True)
    
    # Select all mesh objects that are children of the armature
    for obj in bpy.context.scene.objects:
        if obj.type == 'MESH' and obj.parent == armature:
            obj.select_set(True)
    
    bpy.context.view_layer.objects.active = armature
    
    # Export as GLB
    output_path = f"{base_dir}/cowboy.glb"
    print(f"Exporting to: {output_path}")
    
    bpy.ops.export_scene.gltf(
        filepath=output_path,
        use_selection=True,
        export_animations=True,
        export_frame_range=False,
        export_nla_strips=True,
        export_def_bones=True,
        export_optimize_animation_size=False
    )
    
    print("Export complete!")
    print(f"Your cowboy.glb file is ready at: {output_path}")
    print("\nAnimation indices:")
    print("0: idle")
    print("1: walking") 
    print("2: running")
    print("3: aiming")
    print("4: shooting")
    print("5: holster")

if __name__ == "__main__":
    main()
