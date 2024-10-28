use std::sync::{Arc, Mutex, MutexGuard};

use floem::common::nav_button;
use floem::event::{Event, EventListener, EventPropagation};
use floem::keyboard::{Key, KeyCode, NamedKey};
use floem::peniko::Color;
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::views::{
    container, dyn_container, empty, label, scroll, stack, tab, text_input, v_stack, virtual_stack,
    VirtualDirection, VirtualItemSize,
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
use floem::{GpuHelper, View, WindowHandle};

pub fn editor_settings(
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    v_stack(((label(|| "Editor Settings"),))).style(|s| s.height_full())
}
