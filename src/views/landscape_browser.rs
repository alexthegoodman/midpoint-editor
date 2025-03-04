use std::fs;
use std::sync::{Arc, Mutex, MutexGuard};

use super::shared::dynamic_img;
use midpoint_engine::core::RendererState::ObjectConfig;
use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::floem::common::{simple_button, small_button};
use midpoint_engine::floem::reactive::SignalGet;
use midpoint_engine::floem::reactive::{create_effect, create_rw_signal, RwSignal, SignalUpdate};
use midpoint_engine::floem::taffy::{FlexDirection, FlexWrap};
use midpoint_engine::floem::views::{
    container, dyn_container, dyn_stack, empty, label, scroll, v_stack,
};
use midpoint_engine::floem::IntoView;
use midpoint_engine::floem_renderer::gpu_resources;
use midpoint_engine::handlers::handle_add_landscape;
use midpoint_engine::helpers::landscapes::upscale_tiff_heightmap;
use midpoint_engine::helpers::saved_data::{
    ComponentData, ComponentKind, File, GenericProperties, LandscapeData, LandscapeProperties,
};
use midpoint_engine::helpers::utilities::{get_heightmap_dir, get_soilmap_dir};
use rfd::FileDialog;
use uuid::Uuid;
use wgpu::util::DeviceExt;

use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::{EditorState, StateHelper};
use crate::helpers::utilities::get_common_os_dir;

pub fn landscape_item(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    landscape: LandscapeData,
) -> impl View {
    let state_2 = Arc::clone(&state_helper);
    let active = create_rw_signal(false);
    let upscale_active = create_rw_signal(false);
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

    let landscape_id = landscape.id.clone();

    v_stack((
        dynamic_img(
            landscape
                .rockmap
                .as_ref()
                .expect("Couldn't get rockmap")
                .normalFilePath
                .clone(),
            rockmap_filename.clone(),
            120.0,
            120.0,
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
                            position: [0.0, 0.0, 0.0],
                            rotation: [0.0, 0.0, 0.0],
                            scale: [1.0, 1.0, 1.0],
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
                        .push(landscape_component.clone());

                    let project_id = state_helper
                        .project_selected_signal
                        .expect("Couldn't get project signal")
                        .get();

                    state_helper.save_saved_state(project_id, saved_state);

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
                        [0.0, 0.0, 0.0],
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
                    selected_object_data_signal.set(landscape_component.clone());

                    println!("Landscape added!");

                    disabled.set(false);
                }
            },
            active,
        )
        .disabled(move || disabled.get()),
        small_button(
            "Upscale by 10x",
            "plus",
            {
                let landscape_id = landscape_id.clone();
                let state_helper = state_2.clone();
                let heightmap_filepath = landscape
                    .heightmap
                    .as_ref()
                    .expect("Couldn't get heightmap path")
                    .normalFilePath
                    .clone();
                let heightmap_filename = landscape
                    .heightmap
                    .as_ref()
                    .expect("Couldn't get heightmap name")
                    .fileName
                    .clone();

                move |_| {
                    let state_helper = state_helper.lock().unwrap();
                    let renderer_state = state_helper
                        .renderer_state
                        .as_ref()
                        .expect("Couldn't get RendererState");
                    let renderer_state = renderer_state.lock().unwrap();
                    let project_id = renderer_state
                        .project_selected
                        .expect("Couldn't get selected project id");

                    let sync_dir = get_common_os_dir().expect("Couldn't get CommonOS directory");
                    let upscaled_dir = sync_dir.join(format!(
                        "midpoint/projects/{}/landscapes/{}/heightmaps/upscaled",
                        project_id.to_string(),
                        landscape_id,
                    ));
                    let upscaled_dir = upscaled_dir.as_path();

                    let heightmap_full_path =
                        sync_dir.join(format!("{}/{}", heightmap_filepath, heightmap_filename));
                    let heightmap_full_path = heightmap_full_path.as_path();

                    upscale_tiff_heightmap(heightmap_full_path, upscaled_dir, 16, 16, 0.0)
                        .expect("Couldn't upscale landscape");
                }
            },
            upscale_active,
        ),
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
    let state_3 = Arc::clone(&state_helper);

    let landscape_modal_open = create_rw_signal(false);
    let new_heightmap_path = create_rw_signal(None);
    let new_soilmap_path = create_rw_signal(None);
    let new_rockmap_path = create_rw_signal(None);

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
        simple_button("Add Landscape".to_string(), move |_| {
            landscape_modal_open.set(true);
        }),
        dyn_container(
            move || landscape_modal_open.get(),
            move |is_open| {
                let state_3 = state_3.clone();

                if is_open {
                    v_stack((
                        simple_button("Add Heightmap".to_string(), move |_| {
                            let file = FileDialog::new()
                                .add_filter("image", &["tiff"])
                                // .add_filter("rust", &["rs", "toml"])
                                .set_directory("/")
                                .pick_file();

                            new_heightmap_path.set(file);
                        }),
                        simple_button("Add Soil Map".to_string(), move |_| {
                            let file = FileDialog::new()
                                .add_filter("image", &["png"])
                                // .add_filter("rust", &["rs", "toml"])
                                .set_directory("/")
                                .pick_file();

                            new_soilmap_path.set(file);
                        }),
                        simple_button("Add Rock Map".to_string(), move |_| {
                            let file = FileDialog::new()
                                .add_filter("image", &["png"])
                                // .add_filter("rust", &["rs", "toml"])
                                .set_directory("/")
                                .pick_file();

                            new_rockmap_path.set(file);
                        }),
                        simple_button("Save Landscape".to_string(), move |_| {
                            if (new_heightmap_path.get().is_some()
                                && new_soilmap_path.get().is_some()
                                && new_rockmap_path.get().is_some())
                            {
                                let new_id = Uuid::new_v4();

                                let state_helper = state_3.lock().unwrap();
                                let project_id = state_helper
                                    .project_selected_signal
                                    .expect("Couldn't get project signal")
                                    .get();

                                // Move Heightmap
                                let original_heightmap_path = new_heightmap_path
                                    .get()
                                    .expect("Couldn't get heightmap path");
                                let heightmap_dir =
                                    get_heightmap_dir(&project_id.to_string(), &new_id.to_string())
                                        .expect("Couldn't get heightmap dir");

                                let heightmap_name = original_heightmap_path
                                    .file_name()
                                    .expect("Couldn't get file name");
                                let new_heightmap_path = heightmap_dir.join(heightmap_name);

                                fs::copy(&original_heightmap_path, &new_heightmap_path)
                                    .expect("Couldn't copy heightmap to storage directory");

                                // Move Soilmap
                                let original_soilmap_path =
                                    new_soilmap_path.get().expect("Couldn't get soilmap path");
                                let soilmap_dir =
                                    get_soilmap_dir(&project_id.to_string(), &new_id.to_string())
                                        .expect("Couldn't get soilmap dir");

                                let soilmap_name = original_soilmap_path
                                    .file_name()
                                    .expect("Couldn't get file name");
                                let new_soilmap_path = soilmap_dir.join(soilmap_name);

                                fs::copy(&original_soilmap_path, &new_soilmap_path)
                                    .expect("Couldn't copy soilmap to storage directory");

                                // Move Rockmap
                                let original_rockmap_path =
                                    new_rockmap_path.get().expect("Couldn't get rockmap path");
                                let rockmap_dir =
                                    get_heightmap_dir(&project_id.to_string(), &new_id.to_string())
                                        .expect("Couldn't get rockmap dir");

                                let rockmap_name = original_rockmap_path
                                    .file_name()
                                    .expect("Couldn't get file name");
                                let new_rockmap_path = rockmap_dir.join(rockmap_name);

                                fs::copy(&original_rockmap_path, &new_rockmap_path)
                                    .expect("Couldn't copy rockmap to storage directory");

                                // Set Landscape in SavedState and update landscape_data list
                                let mut saved_state = state_helper
                                    .saved_state
                                    .as_ref()
                                    .expect("Couldn't get saved state")
                                    .lock()
                                    .unwrap();

                                let landscapes = saved_state
                                    .landscapes
                                    .as_mut()
                                    .expect("Couldn't get saved landscapes");

                                let new_landscape = LandscapeData {
                                    id: new_id.to_string(),
                                    heightmap: Some(File {
                                        id: Uuid::new_v4().to_string(),
                                        fileName: heightmap_name
                                            .to_str()
                                            .expect("Couldn't get string")
                                            .to_string(),
                                        cloudfrontUrl: "".to_string(),
                                        normalFilePath: new_heightmap_path
                                            .to_str()
                                            .expect("Couldn't get path string")
                                            .to_string(),
                                    }),
                                    rockmap: Some(File {
                                        id: Uuid::new_v4().to_string(),
                                        fileName: rockmap_name
                                            .to_str()
                                            .expect("Couldn't get string")
                                            .to_string(),
                                        cloudfrontUrl: "".to_string(),
                                        normalFilePath: new_rockmap_path
                                            .to_str()
                                            .expect("Couldn't get path string")
                                            .to_string(),
                                    }),
                                    soil: Some(File {
                                        id: Uuid::new_v4().to_string(),
                                        fileName: soilmap_name
                                            .to_str()
                                            .expect("Couldn't get string")
                                            .to_string(),
                                        cloudfrontUrl: "".to_string(),
                                        normalFilePath: new_soilmap_path
                                            .to_str()
                                            .expect("Couldn't get path string")
                                            .to_string(),
                                    }),
                                };

                                landscapes.push(new_landscape);

                                landscape_data.set(
                                    saved_state
                                        .landscapes
                                        .as_ref()
                                        .expect("Couldn't get landscape data")
                                        .clone(),
                                );

                                state_helper.save_saved_state(project_id, saved_state);
                            }
                        }),
                    ))
                    .into_any()
                } else {
                    empty().into_any()
                }
            },
        ),
        scroll(
            dyn_stack(
                move || landscape_data.get(),
                move |landscape_data| landscape_data.id.clone(),
                move |landscape_data| {
                    landscape_item(state_2.clone(), gpu_helper.clone(), landscape_data)
                },
            )
            .into_view(),
        ),
    ))
    .style(|s| {
        s.flex_direction(FlexDirection::Row)
            .flex_wrap(FlexWrap::Wrap)
    })
    .style(|s| s.width(260.0))
}
