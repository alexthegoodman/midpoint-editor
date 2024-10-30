use floem::reactive::create_effect;
use floem::reactive::create_rw_signal;
use floem::reactive::SignalGet;
use floem::views::{
    container, dyn_container, empty, label, scroll, stack, tab, text_input, virtual_stack,
    VirtualDirection, VirtualItemSize,
};
use midpoint_engine::core::RendererState::ObjectConfig;
use midpoint_engine::core::Viewport::Viewport;
use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;
use wgpu::util::DeviceExt;

use floem::GpuHelper;
use floem::IntoView;

use crate::editor_state::StateHelper;
use crate::helpers::websocket::WebSocketManager;

use super::aside::project_tab_interface;
use super::aside::welcome_tab_interface;
use super::properties_panel::properties_view;

pub fn project_view(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> impl IntoView {
    // object_selected? model_selected?
    let object_selected_signal = create_rw_signal(false);
    let selected_object_id_signal = create_rw_signal(Uuid::nil());
    let selected_object_data_signal = create_rw_signal(ObjectConfig {
        id: Uuid::nil(),
        name: "".to_string(),
        position: (0.0, 0.0, 0.0),
    });

    let state_2 = Arc::clone(&state_helper);

    create_effect(move |_| {
        let state_helper = state_2.clone();
        let mut state_helper = state_helper.lock().unwrap();
        state_helper.object_selected_signal = Some(object_selected_signal);
        state_helper.selected_object_id_signal = Some(selected_object_id_signal);
        state_helper.selected_object_data_signal = Some(selected_object_data_signal);
    });

    container((
        project_tab_interface(
            state_helper.clone(),
            gpu_helper.clone(),
            viewport.clone(),
            object_selected_signal,
        ),
        // this properties pabel "covers" the tools panels which are inserted within tab_interface
        dyn_container(
            move || object_selected_signal.get(),
            move |object_selected_real| {
                if object_selected_real {
                    properties_view(
                        state_helper.clone(),
                        gpu_helper.clone(),
                        viewport.clone(),
                        object_selected_signal,
                        selected_object_id_signal,
                        selected_object_data_signal,
                    )
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
    ))
}

pub fn selection_view(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    manager: Arc<WebSocketManager>,
) -> impl IntoView {
    container((welcome_tab_interface(
        state_helper.clone(),
        gpu_helper.clone(),
        viewport.clone(),
        manager.clone(),
    ),))
}

pub fn app_view(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    manager: Arc<WebSocketManager>,
) -> impl IntoView {
    let project_selected = create_rw_signal(Uuid::nil());

    let state_2 = Arc::clone(&state_helper);

    create_effect(move |_| {
        let mut state_helper = state_2.lock().unwrap();
        state_helper.project_selected_signal = Some(project_selected);
    });

    dyn_container(
        move || project_selected.get(),
        move |project_selected_real| {
            if project_selected_real != Uuid::nil() {
                project_view(state_helper.clone(), gpu_helper.clone(), viewport.clone()).into_any()
            } else {
                selection_view(
                    state_helper.clone(),
                    gpu_helper.clone(),
                    viewport.clone(),
                    manager.clone(),
                )
                .into_any()
            }
        },
    )
}
