use floem::views::{
    container, dyn_container, empty, label, scroll, stack, tab, text_input, virtual_stack,
    VirtualDirection, VirtualItemSize,
};
use midpoint_engine::core::Viewport::Viewport;
use std::sync::{Arc, Mutex, MutexGuard};
use wgpu::util::DeviceExt;

use floem::GpuHelper;
use floem::IntoView;

// use crate::editor_state::EditorState;
// use crate::PolygonClickHandler;

use super::aside::tab_interface;
// use super::properties_panel::properties_view;

pub fn app_view(
    //     editor_state: Arc<Mutex<EditorState>>,
    //     editor: std::sync::Arc<Mutex<common_vector::editor::Editor>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> impl IntoView {
    container((
        // label(move || format!("Value: {counter}")).style(|s| s.margin_bottom(10)),
        tab_interface(
            gpu_helper.clone(),
            // editor,
            // editor_cloned,
            viewport.clone(),
            // handler,
            // square_handler,
            // polygon_selected,
        ),
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
    // .style(|s| s.flex_col().items_center())
}
