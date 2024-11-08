use std::sync::{Arc, Mutex, MutexGuard};

use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::floem::common::card_styles;
use midpoint_engine::floem::reactive::RwSignal;
use midpoint_engine::floem::reactive::SignalGet;
use midpoint_engine::floem::reactive::SignalUpdate;
use midpoint_engine::floem::reactive::{create_effect, create_rw_signal};
use midpoint_engine::floem::views::{container, dyn_container, empty, h_stack, label, v_stack};
use midpoint_engine::floem::IntoView;
use wgpu::util::DeviceExt;

use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::StateHelper;

use super::topographic_map::create_topographic_map;

pub fn maps_view(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    let topo_heights: RwSignal<Option<nalgebra::DMatrix<f32>>> = create_rw_signal(None);

    // TODO: reimplement now with upscaling (use original?)
    // create_effect(move |_| {
    //     let state_helper = state_helper.lock().unwrap();
    //     let renderer_state = state_helper
    //         .renderer_state
    //         .as_ref()
    //         .expect("Couldn't get saved state")
    //         .lock()
    //         .unwrap();

    //     let landscape = renderer_state
    //         .landscapes
    //         .get(0)
    //         .expect("Couldn't get first landscape");

    //     // all new projects should have 1 level created upon creation
    //     topo_heights.set(Some(landscape.heights.clone()));
    // });

    h_stack((
        v_stack(((label(|| "Maps"),)))
            .style(|s| card_styles(s))
            .style(|s| s.width(300.0)),
        dyn_container(
            move || topo_heights.get(),
            move |topo_heights_real| {
                if topo_heights_real.is_some() {
                    let topo_heights_real = topo_heights_real.expect("Couldn't get heights");
                    let topo_map = create_topographic_map(topo_heights_real);

                    container((topo_map)).into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
    ))
}
