use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};

use floem::common::{card_styles, create_icon, nav_button};
use floem::event::{Event, EventListener, EventPropagation};
use floem::keyboard::{Key, KeyCode, NamedKey};
use floem::peniko::Color;
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::style::CursorStyle;
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

use crate::editor_state::StateHelper;
use crate::helpers::projects::{get_projects, load_project_state, ProjectInfo};

pub fn project_item(
    project_info: ProjectInfo,
    sortable_items: RwSignal<Vec<ProjectInfo>>,
    project_label: String,
    icon_name: &'static str,
) -> impl IntoView {
    h_stack((
        svg(create_icon(icon_name))
            .style(|s| s.width(24).height(24).color(Color::BLACK))
            .style(|s| s.margin_right(7.0)),
        // .on_event_stop(
        //     floem::event::EventListener::PointerDown,
        //     |_| { /* Disable dragging for this view */ },
        // ),
        label(move || project_label.to_string()),
    ))
    .style(|s| {
        s.width(260.0)
            .border_radius(15.0)
            .align_items(AlignItems::FlexStart)
            .justify_start()
            .padding_vert(8)
            .background(Color::rgb(255.0, 255.0, 255.0))
            .border_bottom(1)
            .border_color(Color::rgb(200.0, 200.0, 200.0))
            .hover(|s| {
                s.background(Color::rgb(100.0, 100.0, 100.0))
                    .cursor(CursorStyle::Pointer)
            })
            .active(|s| s.background(Color::rgb(237.0, 218.0, 164.0)))
    })
    // .on_click(|_| {
    //     println!("Layer selected");
    //     EventPropagation::Stop
    // })
}

pub fn project_browser(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    // TODO: Start CommonOS File Manager to use Midpoint
    let projects = get_projects().expect("Couldn't get projects");
    // v_stack((,))).style(|s| s.height_full())

    let project_list = create_rw_signal(projects); // for long lists technically

    v_stack((
        (label(|| "Select a Project").style(|s| s.margin_bottom(4.0))),
        scroll(
            dyn_stack(
                move || project_list.get(),
                move |project| project.name.clone(),
                move |project| {
                    project_item(
                        project.clone(),
                        project_list,
                        "Project".to_string(),
                        "sphere",
                    )
                    .on_click({
                        let state_helper = state_helper.clone();

                        move |_| {
                            let mut state_helper = state_helper.lock().unwrap();

                            // retrieve saved state of project and set on helper
                            let saved_state = load_project_state(&project.name)
                                .expect("Couldn't get project saved state");
                            let saved_state = Arc::new(Mutex::new(saved_state));
                            state_helper.saved_state = Some(saved_state);

                            // update the UI signal
                            let project_selected = state_helper
                                .project_selected_signal
                                .expect("Couldn't get project selection signal");
                            let uuid = Uuid::from_str(&project.name.clone())
                                .expect("Couldn't convert project name to id");
                            project_selected.set(uuid.clone());

                            // update renderer_state with project_selected (and current_view if necessary)
                            let mut renderer_state = state_helper
                                .renderer_state
                                .as_mut()
                                .expect("Couldn't find RendererState")
                                .lock()
                                .unwrap();
                            renderer_state.project_selected = Some(uuid.clone());
                            renderer_state.current_view = "scene".to_string();

                            println!("Project selected {:?}", project.name.clone());

                            EventPropagation::Stop
                        }
                    })
                },
            )
            // .style(|s| s.flex_col().column_gap(5).padding(10))
            .into_view(),
        ),
    ))
    .style(|s| card_styles(s))
    .style(|s| s.width(300.0))
}
