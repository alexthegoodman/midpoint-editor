use std::sync::{Arc, Mutex, MutexGuard};

use floem::common::card_styles;
use floem::views::{container, dyn_container, empty, label, v_stack};
use midpoint_engine::core::Viewport::Viewport;
use wgpu::util::DeviceExt;

use floem::views::Decorators;
use floem::{GpuHelper, View, WindowHandle};

pub fn scene_view(gpu_helper: Arc<Mutex<GpuHelper>>, viewport: Arc<Mutex<Viewport>>) -> impl View {
    v_stack(((label(|| "Scene"),)))
        .style(|s| card_styles(s))
        .style(|s| s.width(300.0))
}
