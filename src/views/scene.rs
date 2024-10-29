use std::sync::{Arc, Mutex, MutexGuard};

use floem::common::{card_styles, tab_button};
use floem::event::{Event, EventListener, EventPropagation};
use floem::keyboard::{Key, NamedKey};
use floem::peniko::Color;
use floem::reactive::create_signal;
use floem::views::{
    container, dyn_container, empty, label, scroll, stack, tab, v_stack, virtual_stack,
    VirtualDirection,
};
use midpoint_engine::core::Viewport::Viewport;
use wgpu::util::DeviceExt;

use floem::reactive::SignalGet;
use floem::reactive::SignalUpdate;
use floem::views::Decorators;
use floem::views::VirtualItemSize;
use floem::IntoView;
use floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::StateHelper;

use super::landscape_browser::landscape_browser;
use super::model_browser::model_browser;
use super::texture_browser::texture_browser;

pub fn scene_view(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    let state_2 = Arc::clone(&state_helper);

    let tabs: im::Vector<&str> = vec!["Models", "Landscapes", "Textures"]
        .into_iter()
        .collect();
    let (tabs, _set_tabs) = create_signal(tabs);
    let (active_tab, set_active_tab) = create_signal(0);

    let list = scroll({
        virtual_stack(
            VirtualDirection::Horizontal,
            VirtualItemSize::Fixed(Box::new(|| 120.0)),
            move || tabs.get(),
            move |item| *item,
            move |item| {
                let index = tabs
                    .get_untracked()
                    .iter()
                    .position(|it| *it == item)
                    .unwrap();
                let active = index == active_tab.get();
                // let icon_name = match item {
                //     "Projects" => "folder-plus",
                //     "Settings" => "gear",
                //     _ => "plus",
                // };
                // let destination_view = match item {
                //     "Projects" => "projects",
                //     "Settings" => "editor_settings",
                //     _ => "plus",
                // };
                stack((
                    // label(move || item).style(|s| s.font_size(18.0)),
                    // svg(create_icon("plus")).style(|s| s.width(24).height(24)),
                    tab_button(
                        item,
                        // icon_name,
                        Some({
                            let state_helper = state_helper.clone();

                            move || {
                                println!("Click...");
                                set_active_tab.update(|v: &mut usize| {
                                    *v = tabs
                                        .get_untracked()
                                        .iter()
                                        .position(|it| *it == item)
                                        .unwrap();
                                });

                                // EventPropagation::Continue
                            }
                        }),
                        index,
                        active_tab,
                    ),
                ))
                // .on_click()
                .on_event(EventListener::KeyDown, move |e| {
                    if let Event::KeyDown(key_event) = e {
                        let active = active_tab.get();
                        if key_event.modifiers.is_empty() {
                            match key_event.key.logical_key {
                                Key::Named(NamedKey::ArrowUp) => {
                                    if active > 0 {
                                        set_active_tab.update(|v| *v -= 1)
                                    }
                                    EventPropagation::Stop
                                }
                                Key::Named(NamedKey::ArrowDown) => {
                                    if active < tabs.get().len() - 1 {
                                        set_active_tab.update(|v| *v += 1)
                                    }
                                    EventPropagation::Stop
                                }
                                _ => EventPropagation::Continue,
                            }
                        } else {
                            EventPropagation::Continue
                        }
                    } else {
                        EventPropagation::Continue
                    }
                })
                .keyboard_navigatable()
            },
        )
        .style(|s| s.flex_row().width(260.0).padding_vert(7.0).height(55.0))
    })
    // .scroll_style(|s| s.shrink_to_fit())
    .style(|s| s.height(55.0));

    v_stack((
        (label(|| "Scene"),),
        v_stack((
            list, // tab list
            tab(
                // active tab
                move || active_tab.get(),
                move || tabs.get(),
                |it| *it,
                move |it| match it {
                    "Models" => {
                        model_browser(state_2.clone(), gpu_helper.clone(), viewport.clone())
                            .into_any()
                    }
                    "Landscapes" => {
                        landscape_browser(state_2.clone(), gpu_helper.clone(), viewport.clone())
                            .into_any()
                    }
                    "Textures" => {
                        texture_browser(state_2.clone(), gpu_helper.clone(), viewport.clone())
                            .into_any()
                    }
                    _ => label(|| "Not implemented".to_owned()).into_any(),
                },
            )
            .style(|s| s.flex_col().items_start().margin_top(20.0)),
        )),
    ))
    .style(|s| card_styles(s))
    .style(|s| s.width(300.0))
}
