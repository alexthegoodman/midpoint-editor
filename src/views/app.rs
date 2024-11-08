use midpoint_engine::core::RendererState::ObjectConfig;
use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::floem::common::toggle_button;
use midpoint_engine::floem::reactive::create_effect;
use midpoint_engine::floem::reactive::create_rw_signal;
use midpoint_engine::floem::reactive::SignalGet;
use midpoint_engine::floem::reactive::SignalUpdate;
use midpoint_engine::floem::views::h_stack;
use midpoint_engine::floem::views::{
    container, dyn_container, empty, label, scroll, stack, tab, text_input, virtual_stack,
    VirtualDirection, VirtualItemSize,
};
use midpoint_engine::helpers::saved_data::ComponentData;
use midpoint_engine::helpers::saved_data::ComponentKind;
use midpoint_engine::helpers::saved_data::GenericProperties;
use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;
use wgpu::util::DeviceExt;

use midpoint_engine::floem::GpuHelper;
use midpoint_engine::floem::IntoView;

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
    let object_selected_signal = create_rw_signal(false);
    let selected_object_id_signal = create_rw_signal(Uuid::nil());
    let active_gizmo_signal = create_rw_signal("translation".to_string());
    let current_view_signal = create_rw_signal("scene".to_string());

    let selected_object_data_signal = create_rw_signal(ComponentData {
        id: "".to_string(),
        kind: Some(ComponentKind::Model),
        asset_id: "".to_string(),
        generic_properties: GenericProperties {
            name: "".to_string(),
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        },
        landscape_properties: None,
        model_properties: None,
    });

    let state_2 = Arc::clone(&state_helper);
    let state_3 = Arc::clone(&state_helper);
    let state_4 = Arc::clone(&state_helper);
    let state_5 = Arc::clone(&state_helper);

    create_effect(move |_| {
        let state_helper = state_2.clone();
        let mut state_helper = state_helper.lock().unwrap();
        state_helper.object_selected_signal = Some(object_selected_signal);
        state_helper.selected_object_id_signal = Some(selected_object_id_signal);
        state_helper.selected_object_data_signal = Some(selected_object_data_signal);

        // also current_view
        state_helper.current_view_signal = Some(current_view_signal);
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
        dyn_container(
            move || current_view_signal.get(),
            move |current_view_real| {
                if current_view_real == "scene".to_string() {
                    h_stack((
                        toggle_button(
                            "Translate",
                            "translate",
                            "translate".to_string(),
                            {
                                let state_3 = state_3.clone();

                                move |_| {
                                    let mut state_helper = state_3.lock().unwrap();
                                    let mut renderer_state = state_helper
                                        .renderer_state
                                        .as_mut()
                                        .expect("Couldn't get RendererState")
                                        .lock()
                                        .unwrap();

                                    renderer_state.active_gizmo = "translate".to_string();

                                    active_gizmo_signal.set("translate".to_string());
                                }
                            },
                            active_gizmo_signal,
                        ),
                        toggle_button(
                            "Rotate",
                            "rotate",
                            "rotate".to_string(),
                            {
                                let state_4 = state_4.clone();

                                move |_| {
                                    let mut state_helper = state_4.lock().unwrap();
                                    let mut renderer_state = state_helper
                                        .renderer_state
                                        .as_mut()
                                        .expect("Couldn't get RendererState")
                                        .lock()
                                        .unwrap();

                                    renderer_state.active_gizmo = "rotate".to_string();

                                    active_gizmo_signal.set("rotate".to_string());
                                }
                            },
                            active_gizmo_signal,
                        ),
                        toggle_button(
                            "Scale",
                            "scale",
                            "scale".to_string(),
                            {
                                let state_5 = state_5.clone();

                                move |_| {
                                    let mut state_helper = state_5.lock().unwrap();
                                    let mut renderer_state = state_helper
                                        .renderer_state
                                        .as_mut()
                                        .expect("Couldn't get RendererState")
                                        .lock()
                                        .unwrap();

                                    renderer_state.active_gizmo = "scale".to_string();

                                    active_gizmo_signal.set("scale".to_string());
                                }
                            },
                            active_gizmo_signal,
                        ),
                    ))
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
