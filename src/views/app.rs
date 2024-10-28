use floem::reactive::create_effect;
use floem::reactive::create_rw_signal;
use floem::reactive::SignalGet;
use floem::views::{
    container, dyn_container, empty, label, scroll, stack, tab, text_input, virtual_stack,
    VirtualDirection, VirtualItemSize,
};
use midpoint_engine::core::Viewport::Viewport;
use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;
use wgpu::util::DeviceExt;

use floem::GpuHelper;
use floem::IntoView;

use crate::editor_state::StateHelper;

use super::aside::project_tab_interface;
use super::aside::welcome_tab_interface;

pub fn project_view(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> impl IntoView {
    // object_selected? model_selected?

    container((
        project_tab_interface(state_helper.clone(), gpu_helper.clone(), viewport.clone()),
        // this properties pabel "covers" the tools panels which are inserted within tab_interface
        // dyn_container(
        //     move || polygon_selected.get(),
        //     move |polygon_selected_real| {
        //         if polygon_selected_real {
        //             properties_view(
        //                 editor_state.clone(),
        //                 gpu_helper.clone(),
        //                 editor_cloned4.clone(),
        //                 viewport.clone(),
        //                 polygon_selected,
        //                 selected_polygon_id,
        //                 selected_polygon_data,
        //             )
        //             .into_any()
        //         } else {
        //             empty().into_any()
        //         }
        //     },
        // ),
    ))
}

pub fn selection_view(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> impl IntoView {
    container((welcome_tab_interface(
        state_helper.clone(),
        gpu_helper.clone(),
        viewport.clone(),
    ),))
}

pub fn app_view(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
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
                selection_view(state_helper.clone(), gpu_helper.clone(), viewport.clone())
                    .into_any()
            }
        },
    )
}
