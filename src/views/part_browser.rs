use std::sync::{Arc, Mutex, MutexGuard};

use super::shared::dynamic_img;
use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::floem::common::small_button;
use midpoint_engine::floem::ext_event::create_signal_from_tokio_channel;
use midpoint_engine::floem::reactive::SignalGet;
use midpoint_engine::floem::reactive::SignalUpdate;
use midpoint_engine::floem::reactive::{create_effect, create_rw_signal, RwSignal};
use midpoint_engine::floem::taffy::{FlexDirection, FlexWrap};
use midpoint_engine::floem::views::h_stack;
use midpoint_engine::floem::views::text_input;
use midpoint_engine::floem::views::{
    container, dyn_container, dyn_stack, empty, label, scroll, v_stack,
};
use midpoint_engine::floem::IntoView;
use midpoint_engine::helpers::saved_data::File;
use tokio::spawn;
use uuid::Uuid;
use wgpu::util::DeviceExt;

use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::{GpuHelper, View, WindowHandle};

use crate::editor_state::UIMessage;
use crate::editor_state::{EditorState, StateHelper};
use crate::gql::generateTexture::generate_texture;
use crate::helpers::auth::read_auth_token;
use crate::helpers::textures::save_texture;
use crate::helpers::utilities::get_filename;

pub fn part_item(image_path: String, label_text: String) -> impl View {
    v_stack(
        ((
            dynamic_img(image_path, label_text.clone())
                .style(|s| s.width(120.0).height(120.0).border_radius(5.0)),
            label(move || label_text.clone()),
        )),
    )
    .style(|s| s.width(120.0))
}

pub fn part_browser(
    state_helper: Arc<Mutex<StateHelper>>,
    gpu_helper: Arc<Mutex<GpuHelper>>,
    viewport: Arc<Mutex<Viewport>>,
) -> impl View {
    // let part_data: RwSignal<Vec<File>> = create_rw_signal(Vec::new());

    let state_2 = Arc::clone(&state_helper);

    let generate_field = create_rw_signal("".to_string());
    let generate_active = create_rw_signal(false);
    let generate_disabled = create_rw_signal(false);

    // let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    // let tx = Arc::new(tx);
    let part_data: RwSignal<Vec<File>> = create_rw_signal(Vec::new());

    // Create signal from channel
    // let update_signal = create_signal_from_tokio_channel(rx);

    // Handle updates in UI thread
    // create_effect(move |_| {
    //     if let Some(msg) = update_signal.get() {
    //         match msg {
    //             UIMessage::UpdateTextures(textures) => part_data.set(textures),
    //             UIMessage::AddTexture(file) => part_data.update(|t| t.push(file)),
    //             _ => return,
    //         }
    //     }
    // });

    // create_effect({
    //     let tx = tx.clone();
    //     move |_| {
    //         let tx = tx.clone();
    //         let mut state_helper = state_helper.lock().unwrap();
    //         // let mut named_signals = state_helper.named_signals.lock().unwrap();

    //         // named_signals.texture_browser = Some(part_data);
    //         // state_helper.register_file_signal("texture_browser".to_string(), part_data);
    //         state_helper.register_file_signal("texture_browser".to_string(), tx);

    //         let saved_state = state_helper
    //             .saved_state
    //             .as_ref()
    //             .expect("Couldn't get saved state")
    //             .lock()
    //             .unwrap();

    //         part_data.set(
    //             saved_state
    //                 .textures
    //                 .as_ref()
    //                 .expect("Couldn't get texture data")
    //                 .clone(),
    //         );
    //     }
    // });

    v_stack((
        h_stack((
            // rich_text?
            text_input(generate_field).style(|s| s.width(200.0)),
            small_button(
                if generate_disabled.get() {
                    "Generating..."
                } else {
                    "Generate"
                },
                "plus",
                {
                    let generate_field = generate_field.clone();

                    move |_| {
                        let state_helper = state_2.lock().unwrap();
                        let generate_field = generate_field.clone();

                        println!("Preparing generation...");

                        generate_disabled.set(true);

                        // Get the data you need before spawning
                        // let renderer_state = state_helper
                        //     .renderer_state
                        //     .as_ref()
                        //     .expect("Couldn't get RendererState")
                        //     .lock()
                        //     .unwrap();
                        // let selected_project_id = renderer_state
                        //     .project_selected
                        //     .as_ref()
                        //     .expect("Couldn't get current project")
                        //     .to_string();

                        // let generated_field_val = generate_field.get();

                        // // Use the runtime handle to spawn
                        // tokio::runtime::Handle::current().spawn(async move {
                        //     // Now we can safely use generate_field inside async block
                        //     let auth_token = read_auth_token();

                        //     println!(
                        //         "Generating... {:?} {:?}",
                        //         auth_token,
                        //         generated_field_val.clone()
                        //     );

                        //     let part_data =
                        //         generate_texture(auth_token, generated_field_val.clone()).await;
                        //     let textureBase64 = part_data
                        //         .expect("Couldn't unwrap texture data")
                        //         .generateTexture;

                        //     println!("Saving...");
                        //     // save texture to sync directory (to be uploaded to S3)
                        //     let textureFilename = get_filename(generated_field_val.clone());
                        //     let textureFilename = textureFilename + ".png";

                        //     save_texture(selected_project_id, textureBase64, textureFilename);

                        //     // Update saved state - rather update on websocket, its the only way to get cloudfrontUrl
                        //     println!("Syncing...");
                        // });
                    }
                },
                generate_active,
            )
            .disabled(move || generate_disabled.get()),
        ))
        .style(|s| s.margin_bottom(7.0)),
        scroll(
            dyn_stack(
                move || part_data.get(),
                move |part_data| part_data.id.clone(),
                move |part_data_real| {
                    let current_textures = part_data.get(); // Add this to ensure reactivity
                    part_item(part_data_real.normalFilePath, part_data_real.fileName)
                },
            )
            .into_view()
            .style(|s| {
                s.width(260.0)
                    .flex_direction(FlexDirection::Row)
                    .flex_wrap(FlexWrap::Wrap)
                    .gap(10.0)
            }),
        ),
    ))
    .style(|s| s.width(260.0))
}
