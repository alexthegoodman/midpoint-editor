use std::sync::{Arc, Mutex, MutexGuard};

use super::shared::dynamic_img;
use floem::common::small_button;
use floem::reactive::SignalGet;
use floem::reactive::{create_effect, create_rw_signal, RwSignal, SignalUpdate};
use floem::taffy::{FlexDirection, FlexWrap};
use floem::views::{container, dyn_container, dyn_stack, empty, label, scroll, v_stack};
use floem::IntoView;
use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::helpers::saved_data::{File, LandscapeData};
use wgpu::util::DeviceExt;

use floem::views::Decorators;
use floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::{EditorState, StateHelper};

pub fn landscape_item(landscape: LandscapeData) -> impl View {
    let active = create_rw_signal(false);

    let heightmap_filename = landscape
        .heightmap
        .as_ref()
        .expect("Couldn't get heightmap")
        .fileName
        .clone();

    let rockmap_filename = landscape
        .rockmap
        .as_ref()
        .expect("Couldn't get rockmap")
        .fileName
        .clone();

    let soil_filename = landscape
        .soil
        .as_ref()
        .expect("Couldn't get soil")
        .fileName
        .clone();

    v_stack((
        dynamic_img(
            landscape
                .rockmap
                .as_ref()
                .expect("Couldn't get rockmap")
                .normalFilePath
                .clone(),
            rockmap_filename.clone(),
        )
        .style(|s| s.width(120.0).height(120.0)),
        label(move || format!("Heightmap: {}", heightmap_filename)),
        label(move || format!("RockMap: {}", rockmap_filename)),
        label(move || format!("Soil: {}", soil_filename)),
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

pub fn landscape_browser(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    let landscape_data: RwSignal<Vec<LandscapeData>> = create_rw_signal(Vec::new());

    create_effect(move |_| {
        let state_helper = state_helper.lock().unwrap();
        let saved_state = state_helper
            .saved_state
            .as_ref()
            .expect("Couldn't get saved state")
            .lock()
            .unwrap();
        landscape_data.set(
            saved_state
                .landscapes
                .as_ref()
                .expect("Couldn't get landscape data")
                .clone(),
        );
    });

    v_stack((
        (label(|| "Landscapes"),),
        container((scroll(
            dyn_stack(
                move || landscape_data.get(),
                move |landscape_data| landscape_data.id.clone(),
                move |landscape_data| landscape_item(landscape_data),
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
