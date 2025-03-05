use std::fs;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};

use super::shared::dynamic_img;
use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::floem::common::{simple_button, small_button};
use midpoint_engine::floem::ext_event::create_signal_from_tokio_channel;
use midpoint_engine::floem::reactive::SignalGet;
use midpoint_engine::floem::reactive::{create_effect, create_rw_signal, RwSignal, SignalUpdate};
use midpoint_engine::floem::taffy::{FlexDirection, FlexWrap};
use midpoint_engine::floem::views::{
    button, container, dyn_container, dyn_stack, empty, label, scroll, v_stack,
};
use midpoint_engine::floem::IntoView;
use midpoint_engine::floem_renderer::gpu_resources;
use midpoint_engine::handlers::handle_add_model;
use midpoint_engine::helpers::saved_data::{ComponentData, ComponentKind, File, GenericProperties};
use midpoint_engine::helpers::utilities::get_models_dir;
use nalgebra::{Isometry3, Vector3};
use rfd::FileDialog;
use uuid::Uuid;
use wgpu::util::DeviceExt;

use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::{EditorState, StateHelper, UIMessage};

// type BoxedAsyncFn = Box<dyn Fn() -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync>;

pub fn model_item(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    label_text: String,
    filename: String,
    model_id: String,
) -> impl View {
    let active = create_rw_signal(false);
    let active_2 = create_rw_signal(false);

    v_stack((
        label(move || label_text.clone()),
        small_button(
            "Add to Scene",
            "plus",
            {
                move |_| {
                    // add to scene
                    let state_helper = state_helper.lock().unwrap();
                    let gpu_helper = gpu_helper.lock().unwrap();
                    let gpu_resources = gpu_helper
                        .gpu_resources
                        .as_ref()
                        .expect("Couldn't get gpu resources");
                    let renderer_state = state_helper
                        .renderer_state
                        .as_ref()
                        .expect("Couldn't get RendererState");
                    let project_id = state_helper
                        .project_selected_signal
                        .expect("Couldn't get project signal")
                        .get();

                    // different than the asset id, this is the component instance id
                    let component_id = Uuid::new_v4();

                    handle_add_model(
                        renderer_state.clone(),
                        &gpu_resources.device,
                        &gpu_resources.queue,
                        project_id.to_string(),
                        model_id.clone(),
                        component_id.to_string(),
                        filename.clone(),
                        Isometry3::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0)),
                    );

                    // create physics
                    let mut renderer_state = renderer_state.lock().unwrap();
                    renderer_state
                        .add_collider(component_id.to_string().clone(), ComponentKind::Model);

                    // update saved data
                    let mut saved_state = state_helper
                        .saved_state
                        .as_ref()
                        .expect("Couldn't get RendererState")
                        .lock()
                        .unwrap();

                    // add to `levels.components` in SavedContext
                    let model_component = ComponentData {
                        id: component_id.to_string().clone(),
                        kind: Some(ComponentKind::Model),
                        asset_id: model_id.clone(),
                        generic_properties: GenericProperties {
                            name: "New Model Component".to_string(),
                            position: [0.0, 0.0, 0.0],
                            rotation: [0.0, 0.0, 0.0],
                            scale: [1.0, 1.0, 1.0],
                        },
                        landscape_properties: None,
                        model_properties: None,
                    };
                    let mut levels = saved_state.levels.as_mut().expect("Couldn't get levels");
                    levels
                        .get_mut(0)
                        .expect("Couldn't get first level")
                        .components
                        .get_or_insert_with(Vec::new)
                        .push(model_component);

                    let project_id = state_helper
                        .project_selected_signal
                        .expect("Couldn't get project signal")
                        .get();

                    state_helper.save_saved_state(project_id, saved_state);
                }
            },
            active,
        ),
        // small_button(
        //     "Add Skeleton",
        //     "plus",
        //     {
        //         move |_| {
        //             // renderer_state.current_view = "animation_retarget".to_string();
        //         }
        //     },
        //     active,
        // ),
    ))
    .style(|s| s.width(120.0))
}

pub fn model_browser(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    let state_2 = Arc::clone(&state_helper);
    let state_3 = Arc::clone(&state_helper);
    let gpu_2 = Arc::clone(&gpu_helper);

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let tx = Arc::new(tx);
    let model_data: RwSignal<Vec<File>> = create_rw_signal(Vec::new());
    let update_signal = create_signal_from_tokio_channel(rx);

    // Handle updates in UI thread
    create_effect(move |_| {
        if let Some(msg) = update_signal.get() {
            match msg {
                UIMessage::UpdateModels(models) => model_data.set(models),
                UIMessage::AddModel(file) => model_data.update(|t| t.push(file)),
                _ => return,
            }
        }
    });

    create_effect({
        let tx = tx.clone();
        move |_| {
            let tx = tx.clone();
            let mut state_helper = state_helper.lock().unwrap();

            state_helper.register_file_signal("model_browser".to_string(), tx);

            let saved_state = state_helper
                .saved_state
                .as_ref()
                .expect("Couldn't get saved state")
                .lock()
                .unwrap();

            model_data.set(saved_state.models.clone());
        }
    });

    v_stack((
        simple_button("Add Model".to_string(), move |_| {
            let original_file_path = FileDialog::new()
                .add_filter("model", &["glb"])
                .set_directory("/")
                .pick_file()
                .expect("Couldn't get file path");

            let new_id = Uuid::new_v4();

            let state_helper = state_3.lock().unwrap();
            let project_id = state_helper
                .project_selected_signal
                .expect("Couldn't get project signal")
                .get();

            let models_dir =
                get_models_dir(&project_id.to_string()).expect("Couldn't get models dir");

            let model_path = models_dir.join(new_id.to_string() + ".glb");

            fs::copy(&original_file_path, &model_path)
                .expect("Couldn't copy heightmap to storage directory");

            // Update SavedState and model_data
            let mut saved_state = state_helper
                .saved_state
                .as_ref()
                .expect("Couldn't get saved state")
                .lock()
                .unwrap();

            let models = &mut saved_state.models;

            let new_model = File {
                id: new_id.to_string(),
                fileName: new_id.to_string() + ".glb",
                cloudfrontUrl: "".to_string(),
                normalFilePath: model_path
                    .to_str()
                    .expect("Couldn't get path string")
                    .to_string(),
            };

            models.push(new_model);

            model_data.set(saved_state.models.clone());

            state_helper.save_saved_state(project_id, saved_state);
        }),
        scroll(
            dyn_stack(
                move || model_data.get(),
                move |model_data| model_data.id.clone(),
                move |model_data| {
                    model_item(
                        state_2.clone(),
                        gpu_2.clone(),
                        model_data.fileName.clone(),
                        model_data.fileName.clone(),
                        model_data.id.clone(),
                    )
                },
            )
            .into_view()
            .style(|s| {
                s.flex_direction(FlexDirection::Row)
                    .flex_wrap(FlexWrap::Wrap)
            }),
        ),
    ))
    .style(|s| s.width(260.0))
}
