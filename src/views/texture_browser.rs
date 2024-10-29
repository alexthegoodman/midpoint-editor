use std::sync::{Arc, Mutex, MutexGuard};

use super::shared::dynamic_img;
use floem::reactive::SignalGet;
use floem::reactive::SignalUpdate;
use floem::reactive::{create_effect, create_rw_signal, RwSignal};
use floem::taffy::{FlexDirection, FlexWrap};
use floem::views::{container, dyn_container, dyn_stack, empty, label, scroll, v_stack};
use floem::IntoView;
use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::helpers::saved_data::File;
use wgpu::util::DeviceExt;

use floem::views::Decorators;
use floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::{EditorState, StateHelper};

pub fn texture_item(image_path: String, label_text: String) -> impl View {
    v_stack(
        ((
            dynamic_img(image_path, label_text.clone())
                .style(|s| s.width(120.0).height(120.0).border_radius(5.0)),
            label(move || label_text.clone()),
        )),
    )
    .style(|s| s.width(120.0))
}

pub fn texture_browser(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    let texture_data: RwSignal<Vec<File>> = create_rw_signal(Vec::new());

    create_effect(move |_| {
        let state_helper = state_helper.lock().unwrap();
        let saved_state = state_helper
            .saved_state
            .as_ref()
            .expect("Couldn't get saved state")
            .lock()
            .unwrap();
        texture_data.set(
            saved_state
                .textures
                .as_ref()
                .expect("Couldn't get texture data")
                .clone(),
        );
    });

    container((scroll(
        dyn_stack(
            move || texture_data.get(),
            move |texture_data| texture_data.id.clone(),
            move |texture_data| texture_item(texture_data.normalFilePath, texture_data.fileName),
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
