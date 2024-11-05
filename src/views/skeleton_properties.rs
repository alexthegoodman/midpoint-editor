use midpoint_engine::animations::skeleton::Joint;
use midpoint_engine::floem::common::create_icon;
use midpoint_engine::floem::common::small_button;
use midpoint_engine::floem::event::EventListener;
use midpoint_engine::floem::event::EventPropagation;
use midpoint_engine::floem::peniko::Color;
use midpoint_engine::floem::reactive::create_rw_signal;
use midpoint_engine::floem::reactive::RwSignal;
use midpoint_engine::floem::reactive::SignalGet;
use midpoint_engine::floem::reactive::SignalUpdate;
use midpoint_engine::floem::style::CursorStyle;
use midpoint_engine::floem::taffy::AlignItems;
use midpoint_engine::floem::text::Weight;
use midpoint_engine::floem::views::h_stack;
use midpoint_engine::floem::views::svg;
use std::sync::{Arc, Mutex};

use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::floem::common::card_styles;
use midpoint_engine::floem::views::{container, dyn_container, dyn_stack, empty, label, v_stack};
use wgpu::util::DeviceExt;

use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{GpuHelper, IntoView, View, WindowHandle};

use crate::editor_state::StateHelper;

pub fn skeleton_properties(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    let back_active = create_rw_signal(false);

    v_stack((h_stack((
        small_button(
            "",
            "arrow-left",
            {
                move |_| {
                    println!("Click back!");
                    // this action runs on_click_stop so should stop propagation
                    // object_selected_signal.update(|v| {
                    //     *v = false;
                    // });
                    // selected_object_id_signal.update(|v| {
                    //     *v = Uuid::nil();
                    // });
                }
            },
            back_active,
        )
        .style(|s| s.margin_right(7.0)),
        label(|| "Properties").style(|s| s.font_size(24.0).font_weight(Weight::THIN)),
        // joint_tree(state_helper, joints, dragger_id),
    ))
    .style(|s| s.margin_bottom(12.0)),))
    .style(|s| card_styles(s))
    .style(|s| {
        s.width(300)
            // .absolute()
            .height(800.0)
            .margin_left(0.0)
            .margin_top(20)
        // .z_index(10)
    })
}
