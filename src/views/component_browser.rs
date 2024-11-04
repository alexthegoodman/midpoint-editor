use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};

use super::shared::dynamic_img;
use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::floem::common::small_button;
use midpoint_engine::floem::reactive::SignalGet;
use midpoint_engine::floem::reactive::{create_effect, create_rw_signal, RwSignal, SignalUpdate};
use midpoint_engine::floem::taffy::{FlexDirection, FlexWrap};
use midpoint_engine::floem::views::{
    button, container, dyn_container, dyn_stack, empty, label, scroll, v_stack,
};
use midpoint_engine::floem::IntoView;
use midpoint_engine::floem_renderer::gpu_resources;
use midpoint_engine::handlers::handle_add_model;
use midpoint_engine::helpers::saved_data::{ComponentData, ComponentKind, File, GenericProperties};
use uuid::Uuid;
use wgpu::util::DeviceExt;

use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::{EditorState, StateHelper};

pub fn component_item(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    label_text: String,
    component_data: ComponentData,
) -> impl View {
    let active = create_rw_signal(false);

    v_stack((
        label(move || label_text.clone()),
        small_button(
            "Select Component",
            "plus",
            {
                move |_| {
                    let mut state_helper = state_helper.lock().unwrap();

                    // open properties panel, set selected id
                    let object_selected_signal = state_helper
                        .object_selected_signal
                        .expect("Couldn't get signal");
                    object_selected_signal.set(true);
                    let selected_object_id_signal = state_helper
                        .selected_object_id_signal
                        .expect("Couldn't get signal");
                    selected_object_id_signal
                        .set(Uuid::from_str(&component_data.id).expect("Couldn't extract uuid"));
                    let selected_object_data_signal = state_helper
                        .selected_object_data_signal
                        .expect("Couldn't get signal");
                    selected_object_data_signal.set(component_data.clone());

                    // sync other ids
                    let mut renderer_state = state_helper
                        .renderer_state
                        .as_mut()
                        .expect("Couldn't get renderer state")
                        .lock()
                        .unwrap();
                    renderer_state.object_selected =
                        Some(Uuid::from_str(&component_data.id).expect("Couldn't extract uuid"));
                    renderer_state.object_selected_kind = component_data.kind.clone();
                    renderer_state.object_selected_data = Some(component_data.clone());

                    // gizmo displays automatically (but need to update gizmo position)
                    let component_data = component_data.clone();
                    renderer_state
                        .gizmo
                        .transform
                        .update_position(component_data.generic_properties.position);
                    // don't forget the rapier positions
                    // let arrows = &renderer_state.gizmo.arrows;
                    // let cloned_arrows = [renderer_state.gizmo.arrows[0]]

                    renderer_state.update_arrow_collider_position(
                        // arrows,
                        component_data.generic_properties.position,
                    );
                }
            },
            active,
        ),
    ))
    .style(|s| s.width(120.0))
}

pub fn component_browser(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    let component_data: RwSignal<Vec<ComponentData>> = create_rw_signal(Vec::new());

    let state_2 = Arc::clone(&state_helper);
    let gpu_2 = Arc::clone(&gpu_helper);

    create_effect(move |_| {
        let mut state_helper = state_helper.lock().unwrap();
        let mut saved_state = state_helper
            .saved_state
            .as_mut()
            .expect("Couldn't get saved state")
            .lock()
            .unwrap();
        let mut selected_level = saved_state
            .levels
            .as_mut()
            .expect("Couldn't get levels")
            .get_mut(0)
            .expect("Couldn't get first level");
        component_data.set(
            selected_level
                .components
                .get_or_insert_with(Vec::new)
                .clone(),
        );
    });

    container((scroll(
        dyn_stack(
            move || component_data.get(),
            move |component_data| component_data.id.clone(),
            move |component_data| {
                component_item(
                    state_2.clone(),
                    gpu_2.clone(),
                    component_data.generic_properties.name.clone(),
                    component_data,
                )
            },
        )
        .style(|s| {
            s.flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
        })
        .style(|s| s.width(260.0))
        .into_view(),
    ),))
    .style(|s| s.width(260.0))
}
