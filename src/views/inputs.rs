use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};

use bytemuck::Contiguous;
use floem::common::input_styles;
use floem::event::{Event, EventListener, EventPropagation};
use floem::keyboard::{Key, KeyCode, NamedKey};
use floem::kurbo::Size;
use floem::peniko::Color;
use floem::reactive::{create_effect, create_rw_signal, create_signal, RwSignal, SignalRead};
use floem::style::{Background, CursorStyle, Style, Transition};
use floem::taffy::{AlignItems, Position};
use floem::text::Weight;
use floem::views::dropdown::dropdown;
use floem::views::editor::view;
use floem::views::{
    container, dyn_container, empty, label, scroll, stack, tab, text_input, virtual_stack,
    VirtualDirection, VirtualItemSize,
};
use floem::window::WindowConfig;
use floem_renderer::gpu_resources::{self, GpuResources};
use floem_winit::dpi::{LogicalSize, PhysicalSize};
use floem_winit::event::{ElementState, MouseButton};
use uuid::Uuid;
// use views::buttons::{nav_button, option_button, small_button};
// use winit::{event_loop, window};
use wgpu::util::DeviceExt;

use floem::context::PaintState;
// use floem::floem_reactive::SignalGet;
use floem::reactive::{SignalGet, SignalUpdate};
use floem::views::text;
use floem::views::Decorators;
use floem::views::{h_stack, svg, v_stack};
use floem::{
    views::{button, dropdown},
    IntoView,
};
use floem::{Application, CustomRenderCallback};
use floem::{GpuHelper, View, WindowHandle};

use floem::unit::{Auto, DurationUnitExt, Pct, UnitExt};
use std::time::Duration;

use crate::editor_state::{EditorState, StateHelper};

pub fn styled_input(
    label_text: String,
    initial_value: &str,
    placeholder: &str,
    on_event_stop: Box<dyn Fn(MutexGuard<StateHelper>, String) + 'static>,
    // mut values: HashMap<String, RwSignal<String>>,
    mut state_helper: Arc<Mutex<StateHelper>>,
    name: String,
) -> impl IntoView {
    let value = create_rw_signal(initial_value.to_string());

    let state_2 = Arc::clone(&state_helper);

    create_effect({
        let name = name.clone();
        move |_| {
            // need to value.set in undos defined in properties_panel
            // let mut state_helper = state_helper.lock().unwrap();
            // state_helper.register_signal(name.to_string(), value);
        }
    });

    v_stack((
        label(move || label_text.clone()).style(|s| s.font_size(10.0).margin_bottom(1.0)),
        text_input(value)
            .on_event_stop(EventListener::KeyUp, move |event: &Event| {
                if let Event::KeyUp(key_event) = event {
                    let state_helper = state_2.lock().unwrap();

                    // Handle keyboard shortcuts first
                    // if editor_state.current_modifiers.control_key() {
                    //     match key_event.key.logical_key {
                    //         Key::Character(ref c) if c.to_lowercase() == "z" => {
                    //             // Don't trigger value update for Ctrl+Z
                    //             return;
                    //         }
                    //         Key::Character(ref c) if c.to_lowercase() == "y" => {
                    //             // Don't trigger value update for Ctrl+Y
                    //             return;
                    //         }
                    //         _ => {}
                    //     }
                    // }

                    match key_event.key.logical_key {
                        // Ignore all control and navigation keys
                        Key::Named(NamedKey::ArrowUp)
                        | Key::Named(NamedKey::ArrowDown)
                        | Key::Named(NamedKey::ArrowLeft)
                        | Key::Named(NamedKey::ArrowRight)
                        | Key::Named(NamedKey::Enter)
                        | Key::Named(NamedKey::Tab)
                        | Key::Named(NamedKey::Escape)
                        | Key::Named(NamedKey::Home)
                        | Key::Named(NamedKey::End)
                        | Key::Named(NamedKey::PageUp)
                        | Key::Named(NamedKey::PageDown)
                        | Key::Named(NamedKey::Control)
                        | Key::Named(NamedKey::Shift)
                        | Key::Named(NamedKey::Alt)
                        | Key::Named(NamedKey::Meta) => {
                            // Ignore these keys
                            println!("Ignoring control/navigation key");
                            return;
                        }
                        // Only trigger value update for actual content changes
                        _ => {
                            println!("Content change detected: {:?}", key_event.key.logical_key);
                            let current_value = value.get();
                            on_event_stop(state_helper, current_value);
                        }
                    }
                }
            })
            .placeholder(placeholder)
            .style(|s| input_styles(s)),
    ))
    .style(|s| s.margin_bottom(10))
}

// Define an option type for better ergonomics
#[derive(Clone)]
pub struct DropdownOption {
    pub id: String,
    pub label: String,
}

pub fn create_dropdown<F>(
    initial_selection: String,
    options: Vec<DropdownOption>,
    on_selection: F,
) -> impl IntoView
where
    F: Fn(String) + Clone + 'static,
{
    let (selected, set_selected) = create_signal(initial_selection);
    let (options, _set_options) = create_signal(options);

    // // Convert our options to DropdownOption format
    // let dropdown_options = options
    //     .get()
    //     .into_iter()
    //     .map(|opt| DropdownOption {
    //         id: opt.id.clone(),
    //         label: opt.label,
    //     })
    //     .collect::<Vec<_>>();

    // Start with the default option
    let mut dropdown_options = vec![DropdownOption {
        id: "".to_string(),
        label: "Select a file".to_string(),
    }];

    // Add the file options
    dropdown_options.extend(options.get().into_iter().map(|file| DropdownOption {
        id: file.id.clone(),
        label: file.label.clone(),
    }));

    // Create the dropdown
    let dropdown = {
        let dropdown_2 = dropdown_options.clone();
        let set_selected = set_selected.clone();
        let on_selection = on_selection.clone();
        // dropdown(
        //     // Active item selector
        //     move || {
        //         dropdown_options
        //             .clone()
        //             .into_iter()
        //             .find(|opt| opt.id == selected.get())
        //             .unwrap_or_else(|| DropdownOption {
        //                 id: "".to_string(),
        //                 label: "Select an option".to_string(),
        //             })
        //     },
        //     // Main view (what's shown when dropdown is closed)
        //     |item: DropdownOption| Box::new(container(label(move || item.label.clone()))),
        //     // Iterator for options
        //     dropdown_2.clone(),
        //     // List item view (how each option is rendered)
        //     move |item: DropdownOption| {
        //         let set_selected = set_selected.clone();
        //         let on_selection = on_selection.clone();
        //         Box::new(
        //             container(label(move || item.label.clone())).on_click(move |_| {
        //                 println!("Select dropdown option");
        //                 set_selected.set(item.id.clone());
        //                 on_selection(item.id.clone());
        //                 EventPropagation::Continue
        //             }),
        //         )
        //     },
        // )
        dropdown(
            move || {
                let selected = selected.get();
                dropdown_options
                    .clone()
                    .into_iter()
                    .find(|opt| opt.id == selected)
                    .expect("Couldn't find dropdown option")
            },
            // Main view (selected item)
            move |item: DropdownOption| {
                text(item.label.to_string())
                    .style(|s| {
                        s.background(Color::rgba(0.5, 0.5, 0.5, 1.0))
                            .padding_left(8)
                            .padding_right(8)
                            .width_full()
                    })
                    .into_any()
            },
            // Options iterator
            dropdown_2.clone(),
            // List item view
            move |item: DropdownOption| {
                text(item.label.to_string())
                    .style(|s| {
                        s.background(Color::rgba(0.5, 0.5, 0.5, 1.0))
                            .padding(8)
                            .hover(|s| s.background(Color::rgba(0.5, 0.5, 0.5, 1.0)))
                            .width_full()
                            .cursor(CursorStyle::Pointer)
                    })
                    .into_any()
            },
        )
        .on_accept(move |new: DropdownOption| {
            set_selected.set(new.id.clone());
            on_selection(new.id.clone());
        })
        .style(|s| {
            s.width(200)
                .background(Color::rgba(0.5, 0.5, 0.5, 1.0))
                .border(1)
                .border_color(Color::rgba(0.5, 0.5, 0.5, 1.0))
                .border_radius(4)
                .position(Position::Relative)
                // Style for the dropdown menu container
                .class(dropdown::DropdownClass, |s| {
                    s.background(Color::rgba(0.5, 0.5, 0.5, 1.0))
                        .border(1)
                        .border_color(Color::rgba(0.5, 0.5, 0.5, 1.0))
                        .border_radius(4)
                        .z_index(999)
                        .position(Position::Absolute)
                })
        })
    };

    dropdown
}
