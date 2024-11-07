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
use midpoint_engine::helpers::saved_data::File;
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

pub fn skeleton_item(label_text: String) -> impl View {
    let active_btn = create_rw_signal(false);

    v_stack(
        ((
            // dynamic_img(image_path, label_text.clone())
            //     .style(|s| s.width(120.0).height(120.0).border_radius(5.0)),
            label(move || label_text.clone()),
            small_button(
                "Edit Assembly",
                "plus",
                move |_| {
                    // renderer_state.current_view = "animation_skeleton".to_string();
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
                skeleton_item(skeleton_data_real.name)
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
