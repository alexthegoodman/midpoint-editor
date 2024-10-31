use std::sync::{Arc, Mutex, MutexGuard};

use midpoint_engine::floem::common::card_styles;
use midpoint_engine::floem::views::{container, dyn_container, empty, label, v_stack};
use midpoint_engine::core::Viewport::Viewport;
use wgpu::util::DeviceExt;

use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{GpuHelper, View, WindowHandle};

pub fn story_view(gpu_helper: Arc<Mutex<GpuHelper>>, viewport: Arc<Mutex<Viewport>>) -> impl View {
    v_stack(((label(|| "Story"),)))
        .style(|s| card_styles(s))
        .style(|s| s.width(300.0))
}
