use std::sync::{Arc, Mutex, MutexGuard};

use floem::common::{create_icon, nav_button};
use floem::event::{Event, EventListener, EventPropagation};
use floem::keyboard::{Key, KeyCode, NamedKey};
use floem::peniko::Color;
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::taffy::AlignItems;
use floem::views::{
    container, dyn_container, dyn_stack, empty, h_stack, label, scroll, stack, svg, tab,
    text_input, v_stack, virtual_list, virtual_stack, VirtualDirection, VirtualItemSize,
};
use midpoint_engine::core::Viewport::Viewport;
use uuid::Uuid;
// use views::buttons::{nav_button, option_button, small_button};
// use winit::{event_loop, window};
use wgpu::util::DeviceExt;

use floem::context::PaintState;
// use floem::floem_reactive::SignalGet;
use floem::reactive::{SignalGet, SignalUpdate};
use floem::views::text;
use floem::views::Decorators;
use floem::IntoView;
use floem::{GpuHelper, View, WindowHandle};

use crate::helpers::projects::{get_projects, ProjectInfo};

pub fn project_item(
    project_info: ProjectInfo,
    sortable_items: RwSignal<Vec<ProjectInfo>>,
    project_label: String,
    icon_name: &'static str,
) -> impl IntoView {
    h_stack((
        svg(create_icon(icon_name))
            .style(|s| s.width(24).height(24).color(Color::BLACK))
            .style(|s| s.margin_right(7.0))
            .on_event_stop(
                floem::event::EventListener::PointerDown,
                |_| { /* Disable dragging for this view */ },
            ),
        label(move || project_label.to_string()),
    ))
    .style(|s| {
        s.width(220.0)
            .border_radius(15.0)
            .align_items(AlignItems::Center)
            .padding_vert(8)
            .background(Color::rgb(255.0, 239.0, 194.0))
            .border_bottom(1)
            .border_color(Color::rgb(200.0, 200.0, 200.0))
            .hover(|s| s.background(Color::rgb(222.0, 206.0, 160.0)))
            .active(|s| s.background(Color::rgb(237.0, 218.0, 164.0)))
    })
    // .on_click(|_| {
    //     println!("Layer selected");
    //     EventPropagation::Stop
    // })
}

pub fn project_browser(
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    // TODO: Start CommonOS File Manager to use Midpoint
    let projects = get_projects().expect("Couldn't get projects");
    // v_stack((,))).style(|s| s.height_full())

    let project_list = create_rw_signal(projects); // for long lists technically

    container((
        (label(|| "Select a Project")),
        scroll(
            dyn_stack(
                move || project_list.get(),
                move |project| project.clone(),
                move |project| project_item(project, project_list, "Project".to_string(), "sphere"),
            )
            .style(|s| s.flex_col().column_gap(5).padding(10))
            .into_view(),
        ),
    ))
    .style(|s| s.padding_vert(20.0).flex_col().items_center())
}
