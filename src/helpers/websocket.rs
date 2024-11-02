use async_trait::async_trait;
use ezsockets::{ClientConfig, CloseCode, CloseFrame, Error};
use midpoint_engine::floem::reactive::SignalUpdate;
use midpoint_engine::helpers::saved_data::File;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};
// use tokio::sync::Mutex;
use midpoint_engine::floem::reactive::SignalGet;
use url::Url;

// use midpoint_engine::floem::reactive::RUNTIME;

use crate::editor_state::{StateHelper, UIMessage};
use crate::helpers::utilities::parse_ws_command;

// Types for our messages
#[derive(Debug, Serialize)]
pub struct JoinGroupPayload {
    pub group_id: String,
}

// #[derive(Debug, Clone)]
// pub struct LocalState_helper {
//     pub token: Option<String>,
//     pub current_project_id: Option<String>,
// }

// Commands that can be sent to our client
#[derive(Debug)]
pub enum Call {
    JoinGroup,
    Disconnect,
    SendMessage(String),
}

// Our WebSocket client
struct WebSocketClient {
    handle: ezsockets::Client<Self>,
    state_helper: Arc<Mutex<StateHelper>>,
    on_message: Arc<dyn Fn(String, String, Vec<File>) + Send + Sync>,
}

#[async_trait]
impl ezsockets::ClientExt for WebSocketClient {
    type Call = Call;

    async fn on_text(&mut self, text: String) -> Result<(), Error> {
        println!("Received message: {}", text);

        // Handle refresh state_helper command
        if let Ok(command_data) = parse_ws_command(&text) {
            // let command_data = parse_ws_command(&text).expect("Couldn't parse WebSocket command");

            let mut state_helper = self.state_helper.lock().unwrap();
            let file_signals = Arc::clone(&state_helper.file_signals);

            let mut saved_state = state_helper
                .saved_state
                .as_mut()
                .expect("Couldn't get Saved State")
                .lock()
                .unwrap();

            let new_file = File {
                id: command_data.new_id,
                cloudfrontUrl: command_data.cloudfront_url,
                fileName: command_data.filename,
                normalFilePath: command_data.normal_file_path,
            };

            if (command_data.command == "add_model") {
                saved_state.models.push(new_file);
            } else if (command_data.command == "add_concept") {
                saved_state.concepts.push(new_file.clone());

                println!("Updating signal...");

                let concept_browser_signal = file_signals
                    .lock()
                    .unwrap()
                    .get("concept_browser")
                    .cloned()
                    .expect("Couldn't get concept browser file signal");

                // let new_concepts = saved_state.concepts.to_vec();

                let tx = Arc::clone(&concept_browser_signal);
                tokio::spawn(async move {
                    // Process in another thread, then send update
                    println!("send tx");
                    tx.send(UIMessage::AddConcept(new_file.clone())).unwrap();
                });

                drop(saved_state);
                drop(state_helper);

                let mut state_helper = self.state_helper.lock().unwrap();
                state_helper.save_current_saved_state();
                drop(state_helper);

                println!("Texture Finished!");
            } else if (command_data.command == "add_landscape_heightmap") {
                let mut landscape = saved_state
                    .landscapes
                    .as_mut()
                    .expect("Couldn't get landscape data")
                    .iter_mut()
                    .find(|l| l.id == command_data.parent_id)
                    .expect("Couldn't get associated landscape");
                landscape.heightmap = Some(new_file);
            } else if (command_data.command == "add_landscape_rockmap") {
                let mut landscape = saved_state
                    .landscapes
                    .as_mut()
                    .expect("Couldn't get landscape data")
                    .iter_mut()
                    .find(|l| l.id == command_data.parent_id)
                    .expect("Couldn't get associated landscape");
                landscape.rockmap = Some(new_file);
            } else if (command_data.command == "add_landscape_soil") {
                let mut landscape = saved_state
                    .landscapes
                    .as_mut()
                    .expect("Couldn't get landscape data")
                    .iter_mut()
                    .find(|l| l.id == command_data.parent_id)
                    .expect("Couldn't get associated landscape");
                landscape.soil = Some(new_file);
            } else if (command_data.command == "add_texture") {
                println!("adding texture... {:?}", new_file);
                saved_state
                    .textures
                    .get_or_insert_with(Vec::new)
                    .push(new_file.clone());

                println!("Updating signal...");

                let texture_browser_signal = file_signals
                    .lock()
                    .unwrap()
                    .get("texture_browser")
                    .cloned()
                    .expect("Couldn't get texture browser file signal");

                let new_textures = saved_state
                    .textures
                    .as_ref()
                    .expect("Couldn't get saved textures")
                    .to_vec();

                let tx = Arc::clone(&texture_browser_signal);
                tokio::spawn(async move {
                    // Process in another thread, then send update
                    println!("send tx");
                    tx.send(UIMessage::AddTexture(new_file.clone())).unwrap();
                });

                drop(saved_state);
                drop(state_helper);

                let mut state_helper = self.state_helper.lock().unwrap();
                state_helper.save_current_saved_state();
                drop(state_helper);

                println!("Texture Finished!");
            } else {
                println!("Unhandled command {:?}", command_data.command);
            }
        }
        // else {
        //     // Handle other messages
        //     (self.on_message)(text);
        // }

        Ok(())
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), Error> {
        println!("Received binary message: {:?}", bytes);
        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), Error> {
        match call {
            Call::JoinGroup => {
                let state_helper = self.state_helper.lock().unwrap();
                let token = state_helper.auth_token.clone();
                let project_id = state_helper
                    .renderer_state
                    .as_ref()
                    .expect("Couldn't get RendererState")
                    .lock()
                    .unwrap()
                    .project_selected
                    .clone();
                let project_id = project_id.expect("Couldn't get current project");

                let payload = json!({
                    "Authorization": format!("Bearer {}", token),
                    "event": "join",
                    "payload": JoinGroupPayload {
                        group_id: project_id.to_string()
                    }
                });
                println!("Joining group: {}", project_id);
                self.handle.text(payload.to_string())?;

                drop(state_helper);
            }
            Call::Disconnect => {
                self.handle.close(Some(CloseFrame {
                    code: CloseCode::Normal,
                    reason: "Client disconnecting".to_string(),
                }))?;
            }
            Call::SendMessage(message) => {
                self.handle.text(message)?;
            }
        }
        Ok(())
    }

    async fn on_connect(&mut self) -> Result<(), Error> {
        println!("Connected to WebSocket server");
        // Automatically join group upon connection
        // may want to do this upon opening a project, or perhaps start WebSocketManager after opening project
        // self.handle.call(Call::JoinGroup)?;
        Ok(())
    }

    // async fn on_disconnect(&mut self) -> Result<(), Error> {
    //     println!("Disconnected from WebSocket server");
    //     Ok(())
    // }
}

pub struct WebSocketManager {
    handle: Option<ezsockets::Client<WebSocketClient>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        WebSocketManager { handle: None }
    }

    pub async fn connect(
        &mut self,
        state_helper: Arc<Mutex<StateHelper>>,
        on_message: impl Fn(String, String, Vec<File>) + Send + Sync + 'static,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = Url::parse("ws://localhost:4000")?;
        let config = ClientConfig::new(url);

        // let state_helper = Arc::new(Mutex::new(state_helper));
        let on_message = Arc::new(on_message);

        let (handle, future) = ezsockets::connect(
            move |handle| WebSocketClient {
                handle,
                state_helper: state_helper.clone(),
                on_message: on_message.clone(),
            },
            config,
        )
        .await;

        // Store handle for later use
        self.handle = Some(handle.clone());

        // Spawn the WebSocket future
        tokio::spawn(async move {
            if let Err(e) = future.await {
                tracing::error!("WebSocket error: {:?}", e);
            }
        });

        Ok(())
    }

    pub fn disconnect(&self) {
        if let Some(handle) = &self.handle {
            tracing::warn!("Disconnecting WebSocket client...");
            let _ = handle.call(Call::Disconnect);
        }
    }

    pub fn send_message(&self, message: String) {
        if let Some(handle) = &self.handle {
            let _ = handle.call(Call::SendMessage(message));
        }
    }

    pub fn join_group(&self) {
        if let Some(handle) = &self.handle {
            let _ = handle.call(Call::JoinGroup);
        }
    }
}
