use std::sync::{Arc, Mutex, MutexGuard};

use super::shared::dynamic_img;
use floem::common::small_button;
use floem::reactive::SignalGet;
use floem::reactive::{create_effect, create_rw_signal, RwSignal, SignalUpdate};
use floem::taffy::{FlexDirection, FlexWrap};
use floem::views::{container, dyn_container, dyn_stack, empty, label, scroll, v_stack};
use floem::IntoView;
use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::helpers::saved_data::File;
use wgpu::util::DeviceExt;

use floem::views::Decorators;
use floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::{EditorState, StateHelper};

pub fn model_item(label_text: String) -> impl View {
    let active = create_rw_signal(false);

    v_stack((
        label(move || label_text.clone()),
        small_button(
            "Add to Scene",
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

pub fn model_browser(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    let model_data: RwSignal<Vec<File>> = create_rw_signal(Vec::new());

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
                move |model_data| model_item(model_data.fileName),
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
