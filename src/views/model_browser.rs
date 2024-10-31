use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};

use super::shared::dynamic_img;
use midpoint_engine::floem::common::small_button;
use midpoint_engine::floem::reactive::SignalGet;
use midpoint_engine::floem::reactive::{create_effect, create_rw_signal, RwSignal, SignalUpdate};
use midpoint_engine::floem::taffy::{FlexDirection, FlexWrap};
use midpoint_engine::floem::views::{button, container, dyn_container, dyn_stack, empty, label, scroll, v_stack};
use midpoint_engine::floem::IntoView;
use midpoint_engine::floem_renderer::gpu_resources;
use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::helpers::saved_data::File;
use midpoint_engine::startup::handle_add_model;
use wgpu::util::DeviceExt;

use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::{EditorState, StateHelper};

// type BoxedAsyncFn = Box<dyn Fn() -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync>;

pub fn model_item(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    label_text: String,
    filename: String,
) -> impl View {
    let active = create_rw_signal(false);

    v_stack((
        label(move || label_text.clone()),
        small_button(
            "Add to Scene",
            "plus",
            {
                move |_| {
                    // add to scene
                    let state_helper = state_helper.lock().unwrap();
                    let gpu_helper = gpu_helper.lock().unwrap();
                    let gpu_resources = gpu_helper
                        .gpu_resources
                        .as_ref()
                        .expect("Couldn't get gpu resources");
                    let renderer_state = state_helper
                        .renderer_state
                        .as_ref()
                        .expect("Couldn't get RendererState");
                    handle_add_model(
                        renderer_state.clone(),
                        &gpu_resources.device,
                        &gpu_resources.queue,
                        filename.clone(),
                    );
                }
            },
            active,
        ),
    ))
    .style(|s| s.width(120.0))
}

pub fn model_browser(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    let model_data: RwSignal<Vec<File>> = create_rw_signal(Vec::new());

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
        model_data.set(saved_state.models.clone());
    });

    v_stack((
        (label(|| "Models"),),
        container((scroll(
            dyn_stack(
                move || model_data.get(),
                move |model_data| model_data.id.clone(),
                move |model_data| {
                    model_item(
                        state_2.clone(),
                        gpu_2.clone(),
                        model_data.fileName.clone(),
                        model_data.fileName.clone(),
                    )
                },
            )
            .into_view(),
        ),))
        .style(|s| {
            s.flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
        }),
    ))
    .style(|s| s.width(260.0))
}
