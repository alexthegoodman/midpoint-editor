use std::future::Future;
use std::pin::Pin;
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
use midpoint_engine::helpers::saved_data::{
    ComponentData, ComponentKind, File, GenericProperties, LevelData,
};
use uuid::Uuid;
use wgpu::util::DeviceExt;

use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::{EditorState, StateHelper};

pub fn level_item(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    label_text: String,
    filename: String,
    model_id: String,
) -> impl View {
    let active = create_rw_signal(false);

    v_stack((
        label(move || label_text.clone()),
        small_button(
            "Open Level",
            "plus",
            {
                move |_| {
                    // add to scene
                }
            },
            active,
        ),
    ))
    .style(|s| s.width(120.0))
}

pub fn level_browser(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    let level_data: RwSignal<Vec<LevelData>> = create_rw_signal(Vec::new());

    let state_2 = Arc::clone(&state_helper);
    let gpu_2 = Arc::clone(&gpu_helper);

    create_effect(move |_| {
        let state_helper = state_helper.lock().unwrap();
        let saved_state = state_helper
            .saved_state
            .as_ref()
            .expect("Couldn't get saved state")
            .lock()
            .unwrap();
        // all new projects should have 1 level created upon creation
        level_data.set(
            saved_state
                .levels
                .as_ref()
                .expect("Couldn't get levels")
                .clone(),
        );
    });

    container((scroll(
        dyn_stack(
            move || level_data.get(),
            move |level_data| level_data.id.clone(),
            move |level_data| {
                level_item(
                    state_2.clone(),
                    gpu_2.clone(),
                    level_data.id.clone(),
                    level_data.id.clone(),
                    level_data.id.clone(),
                )
            },
        )
        .into_view(),
    ),))
    .style(|s| {
        s.flex_direction(FlexDirection::Row)
            .flex_wrap(FlexWrap::Wrap)
    })
    .style(|s| s.width(260.0))
}
