use midpoint_engine::core::RendererState::ObjectConfig;
use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::floem::common::card_styles;
use midpoint_engine::floem::common::small_button;
use midpoint_engine::floem::views::dropdown::dropdown;
use midpoint_engine::floem::views::text;
use midpoint_engine::floem_renderer::gpu_resources;
use midpoint_engine::handlers::handle_add_landscape_texture;
use midpoint_engine::helpers::saved_data::File;
use midpoint_engine::helpers::saved_data::LandscapeTextureKinds;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;
use wgpu::util::DeviceExt;

use midpoint_engine::floem::peniko::{Brush, Color};
use midpoint_engine::floem::reactive::{
    create_effect, create_rw_signal, create_signal, RwSignal, SignalRead,
};
use midpoint_engine::floem::reactive::{SignalGet, SignalUpdate};
use midpoint_engine::floem::text::Weight;
use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::views::{container, dyn_container, empty, label};
use midpoint_engine::floem::views::{h_stack, v_stack};
use midpoint_engine::floem::GpuHelper;
use midpoint_engine::floem::IntoView;

use crate::editor_state::EditorState;
use crate::editor_state::StateHelper;
use crate::helpers::landscapes::save_landscape_texture;

use super::inputs::create_dropdown;
use super::inputs::styled_input;
use super::inputs::DropdownOption;

pub fn properties_view(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    object_selected_signal: RwSignal<bool>,
    selected_object_id_signal: RwSignal<Uuid>,
    selected_object_data: RwSignal<ObjectConfig>,
) -> impl IntoView {
    // let polygon_data = selected_polygon_data.read();

    let state_2 = Arc::clone(&state_helper);
    let state_3 = Arc::clone(&state_helper);
    let state_4 = Arc::clone(&state_helper);
    let state_5 = Arc::clone(&state_helper);
    let state_6 = Arc::clone(&state_helper);
    let state_7 = Arc::clone(&state_helper);

    let gpu_2 = Arc::clone(&gpu_helper);
    let gpu_3 = Arc::clone(&gpu_helper);

    let aside_width = 260.0;
    let quarters = (aside_width / 4.0) + (5.0 * 4.0);
    let thirds = (aside_width / 3.0) + (5.0 * 3.0);
    let halfs = (aside_width / 2.0) + (5.0 * 2.0);

    let back_active = RwSignal::new(false);
    let texture_options: RwSignal<Vec<DropdownOption>> = create_rw_signal(Vec::new());
    let initial_rockmap = create_rw_signal("".to_string());
    let initial_soil = create_rw_signal("".to_string());

    create_effect(move |_| {
        let mut state_helper = state_5.lock().unwrap();
        let mut saved_state = state_helper
            .saved_state
            .as_mut()
            .expect("Couldn't get SavedState")
            .lock()
            .unwrap();

        let dropdown_options = saved_state
            .textures
            .get_or_insert_with(Vec::new)
            .into_iter()
            .map(|file| DropdownOption {
                id: file.id.clone(),
                label: file.fileName.clone(),
            })
            .collect::<Vec<_>>();

        texture_options.set(dropdown_options);
    });

    v_stack((
        h_stack((
            small_button(
                "",
                "arrow-left",
                {
                    move |_| {
                        println!("Click back!");
                        // this action runs on_click_stop so should stop propagation
                        object_selected_signal.update(|v| {
                            *v = false;
                        });
                        selected_object_id_signal.update(|v| {
                            *v = Uuid::nil();
                        });
                        // let mut editor_state = editor_state2.lock().unwrap();
                        // editor_state.selected_polygon_id = Uuid::nil();
                        // editor_state.polygon_selected = false;
                        let mut state_helper = state_helper.lock().unwrap();
                        let mut renderer_state = state_helper
                            .renderer_state
                            .as_mut()
                            .expect("Couldn't get RendererState")
                            .lock()
                            .unwrap();
                        renderer_state.object_selected = None;
                    }
                },
                back_active,
            )
            .style(|s| s.margin_right(7.0)),
            label(|| "Properties").style(|s| s.font_size(24.0).font_weight(Weight::THIN)),
        ))
        .style(|s| s.margin_bottom(12.0)),
        h_stack((
            styled_input(
                "X:".to_string(),
                &selected_object_data.read().borrow().position.0.to_string(),
                "X Position",
                Box::new({
                    move |mut editor_state, value| {
                        // editor_state.update_position(&value);
                    }
                }),
                state_2,
                "x".to_string(),
            )
            .style(move |s| s.width(thirds).margin_right(5.0)),
            styled_input(
                "Y:".to_string(),
                &selected_object_data.read().borrow().position.1.to_string(),
                "Y Position",
                Box::new({
                    move |mut editor_state, value| {
                        // editor_state.update_position(&value);
                    }
                }),
                state_3,
                "y".to_string(),
            )
            .style(move |s| s.width(thirds).margin_right(5.0)),
            styled_input(
                "Z:".to_string(),
                &selected_object_data.read().borrow().position.2.to_string(),
                "Z Position",
                Box::new({
                    move |mut editor_state, value| {
                        // editor_state.update_position(&value);
                    }
                }),
                state_4,
                "z".to_string(),
            )
            .style(move |s| s.width(thirds)),
        ))
        .style(move |s| s.width(aside_width)),
        v_stack((
            label(|| "Rockmap Texture"),
            // selected_object_data.get() or saved_data? saved_data requires lock and signals
            create_dropdown(
                initial_rockmap.get(),
                texture_options.get(),
                move |selected_id| {
                    println!("Selected Rockmap: {}", selected_id);

                    // TODO: make DRY with Soil

                    let state_helper = state_6.lock().unwrap();
                    let saved_state = state_helper
                        .saved_state
                        .as_ref()
                        .expect("Couldn't get RendererState")
                        .lock()
                        .unwrap();
                    let levels = saved_state.levels.clone();
                    let component_id = selected_object_id_signal.get();

                    // add to saved_state
                    save_landscape_texture(
                        levels,
                        component_id.to_string(),
                        LandscapeTextureKinds::Rockmap,
                        selected_id.clone(),
                    );

                    let available_textures = saved_state
                        .textures
                        .clone()
                        .unwrap_or(Vec::new())
                        .to_owned();

                    let components = saved_state
                        .levels
                        .as_ref()
                        .expect("Couldn't get levels")
                        .get(0)
                        .as_ref()
                        .expect("Couldn't get first level")
                        .components
                        .as_ref()
                        .expect("Couldn't get components");
                    let landscape_component = components
                        .iter()
                        .find(|l| l.id == component_id.to_string())
                        .to_owned()
                        .expect("Couldn't get landscape component")
                        .to_owned();
                    let landscapes = saved_state
                        .landscapes
                        .as_ref()
                        .expect("No landscapes?")
                        .to_owned();
                    let landscape = landscapes
                        .iter()
                        .find(|l| l.id == landscape_component.asset_id);

                    drop(saved_state);
                    let renderer_state = state_helper
                        .renderer_state
                        .as_ref()
                        .expect("Couldn't get RendererState")
                        .lock()
                        .unwrap();

                    let project_id = renderer_state
                        .project_selected
                        .expect("Couldn't get selected project");

                    drop(renderer_state);

                    let renderer_state = state_helper
                        .renderer_state
                        .as_ref()
                        .expect("Couldn't get RendererState");

                    println!("Adding to scene...");

                    let gpu_helper = gpu_3.lock().unwrap();
                    let gpu_resources = gpu_helper
                        .gpu_resources
                        .as_ref()
                        .expect("Couldn't get gpu resources");

                    if let Some(texture) = available_textures
                        .clone()
                        .iter()
                        .find(move |t| t.id.clone() == selected_id.clone())
                    {
                        // add to scene
                        handle_add_landscape_texture(
                            renderer_state.clone(),
                            &gpu_resources.device,
                            &gpu_resources.queue,
                            project_id.to_string(),
                            landscape_component.id.clone(),
                            landscape_component.asset_id.clone(),
                            texture.fileName.clone(),
                            "Rockmap".to_string(),
                            landscape
                                .clone()
                                .expect("No landscape?")
                                .rockmap
                                .clone()
                                .expect("No rockmap?")
                                .fileName,
                        );
                    } else {
                        println!("Texture not available!");
                    }
                },
            ),
            label(|| "Soil Texture"),
            create_dropdown(
                initial_soil.get(),
                texture_options.get(),
                move |selected_id| {
                    println!("Selected Soil: {}", selected_id);

                    let state_helper = state_7.lock().unwrap();
                    let saved_state = state_helper
                        .saved_state
                        .as_ref()
                        .expect("Couldn't get RendererState")
                        .lock()
                        .unwrap();
                    let levels = saved_state.levels.clone();
                    let component_id = selected_object_id_signal.get();

                    save_landscape_texture(
                        levels,
                        component_id.to_string(),
                        LandscapeTextureKinds::Rockmap,
                        selected_id.clone(),
                    );

                    let available_textures = saved_state
                        .textures
                        .clone()
                        .unwrap_or(Vec::new())
                        .to_owned();

                    // let landscapes = saved_state
                    //     .landscapes
                    //     .as_ref()
                    //     .expect("No landscapes?")
                    //     .to_owned();
                    // let landscape = landscapes
                    //     .iter()
                    //     .find(|l| l.id == component_id.to_string())
                    //     .to_owned();

                    let components = saved_state
                        .levels
                        .as_ref()
                        .expect("Couldn't get levels")
                        .get(0)
                        .as_ref()
                        .expect("Couldn't get first level")
                        .components
                        .as_ref()
                        .expect("Couldn't get components");
                    let landscape_component = components
                        .iter()
                        .find(|l| l.id == component_id.to_string())
                        .to_owned()
                        .expect("Couldn't get landscape component")
                        .to_owned();
                    let landscapes = saved_state
                        .landscapes
                        .as_ref()
                        .expect("No landscapes?")
                        .to_owned();
                    let landscape = landscapes
                        .iter()
                        .find(|l| l.id == landscape_component.asset_id);

                    drop(saved_state);
                    let renderer_state = state_helper
                        .renderer_state
                        .as_ref()
                        .expect("Couldn't get RendererState")
                        .lock()
                        .unwrap();

                    let project_id = renderer_state
                        .project_selected
                        .expect("Couldn't get selected project");

                    drop(renderer_state);

                    let renderer_state = state_helper
                        .renderer_state
                        .as_ref()
                        .expect("Couldn't get RendererState");

                    let gpu_helper = gpu_2.lock().unwrap();
                    let gpu_resources = gpu_helper
                        .gpu_resources
                        .as_ref()
                        .expect("Couldn't get gpu resources");

                    println!("Adding to scene...");

                    if let Some(texture) = available_textures
                        .clone()
                        .iter()
                        .find(move |t| t.id.clone() == selected_id.clone())
                    {
                        handle_add_landscape_texture(
                            renderer_state.clone(),
                            &gpu_resources.device,
                            &gpu_resources.queue,
                            project_id.to_string(),
                            landscape_component.id.clone(),
                            landscape_component.asset_id.clone(),
                            texture.fileName.clone(),
                            "Soil".to_string(),
                            landscape
                                .clone()
                                .expect("No landscape?")
                                .soil
                                .clone()
                                .expect("No soil?")
                                .fileName,
                        );
                    } else {
                        println!("Texture not available!");
                    }
                },
            ),
        ))
        .style(move |s| s.width(aside_width)),
    ))
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
