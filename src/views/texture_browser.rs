use std::sync::{Arc, Mutex, MutexGuard};

use floem::common::card_styles;
use floem::views::{container, dyn_container, empty, label, v_stack};
use midpoint_engine::core::Viewport::Viewport;
use wgpu::util::DeviceExt;

use floem::views::Decorators;
use floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::{EditorState, StateHelper};

pub fn texture_browser(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    v_stack(((label(|| "Textures"),))).style(|s| s.width(260.0))
}
