use std::sync::{Arc, Mutex, MutexGuard};

use super::shared::dynamic_img;
use midpoint_engine::core::RendererState::ObjectConfig;
use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::floem::common::small_button;
use midpoint_engine::floem::reactive::SignalGet;
use midpoint_engine::floem::reactive::{create_effect, create_rw_signal, RwSignal, SignalUpdate};
use midpoint_engine::floem::taffy::{FlexDirection, FlexWrap};
use midpoint_engine::floem::views::{
    container, dyn_container, dyn_stack, empty, label, scroll, v_stack,
};
use midpoint_engine::floem::IntoView;
use midpoint_engine::floem_renderer::gpu_resources;
use midpoint_engine::handlers::handle_add_landscape;
use midpoint_engine::helpers::saved_data::{
    ComponentData, ComponentKind, File, GenericProperties, LandscapeData, LandscapeProperties,
};
use uuid::Uuid;
use wgpu::util::DeviceExt;

use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::{EditorState, StateHelper};

pub fn landscape_item(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    landscape: LandscapeData,
) -> impl View {
    let active = create_rw_signal(false);
    let disabled = create_rw_signal(false);

    let heightmap_filename = landscape
        .heightmap
        .as_ref()
        .expect("Couldn't get heightmap")
        .fileName
        .clone();

    let rockmap_filename = landscape
        .rockmap
        .as_ref()
        .expect("Couldn't get rockmap")
        .fileName
        .clone();

    let soil_filename = landscape
        .soil
        .as_ref()
        .expect("Couldn't get soil")
        .fileName
        .clone();

    v_stack((
        dynamic_img(
            landscape
                .rockmap
                .as_ref()
                .expect("Couldn't get rockmap")
                .normalFilePath
                .clone(),
            rockmap_filename.clone(),
        )
        .style(|s| s.width(120.0).height(120.0)),
        label(move || format!("Heightmap: {}", heightmap_filename)),
        label(move || format!("RockMap: {}", rockmap_filename)),
        label(move || format!("Soil: {}", soil_filename)),
        small_button(
            "Add to Scene",
            "plus",
            {
                let state_helper = state_helper.clone();
                // let heightmap_filename = heightmap_filename.clone();
                let heightmap_filename = landscape
                    .heightmap
                    .as_ref()
                    .expect("Couldn't get heightmap")
                    .fileName
                    .clone();

                move |_| {
                    let mut state_helper = state_helper.lock().unwrap();
                    let disabled = disabled.clone();

                    println!("Prearting to landscape add to scene...");

                    disabled.set(true);

                    // different than the landscape asset id, this is the component instance id
                    let landscapeComponentId = Uuid::new_v4();

                    let mut saved_state = state_helper
                        .saved_state
                        .as_ref()
                        .expect("Couldn't get RendererState")
                        .lock()
                        .unwrap();

                    // add to `levels.components` in SavedContext
                    let landscape_component = ComponentData {
                        id: landscapeComponentId.to_string().clone(),
                        kind: Some(ComponentKind::Landscape),
                        asset_id: landscape.id.clone(),
                        generic_properties: GenericProperties {
                            name: "New Landscape Component".to_string(),
                        },
                        landscape_properties: Some(LandscapeProperties {
                            // these are the visible texture ids, not the map ids, so are added after adding
                            primary_texture_id: None,
                            rockmap_texture_id: None,
                            soil_texture_id: None,
                        }),
                        model_properties: None,
                    };
                    let mut levels = saved_state.levels.as_mut().expect("Couldn't get levels");
                    levels
                        .get_mut(0)
                        .expect("Couldn't get first level")
                        .components
                        .get_or_insert_with(Vec::new)
                        .push(landscape_component);

                    state_helper.save_saved_state(saved_state);

                    // drop(saved_state);

                    let mut renderer_state = state_helper
                        .renderer_state
                        .as_mut()
                        .expect("Couldn't get RendererState")
                        .lock()
                        .unwrap();

                    // update selected_component_id in renderer state
                    renderer_state.object_selected = Some(landscapeComponentId);

                    // actually render the landscape in wgpu
                    let gpu_helper = gpu_helper.lock().unwrap();
                    let gpu_resources = gpu_helper
                        .gpu_resources
                        .as_ref()
                        .expect("Couldn't get gpu resources");

                    let proejct_selected = renderer_state
                        .project_selected
                        .as_ref()
                        .expect("Couldn't get selected project")
                        .to_string();

                    drop(renderer_state);

                    println!("Loading landscape to scene...");

                    handle_add_landscape(
                        state_helper
                            .renderer_state
                            .as_ref()
                            .expect("Couldn't get RendererState")
                            .clone(),
                        &gpu_resources.device,
                        &gpu_resources.queue,
                        proejct_selected,
                        landscape.id.clone(),
                        landscapeComponentId.to_string().clone(),
                        heightmap_filename.clone(),
                        // js_callback,
                    );

                    // update selected_component_id in signal
                    let object_selected_signal = state_helper
                        .object_selected_signal
                        .expect("Couldn't get signal");
                    object_selected_signal.set(true);
                    let selected_object_id_signal = state_helper
                        .selected_object_id_signal
                        .expect("Couldn't get signal");
                    selected_object_id_signal.set(landscapeComponentId);
                    let selected_object_data_signal = state_helper
                        .selected_object_data_signal
                        .expect("Couldn't get signal");
                    selected_object_data_signal.set(ObjectConfig {
                        id: landscapeComponentId,
                        name: "New Landscape".to_string(),
                        position: (0.0, 0.0, 0.0),
                    });

                    println!("Landscape added!");

                    disabled.set(false);
                }
            },
            active,
        )
        .disabled(move || disabled.get()),
    ))
    .style(|s| s.width(120.0))
}

pub fn landscape_browser(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    let landscape_data: RwSignal<Vec<LandscapeData>> = create_rw_signal(Vec::new());

    let state_2 = Arc::clone(&state_helper);

    create_effect(move |_| {
        let state_helper = state_helper.lock().unwrap();
        let saved_state = state_helper
            .saved_state
            .as_ref()
            .expect("Couldn't get saved state")
            .lock()
            .unwrap();
        landscape_data.set(
            saved_state
                .landscapes
                .as_ref()
                .expect("Couldn't get landscape data")
                .clone(),
        );
    });

    v_stack((
        (label(|| "Landscapes"),),
        container((scroll(
            dyn_stack(
                move || landscape_data.get(),
                move |landscape_data| landscape_data.id.clone(),
                move |landscape_data| {
                    landscape_item(state_2.clone(), gpu_helper.clone(), landscape_data)
                },
            )
            .into_view(),
        ),))
        .style(|s| {
            s.flex_direction(FlexDirection::Row)
                .flex_wrap(FlexWrap::Wrap)
        }),
    ))
    .style(|s| s.width(260.0))
}
