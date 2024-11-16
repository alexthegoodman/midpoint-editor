use midpoint_engine::animations::motion_path::{
    AnimationPlayback, SkeletonKeyframe, SkeletonMotionPath,
};
use midpoint_engine::floem::common::small_button;
use midpoint_engine::floem::reactive::{
    create_effect, create_rw_signal, RwSignal, SignalGet, SignalUpdate,
};
use midpoint_engine::floem::text::Weight;
use midpoint_engine::floem::views::h_stack;
use midpoint_engine::helpers::saved_data::{self, SavedState};
use nalgebra::{Quaternion, UnitQuaternion};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex, MutexGuard};

use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::floem::common::card_styles;
use midpoint_engine::floem::views::{container, dyn_container, dyn_stack, empty, label, v_stack};
use wgpu::util::DeviceExt;

use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{GpuHelper, IntoView, View, WindowHandle};

use crate::editor_state::StateHelper;
use crate::helpers::animations::{AnimationData, KeyframeValue, UIKeyframe};
use crate::helpers::utilities::parse_string_to_float;

use super::inputs::styled_input;

pub fn refresh_animation_data(
    saved_state: &MutexGuard<SavedState>,
    renderable_paths: Vec<SkeletonMotionPath>,
    selected_skeleton_id_signal: RwSignal<String>,
    motion_paths_signal: RwSignal<Vec<SkeletonMotionPath>>,
    animation_data_signal: RwSignal<Option<AnimationData>>,
) {
    let relevant_paths: Vec<SkeletonMotionPath> = renderable_paths
        .iter()
        .filter(|mp| mp.target.skeleton_id == selected_skeleton_id_signal.get())
        .map(|mp| mp.clone())
        .collect();

    let animations = AnimationData::from_motion_paths(relevant_paths.clone());

    motion_paths_signal.set(relevant_paths);
    animation_data_signal.set(Some(animations));
}

// pub fn update_position(
//     // state_helper: Arc<Mutex<StateHelper>>,
//     mut state_helper: MutexGuard<StateHelper>, // may be truly unecessary as guard here
//     active_keyframe_data: RwSignal<Option<UIKeyframe>>,
//     active_skel_keyframe_data: RwSignal<Option<SkeletonKeyframe>>,
//     value: String,
//     axis: &str,
//     selected_skeleton_id_signal: RwSignal<String>,
//     motion_paths_signal: RwSignal<Vec<SkeletonMotionPath>>,
//     animation_data_signal: RwSignal<Option<AnimationData>>,
//     active_position: RwSignal<[f32; 3]>,
//     selected_keyframes: RwSignal<Vec<UIKeyframe>>,
// ) {
//     println!("running update_position");
//     // Check if we have selected keyframes first
//     if selected_keyframes.get().is_empty() {
//         return; // or handle empty case
//     }

//     println!("updating keyframe position");
//     // let mut renderer_state = state_helper
//     //     .renderer_state
//     //     .as_mut()
//     //     .expect("Couldn't get RendererState");
//     // let mut renderer_state = renderer_state.lock().unwrap();

//     // let current_position = renderer_state.translation_gizmo.transform.position;

//     let active_keyframe = active_keyframe_data
//         .get()
//         .expect("Couldn't get active keyframe");
//     let active_skel_keyframe = active_skel_keyframe_data
//         .get()
//         .expect("Couldn't get active keyframe");

//     // for X only
//     let mut new_position = active_skel_keyframe.base.position.clone();
//     println!("parsing value {:?}", value);
//     let parsed_value = parse_string_to_float(&value);

//     if parsed_value.is_none() {
//         println!("invalid input");
//         return;
//     }

//     let parsed_value = parsed_value.expect("Couldn't get parsed value");

//     match axis {
//         "x" => new_position[0] = parsed_value,
//         "y" => new_position[1] = parsed_value,
//         "z" => new_position[2] = parsed_value,
//         _ => println!("not real axis"),
//     }

//     let new_position = [new_position[0], new_position[1], new_position[2]];

//     // update gizmo transform position
//     // renderer_state
//     //     .translation_gizmo
//     //     .transform
//     //     .update_position(new_position);

//     // update_arrow_collider_position
//     // renderer_state.update_arrow_collider_position(new_position);

//     println!("new_position {:?}", new_position);

//     active_position.set(new_position);

//     // update renderer_state.selected_object_data with new ComponentData
//     let mut new_skel_data = active_skel_keyframe.clone();
//     let mut new_data = active_keyframe.clone();

//     // new_data.generic_properties.position = new_position;
//     new_skel_data.base.position = new_position;
//     // for now, set base and ik as same, but maybe get rid of base?
//     new_skel_data.ik_target_position = Some(new_position);
//     new_data.value = KeyframeValue::Position(new_position);

//     active_skel_keyframe_data.set(Some(new_skel_data.clone()));
//     active_keyframe_data.set(Some(new_data.clone()));

//     // drop(renderer_state);

//     // // save in saved_state
//     // let mut saved_state = state_helper
//     //     .saved_state
//     //     .as_mut()
//     //     .expect("Couldn't get saved state")
//     //     .lock()
//     //     .unwrap();

//     // // Update the component within the saved state
//     // let motion_paths = &mut saved_state.motion_paths;

//     // if let Some(mutable_keyframe) = motion_paths.iter_mut().find_map(|mp| {
//     //     mp.keyframes
//     //         .iter_mut()
//     //         .find(|kf| kf.id == active_keyframe.skel_key_id)
//     // }) {
//     //     mutable_keyframe.base.position = new_position;
//     //     mutable_keyframe.ik_target_position = Some(new_position);
//     // }

//     // let renderable_paths = motion_paths.clone();

//     // drop(saved_state);

//     // save in saved_state
//     let mut saved_state = state_helper
//         .saved_state
//         .as_mut()
//         .expect("Couldn't get saved state")
//         .lock()
//         .unwrap();

//     // Update the component within the saved state
//     let motion_paths = &mut saved_state.motion_paths;

//     // Find and update the correct motion path
//     for motion_path in motion_paths.iter_mut() {
//         if let Some(keyframe) = motion_path
//             .keyframes
//             .iter_mut()
//             .find(|kf| kf.id == active_keyframe.skel_key_id)
//         {
//             keyframe.base.position = new_position;
//             keyframe.ik_target_position = Some(new_position);
//             println!("set key prop {:?}", new_position);
//             break; // Exit once we've found and updated the keyframe
//         }
//     }

//     let renderable_paths = motion_paths.clone();

//     drop(saved_state);

//     let project_id = state_helper
//         .project_selected_signal
//         .expect("Couldn't get project signal")
//         .get();

//     let saved_state = state_helper
//         .saved_state
//         .as_ref()
//         .expect("Couldn't get saved state")
//         .lock()
//         .unwrap();

//     // println!(
//     //     "renderable_paths {:?} {:?}",
//     //     renderable_paths, saved_state.motion_paths
//     // );

//     // Save the updated state
//     state_helper.save_saved_state_raw(project_id, saved_state.clone());

//     drop(saved_state);

//     let mut renderer_state = state_helper
//         .renderer_state
//         .as_mut()
//         .expect("Couldn't get RendererState");
//     let mut renderer_state = renderer_state.lock().unwrap();

//     // update rendererstate for visuals
//     let animation_playback = AnimationPlayback::new(renderable_paths.clone());
//     let mut current_animations = Vec::new();
//     current_animations.push(animation_playback);
//     renderer_state.active_animations = current_animations;

//     println!("Inserted AnimationPlayback!");

//     drop(renderer_state);

//     let saved_state = state_helper
//         .saved_state
//         .as_ref()
//         .expect("Couldn't get saved state")
//         .lock()
//         .unwrap();

//     refresh_animation_data(
//         &saved_state,
//         renderable_paths.clone(),
//         selected_skeleton_id_signal,
//         motion_paths_signal,
//         animation_data_signal,
//     );

//     drop(saved_state);
//     drop(state_helper);

//     let mut new_selections = Vec::new();
//     new_selections.push(new_data.clone());
//     selected_keyframes.set(new_selections);
// }

// pub fn update_rotation(
//     // state_helper: Arc<Mutex<StateHelper>>,
//     mut state_helper: MutexGuard<StateHelper>, // may be truly unecessary as guard here
//     active_keyframe_data: RwSignal<Option<UIKeyframe>>,
//     active_skel_keyframe_data: RwSignal<Option<SkeletonKeyframe>>,
//     value: String,
//     axis: &str,
//     selected_skeleton_id_signal: RwSignal<String>,
//     motion_paths_signal: RwSignal<Vec<SkeletonMotionPath>>,
//     animation_data_signal: RwSignal<Option<AnimationData>>,
//     active_rotation: RwSignal<[f32; 3]>,
//     selected_keyframes: RwSignal<Vec<UIKeyframe>>,
// ) {
//     println!("updating keyframe rotation");
//     let mut renderer_state = state_helper
//         .renderer_state
//         .as_mut()
//         .expect("Couldn't get RendererState");
//     let mut renderer_state = renderer_state.lock().unwrap();

//     // let current_position = renderer_state.translation_gizmo.transform.position;

//     let active_keyframe = active_keyframe_data
//         .get()
//         .expect("Couldn't get active keyframe");
//     let active_skel_keyframe = active_skel_keyframe_data
//         .get()
//         .expect("Couldn't get active keyframe");

//     // for X only
//     let mut new_rotation = active_skel_keyframe.base.rotation.clone();
//     println!("parsing value {:?}", value);
//     let parsed_value = parse_string_to_float(&value);

//     if parsed_value.is_none() {
//         println!("invalid input");
//         return;
//     }

//     let parsed_value = parsed_value.expect("Couldn't get parsed value");

//     // let degrees = parsed_value.parse::<f32>().unwrap_or(0.0);
//     let degrees = parsed_value;
//     let radians = degrees * PI / 180.0;

//     let current_quat = UnitQuaternion::from_quaternion(Quaternion::new(
//         new_rotation[0], // w
//         new_rotation[1], // x
//         new_rotation[2], // y
//         new_rotation[3], // z
//     ));

//     // Get current angles in radians
//     let (x_rad, y_rad, z_rad) = current_quat.euler_angles();

//     // Then when creating new rotation, preserve other angles:
//     let rotation = match axis {
//         "x" => UnitQuaternion::from_euler_angles(radians, y_rad, z_rad),
//         "y" => UnitQuaternion::from_euler_angles(x_rad, radians, z_rad),
//         "z" => UnitQuaternion::from_euler_angles(x_rad, y_rad, radians),
//         _ => current_quat,
//     };

//     let new_rotation = [
//         rotation.quaternion().w,
//         rotation.quaternion().i,
//         rotation.quaternion().j,
//         rotation.quaternion().k,
//     ];

//     // update gizmo transform position
//     // renderer_state
//     //     .translation_gizmo
//     //     .transform
//     //     .update_position(new_position);

//     // update_arrow_collider_position
//     // renderer_state.update_arrow_collider_position(new_position);

//     // aaaand back to degrees
//     let current_quat = UnitQuaternion::from_quaternion(Quaternion::new(
//         new_rotation[0], // w
//         new_rotation[1], // x
//         new_rotation[2], // y
//         new_rotation[3], // z
//     ));

//     // Get angles in radians
//     let (x_rad, y_rad, z_rad) = current_quat.euler_angles();

//     // Convert to degrees for display
//     let x_degrees = (x_rad * 180.0 / PI).round();
//     let y_degrees = (y_rad * 180.0 / PI).round();
//     let z_degrees = (z_rad * 180.0 / PI).round();

//     active_rotation.set([x_degrees, y_degrees, z_degrees]);

//     // update renderer_state.selected_object_data with new ComponentData
//     let mut new_skel_data = active_skel_keyframe.clone();
//     let mut new_data = active_keyframe.clone();

//     // new_data.generic_properties.position = new_position;
//     new_skel_data.base.rotation = new_rotation;
//     new_data.value = KeyframeValue::Rotation(new_rotation);

//     active_skel_keyframe_data.set(Some(new_skel_data.clone()));
//     active_keyframe_data.set(Some(new_data.clone()));

//     drop(renderer_state);

//     // // save in saved_state
//     // let mut saved_state = state_helper
//     //     .saved_state
//     //     .as_mut()
//     //     .expect("Couldn't get saved state")
//     //     .lock()
//     //     .unwrap();

//     // // Update the component within the saved state
//     // let motion_paths = &mut saved_state.motion_paths;

//     // if let Some(mutable_keyframe) = motion_paths.iter_mut().find_map(|mp| {
//     //     mp.keyframes
//     //         .iter_mut()
//     //         .find(|kf| kf.id == active_keyframe.skel_key_id)
//     // }) {
//     //     mutable_keyframe.base.rotation = new_rotation;
//     // }

//     // let renderable_paths = motion_paths.clone();

//     // drop(saved_state);

//     // save in saved_state
//     let mut saved_state = state_helper
//         .saved_state
//         .as_mut()
//         .expect("Couldn't get saved state")
//         .lock()
//         .unwrap();

//     // Update the component within the saved state
//     let motion_paths = &mut saved_state.motion_paths;

//     // Find and update the correct motion path
//     for motion_path in motion_paths.iter_mut() {
//         if let Some(keyframe) = motion_path
//             .keyframes
//             .iter_mut()
//             .find(|kf| kf.id == active_keyframe.skel_key_id)
//         {
//             keyframe.base.rotation = new_rotation;
//             break; // Exit once we've found and updated the keyframe
//         }
//     }

//     let renderable_paths = motion_paths.clone();

//     drop(saved_state);

//     let project_id = state_helper
//         .project_selected_signal
//         .expect("Couldn't get project signal")
//         .get();

//     let saved_state = state_helper
//         .saved_state
//         .as_ref()
//         .expect("Couldn't get saved state")
//         .lock()
//         .unwrap();

//     // Save the updated state
//     state_helper.save_saved_state_raw(project_id, saved_state.clone());

//     drop(saved_state);

//     let mut renderer_state = state_helper
//         .renderer_state
//         .as_mut()
//         .expect("Couldn't get RendererState");
//     let mut renderer_state = renderer_state.lock().unwrap();

//     // update rendererstate for visuals
//     let animation_playback = AnimationPlayback::new(renderable_paths.clone());
//     let mut current_animations = Vec::new();
//     current_animations.push(animation_playback);
//     renderer_state.active_animations = current_animations;

//     drop(renderer_state);
//     // drop(state_helper);

//     let saved_state = state_helper
//         .saved_state
//         .as_ref()
//         .expect("Couldn't get saved state")
//         .lock()
//         .unwrap();

//     refresh_animation_data(
//         &saved_state,
//         renderable_paths.clone(),
//         selected_skeleton_id_signal,
//         motion_paths_signal,
//         animation_data_signal,
//     );

//     drop(saved_state);
//     drop(state_helper);

//     let mut new_selections = Vec::new();
//     new_selections.push(new_data.clone());
//     selected_keyframes.set(new_selections);
// }

// pub fn update_joint_rotation(
//     // state_helper: Arc<Mutex<StateHelper>>,
//     mut state_helper: MutexGuard<StateHelper>, // may be truly unecessary as guard here
//     active_keyframe_data: RwSignal<Option<UIKeyframe>>,
//     active_skel_keyframe_data: RwSignal<Option<SkeletonKeyframe>>,
//     joint_id: String,
//     value: String,
//     axis: &str,
//     selected_skeleton_id_signal: RwSignal<String>,
//     motion_paths_signal: RwSignal<Vec<SkeletonMotionPath>>,
//     animation_data_signal: RwSignal<Option<AnimationData>>,
//     active_joint_rotations: RwSignal<Option<std::collections::HashMap<String, [f32; 4]>>>,
//     selected_keyframes: RwSignal<Vec<UIKeyframe>>,
// ) {
//     println!("updating keyframe joint rotation");
//     let mut renderer_state = state_helper
//         .renderer_state
//         .as_mut()
//         .expect("Couldn't get RendererState");
//     let mut renderer_state = renderer_state.lock().unwrap();

//     // let current_position = renderer_state.translation_gizmo.transform.position;

//     let active_keyframe = active_keyframe_data
//         .get()
//         .expect("Couldn't get active keyframe");
//     let active_skel_keyframe = active_skel_keyframe_data
//         .get()
//         .expect("Couldn't get active keyframe");

//     // for X only
//     let joint_rotations = active_skel_keyframe
//         .clone()
//         .joint_rotations
//         .expect("Couldn't get joint rotations");
//     let mut new_rotation = joint_rotations.get(&joint_id).expect("Couldn't get joint");
//     println!("parsing value {:?}", value);
//     let parsed_value = parse_string_to_float(&value);

//     if parsed_value.is_none() {
//         println!("invalid input");
//         return;
//     }

//     let parsed_value = parsed_value.expect("Couldn't get parsed value");

//     let degrees = parsed_value;
//     let radians = degrees * PI / 180.0;

//     let current_quat = UnitQuaternion::from_quaternion(Quaternion::new(
//         new_rotation[0], // w
//         new_rotation[1], // x
//         new_rotation[2], // y
//         new_rotation[3], // z
//     ));

//     // Get current angles in radians
//     let (x_rad, y_rad, z_rad) = current_quat.euler_angles();

//     // Then when creating new rotation, preserve other angles:
//     let rotation = match axis {
//         "x" => UnitQuaternion::from_euler_angles(radians, y_rad, z_rad),
//         "y" => UnitQuaternion::from_euler_angles(x_rad, radians, z_rad),
//         "z" => UnitQuaternion::from_euler_angles(x_rad, y_rad, radians),
//         _ => current_quat,
//     };

//     let new_rotation = [
//         rotation.quaternion().w,
//         rotation.quaternion().i,
//         rotation.quaternion().j,
//         rotation.quaternion().k,
//     ];

//     // update gizmo transform position
//     // renderer_state
//     //     .translation_gizmo
//     //     .transform
//     //     .update_position(new_position);

//     // update_arrow_collider_position
//     // renderer_state.update_arrow_collider_position(new_position);

//     // TODO: need to update to degrees for display and input
//     // active_joint_rotations.update(|jr| {
//     //     let mut jr = jr.as_mut().expect("Couldn't get jr");
//     //     let joint = jr.get_mut(&joint_id).expect("Couldn't get joint");
//     //     joint[0] = new_rotation[0];
//     //     joint[1] = new_rotation[1];
//     //     joint[2] = new_rotation[2];
//     //     joint[3] = new_rotation[3];
//     // });

//     // update renderer_state.selected_object_data with new ComponentData
//     let mut new_skel_data = active_skel_keyframe.clone();
//     let mut new_data = active_keyframe.clone();

//     // new_data.generic_properties.position = new_position;
//     // this is done below
//     // let mutable_skel_rotation = new_skel_data
//     //     .joint_rotations
//     //     .expect("Couldn't get rotations")
//     //     .get_mut(&joint_id)
//     //     .expect("Couldn't get specific rotation");

//     // mutable_rotation[0] = new_rotation[0];
//     // mutable_rotation[1] = new_rotation[1];
//     // mutable_rotation[2] = new_rotation[2];
//     // mutable_rotation[3] = new_rotation[3];
//     // new_data.value = KeyframeValue::Rotation(new_rotation);

//     active_skel_keyframe_data.set(Some(new_skel_data.clone()));
//     active_keyframe_data.set(Some(new_data.clone()));

//     drop(renderer_state);

//     // // save in saved_state
//     // let mut saved_state = state_helper
//     //     .saved_state
//     //     .as_mut()
//     //     .expect("Couldn't get saved state")
//     //     .lock()
//     //     .unwrap();

//     // // Update the component within the saved state
//     // let motion_paths = &mut saved_state.motion_paths;

//     // if let Some(mutable_keyframe) = motion_paths.iter_mut().find_map(|mp| {
//     //     mp.keyframes
//     //         .iter_mut()
//     //         .find(|kf| kf.id == active_keyframe.skel_key_id)
//     // }) {
//     //     // mutable_keyframe.base.rotation = new_rotation;
//     //     let mutable_rotation = mutable_keyframe
//     //         .joint_rotations
//     //         .as_mut()
//     //         .expect("Couldn't get rotations")
//     //         .get_mut(&joint_id)
//     //         .expect("Couldn't get specific rotation");
//     //     mutable_rotation[0] = new_rotation[0];
//     //     mutable_rotation[1] = new_rotation[1];
//     //     mutable_rotation[2] = new_rotation[2];
//     //     mutable_rotation[3] = new_rotation[3];
//     // }

//     // let renderable_paths = motion_paths.clone();

//     // drop(saved_state);

//     // save in saved_state
//     let mut saved_state = state_helper
//         .saved_state
//         .as_mut()
//         .expect("Couldn't get saved state")
//         .lock()
//         .unwrap();

//     // Update the component within the saved state
//     let motion_paths = &mut saved_state.motion_paths;

//     // Find and update the correct motion path
//     for motion_path in motion_paths.iter_mut() {
//         if let Some(keyframe) = motion_path
//             .keyframes
//             .iter_mut()
//             .find(|kf| kf.id == active_keyframe.skel_key_id)
//         {
//             // keyframe.base.rotation = new_rotation;
//             let mutable_rotation = keyframe
//                 .joint_rotations
//                 .as_mut()
//                 .expect("Couldn't get rotations")
//                 .get_mut(&joint_id)
//                 .expect("Couldn't get specific rotation");
//             mutable_rotation[0] = new_rotation[0];
//             mutable_rotation[1] = new_rotation[1];
//             mutable_rotation[2] = new_rotation[2];
//             mutable_rotation[3] = new_rotation[3];

//             break; // Exit once we've found and updated the keyframe
//         }
//     }

//     let renderable_paths = motion_paths.clone();

//     drop(saved_state);

//     let project_id = state_helper
//         .project_selected_signal
//         .expect("Couldn't get project signal")
//         .get();

//     let saved_state = state_helper
//         .saved_state
//         .as_ref()
//         .expect("Couldn't get saved state")
//         .lock()
//         .unwrap();

//     // Save the updated state
//     state_helper.save_saved_state_raw(project_id, saved_state.clone());

//     drop(saved_state);

//     let mut renderer_state = state_helper
//         .renderer_state
//         .as_mut()
//         .expect("Couldn't get RendererState");
//     let mut renderer_state = renderer_state.lock().unwrap();

//     // update rendererstate for visuals
//     let animation_playback = AnimationPlayback::new(renderable_paths.clone());
//     let mut current_animations = Vec::new();
//     current_animations.push(animation_playback);
//     renderer_state.active_animations = current_animations;

//     drop(renderer_state);
//     // drop(state_helper);

//     let saved_state = state_helper
//         .saved_state
//         .as_ref()
//         .expect("Couldn't get saved state")
//         .lock()
//         .unwrap();

//     // causes remounting of keyframe_properties?
//     refresh_animation_data(
//         &saved_state,
//         renderable_paths.clone(),
//         selected_skeleton_id_signal,
//         motion_paths_signal,
//         animation_data_signal,
//     );

//     drop(saved_state);
//     drop(state_helper);

//     let mut new_selections = Vec::new();
//     new_selections.push(new_data.clone());
//     selected_keyframes.set(new_selections);
// }

pub fn keyframe_properties(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
    selected_keyframes: RwSignal<Vec<UIKeyframe>>,
    selected_skeleton_id_signal: RwSignal<String>,
    motion_paths_signal: RwSignal<Vec<SkeletonMotionPath>>,
    animation_data_signal: RwSignal<Option<AnimationData>>,
) -> impl View {
    println!("compute keyframe_properties layout");

    let state_2 = Arc::clone(&state_helper);
    let state_3 = Arc::clone(&state_helper);
    let state_4 = Arc::clone(&state_helper);
    let state_5 = Arc::clone(&state_helper);
    let state_6 = Arc::clone(&state_helper);
    let state_7 = Arc::clone(&state_helper);
    let state_8 = Arc::clone(&state_helper);
    let gpu_2 = Arc::clone(&gpu_helper);

    let back_active = create_rw_signal(false);

    let active_keyframe = create_rw_signal(None);
    let active_skel_keyframe = create_rw_signal(None);

    let active_keyframe_type = create_rw_signal(String::new());
    let active_position = create_rw_signal([0.0, 0.0, 0.0]);
    let active_rotation = create_rw_signal([0.0, 0.0, 0.0]);
    // let active_ik_target_position = create_rw_signal(None);
    // let active_joint_rotations: RwSignal<Option<std::collections::HashMap<String, [f32; 4]>>> =
    //     create_rw_signal(None);

    let aside_width = 260.0;
    let quarters = (aside_width / 4.0) + (5.0 * 4.0);
    let thirds = (aside_width / 3.0) + (5.0 * 3.0);
    let halfs = (aside_width / 2.0) + (5.0 * 2.0);

    create_effect(move |_| {
        println!("starting effect");
        let state_5 = state_5.clone();
        let state_helper = state_5.lock().unwrap();
        let saved_state = state_helper
            .saved_state
            .as_ref()
            .expect("Couldn't get saved state");
        let saved_state = saved_state.lock().unwrap();

        let active_keyframe_data = selected_keyframes
            .get()
            .get(0)
            .expect("Couldn't get active keyframe")
            .clone(); // only support editing one for now
        let active_skel_keyframe_data = saved_state
            .motion_paths
            .iter()
            .find_map(|mp| {
                mp.keyframes
                    .iter()
                    .find(|kf| kf.id == active_keyframe_data.skel_key_id)
            })
            .cloned();

        println!("continuing effect");

        // Update the component within the saved state
        let motion_paths = &saved_state.motion_paths;

        // Find and update the correct motion path
        for motion_path in motion_paths.iter() {
            if let Some(keyframe) = motion_path
                .keyframes
                .iter()
                .find(|kf| kf.id == active_keyframe_data.skel_key_id)
            {
                // keyframe.base.position = new_position;
                // keyframe.ik_target_position = Some(new_position);
                println!(
                    "key prop on mount {:?} {:?}",
                    keyframe.fk_settings, keyframe.ik_settings
                );
                break; // Exit once we've found and updated the keyframe
            }
        }

        let mut keyframe_type = "".to_string();
        let position_data = match active_keyframe_data.value {
            KeyframeValue::Position(pos) => {
                keyframe_type = "position".to_string();
                pos
            }
            // Rotation(rot) => ,
            // Scale(scale) => ,
            // Custom(cust) => ,
            _ => [0.0, 0.0, 0.0],
        };

        let rotation_data = match active_keyframe_data.value {
            // Position(pos) => pos,
            KeyframeValue::Rotation(rot) => {
                keyframe_type = "rotation".to_string();

                let current_quat = UnitQuaternion::from_quaternion(Quaternion::new(
                    rot[0], // w
                    rot[1], // x
                    rot[2], // y
                    rot[3], // z
                ));

                // Get angles in radians
                let (x_rad, y_rad, z_rad) = current_quat.euler_angles();

                // Convert to degrees for display
                let x_degrees = (x_rad * 180.0 / PI).round();
                let y_degrees = (y_rad * 180.0 / PI).round();
                let z_degrees = (z_rad * 180.0 / PI).round();

                [x_degrees, y_degrees, z_degrees]
            }
            // Scale(scale) => ,
            // Custom(cust) => ,
            _ => [0.0, 0.0, 0.0],
        };

        // let ik_target_position = active_skel_keyframe_data
        //     .as_ref()
        //     .expect("Couldn't get keyframe data")
        //     .ik_target_position;

        // let joint_rotations = active_skel_keyframe_data
        //     .as_ref()
        //     .expect("Couldn't get keyframe data")
        //     .joint_rotations
        //     .clone();

        println!("finishing effect");

        active_keyframe.set(Some(active_keyframe_data));
        active_skel_keyframe.set(active_skel_keyframe_data);
        active_position.set(position_data);
        active_rotation.set(rotation_data);
        active_keyframe_type.set(keyframe_type);
        // active_ik_target_position.set(ik_target_position);
        // active_joint_rotations.set(joint_rotations);

        println!("Properties effect!");
    });

    v_stack((
        h_stack((
            small_button(
                "",
                "arrow-left",
                {
                    move |_| {
                        println!("Click back!");
                        // this action runs on_click_stop so should stop propagation
                        // object_selected_signal.update(|v| {
                        //     *v = false;
                        // });
                        // selected_object_id_signal.update(|v| {
                        //     *v = Uuid::nil();
                        // });
                    }
                },
                back_active,
            )
            .style(|s| s.margin_right(7.0)),
            label(|| "Properties").style(|s| s.font_size(24.0).font_weight(Weight::THIN)),
        ))
        .style(|s| s.margin_bottom(12.0)),
        label(move || {
            format!(
                "ID: {}",
                active_keyframe
                    .get()
                    .expect("Couldn't get active keyframe")
                    .id
            )
        })
        .style(|s| s.font_size(12.0).font_weight(Weight::THIN)),
        label(move || {
            format!(
                "SK ID: {}",
                active_keyframe
                    .get()
                    .expect("Couldn't get active keyframe")
                    .skel_key_id
            )
        })
        .style(|s| s.font_size(12.0).font_weight(Weight::THIN)),
        dyn_container(
            move || active_keyframe_type.get(),
            move |keyframe_type| {
                let state_2 = Arc::clone(&state_2);
                let state_3 = Arc::clone(&state_3);
                let state_4 = Arc::clone(&state_4);
                let state_6 = Arc::clone(&state_6);

                if keyframe_type == "position".to_string() {
                    h_stack((
                        styled_input(
                            "X Position:".to_string(),
                            &active_position
                                .try_get()
                                .map_or("0".to_string(), |pos| pos[0].to_string()),
                            "X Position",
                            Box::new({
                                move |mut state_helper_passed, value| {
                                    // update_position(
                                    //     state_helper_passed,
                                    //     active_keyframe,
                                    //     active_skel_keyframe,
                                    //     value,
                                    //     "x",
                                    //     selected_skeleton_id_signal,
                                    //     motion_paths_signal,
                                    //     animation_data_signal,
                                    //     active_position,
                                    //     selected_keyframes,
                                    // )
                                }
                            }),
                            state_2,
                            "x".to_string(),
                        )
                        .style(move |s| s.width(thirds).margin_right(5.0)),
                        styled_input(
                            "Y Position:".to_string(),
                            &active_position
                                .try_get()
                                .map_or("0".to_string(), |pos| pos[1].to_string()),
                            "Y Position",
                            Box::new({
                                move |mut state_helper_passed, value| {
                                    // update_position(
                                    //     state_helper_passed,
                                    //     active_keyframe,
                                    //     active_skel_keyframe,
                                    //     value,
                                    //     "y",
                                    //     selected_skeleton_id_signal,
                                    //     motion_paths_signal,
                                    //     animation_data_signal,
                                    //     active_position,
                                    //     selected_keyframes,
                                    // )
                                }
                            }),
                            state_3,
                            "y".to_string(),
                        )
                        .style(move |s| s.width(thirds).margin_right(5.0)),
                        styled_input(
                            "Z Position:".to_string(),
                            &active_position
                                .try_get()
                                .map_or("0".to_string(), |pos| pos[2].to_string()),
                            "Z Position",
                            Box::new({
                                move |mut state_helper_passed, value| {
                                    // update_position(
                                    //     state_helper_passed,
                                    //     active_keyframe,
                                    //     active_skel_keyframe,
                                    //     value,
                                    //     "z",
                                    //     selected_skeleton_id_signal,
                                    //     motion_paths_signal,
                                    //     animation_data_signal,
                                    //     active_position,
                                    //     selected_keyframes,
                                    // )
                                }
                            }),
                            state_4,
                            "z".to_string(),
                        )
                        .style(move |s| s.width(thirds)),
                    ))
                    .style(move |s| s.width(aside_width))
                    .into_any()
                } else if keyframe_type == "rotation".to_string() {
                    h_stack((
                        styled_input(
                            "X Rotation:".to_string(),
                            &active_rotation
                                .try_get()
                                .map_or("0".to_string(), |pos| pos[0].to_string()),
                            "X Degrees",
                            Box::new({
                                move |mut state_helper_passed, value| {
                                    // update_rotation(
                                    //     state_helper_passed,
                                    //     active_keyframe,
                                    //     active_skel_keyframe,
                                    //     value,
                                    //     "x",
                                    //     selected_skeleton_id_signal,
                                    //     motion_paths_signal,
                                    //     animation_data_signal,
                                    //     active_rotation,
                                    //     selected_keyframes,
                                    // )
                                }
                            }),
                            state_2,
                            "x".to_string(),
                        )
                        .style(move |s| s.width(thirds).margin_right(5.0)),
                        styled_input(
                            "Y Rotation:".to_string(),
                            &active_rotation
                                .try_get()
                                .map_or("0".to_string(), |pos| pos[1].to_string()),
                            "Y Degrees",
                            Box::new({
                                move |mut state_helper_passed, value| {
                                    // update_rotation(
                                    //     state_helper_passed,
                                    //     active_keyframe,
                                    //     active_skel_keyframe,
                                    //     value,
                                    //     "y",
                                    //     selected_skeleton_id_signal,
                                    //     motion_paths_signal,
                                    //     animation_data_signal,
                                    //     active_rotation,
                                    //     selected_keyframes,
                                    // )
                                }
                            }),
                            state_3,
                            "y".to_string(),
                        )
                        .style(move |s| s.width(thirds).margin_right(5.0)),
                        styled_input(
                            "Z Rotation:".to_string(),
                            &active_rotation
                                .try_get()
                                .map_or("0".to_string(), |pos| pos[2].to_string()),
                            "Z Degrees",
                            Box::new({
                                move |mut state_helper_passed, value| {
                                    // update_rotation(
                                    //     state_helper_passed,
                                    //     active_keyframe,
                                    //     active_skel_keyframe,
                                    //     value,
                                    //     "z",
                                    //     selected_skeleton_id_signal,
                                    //     motion_paths_signal,
                                    //     animation_data_signal,
                                    //     active_rotation,
                                    //     selected_keyframes,
                                    // )
                                }
                            }),
                            state_4,
                            "z".to_string(),
                        )
                        .style(move |s| s.width(thirds)),
                    ))
                    .style(move |s| s.width(aside_width))
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
        // dyn_container(
        //     move || active_joint_rotations.get().is_some(),
        //     move |has_joint_roations| {
        //         if (has_joint_roations) {
        //             let state_7 = state_7.clone();

        //             dyn_stack(
        //                 move || {
        //                     active_joint_rotations
        //                         .try_get()
        //                         .map_or(HashMap::new(), |map| {
        //                             if let Some(hashmap) = map {
        //                                 hashmap
        //                             } else {
        //                                 HashMap::new()
        //                             }
        //                         })
        //                         .into_iter()
        //                         .enumerate()
        //                 },
        //                 move |(idx, (key, value))| idx.clone(), // use index as the key
        //                 move |(idx, (joint_id, rotation))| {
        //                     let state_10 = Arc::clone(&state_7);
        //                     let state_11 = Arc::clone(&state_7);
        //                     let state_12 = Arc::clone(&state_7);

        //                     let joint_id_2 = joint_id.clone();
        //                     let joint_id_3 = joint_id.clone();

        //                     // For display - converting quaternion to degrees:
        //                     let current_quat = UnitQuaternion::from_quaternion(Quaternion::new(
        //                         rotation[0], // w
        //                         rotation[1], // x
        //                         rotation[2], // y
        //                         rotation[3], // z
        //                     ));

        //                     // Get angles in radians
        //                     let (x_rad, y_rad, z_rad) = current_quat.euler_angles();

        //                     // Convert to degrees for display
        //                     let x_degrees = (x_rad * 180.0 / PI).round();
        //                     let y_degrees = (y_rad * 180.0 / PI).round();
        //                     let z_degrees = (z_rad * 180.0 / PI).round();

        //                     h_stack((
        //                         styled_input(
        //                             format!("{} X:", joint_id.clone()),
        //                             &x_degrees.to_string(),
        //                             &format!("{} X Rotation", joint_id.clone()),
        //                             Box::new({
        //                                 move |mut state_helper_passed, value| {
        //                                     update_joint_rotation(
        //                                         state_helper_passed,
        //                                         active_keyframe,
        //                                         active_skel_keyframe,
        //                                         joint_id.clone(),
        //                                         value,
        //                                         "x",
        //                                         selected_skeleton_id_signal,
        //                                         motion_paths_signal,
        //                                         animation_data_signal,
        //                                         active_joint_rotations,
        //                                         selected_keyframes,
        //                                     )
        //                                 }
        //                             }),
        //                             state_10.clone(),
        //                             "x".to_string(),
        //                         )
        //                         .style(move |s| s.width(thirds).margin_right(5.0)),
        //                         styled_input(
        //                             format!("{} Y:", joint_id_2.clone()),
        //                             &y_degrees.to_string(),
        //                             &format!("{} Y Rotation", joint_id_2.clone()),
        //                             Box::new({
        //                                 move |mut state_helper_passed, value| {
        //                                     update_joint_rotation(
        //                                         state_helper_passed,
        //                                         active_keyframe,
        //                                         active_skel_keyframe,
        //                                         joint_id_2.clone(),
        //                                         value,
        //                                         "y",
        //                                         selected_skeleton_id_signal,
        //                                         motion_paths_signal,
        //                                         animation_data_signal,
        //                                         active_joint_rotations,
        //                                         selected_keyframes,
        //                                     )
        //                                 }
        //                             }),
        //                             state_11,
        //                             "y".to_string(),
        //                         )
        //                         .style(move |s| s.width(thirds).margin_right(5.0)),
        //                         styled_input(
        //                             format!("{} Z:", joint_id_3.clone()),
        //                             &z_degrees.to_string(),
        //                             &format!("{} Z Rotation", joint_id_3.clone()),
        //                             Box::new({
        //                                 move |mut state_helper_passed, value| {
        //                                     update_joint_rotation(
        //                                         state_helper_passed,
        //                                         active_keyframe,
        //                                         active_skel_keyframe,
        //                                         joint_id_3.clone(),
        //                                         value,
        //                                         "z",
        //                                         selected_skeleton_id_signal,
        //                                         motion_paths_signal,
        //                                         animation_data_signal,
        //                                         active_joint_rotations,
        //                                         selected_keyframes,
        //                                     )
        //                                 }
        //                             }),
        //                             state_12,
        //                             "z".to_string(),
        //                         )
        //                         .style(move |s| s.width(thirds)),
        //                     ))
        //                     .style(move |s| s.width(aside_width))
        //                 },
        //             )
        //             .into_any()
        //         } else {
        //             empty().into_any()
        //         }
        //     },
        // ),
    ))
    .style(|s| card_styles(s))
    .style(|s| {
        s.width(300)
            // .absolute()
            .height(800.0)
            .margin_left(0.0)
            .margin_top(20)
        // .z_index(10)
    })
}
