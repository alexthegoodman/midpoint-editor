use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use super::keyframe_properties::keyframe_properties;
use super::keyframe_timeline::{
    create_timeline, KeyframeId, TimelineConfig, TimelineGridView, TimelineState,
};
use super::part_browser::part_browser;
use super::part_properties::{self, part_properties};
use super::shared::dynamic_img;
use super::skeleton_browser::skeleton_browser;
use super::skeleton_properties::skeleton_properties;
use midpoint_engine::animations::motion_path::{AnimationPlayback, Keyframe, SkeletonMotionPath};
use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::floem::common::{card_styles, small_button, tab_button};
use midpoint_engine::floem::event::{Event, EventListener, EventPropagation};
use midpoint_engine::floem::keyboard::{Key, NamedKey};
use midpoint_engine::floem::peniko::Color;
use midpoint_engine::floem::reactive::SignalUpdate;
use midpoint_engine::floem::reactive::{create_effect, SignalGet};
use midpoint_engine::floem::reactive::{create_rw_signal, create_signal};
use midpoint_engine::floem::views::{
    container, dyn_container, dyn_stack, empty, h_stack, label, scroll, stack, tab, v_stack,
};
use midpoint_engine::floem::IntoView;

use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::UIMessage;
use crate::editor_state::{EditorState, StateHelper};
use crate::helpers::animations::{AnimationData, UIKeyframe};

pub fn animations_view(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    let state_2 = Arc::clone(&state_helper);
    let state_3 = Arc::clone(&state_helper);
    let state_4 = Arc::clone(&state_helper);
    let state_5 = Arc::clone(&state_helper);
    let state_6 = Arc::clone(&state_helper);
    let state_7 = Arc::clone(&state_helper);

    let gpu_2 = Arc::clone(&gpu_helper);
    let gpu_3 = Arc::clone(&gpu_helper);

    let viewport_2 = Arc::clone(&viewport);
    let viewport_3 = Arc::clone(&viewport);

    let tabs: im::Vector<&str> = vec!["Parts", "Skeletons"].into_iter().collect();
    let (tabs, _set_tabs) = create_signal(tabs);
    let (active_tab, set_active_tab) = create_signal(0);

    let part_selected_signal = create_rw_signal(false);
    let selected_part_id_signal = create_rw_signal(String::new());
    let skeleton_selected_signal = create_rw_signal(false);
    let selected_skeleton_id_signal = create_rw_signal(String::new());

    let active_1 = create_rw_signal(false);
    let active_2 = create_rw_signal(false);

    let motion_paths = create_rw_signal(Vec::new());
    let animation_data: midpoint_engine::floem::reactive::RwSignal<Option<AnimationData>> =
        create_rw_signal(None);

    let selected_keyframes: midpoint_engine::floem::reactive::RwSignal<Vec<UIKeyframe>> =
        create_rw_signal(Vec::new());

    create_effect(move |_| {
        println!("start effect");
        let state_helper = state_6.lock().unwrap();
        let saved_state = state_helper
            .saved_state
            .as_ref()
            .expect("Couldn't get SavedState");
        let saved_state = saved_state.lock().unwrap();

        let relevant_paths: Vec<SkeletonMotionPath> = saved_state
            .motion_paths
            .iter()
            .filter(|mp| mp.target.skeleton_id == selected_skeleton_id_signal.get())
            .map(|mp| mp.clone())
            .collect();

        let animations = AnimationData::from_motion_paths(relevant_paths.clone());

        motion_paths.set(relevant_paths);
        animation_data.set(Some(animations));

        println!("finish effect");
    });

    h_stack((
        /* Object Browsers */
        dyn_container(
            move || !part_selected_signal.get() && !skeleton_selected_signal.get(),
            move |either_selected_real| {
                let state_helper = state_helper.clone();

                let list = scroll({
                    dyn_stack(
                        move || tabs.get(),
                        move |item| *item,
                        move |item| {
                            let index = tabs
                                .get_untracked()
                                .iter()
                                .position(|it| *it == item)
                                .unwrap();
                            let active = index == active_tab.get();
                            stack((tab_button(
                                item,
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
                            ),))
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
                    .style(|s| s.flex_row().padding_vert(7.0).height(55.0))
                })
                .style(|s| s.height(55.0).width(260.0));

                if either_selected_real {
                    container(
                        (container((v_stack((
                            list, // tab list
                            tab(
                                // active tab
                                move || active_tab.get(),
                                move || tabs.get(),
                                |it| *it,
                                {
                                    let state_2 = state_2.clone();
                                    let state_5 = state_5.clone();
                                    let gpu_helper = gpu_helper.clone();
                                    let viewport = viewport.clone();

                                    move |it| match it {
                                        "Parts" => part_browser(
                                            state_2.clone(),
                                            gpu_helper.clone(),
                                            viewport.clone(),
                                            part_selected_signal,
                                            selected_part_id_signal,
                                        )
                                        .into_any(),
                                        "Skeletons" => skeleton_browser(
                                            state_5.clone(),
                                            gpu_helper.clone(),
                                            viewport.clone(),
                                            skeleton_selected_signal,
                                            selected_skeleton_id_signal,
                                        )
                                        .into_any(),
                                        _ => label(|| "Not implemented".to_owned()).into_any(),
                                    }
                                },
                            )
                            .style(|s| s.flex_col().items_start().margin_top(20.0)),
                        ))
                        .style(|s| card_styles(s))
                        .style(|s| s.width(300.0)),))),
                    )
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
        /* Part Properties */
        dyn_container(
            move || part_selected_signal.get(),
            move |part_selected_real| {
                if part_selected_real {
                    part_properties(
                        state_3.clone(),
                        gpu_2.clone(),
                        viewport_2.clone(),
                        part_selected_signal,
                        selected_part_id_signal,
                    )
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
        /* Skeleton Properties + Editor (sidebar + timeline) */
        dyn_container(
            move || skeleton_selected_signal.get() && animation_data.get().is_some(),
            move |skeleton_selected_real| {
                if skeleton_selected_real {
                    let state = TimelineState {
                        current_time: Duration::from_secs_f64(0.0),
                        zoom_level: 1.0,
                        scroll_offset: 0.0,
                        // selected_keyframes: Vec::new(),
                        property_expansions: im::HashMap::from_iter([
                            ("position".to_string(), true),
                            ("rotation".to_string(), true),
                        ]),
                        dragging: None,
                        hovered_keyframe: None,
                        selected_keyframes,
                    };

                    let config = TimelineConfig {
                        width: 1200.0,
                        height: 300.0,
                        header_height: 30.0,
                        property_width: 200.0,
                        row_height: 24.0,
                        // offset_x: 325.0,
                        // offset_y: 300.0,
                        offset_x: 0.0,
                        offset_y: 0.0,
                    };

                    let keyframe_timeline = create_timeline(state, config, animation_data);

                    let state_4 = Arc::clone(&state_4);
                    let state_7 = Arc::clone(&state_7);
                    let gpu_3 = Arc::clone(&gpu_3);
                    let viewport_3 = Arc::clone(&viewport_3);
                    let state_7b = Arc::clone(&state_7);
                    let gpu_3b = Arc::clone(&gpu_3);
                    let viewport_3b = Arc::clone(&viewport_3);

                    h_stack((
                        dyn_container(
                            move || selected_keyframes.get().len() > 0,
                            move |selected_keyframe_real| {
                                if selected_keyframe_real {
                                    keyframe_properties(
                                        state_7b.clone(),
                                        gpu_3b.clone(),
                                        viewport_3b.clone(),
                                        selected_keyframes,
                                        selected_skeleton_id_signal,
                                        motion_paths,
                                        animation_data,
                                    )
                                    .into_any()
                                } else {
                                    skeleton_properties(
                                        state_4.clone(),
                                        gpu_3.clone(),
                                        viewport_3.clone(),
                                        selected_skeleton_id_signal.get(),
                                        motion_paths,
                                    )
                                    .into_any()
                                }
                            },
                        ),
                        v_stack((
                            h_stack((
                                small_button(
                                    "Play",
                                    "plus",
                                    {
                                        let state_7 = state_7.clone();

                                        move |_| {
                                            let state_helper = state_7.lock().unwrap();
                                            let renderer_state = state_helper
                                                .renderer_state
                                                .as_ref()
                                                .expect("Couldn't get RendererState");
                                            let mut renderer_state = renderer_state.lock().unwrap();

                                            let animation_playback =
                                                AnimationPlayback::new(motion_paths.get());

                                            println!("insert animation_playback");

                                            renderer_state
                                                .active_animations
                                                .push(animation_playback);
                                        }
                                    },
                                    active_1,
                                ),
                                small_button("Insert Keyframe", "plus", |_| {}, active_2),
                            ))
                            .style(|s| s.margin_top(300.0)),
                            keyframe_timeline,
                        )),
                    ))
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
    ))
}
