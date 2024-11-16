use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use super::shared::dynamic_img;
use midpoint_engine::animations::skeleton::SkeletonAssemblyConfig;
use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::floem::common::small_button;
use midpoint_engine::floem::ext_event::create_signal_from_tokio_channel;
use midpoint_engine::floem::reactive::SignalGet;
use midpoint_engine::floem::reactive::SignalUpdate;
use midpoint_engine::floem::reactive::{create_effect, create_rw_signal, RwSignal};
use midpoint_engine::floem::taffy::{FlexDirection, FlexWrap};
use midpoint_engine::floem::views::h_stack;
use midpoint_engine::floem::views::text_input;
use midpoint_engine::floem::views::{
    container, dyn_container, dyn_stack, empty, label, scroll, v_stack,
};
use midpoint_engine::floem::IntoView;
use midpoint_engine::handlers::handle_add_skeleton_part;
use midpoint_engine::helpers::saved_data::File;
use nalgebra::Point3;
use tokio::spawn;
use uuid::Uuid;
use wgpu::util::DeviceExt;

use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::UIMessage;
use crate::editor_state::{EditorState, StateHelper};
use crate::gql::generateTexture::generate_texture;
use crate::helpers::auth::read_auth_token;
use crate::helpers::textures::save_texture;
use crate::helpers::utilities::get_filename;

pub fn skeleton_item(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
    skeleton_id: String,
    label_text: String,
    skeleton_selected_signal: RwSignal<bool>,
    selected_skeleton_id_signal: RwSignal<String>,
) -> impl View {
    let active_btn = create_rw_signal(false);

    v_stack(
        ((
            label(move || label_text.clone()),
            small_button(
                "Edit Assembly",
                "plus",
                move |_| {
                    let mut state_helper_guard = state_helper.lock().unwrap();

                    let destination_view = "animation_skeleton".to_string();

                    let mut renderer_state = state_helper_guard
                        .renderer_state
                        .as_mut()
                        .expect("Couldn't get RendererState")
                        .lock()
                        .unwrap();

                    renderer_state.current_view = destination_view.clone();

                    drop(renderer_state);
                    drop(state_helper_guard);

                    let state_helper = state_helper.lock().unwrap();
                    let gpu_helper = gpu_helper.lock().unwrap();
                    let gpu_resources = gpu_helper
                        .gpu_resources
                        .as_ref()
                        .expect("Couldn't get gpu resources");
                    let saved_state = state_helper
                        .saved_state
                        .as_ref()
                        .expect("Couldn't get SavedState")
                        .lock()
                        .unwrap();

                    // Find and clone the necessary data before the iterations
                    let selected_data = saved_state
                        .skeletons
                        .iter()
                        .find(|sp| sp.id == skeleton_id)
                        .expect("Couldn't find selected skeleton");

                    let connections = selected_data.connections.clone();
                    let skeleton_parts = saved_state.skeleton_parts.clone();

                    // Handle root part first
                    let display_part = skeleton_parts
                        .iter()
                        .find(|sp| {
                            &sp.id
                                == selected_data
                                    .root_part_id
                                    .as_ref()
                                    .expect("Couldn't get root part id")
                        })
                        .expect("Couldn't find display part");

                    let part_id = display_part.id.clone();
                    let joints = display_part.joints.clone();
                    // TOOD: precaulcate FK positions as well for idle pose
                    let joint_positions = HashMap::from_iter(joints.iter().map(|joint| {
                        if let Some(ik_settings) = &joint.ik_settings {
                            let position = Point3::new(
                                ik_settings.position[0],
                                ik_settings.position[1],
                                ik_settings.position[2],
                            );
                            (joint.id.clone(), position)
                        } else {
                            let position = Point3::origin();
                            (joint.id.clone(), position)
                        }
                    }));

                    handle_add_skeleton_part(
                        state_helper
                            .renderer_state
                            .as_ref()
                            .expect("Couldn't get RendererState")
                            .clone(),
                        &gpu_resources.device,
                        &gpu_resources.queue,
                        part_id,
                        [0.0, 0.0, 0.0],
                        joints,
                        display_part.k_chains.clone(),
                        display_part.attach_points.clone(),
                        &joint_positions,
                        None,
                    );

                    // Now handle the connections using the cloned data
                    connections.iter().for_each(|connection| {
                        let display_part = skeleton_parts
                            .iter()
                            .find(|sp| {
                                &sp.id
                                    == connection
                                        .child_part_id
                                        .as_ref()
                                        .expect("Couldn't get child part id")
                            })
                            .expect("Couldn't find display part");

                        let part_id = display_part.id.clone();
                        let joints = display_part.joints.clone();
                        // TOOD: precaulcate FK positions as well for idle pose
                        let joint_positions = HashMap::from_iter(joints.iter().map(|joint| {
                            if let Some(ik_settings) = &joint.ik_settings {
                                let position = Point3::new(
                                    ik_settings.position[0],
                                    ik_settings.position[1],
                                    ik_settings.position[2],
                                );
                                (joint.id.clone(), position)
                            } else {
                                let position = Point3::origin();
                                (joint.id.clone(), position)
                            }
                        }));

                        let position = connection
                            .transform_offset
                            .as_ref()
                            .map(|transform| transform.position)
                            .unwrap_or([0.0, 0.0, 0.0]);

                        handle_add_skeleton_part(
                            state_helper
                                .renderer_state
                                .as_ref()
                                .expect("Couldn't get RendererState")
                                .clone(),
                            &gpu_resources.device,
                            &gpu_resources.queue,
                            part_id,
                            position,
                            joints,
                            display_part.k_chains.clone(),
                            display_part.attach_points.clone(),
                            &joint_positions,
                            Some(connection.clone()),
                        );
                    });

                    // load view later to avoid lock?
                    println!("update view");
                    let current_view_signal = state_helper
                        .current_view_signal
                        .expect("Couldn't get current view signal");
                    current_view_signal.set(destination_view.clone());

                    drop(saved_state);
                    drop(state_helper);

                    skeleton_selected_signal.set(true);
                    selected_skeleton_id_signal.set(skeleton_id.clone());

                    println!("view updated");
                },
                active_btn,
            ),
        )),
    )
    .style(|s| s.width(120.0))
}

pub fn skeleton_browser(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
    skeleton_selected_signal: RwSignal<bool>,
    selected_skeleton_id_signal: RwSignal<String>,
) -> impl View {
    let state_2 = Arc::clone(&state_helper);

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let tx = Arc::new(tx);
    let skeleton_data: RwSignal<Vec<SkeletonAssemblyConfig>> = create_rw_signal(Vec::new());

    // Create signal from channel
    let update_signal = create_signal_from_tokio_channel(rx);

    // Handle updates in UI thread
    create_effect(move |_| {
        if let Some(msg) = update_signal.get() {
            match msg {
                UIMessage::UpdateSkeletons(skeletons) => skeleton_data.set(skeletons),
                UIMessage::AddSkeleton(skeleton) => skeleton_data.update(|t| t.push(skeleton)),
                _ => return,
            }
        }
    });

    create_effect({
        let tx = tx.clone();
        move |_| {
            let tx = tx.clone();
            let mut state_helper = state_helper.lock().unwrap();

            state_helper.register_file_signal("skeleton_browser".to_string(), tx);

            let saved_state = state_helper
                .saved_state
                .as_ref()
                .expect("Couldn't get saved state")
                .lock()
                .unwrap();

            skeleton_data.set(saved_state.skeletons.clone());
        }
    });

    v_stack((scroll(
        dyn_stack(
            move || skeleton_data.get(),
            move |skeleton_data| skeleton_data.id.clone(),
            move |skeleton_data_real| {
                let current_textures = skeleton_data.get(); // Add this to ensure reactivity
                skeleton_item(
                    state_2.clone(),
                    gpu_helper.clone(),
                    viewport.clone(),
                    skeleton_data_real.id,
                    skeleton_data_real.name,
                    skeleton_selected_signal,
                    selected_skeleton_id_signal,
                )
            },
        )
        .into_view()
        .style(|s| {
            s.width(260.0)
                .flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
                .gap(10.0)
        }),
    ),))
    .style(|s| s.width(260.0))
}
